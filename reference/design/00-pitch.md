# 00 — Pitch

> **Loams: the unified hybrid retrieval engine on object storage.**
> The open-source, S3-native engine for AI retrieval — vectors, full text and GraphRAG expansion in one planned query, over open formats in *your* bucket, with stateless compute and a RAM + NVMe hot tier.

Status: **Approved** · 2026-09-22 · revised 2026-09-25 (architecture review; positioning D50, surfaces D42–D45)

> **Amendment (approved 2026-10-21; [§77](77-kv-gateways.md), D1640).** RESP (Redis) and CQL (Cassandra) join the protocols Loams speaks (D42), as `loams-gateway` modules on the `loams-kv` store, with tight scopes and published out-of-scope lists. They are designed, not built.

---

## 1. The problem

A production AI application today typically runs five stateful systems — six, once agent runs need a workflow engine — each with its own cluster, replicas, upgrade cadence, security model and on-call burden:

| Need | Typical system | What it holds |
|---|---|---|
| Event ingest, agent traces, CDC | **Kafka** | Raw events |
| Keyword / hybrid search | **Elasticsearch** | Copy #2 of documents |
| Semantic retrieval | **Qdrant** | Copy #3 (embeddings + payload) |
| Knowledge graph / GraphRAG / agent memory | **Neo4j** | Copy #4 (entities, relations) |
| Product analytics, evals, cost/usage dashboards | **ClickHouse** | Copy #5 |
| Durable agent runs: retries, long waits, human-in-the-loop, fan-out | **Temporal**, or a queue + cron + Postgres | Workflow state, in yet another database |

The consequences:

1. **Cost.** Each system replicates 2–3× on block storage (EBS gp3 ≈ $0.08/GB-month ⇒ ≈ $0.16–0.24 effective) versus S3 at ≈ $0.023. Clusters are sized for peak and run 24/7. Kafka additionally pays cross-AZ replication traffic on every byte.
2. **Copies and drift.** The same entity lives in four stores glued by connectors and CDC jobs. When they drift, agents retrieve stale or contradictory context — a correctness bug, not just an ops annoyance.
3. **Retrieval glue lives in app code.** A single "hybrid GraphRAG" retrieval is BM25 (ES) + ANN (Qdrant) + k-hop expansion (Neo4j) + fusion in Python across three network hops. Nothing can plan or optimize it as a whole.
4. **No cross-store consistency.** "I just wrote this memory; can the next agent step see it in search, vectors and graph?" has no answer in a five-store stack.
5. **Agent runs are not durable.** A crash in step 7 of a 10-step agent run repeats paid model calls or loses the run, unless a sixth system tracks workflow state.

## 2. What Loams is

A single Rust engine that serves **hybrid retrieval** — dense and sparse vectors, BM25 full text, filters and graph expansion, fused in one DataFusion plan — from **open formats on object storage**, with **five first-class objects**: *streams, tables, collections, graphs, links*.

Loams takes over the role each system plays in an AI retrieval stack, not its wire protocol. It speaks four protocols (D42): its **native REST/gRPC API**, **Arrow Flight SQL**, the **Qdrant** REST + gRPC API and a **targeted Elasticsearch subset**. It does not emulate Neo4j or ClickHouse. OTLP logs ingest joins them in v1.0 (D73), and a Kafka wire-protocol gateway in M5 (D74).

| Role in the stack (today) | Loams object | Durable format | Surface | Milestone |
|---|---|---|---|---|
| Semantic retrieval (Qdrant) | Collection (vectors) | Lance | Qdrant REST + gRPC | M1 |
| Keyword / hybrid search (Elasticsearch) | Collection (text) | Tantivy splits | ES subset: document APIs, `_bulk`, `_search` with the core Query DSL, `knn`, hybrid + RRF (D48) | M1 |
| Graph expansion for GraphRAG (Neo4j) | Graph, mapped over collections and tables | CSR/CSC sidecars | `expand` stage in the native hybrid search API; `graph_expand` / `graph_neighbors` SQL table functions; Loams-native graph-store adapters for LightRAG and the LlamaIndex property graph (D44) | M3 |
| Analytics, evals, dashboards (ClickHouse) | Table | **Apache Iceberg** (via Lakekeeper) | Flight SQL and the native API; DuckDB, Trino, Spark or ClickHouse through the Iceberg REST catalog (D45) | M4 |
| Event ingest, agent traces (Kafka) | Stream | Loams log segments on S3 | Flight `DoPut` bulk ingest (D49); native streaming API over HTTP and gRPC (idempotent produce, streaming subscribe, named consumers; D72); OTLP logs ingest (D73); Flight `DoGet` replay (D43); the Kafka wire protocol (D74) | M1 (`DoPut` ingest), M2 (stream API, OTLP logs), M5 (replay, Kafka) |
| Connectors/CDC glue | Link, changelog stream | — | Declarative DDL; changelogs read through the native streaming API | M0 (links), M5 (changelogs) |
| Temporal / queue + cron for agent runs | Durable promises (a service, §14) | One document per workflow origin on S3 | **Resonate protocol** (TS, Python, Rust, Go, Java SDKs) | M3 |

Every object is also reachable through the **native API/SDK**, where one request does vector + BM25 + filter + graph expansion + fusion as a single planned query, and through **Flight SQL**, which carries SQL queries and bulk `DoPut` ingest for any language with an ADBC driver (D49).

AI labs' data pipelines read and write the same collections directly: Ray Data, PySpark on Spark 4 or Sail, PyTorch loaders and Polars get pinned snapshots through the Python SDK (scan pinning and retained dataset tags), and Iceberg engines read the tables (§17, D51).

v1.0 is hybrid retrieval (M1) plus production hardening (M2): authentication and authorization, tenant quotas, telemetry, multi-node clusters, a Postgres metastore option and a Kubernetes operator (D46).

Agent code uses the Resonate SDKs unmodified: every step of an agent run is a durable promise stored in the same bucket as the agent's memory, so a crashed run resumes where it stopped and a step's result can carry the consistency token of the memory it wrote.

## 3. Three core ideas

1. **Object storage is the only source of truth.** Every byte at rest lives in S3/GCS/Azure Blob in open formats (Iceberg, Lance, Tantivy, Parquet). Compute nodes are stateless; losing any node loses no data. Storage costs S3 prices; compute scales independently and to zero per namespace.
2. **The log is the spine.** Every write — a native write, a Flight `DoPut`, an ES `_bulk`, a Qdrant upsert — lands in a stream first. Everything else is a *materialization* of the log, maintained by declared **links**. That kills the connector zoo and gives every write a **consistency token** usable in any read on any object.
3. **Hot tiers everywhere.** Durable tier = open format on S3; hot tier = derived, node-local, rebuildable acceleration in RAM and NVMe (Qdrant-style HNSW for vectors, pinned splits for text, sorted hot projections for Iceberg tables, cached CSR for graphs) + an in-memory **tail** of not-yet-indexed data. This is the StarRocks data-cache pattern — stateless compute over open files with a RAM + NVMe cache — applied to Lance vector pages, Tantivy splits and graph adjacency as well as Parquet. Cheap by default, fast where it matters, always fresh.

## 4. Why now

| Enabler | Status (2026) |
|---|---|
| S3 conditional writes (`If-None-Match`, `If-Match`) | GA since 2024; extended to CopyObject (2025). GCS generation preconditions, Azure ETags equivalent. |
| Low-latency object storage | S3 Express One Zone (single-digit ms, append support), GCS Rapid (zonal, appendable) |
| Rust data stack maturity | DataFusion 54, Lance 12, Tantivy 0.26, SlateDB 0.16, openraft, iceberg-rust 0.10, arrow-flight 58, foyer |
| Stateless compute over open files is proven | StarRocks' shared-data mode serves Iceberg on S3 from a two-tier (RAM + NVMe) data cache at close to local-disk speed; ClickHouse Cloud separates compute from storage the same way |
| GraphRAG has a narrow query shape | GraphRAG, LightRAG and Cognee retrieve by vector/BM25 seeds → 1–2 hop expansion → rerank, not arbitrary deep traversal. A separate graph database (Neo4j, or Nebula Graph with three stateful daemons: `metad`, `graphd`, `storaged`) adds a silo and an ETL path for that one step |
| Iceberg as the lakehouse lingua franca | v3 (deletion vectors, variant, row lineage) shipping in Snowflake, Databricks, AWS |
| Durable execution as an open protocol | Resonate (Apache-2.0, 2025–26): formally specified distributed async/await, with a server that runs on nothing but a bucket |
| The best designs are closed | turbopuffer (closed), WarpStream (proprietary), Bufstream (acquired by CoreWeave), LanceDB Enterprise serving layer (closed), AutoMQ low-latency WAL (commercial-only), Kuzu (archived after Apple acquisition), Neon (public repo dormant after Databricks acquisition) |

The architecture has been **proven in production**: for search and vectors by a closed product (turbopuffer), for analytics by StarRocks and ClickHouse Cloud. No open-source project applies it to vector + text + graph retrieval in one engine. That is the slot.

## 5. Positioning

**"The Unified Hybrid Retrieval Engine on Object Storage"** (D50): StarRocks-style stateless compute over open formats with a RAM + NVMe hot tier, for vector + text + GraphRAG retrieval. Concretely: the open-source turbopuffer, with drop-in Qdrant and Elasticsearch-subset compatibility for existing AI frameworks, Arrow Flight SQL for every language, Iceberg tables any engine can read, and durable agent workflows through the Resonate SDKs.

Primary buyer: platform teams at companies running AI apps at scale who are paying for (and operating) a vector database, a search cluster and a graph database side by side. Primary user: application engineers building RAG, agents, GraphRAG and eval pipelines.

## 6. Competitive landscape

| Competitor | License | Overlap | Gap Loams exploits |
|---|---|---|---|
| turbopuffer | Closed SaaS | Search + vector on S3 | Closed; no graph expansion, SQL analytics or open formats |
| LanceDB | OSS format; closed serving | Vector + FTS on S3 | Distributed serving/caching/indexing closed; no graph expansion, no Qdrant/ES compatibility |
| Milvus 3.0 | Apache-2.0 (Go/C++) | Lake-native vector DB | Vector-first; no graph expansion, no ES-compatible search surface, no Iceberg analytics |
| HelixDB | Apache-2.0 (since 2026-05) | Rust graph+vector on SlateDB/S3 | Own DSL; no SQL/analytics, no Qdrant/ES compatibility. **Closest OSS rival — watch closely or collaborate.** |
| Databend | Apache-2.0 + ELv2 | Warehouse on S3, marketed since 2026 as "agent-ready" (analytics + full-text + vector) | SQL-first; no Qdrant/ES compatibility, no graph expansion; ELv2 parts |
| StarRocks | Apache-2.0 | Stateless compute + RAM/NVMe data cache over Iceberg | Analytics-first; no Qdrant/ES-compatible retrieval surface, no graph expansion |
| Elastic Serverless | Proprietary | Search on object storage | Closed, expensive, search only |
| Qdrant | Apache-2.0 | Vector | Local-disk architecture; vector only |
| Neo4j, Nebula Graph | GPLv3 + commercial; Apache-2.0 | Knowledge graphs for GraphRAG | Stateful graph clusters beside the retrieval stores; expansion cannot be planned together with the vector/BM25 seed query |
| Apache Fluss | Apache-2.0 (Java, incubating) | Streaming storage for the lakehouse: columnar Arrow log, primary-key tables with changelogs, tiering to Iceberg/Paimon/Lance | JVM + ZooKeeper, data on tablet-server disks with S3 as a tier; no search, vector serving or graph; Flink-centric |

## 7. What Loams is *not* (non-goals)

- **Not an OLTP database.** No multi-statement interactive transactions with millisecond commits over mutable rows. Keep a Postgres for application state; stream its CDC into Loams.
- **Not a Neo4j or ClickHouse protocol emulator.** No Bolt or Cypher (D44), no ClickHouse HTTP interface, dialect or MergeTree DDL (D45). *Amendment approved 2026-10-02 ([§32](32-loams-flow-fabric-house.md) D347, Q333): the separate Loams House service serves a declared, tested ClickHouse surface over the Event Fabric; the engine itself still speaks no ClickHouse protocol.* Graphs are reached through native expansion, analytics through Iceberg and Flight SQL. Streams are reached through the native streaming API and Flight, and from M5 through the Kafka wire protocol, without Kafka transactions (D74).
- **Not a full Elasticsearch or Qdrant clone.** Compatibility is scoped by external conformance suites (client libraries, framework integrations; D13), not by feature parity. The Elasticsearch subset is what the LangChain and LlamaIndex ES suites and BEIR send (D48). No Kibana, Painless or full Query DSL.
- **Not a general-purpose graph database.** 1–2 hop expansion, shortest path and graph algorithms as table functions, planned with the retrieval query; no graph query language and no deep recursive traversal.
- **Not a stream processor.** Stateless transforms and mergeable aggregates in links, yes; windowed joins with checkpointed state, no. External stream processors can write their results as Iceberg tables through Lakekeeper. RisingWave, the companion stream processor (D22), connects over the Kafka gateway in M5 (D74); before that it writes to Loams through its Elasticsearch, HTTP and Iceberg sinks (§02 §7.3).

## 8. Governance and business model (recommendation)

- **License:** Apache-2.0 for the entire engine, all gateways and the operator. Big-company adoption requires it; AGPL/BSL/SSPL dependencies are excluded (§11).
- **Governance:** start company-led, plan for a foundation (LF AI & Data — as Vortex did — or CNCF) once there are ≥3 corporate contributors.
- **Business model (updated 2026-10-19, D1119):** no open core and no paid edition; everything, including metering, billing and the control plane, is Apache-2.0. Ostrium Labs is a research lab building open-source Loams (D1323, 2026-10-21; the earlier services offering is removed). **Do not** withhold any feature from OSS, reliability (quorum WAL) and performance (hot tiers) above all; that is exactly the AutoMQ/LanceDB gap Loams wins on.

## 9. Launch demo

A GraphRAG agent stack (LightRAG or a LlamaIndex property-graph app, with LangChain retrieval) running against **one `loams` binary and one bucket** — its vector and text stores through the unmodified Qdrant and Elasticsearch integrations, its graph store through the Loams-native adapter (D44) — side-by-side with the usual docker-compose of Elasticsearch + Qdrant + Neo4j: same answers, one process instead of three clusters, a fraction of the storage cost, and a consistency token proving read-your-writes across search, vectors and graph expansion. The agent's run loop is a Resonate workflow: `kill -9` the agent mid-run and it resumes at the step it was on, without repeating model calls. The demo needs M3 (native graph and Resonate Phase A); evals over the same bucket's Iceberg tables, queried from DuckDB, join it after M4.
