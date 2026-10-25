# 01 — Architecture: Data Model and System Shape

Status: **Approved** · 2026-09-22 (including amendments: Tantivy for text, Lance + hot HNSW for vectors, hot tier for Iceberg) · revised 2026-09-25 (architecture review: protocol surfaces D42–D45, `MetaStore` trait D47) · amended 2026-09-26 (turbopuffer gap analysis: write backpressure, filter and conditional writes, branches, sharding, customer-managed keys; D86–D96) · amended 2026-09-27 (M1.3 as built: cluster mode, the networked metastore, the artifact layout; D104–D110)

---

## 1. Design principles

1. **Object storage is the only durable source of truth.** The sole exception is the seconds-long WAL tail of `quorum`-class streams, which is 3-way replicated across AZs before acknowledgment (§02).
2. **All compute is stateless.** Any node can be killed at any time. Local RAM/NVMe hold only caches and derived structures that can be rebuilt from object storage.
3. **The log is the spine.** Every mutation enters through a stream. Tables, collections and graphs are materializations of streams, maintained by links.
4. **Open formats at rest.** Iceberg (tables), Lance (collection documents + vectors), Tantivy splits (text), Parquet-style sidecars (graph adjacency). External engines can read Loams’ data without Loams.
5. **Namespace is the unit of everything.** Tenancy, quotas, encryption keys, cache affinity, routing and billing are all per namespace. A cold namespace costs only its S3 bytes.
6. **Compatibility is a gateway concern.** Protocol frontends (§3.3) translate into a small set of internal logical operations. No protocol leaks into the storage or query core. The footprint is deliberately narrow (D42): the native REST/gRPC API, Arrow Flight SQL, the Qdrant API and a targeted Elasticsearch subset, plus the Resonate server (§14) and the MCP server (§15).
7. **Every object has a durable tier and a hot tier** (§04). Correctness never depends on the hot tier.

## 2. Data model

A **namespace** contains five kinds of objects.

### 2.1 Stream
- Partitioned, ordered, offset-addressed log of records `(key, value, headers, timestamp)`.
- Per-stream **WAL class**: `standard` | `express` | `quorum` (§02).
- Retention by time/size, or `compacted` (last value per key).
- Segment **encoding** `kafka` (the Kafka RecordBatch v2 layout, used internally) or `arrow` (columnar, for schema'd streams) (§02).
- **Changelog streams:** any keyed table or collection can expose its row-level changes as a stream (`upsert` or `full` mode with before images) (§02; M5).
- **Explicit streams** are created by users and reached through the native streaming API (HTTP and gRPC produce with idempotent producer ids, streaming subscribe, long-poll fetch, named consumers with committed offsets) and Flight `DoPut`/`DoGet` (D43; M5). **Implicit streams** back every table, collection and graph: a write to a collection is appended to that collection's implicit stream first.

### 2.2 Table
- Columnar, schema'd, partitioned, with a sort key. Append-only or keyed (upsert/delete).
- Durable format: **Apache Iceberg** (Parquet data files, v3 deletion vectors), catalogued in **Lakekeeper** (Iceberg REST catalog).
- Queried through Flight SQL and the native API; readable directly by Spark, Trino, DuckDB, ClickHouse, Snowflake, StarRocks, etc. through the Iceberg REST catalog (D45; M4).

### 2.3 Collection (replaces Elasticsearch + Qdrant)
- Documents keyed by primary key, with any mix of: text fields, keyword/numeric/date fields, dense vectors (possibly several named), sparse vectors, JSON payload.
- Durable format: one **Lance** dataset (documents, vectors, scalar indexes, IVF vector index) **plus** a set of **Tantivy splits** (inverted index, fast fields for aggregations), bound together by a single **collection manifest**.
- Hot tier: Qdrant-derived HNSW on NVMe for pinned/hot collections; pinned splits; in-memory tail index.
- Reached through the native API, Flight SQL (queries and `DoPut` bulk ingest), the Qdrant API and the Elasticsearch subset (M1).

### 2.4 Graph
- A **mapped graph** *defined over* tables and/or collections: vertex labels map to a keyed source, edge types map to a source with `(src_key, dst_key)` columns. Every graph is mapped; there are no Cypher-first native graphs (D44). Graph-store adapters (LightRAG, LlamaIndex property graph) write entities and relations as documents in the collections a graph maps over.
- Durable acceleration: forward (CSR) and reverse (CSC) adjacency sidecars per source segment, plus a dense vertex-ID map.
- Traversal: `ExpandExec` (1–2 hop neighbor expansion, filtered) and shortest path as DataFusion operators, reached through the `graph_expand` / `graph_neighbors` SQL table functions and an `expand` stage in the native hybrid search API (seed by vector/BM25 → expand → rerank); `leiden`, `pagerank` and `wcc` as table functions (§07; M3).

### 2.5 Link (replaces connector and CDC glue)
- Declared, continuously maintained materialization with optional stateless transform:
  - `stream → table` (e.g., an event stream → Iceberg table)
  - `stream → collection` (e.g., an event stream → searchable, embedded collection)
  - `table → collection` (search projection of selected columns + key)
  - `tables/collections → graph` (adjacency maintenance)
- Every link records its **applied offset** atomically with each target commit ⇒ exactly-once materialization and consistency tokens.

### 2.6 Durable execution (replaces Temporal-style workflow engines)
- Not a sixth object kind but a **service** on the Resonate protocol (§14): durable promises, tasks with fenced leases, and schedules, used by agent code through the Resonate SDKs.
- State: one canonical document per workflow **origin** at `ns/<ns>/durable/wf/<origin>`, committed with one conditional PUT per transition; deadlines as timer objects. No metastore traffic.

## 3. System shape

One binary, `loams`, runs any combination of five roles. All roles except `meta` are stateless; `meta` holds only metadata (Raft-replicated, snapshotted to S3). With an external metastore backend (§3.2) the `meta` role is not run.

```
 clients:  native REST/gRPC (+ MCP) │ Arrow Flight SQL │ Qdrant REST/gRPC │ ES REST subset │ Resonate HTTP
                                          │
                              ┌──── gateway role ────┐   protocol → logical ops, auth, rate limits
                              ▼                      ▼
             log role                           query role
             ─ WAL writes (standard/express:    ─ DataFusion planning/execution
               leaderless; quorum: journals)    ─ RAM → NVMe cache (foyer)
             ─ offset sequencing via meta       ─ hot tier: HNSW, pinned splits,
             ─ tail cache for fetch               Iceberg projections, CSR, tails
                              │                      ▲   namespace/object-affinity routing
                              ▼                      │
          ┌───────────────────────── object storage bucket ─────────────────────────┐
          │ WAL objects · log segments · Iceberg (Parquet + metadata) · Lance ·     │
          │ Tantivy splits · adjacency sidecars · manifests · hot-tier artifacts ·  │
          │ durable-execution documents and timers                                  │
          └──────────────────────────────────────────────────────────────────────────┘
                              ▲
             worker role: segmenting, link apply, index build, compaction,
                          Iceberg commits (via Lakekeeper), GC, hot-artifact builds

             meta:        trait MetaStore (§3.2) — namespaces, schemas, stream offsets
                          & segment index, consumer offsets, leases, manifest pointers,
                          link state. Default: embedded openraft (meta role);
                          Postgres or DynamoDB (M2), TiKV (R1, D124) as external backends.
             catalog:     Lakekeeper (Iceberg REST) — for Iceberg tables; external engines use it too
```

### 3.1 Roles

| Role | Responsibility | State | Scales with |
|---|---|---|---|
| `gateway` | Protocol frontends (§3.3), authN/Z, request routing, rate limiting | None | Connections, request rate |
| `log` | Accept writes, write WAL, request offset assignment, serve recent fetches; host `quorum` journals | `quorum` WAL tail only (replicated) | Ingest bandwidth |
| `query` | Execute reads/queries; own the hot tier for its routed objects | Cache + hot tier (derived) | Query load, hot data size |
| `worker` | Background tasks (§09) | None (leases in meta) | Ingest volume, index/compaction backlog |
| `meta` | Metadata state machine (openraft backend only) | Raft log + snapshots (→ S3) | Metadata op rate (sharded by namespace in M6, §18 §5) |

Small deployments run everything in one process (`loams dev` / `loams standalone`); large ones split roles into separately autoscaled pools.

As built (M1.3): `loams cluster --roles meta,log,query,worker,gateway …` runs any subset of the roles in one process (`gateway` implies `log`). Every node runs a metastore replica: voters on `meta` nodes, non-voting learners elsewhere (§3.2). A node registers under the lease `node/<id>`, and collections are owned by rendezvous hashing over the live `query` nodes (§04 §5).

### 3.2 Metastore: a semantic trait, Raft by default

Streams generate high-rate metadata: offset assignment per flush, consumer offset commits, partition and task leases. S3 conditional PUT (tens to hundreds of ms, contended per key) cannot sustain that, so metadata lives in a metastore. Durable object data never flows through it: manifest *bodies* stay immutable objects on S3; only *pointers* live in meta.

Every crate reaches the metastore through **`trait MetaStore`** in `loams-common` (D47), as `Arc<dyn MetaStore>` from M1.2a. The trait is semantic, not raw KV: it exposes the domain operations Loams needs — WAL commit and segment swap, trims and retention, catalog operations and schema evolution (namespaces, streams, collections, aliases, links), manifest-pointer CAS, leases and fencing, and retired-object tracking for GC. Each backend implements the sequencer, fencing and CAS natively instead of rebuilding them over bytes (the Lakekeeper model: one catalog trait, several database backends).

As built (M1.2a): the trait and its records live in `loams_common::meta`; the openraft `MetaClient` implements it in `loams-meta/src/store.rs`. Only the composition roots (`loams`, `loams-sim`) depend on `loams-meta`; the CI step `metastore boundary` enforces it. Reads are named domain queries that each return one consistent state; `commit_wal`, `swap_segment` and `cas_pointer` report whether an earlier attempt had an unknown outcome; `watch_changes` is the long-poll wake-up. Raft administration (node start, membership, snapshots, status) stays on the openraft types.

As built (M1.3): the openraft backend runs over the network. Every node holds a local replica (a voter on `meta` nodes, a learner elsewhere), so `Local` reads stay local; writes go to the leader (`POST /internal/v1/meta/write`) and linearizable reads ask it for a read index, and the client then waits until its own replica has applied that index (M1.3 Ruling 10, D107). Raft RPCs travel as postcard over HTTP on the node's `--listen` address. The routes are private to the openraft backend, not a remote `MetaStore` protocol (that is M2.x, D64). It is the same `MetaClient`, and it passes the `MetaStore` conformance suite over HTTP on the leader and on a learner.

| Backend | Crate | Milestone | Use |
|---|---|---|---|
| Embedded **openraft** (redb log, snapshots in the bucket; KRaft / ClickHouse Keeper style) | `loams-meta` | Default (M0) | `loams dev`, standalone, and clusters of 3 or 5 `meta` nodes; no external dependency |
| **Postgres** | `loams-meta-postgres` | M2 | Deployments that already run managed Postgres (RDS/Aurora, Cloud SQL, Azure Database); no `meta` role to operate. Follows Lakekeeper's patterns; the throughput ceiling is open (Q17) |
| **DynamoDB** | `loams-meta-dynamodb` | M2 | AWS-native and serverless deployments; the hosted control plane's store (M2.x) |
| ~~**TiDB**~~ **TiKV** | ~~`loams-meta-tidb`~~ `loams-meta-tikv` | ~~M6~~ R1 | Metadata beyond one Postgres primary, and Loams cloud and cluster metadata: scale-out, transactional KV over `tikv-client`. Replaces the TiDB backend (D124); no TiDB anywhere (D260) |

Every backend serves the same relaxed contract (D59): `commit_wal` is atomic per partition group, commands carry bounded-skew stamps with GC claims instead of one monotonic clock, and composite reads follow documented read orders. The openraft backend is stronger, but callers rely only on the relaxed contract. One conformance suite, with the linearizability checker, runs against every backend, each backend has its own fault matrix, and the crash and fault gates run on each (§12 §2 item 5, §18 §4). The directory, the sharded metastore and the placement rules for millions of namespaces are in §18 §5.

### 3.3 Protocol surfaces

| Surface | Default port | Objects | Scope | Milestone |
|---|---|---|---|---|
| Native REST | 8080 | All | Collections, the hybrid query (§05 §4), SQL; the native streaming API (M0.3 routes, completed in M2, D72). Never serves `/mcp` (D111) | M1 |
| MCP (Streamable HTTP) | 8083 | Collections | The MCP server (§15) at `/mcp` on its own listener, bound to 127.0.0.1 by default (D111) | M1 |
| Native gRPC | 8081 | All | The native API over gRPC, including streaming subscribe | M2 |
| OTLP (HTTP / gRPC) | 4318 / 4317 | Streams (collections through links) | Logs only: OTLP/HTTP (protobuf, JSON) and OTLP/gRPC (D73, §02 §7.1) | M2 |
| Kafka | 9092 | Streams | Produce, Fetch, ListOffsets, Metadata, ApiVersions; idempotent producers; consumer groups; no transactions (D74, §02 §7.2) | M5 |
| Arrow Flight SQL | 8082 | Collections, tables, streams | SQL queries; Flight `DoPut` bulk ingest into collections and streams (D49), `DoGet` replay of streams (M5); used by the ADBC Flight SQL drivers | M1 |
| Qdrant REST / gRPC | 6333 / 6334 | Collections | Qdrant API Phase A with sparse vectors (§06) | M1 |
| Elasticsearch REST | 9200 | Collections | What the LangChain and LlamaIndex ES suites and BEIR send (D48, §06) | M1 |
| Resonate HTTP | 8001 | Durable promises | The Resonate protocol (§14, §21), embedded in the binary behind the `durable` feature, bound to 127.0.0.1 and refusing other addresses until auth exists (D138) | D1 |

Each surface is enabled individually (§10 §2). The Kafka wire protocol follows in M5 (D74); there is no Bolt/Cypher (D44) or ClickHouse (D45) surface.

## 4. Data flow

### 4.1 Write (any protocol)
1. Gateway authenticates and translates the request into a logical write against a stream (explicit or implicit). A collection write is admitted only while the collection's unapplied backlog (records past `applied` and their bytes) is under its budget; otherwise it is refused with HTTP 429 or gRPC `RESOURCE_EXHAUSTED` and `Retry-After` (D86). A bulk load may send `Loams-Backpressure: off`, which admits writes up to 4× the budget; while the backlog is above the budget, strong reads may fall back to range tails or answer `Unavailable`, and `Eventual` keeps serving.
2. A `log` node appends to the WAL per the stream's class and obtains dense offsets from meta (or from its journal for `quorum`).
3. The client is acknowledged with a **consistency token** `{(stream, partition, offset)…}`.
4. Workers asynchronously apply links: build Lance fragments + Tantivy splits, append Iceberg data files, update adjacency — each commit atomically records its applied offset.

### 4.2 Read
1. Gateway translates into a logical plan (DataFusion).
2. Plan executes on `query` nodes chosen by affinity routing (§04). For each object, the read = **durable/hot state @ applied offset ∪ tail (applied offset, requested offset]**.
3. Default is **strong consistency** (read sees all acknowledged writes); `eventual` mode skips the tail for lower latency.

## 5. Consistency model

| Scope | Guarantee |
|---|---|
| Within a stream partition | Total order; acknowledged writes durable per WAL class (§02) |
| Single-object reads (default) | Strong: sees all writes acknowledged before the read began (via tail merge) |
| Cross-object reads | Snapshot per object; with a consistency token, guaranteed to reflect the token's offsets in every object that derives from those streams, on every metastore backend and node, during namespace moves and under stale routing (D76) |
| Atomic multi-record writes | Atomic per request within one stream (a native write batch, an ES `_bulk`, a Qdrant upsert — §02) |
| External Iceberg readers | See Iceberg snapshots at commit cadence (default 10–60 s); no tail |
| Durable promises and tasks (§14) | Linearizable per workflow origin; independent across origins; searches are surveys |
| Changelog streams | Per key, change order = source commit order; exactly-once via fenced appends (§02 §8.1) |
| Filter writes (delete or patch by filter) | The filter is evaluated at one pin; each batch of 1 000 keys is atomic, the call is not. In M1 a key changed after the pin is still written; from M2 the filter is re-checked at apply (Read Committed) (D87, D89) |
| Conditional writes (M2) | Decided at apply in partition offset order, by the same function in link apply and the tail; serializable per key (D89) |
| Branches (M2) | A point-in-time copy of a retained manifest; isolated from the source after creation (D90) |
| Sharded collections (M2.x) | A write is atomic across shards; strong and token reads see one token on every shard; `eventual` may see shards at different points (D95) |
| Not provided | Multi-object serializable transactions; interactive OLTP transactions |

**Consistency tokens are a hard guarantee, not a required input** (D76). A token means the same on openraft, Postgres, DynamoDB and TiDB under the relaxed contract (D59), during namespace moves and under stale routing: offsets come from the metastore, per-partition order is kept, and any serving node merges the tail up to the token (§18 §3.5). Clients may omit tokens. Default single-object reads are strong, and `eventual` skips the tail. Tokens are needed only for reads through derived objects: tables, graphs, or another collection fed by a link.

## 6. Object storage layout

```
s3://<bucket>/<cluster_prefix>/
  meta/snapshots/<node_id>/<raft_term>-<index>.snap # metastore snapshots, one set per meta node (D17; openraft backend)
  _erasure/<org_id>/<erasure_id>/{request,completion}.rec # write-once erasure-log records (conditional put), kept outside every snapshot (M2, D115)
  wal/<class>/<node_id>/<ulid>.wal                  # standard/express WAL objects: multi-partition, multi-namespace (D25); CMEK namespaces' chunks are envelope-encrypted (M2, D96)
  ns/<namespace_id>/
    keys/<ulid>.key                                  # a CMEK namespace's wrapped key-encryption keys (M2, D96)
    streams/<stream_id>/<partition>/<base_offset:020>-<ulid>.seg
    collections/<collection_id>/
      lance/…                                        # Lance dataset (data/, _deletions/, _transactions/, _indices/, _versions/ incl. d<id>.manifest detached versions)
      text/splits/<ulid>.split                       # Tantivy split bundles (hotcache footer + trailer)
      text/deletes/<split_ulid>/<ulid>.bitmap        # one whole roaring delete bitmap per split (OPDB)
      manifests/<version:020>-<ulid>.pb              # immutable collection manifests (OPCM)
      pkdelta/<version:020>-<ulid>.pkd               # the keys one commit changed, for PK-index repair (OPPD)
      deadletters/<version:020>-<ulid>.dlq           # records one commit dead-lettered (OPDL)
      hot/hnsw/<column>/<source_version:020>-<ulid>/{descriptor.bin,covered.bin,files/…} # derived HNSW artifacts (M1.3 Ruling 6): descriptor (OPHD, written last), covered row set (OPHC), engine files in 256 MiB zstd chunks
      shards/<shard>/…                               # a sharded collection (M2.x, D95): each shard's lance/, text/, manifests/, pkdelta/, deadletters/ and hot/
    graphs/<graph_id>/
      idmap/…                                        # SlateDB instance: external key → dense id
      adj/<source_ref>/<segment_ulid>.{csr,csc}
      manifests/<version>.pb
    pk/<object_id>/…                                 # SlateDB instance: primary key → row location (e.g. pk/collection-<cid>/)
    durable/                                         # durable execution (§14), Resonate blob layout
      wf/<enc(origin)>                               # one document per workflow origin (CAS'd)
      sched/<enc(schedule_id)>                       # one object per schedule
      t/<NN>/<deadline>_<enc(target)>@<token>        # zero-byte timer objects
  warehouse/<namespace_id>/<table_id>/               # Iceberg table location (managed via Lakekeeper)
    metadata/…  data/…
```

All data objects are immutable and named by ULID/version. Only metastore pointers and Iceberg catalog pointers move. The one exception is `durable/`: workflow documents are mutable objects replaced by conditional PUT, because the Resonate protocol commits each transition as one atomic write per origin.

## 7. Failure model (summary)

| Failure | Effect | Recovery |
|---|---|---|
| Any `gateway`/`query`/`worker` node | In-flight requests retried (durable-execution requests are idempotent); hot tier for its objects goes cold | Re-route by rendezvous hashing; warm from S3 or prebuilt artifacts |
| `log` node (standard/express) | Unflushed, unacknowledged batches lost (producer retries) | Any node continues; orphan WAL objects GC'd |
| `log` node (quorum) | None for acknowledged data | Raft election in the journal (~1–3 s) |
| One AZ | `standard`/`express`(multi-bucket)/`quorum` survive with RPO 0 | Capacity in remaining AZs |
| Meta minority (openraft) | None | Raft |
| Meta majority (openraft), or the external metastore unavailable | Writes and metadata-dependent reads stall; cached reads continue | openraft: restore from S3 snapshot + Raft log; Postgres, DynamoDB, TiDB: the backend's own failover and backups (§10 §6) |
| Object store regional outage | Unavailable | Cross-region replication + meta restore (§10) |
