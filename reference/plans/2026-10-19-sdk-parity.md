# SDKP — The App-Building API in Every SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. One agent per task. Work test first. Where this plan gives exact names, paths, fields or test names, use them verbatim. Record any deviation under "Rulings made during execution" at the end of this file.
>
> **Status: Planned** (2026-10-19). Track API/SDK. Owner request, 2026-10-19: *"I want the app-building API in all SDKs."*
>
> SDKP is a cut through [API2](2026-10-10-api2-unified-api-sdks.md), ordered for one goal: a developer can build an app in any of the thirteen languages without calling REST. It owns the following parts of API2:
> - **Tasks 1–2** (SQL and streams on Connect), as P0b and P0c.
> - **Task 13** (facade annotations and the facade index), for `loams.collection.v1` and `loams.graph.v1` only, as P0a.
> - **Task 14** (hand-written facades checked against the index), for the app modules.
> - **Task 17** (typed builders).
> - The app half of **Tasks 15 and 20–22**.
>
> Everything else in API2 stays where it is: REST removal, admin, operations, bulk Flight, releases, docs. API2's owner rulings (Q600–Q614, API2-Q1–Q5) bind here too. When a P0 task lands, API2's matching task gets a one-line pointer to it.

**Goal:** every SDK (TypeScript, Python, Go, Rust, C#, C++, Swift, Java, Kotlin, Ruby, PHP, Dart, Objective-C) exposes the same app-building modules, and each passes the same app scenario in its CI job:
- namespaces, collections and their schemas;
- documents;
- search: text, vector and hybrid, with filters, fusion, highlights, `select` and pagination;
- SQL with parameters;
- streams: produce, fetch and subscribe;
- graph: GQL queries, plus graph expansion from a search;
- live and tables (these already exist);
- API keys and consistency tokens.

**Architecture:**
- **The protos are the source** (§44 §7.3).
  - A method appears in an SDK only if it carries `loams.options.v1.facade`.
  - TypeScript, Python, Go and Rust are rendered by `protoc-gen-loams-facade`.
  - Ruby and Kotlin read the committed descriptor set at runtime.
  - The other seven are hand-written. A `<lang>_facade_matches_index` test checks each one against `sdks/fixtures/facade-index.json`.
- **Hand-written in every language, against shared corpora:**
  - the runtime additions: default namespace, timeouts, token merge, `write_batch`, `subscribe`;
  - the typed builders: schema, filter, search;
  - the collection handle.
- **Proof is the conformance corpus.** The app scenario is recorded once from `loams dev`. Every SDK replays it through its **public facade** against `sdks/conformance/fixture-server.mjs`, in its own CI job. The builder and token corpora pin client-side encoding without a server.

**Tech stack:** unchanged from API2 (connect-rust 0.9, buf 1.73.0, `scripts/sdk/pins.lock`, per-language toolchains in each `sdks/<lang>`).

---

## 1. Where the SDKs are today

Checked on `dev` at `7fc5e603` (2026-10-19).

### 1.1 Why no SDK can build an app

1. **`loams.collection.v1` and `loams.graph.v1` carry no method-level `FacadeOptions`.**
   - They carry only the service-level `ModuleOptions`.
   - The generator emits a call only for a method with a `facade` option (`crates/loams-facade-gen/src/lib.rs`), and the descriptor-driven runtimes apply the same rule. So none of the four collection services and neither graph service reaches any SDK, although the server serves all six on the main port (`crates/loams/src/api/connect.rs` `CATALOGUE`).
   - `sdks/typescript/packages/client/src/loams.ts:87` and `sdks/python/src/loams/loams.py:96` say so in their own comments.
2. **SQL, streams and links have no Connect proto at all.**
   - They exist only as native REST: `/v1/namespaces/{ns}/sql`, `.../streams/...`, `.../links/...` (`crates/loams/src/api/mod.rs`).
   - API2 Tasks 1–3 have not started. The tonic `StreamService` (`crates/loams-stream-grpc/proto`) has only `Produce` and `ProduceCloudEvents`.
   - REST SQL takes no parameters (`crates/loams/src/api/sql.rs`, `{query, consistency}`).
3. **Consistency tokens have an encoding, but R4 does not use it.**
   - `crates/loams-collection/src/token.rs` defines `v1:s<stream>/p<partition>@<offset>,…`, with `merge` taking the max offset per (stream, partition).
   - `docs/sdk/runtime-contract.md` R4 still says the encoding "is not in the protos yet", so the session stores refuse to merge two tokens.
4. **The corpus covers only `instance`, `live` and the AP0 mock packages** (`sdks/fixtures/manifest.json`, 33 cases, 28 required).
5. **Ruby's committed descriptor set is out of date.** `sdks/ruby/gen/loams-descriptor.binpb` holds only `devices`, `errors`, `live` and `options`. Kotlin's holds more, but not `collection/v1/{collection,document}.proto`.

### 1.2 Per language

"Today" lists the modules on the public client, not the generated stubs underneath. Every language already has runtime R1–R10: token sources, retry, idempotency keys, errors, pagination, streams, versions.

| Language | Facade today | Modules today | Missing for app parity |
|---|---|---|---|
| TypeScript (`sdks/typescript/packages/client`) | generated (`src/gen/facade.ts`) | `instance`, `live`, `live.admin`, `tables`, `auth`, `admin.{org,projects,agents,keys,audit}` | **Modules:** `namespaces`, `collections`, `documents`, `search`/`vector`, `graph`, `sql`, `streams`. **Builders:** schema, filter, search. **Runtime:** token merge and session store, default namespace, timeouts, `write_batch`, `streams.subscribe`, the collection handle. **Tests:** the app scenario |
| Python (`sdks/python/src/loams`) | generated (`_gen/facade.py`), sync and `aio` | same as TypeScript | same as TypeScript, in both the sync and the async client |
| Go (`sdks/go`) | generated `gen/facade` plus hand-written `modules.go` (D730) | `instance`, `live`, `live.admin`, `tables`. Go's generation input is narrower than the TypeScript one: `gen/facade/facade.go` has no `auth` or `admin` calls | everything in the TypeScript row, plus widening the Go generation inputs (`buf.gen*.yaml`, `template.env`) to every package in `PROTO_PACKAGES`, and wiring `auth` and `admin.keys` into `modules.go` |
| Rust (`sdks/rust`) | generated (`src/facade.rs`) | `instance`, `live`, `live.admin`, `tables`, `auth`, `admin.*` | same as TypeScript |
| C# (`sdks/csharp`) | hand-written (`Modules.cs`, `Gen/*.cs`) | `instance`, `live`, `live.admin`, `tables`, `approvals`, `devices`, `notifications`, `operations` | everything in the TypeScript row, plus `auth`, `admin.keys` and `<lang>_facade_matches_index` |
| C++ (`sdks/cpp`) | hand-written (`facade.hpp`, `bindings.cpp`) | `instance`, `live`, `live.admin`, `tables`, `approvals`, `devices`, `notifications`, `operations` | same as C# |
| Swift (`sdks/swift`) | hand-written (`Modules.swift`) | `instance`, `live`, `live.admin`, `tables` | same as C# |
| Java (`sdks/java`) | hand-written (`Modules.java`, `Catalogue.java`) | `instance`, `live`, `live.admin`, `tables` | same as C# |
| Kotlin (`sdks/kotlin`) | descriptor-driven (`gen/loams-descriptor.binpb`, `Descriptors.kt`), plus typed wrappers for `live` and `tables` | whatever the descriptor set annotates: `instance`, `live`, `tables`, `approvals` (`collection.v1` is missing from the set) | a regenerated descriptor set (P0a), typed wrappers for the app modules, and everything in the TypeScript row |
| Ruby (`sdks/ruby`) | descriptor-driven (`gen/loams-descriptor.binpb`, `descriptors.rb`) | `live`, `devices` (the set is out of date) | same as Kotlin |
| PHP (`sdks/php`) | hand-written, minimal (`Client.php`: `query`, `watch`) | `instance`, `live` | a module layer for every app module (`Loams\Modules\*`), plus everything in the C# row |
| Dart (`sdks/dart`) | hand-written, minimal (`lib/src/client.dart`) | `instance`, `live` | same as PHP |
| Objective-C (`sdks/objc`) | hand-written, minimal (`LoamsClient.{h,m}`, Swift shim) | `instance`, `live` | same as PHP; Q611's "community" label applies if the conformance run fails |

---

## 2. The app-building surface (the contract)

Names below are the **facade names** in `snake_case`, as the proto annotations spell them. §3 gives each language's casing.

**Columns.**
- **Retry:** D610's class. `safe` means `NO_SIDE_EFFECTS`, `IDEMPOTENT` or `retry_safe`. `keyed` means the request carries an idempotency key. `never` means neither.
- **Layer:** `gen` means the call is a proto method behind a facade annotation. `hand` means it is written once per language over `gen` calls.

### 2.1 Client and configuration (hand, runtime)

| Item | Contract |
|---|---|
| Construction | `Loams(endpoint, api_key=…, token_source=…, namespace=…, timeout=…, session_consistency=false, max_retries=3, transport=…)`. `Loams.from_env()` reads `LOAMS_ENDPOINT`, `LOAMS_API_KEY` or `LOAMS_TOKEN`, and `LOAMS_NAMESPACE` (R1) |
| API keys | The `ApiKey(key)` token source (R1). Keys are managed through `admin.keys.create`, `list` and `revoke` (`loams.admin.v1.KeyService`, already annotated). The scenario authenticates with a key in `Authorization: Bearer` |
| Default namespace (**new R11**) | The client's `namespace` fills the `namespace` field of any request that has one and leaves it empty. An explicit argument always wins. No default and no argument raises `invalid_argument` client-side, before any network call |
| Timeouts (**new R12**) | A client default (30 s for unary calls, none for streams) and a per-call override, sent as `connect-timeout-ms` on Connect and `grpc-timeout` on gRPC. When a call runs out of time, the client raises `DeadlineExceeded` with reason `client_deadline_exceeded` and never retries, because the budget is spent |
| Consistency tokens (**R4 amended**, P0e) | `ConsistencyToken` value type: `parse`, `to_string`, `merge` (max offset per `(stream, partition)`), equality. With `session_consistency=true`, every `consistency_token`, `token` or `read_token` in a response, and the `loams-consistency-token` header, merges into the session. Every read whose request has a `consistency` field gets `at_least = session` when the caller left `consistency` unset |
| Errors, retries, pagination, streams | R2, R3, R6, R7 and R8 unchanged. The app scenario checks them on app RPCs: one `not_found`, one `invalid_argument` with `field_violations`, one keyed retry |

### 2.2 `namespaces` and `collections` (gen; the schema builder is hand)

| Call | RPC | Retry | Notes |
|---|---|---|---|
| `namespaces.create` | `NamespaceService/CreateNamespace` | safe (`retry_safe`; `already_exists` on repeat) | `FacadeOptions.module = "namespaces"` |
| `namespaces.list` / `list_all` | `NamespaceService/ListNamespaces` | safe | `pagination: "namespaces:next_page_token"` |
| `collections.create` | `CollectionService/CreateCollection` | safe (`retry_safe`, idempotent by name and request) | `schema` is a `Struct`, built by `Schema` (§2.6) |
| `collections.list` / `list_all` | `ListCollections` | safe | `pagination: "collections:next_page_token"` |
| `collections.get` | `GetCollection` | safe | resolves aliases |
| `collections.drop` | `DropCollection` | safe (`retry_safe`) | |
| `collections.add_fields` | `AddFields` | safe (`retry_safe`, additive) | |
| `collections.list_versions` | `ListVersions` | safe | |
| `collections.update_aliases` | `UpdateAliases` | safe (`retry_safe`) | |
| `collections.scan`, `set_hot`, `warm` | `Scan`, `SetHot`, `WarmCollection` | never, safe, safe | in the facade, but not in the app scenario |

### 2.3 `documents` (gen, plus hand-written conveniences)

| Call | RPC / layer | Retry | Notes |
|---|---|---|---|
| `documents.write` | `DocumentService/WriteDocuments` (gen) | keyed (field 5) | raw ops; returns `token` |
| `documents.upsert(collection, docs)` | hand, over `write` | keyed | each doc is `{id, source, vectors?}` |
| `documents.delete(collection, ids)` | hand, over `write` | keyed | |
| `documents.patch(collection, id, set=…, unset=…)` | hand, over `write` | keyed | `Patch` with `mode` and `delete_keys` |
| `documents.write_batch(collection, iterable, chunk_rows=10_000, chunk_bytes=4 MiB)` (**new R13**) | hand | keyed per chunk | One chunk in flight at a time. Each chunk gets its own idempotency key, reused on that chunk's retries. Returns the merged token. Stops at the first failed chunk and raises a `BatchError` that reports how many chunks were applied (§44 §7.5) |
| `documents.get` | `GetDocuments` (gen) | safe | `select`, `consistency` |
| `documents.scroll` / `scroll_all` | `ScrollDocuments` (gen) plus a hand-written iterator | safe | The cursor is a `DocumentId` (`next`), not an AIP-158 string, so `scroll_all` is hand-written. It loops until `next` is unset |
| `documents.count` | `CountDocuments` (gen) | safe | |
| `documents.delete_by_filter`, `patch_by_filter` | gen | never (`allow_partial` and a cursor make them resumable, not idempotent) | |

Filters on the document RPCs take the same `Filter` builder as search. P0a adds a typed `Query where` field (§5, P0a.3), so no SDK needs to convert IR to JSON by hand.

### 2.4 `search` and `vector` (gen, plus the search builder)

| Call | RPC | Retry | Notes |
|---|---|---|---|
| `search.query(request)` | `QueryService/Search` | safe | The full IR: `retrievers`, `fusion`, `filter`, `sort`, `limit`, `search_after`, `score_threshold`, `select`, `highlight`, `group_by`, `aggregations`, `expand_spec`, `rerank_spec` |
| `vector.search(collection, vector, field=…, limit=…, filter=…)` | the same RPC, second facade entry (Q613) | safe | Its required arguments are the vector shape |
| `search.text(collection, text, fields=…)`, `search.hybrid(collection, text, vector, fusion=…)` | hand shorthands over `query` | safe | Each shorthand equals one builder corpus case |
| `search.query_all(…)` | hand iterator | safe | Follows `search_after` from the last hit's `sort_values` until fewer than `limit` hits come back. It requires a `sort` with a tiebreaker (`pk`) and raises `invalid_argument` client-side otherwise |

**The search builder** (`Search`), the same verbs in every language, each a pure function onto `SearchRequest`:
- `collection`.
- `text(query | match | multi_match)` → a `TextQuery` retriever.
- `vector(field, values, k, ann params)` → a `VectorQuery` retriever.
- `sparse(field, terms)` → a `SparseQuery` retriever.
- `fuse(rrf(k=60) | dbsf() | weighted(weights…))`; `weighted` requires one weight per retriever.
- `filter(Filter)`.
- `sort(score | pk | field(name, asc/desc, missing))`.
- `limit`, `after(sort_values)`, `min_score`.
- `select(include=[…], exclude=[…])`.
- `highlight(field, pre, post, fragment_size, fragments)`.
- `group_by(field, size, limit)`.
- `expand(graph, from_field, hops, direction, edge_types, limit_per_seed, limit)`, which is graph expansion and fills `expand_spec`.
- `consistency(strong | eventual | at_least(token))`.

**The filter builder** (`Filter`), onto the `Query` oneof:
- `all()`, `none()`;
- `eq` (term), `in` (terms), `range(gt, gte, lt, lte)`;
- `exists`, `is_null`, `is_empty`;
- `prefix`, `wildcard`, `fuzzy`, `match`, `phrase`, `query_string`, `ids`;
- `and` (must), `or` (should, with `minimum_should_match`), `not` (must_not), `filter` (non-scoring);
- `boost`, `constant_score`.

Languages with operator overloading may also offer `&`, `|` and `~`; that sugar is optional.

### 2.5 `sql`, `streams` and `graph`

| Call | RPC | Retry | Notes |
|---|---|---|---|
| `sql.query(sql, params=[…], max_rows=…, consistency=…)` | `loams.sql.v1.SqlService/Query` (P0b) | safe | Positional `$1…$n` params as `google.protobuf.Value`. Returns `columns[]` and `rows[]`. The SDK adds `rows_as_dicts()` / `records()` (hand) |
| `sql.query_arrow` | `SqlService/QueryArrow` (P0b) | safe | Only in languages with an Arrow IPC reader: TS (`apache-arrow`), Python (`pyarrow` extra), Go, Rust, Java, C#, C++. Elsewhere it returns the bytes, documented as Arrow IPC |
| `streams.create`, `get`, `list` / `list_all` | `loams.stream.v1.StreamService/{CreateStream,DescribeStream,ListStreams}` (P0c) | safe, safe, safe | |
| `streams.produce(stream, partition, records, producer_id=…, producer_seq=…)` | `Produce` | never without `producer_id`; keyed with it | Records are `bytes`, each with an optional `key` (SDKP-25). The SDK takes bytes, strings or JSON values and encodes JSON as UTF-8 |
| `streams.fetch(stream, partition, offset, max_records, max_bytes)` | `Fetch` | safe | |
| `streams.produce_events`, `fetch_events` | `ProduceCloudEvents`, `FetchCloudEvents` | as `produce`, as `fetch` | |
| `streams.subscribe(stream, partition, from_offset=…)` (**new R14**) | hand, over `Fetch` | safe | The native async iteration of §44 §7.1. The next offset is the last offset + 1. An empty fetch backs off 100 ms ×2 up to 1 s, or long-polls with `max_wait_ms` when the server offers it (P0c). The iterator exposes `offset` so a caller can resume. Cancelling the iterator stops the loop |
| `links.create`, `get`, `list` | `loams.link.v1.LinkService` (API2 Task 3) | safe | **Second tier.** In the facade when Task 3 lands, but not in the app scenario |
| `graph.create`, `get`, `list`, `update`, `delete`, `schema` | `GraphAdminService/{CreateGraph,GetGraph,ListGraphs,UpdateGraph,DeleteGraph,GetSchema}` | keyed, safe, safe, never, never, safe | `unstable` (GR1e); SDKs mark the module experimental. `delete` returns an `Operation` |
| `graph.query(graph, statement, parameters={…}, read_only=…, consistency=…)` | `GraphService/Execute` | read-only: safe; otherwise keyed (field 10) | GQL is the default `language` |
| `graph.query_batch`, `query_stream`, `explain` | `ExecuteBatch`, `ExecuteStream` (server stream), `Explain` | keyed, safe if read-only, safe | |

### 2.6 The schema builder (hand)

`Schema` produces the `CreateCollectionRequest.schema` `Struct`, in the shape `connect_collections.rs`'s `kb_schema()` uses:
- `text(name, analyzer="standard", positions=true)`, `keyword(name)`;
- `i64`, `f64`, `bool`, `timestamp`;
- `vector(name, dim, distance="cosine")`, `sparse_vector(name)`;
- `dynamic("ignore" | "strict" | "map")`, `max_fields(n)`.

Each field takes `indexed`, `fast` and `source_path` keyword overrides. The builder corpus pins the output.

### 2.7 Live, tables, durable

- **`live`, `live.admin` and `tables`** stay as they are. The app scenario does not cover them; their own fixtures do.
- **Durable is out.** API2-Q3 ruled that there is no `loams.durable.v1` and that the Resonate HTTP protocol (`POST /durable/`) is the durable API, so no SDK gets a `durable` module. Each SDK README links the Resonate SDK for that language with `LOAMS_ENDPOINT/durable/` as its URL.

### 2.8 The collection handle (hand)

`loams.collection("notes", namespace=…)` returns a handle with these calls, each delegating to the module call with `collection` (and `namespace`) filled in:
- `upsert`, `get`, `delete`, `patch`, `write_batch`;
- `count`, `scroll_all`;
- `search(builder)` and `vector_search`;
- `drop`, `info`.

The handle holds no state of its own.

---

## 3. Idiomatic naming

§44 §7.1's rules, shown on the facade calls `search.query` and `documents.write_batch`, the search builder, and `streams.subscribe`:

| Language | Module access | Call | Builder | Server stream / subscribe |
|---|---|---|---|---|
| TypeScript | `loams.search` | `loams.search.query({...})`, `loams.vector.search`, `loams.documents.writeBatch` | `Search.on("notes").text("refund").vector("embedding", v).fuse(rrf())` | `for await (const r of loams.streams.subscribe(...))` |
| Python | `loams.search` | `loams.search.query(...)`, `loams.documents.write_batch` | `Search("notes").text("refund").vector("embedding", v).fuse(rrf())` | `for r in` / `async for r in loams.streams.subscribe(...)` |
| Go | `client.Search` | `client.Search.Query(ctx, req)`, `client.Documents.WriteBatch` | `loams.NewSearch("notes").Text("refund").Fuse(loams.RRF(60))` | `sub := client.Streams.Subscribe(ctx, ...); for sub.Next() { sub.Record() }` |
| Rust | `client.search()` | `client.search().query(req).await`, `client.documents().write_batch(...)` | `Search::on("notes").text("refund").fuse(Fusion::rrf(60))` | `impl Stream<Item = Result<Record>>` |
| C# | `client.Search` | `await client.Search.QueryAsync(req)`, `WriteBatchAsync` | `Search.On("notes").Text("refund").Fuse(Fusion.Rrf(60))` | `await foreach (var r in client.Streams.SubscribeAsync(...))` |
| C++ | `client.search()` | `client.search().query(req)` returning `Result<T>` | `loams::Search::on("notes").text("refund").fuse(loams::rrf(60))` | `auto sub = client.streams().subscribe(...); while (auto r = sub.next())` |
| Swift | `client.search` | `try await client.search.query(req)`, `writeBatch` | `Search.on("notes").text("refund").fuse(.rrf(k: 60))` | `for try await r in client.streams.subscribe(...)` |
| Java | `client.search()` | `client.search().query(req)`, `writeBatch` | `Search.on("notes").text("refund").fuse(Fusion.rrf(60))` | `Iterator<Record>` from `subscribe(...)` |
| Kotlin | `client.search` | `client.search.query(req)` (suspend), `writeBatch` | DSL: `search("notes") { text("refund"); fuse(rrf(60)) }` | `Flow<Record>` |
| Ruby | `client.search` | `client.search.query(**kw)`, `write_batch` | `Loams::Search.on("notes").text("refund").fuse(Loams::Fusion.rrf(60))` | `Enumerator` from `subscribe` |
| PHP | `$client->search()` | `$client->search()->query([...])`, `writeBatch` | `Search::on('notes')->text('refund')->fuse(Fusion::rrf(60))` | `Generator` from `subscribe` |
| Dart | `client.search` | `await client.search.query(req)`, `writeBatch` | `Search.on('notes').text('refund').fuse(Fusion.rrf(60))` | `Stream<Record>` |
| Objective-C | `client.search` | `[client.search query:req completion:^…]`, `writeBatch:` | `[[LMSSearch on:@"notes"] text:@"refund"]` | block callback per record, plus a cancel token |

Module and call names come from the facade index. A hand-written facade may add a language alias but may not rename the index name.

---

## 4. Generated and hand-written layers

| Layer | TS, Python, Go, Rust | Swift, Kotlin, Java, C# (Q604: generate) | Dart, Ruby, PHP, C++, ObjC (Q604: hand) |
|---|---|---|---|
| Messages and stubs | `buf generate` (pinned) | `buf generate` (pinned) | `buf generate` (C++, ObjC, PHP, Dart); Ruby reads the descriptor set |
| Facade (module, call, retry, pagination) | `protoc-gen-loams-facade` (exists) | **Hand-written today.** Generating them is API2 Task 14's Q604 ruling, which SDKP does not block on. SDKP writes them by hand, checked by `<lang>_facade_matches_index` (Kotlin reads the descriptor set) | Hand-written (Ruby: descriptor-driven), checked by `<lang>_facade_matches_index` |
| Runtime R1–R14 | hand, once per language | hand | hand |
| Builders (`Schema`, `Filter`, `Search`), handle, `write_batch`, `subscribe`, `scroll_all`, `query_all` | hand, checked against the builder corpus | hand | hand |

**One rule for every layer:** if a fact can be read from the protos, it is not hand-typed. That covers RPC paths, retry classes, pagination fields and idempotency-key fields. Hand-written facades load them from `facade-index.json` at build time, or assert them against it in a test.

---

## 5. Tasks

### Execution order and launching

The P0 tasks touch protos, the server and the corpus; the language tasks touch only `sdks/<lang>/`, plus one hunk of that language's CI job. Launch all five P0 tasks and all thirteen language tasks **at once**. Each language task has three parts:

- **Part A** (no dependency):
  - the runtime additions R11–R14 and the R4 merge;
  - the `Schema`, `Filter` and `Search` builders;
  - the collection handle.

  Tests use local stubs and the spec in §2, then switch to the corpora when P0d merges.
- **Part B** (after **P0a** and **P0d**): the `namespaces`, `collections`, `documents`, `search`, `vector` and `graph` modules, and the `app_collections`, `app_search` and `app_graph` scenarios.
- **Part C** (after **P0b** and **P0c**): `sql` and `streams`, and the `app_sql` and `app_streams` scenarios.

A language agent opens one PR per part, or one PR if the P0 work has merged by then.

Order of merge:
1. P0a and P0d first; they are the critical path.
2. P0e, at any time.
3. P0b and P0c, independently.
4. Then S1, after all thirteen languages are green.

### P0a — Facade annotations on `loams.collection.v1` and `loams.graph.v1`, the facade index, descriptor sets

**Files:**
- Modify:
  - `proto/loams/collection/v1/{collection,document,query}.proto`;
  - `proto/loams/graph/v1/graph.proto`;
  - `crates/loams-facade-gen/src/{lib.rs,bin/protoc-gen-loams-facade.rs}` (a new `lang=index` renderer);
  - `crates/loams-facade-gen/tests/golden*`;
  - `scripts/sdk/gen.sh` (an `index` target and a `descriptors` target);
  - the regenerated outputs: `sdks/typescript/packages/client/src/gen/facade.ts`, `sdks/python/src/loams/_gen/facade.py`, the Go `gen/facade`, `sdks/rust/src/facade.rs`, and `@loams/proto` if its generated files change.
- Create:
  - `sdks/fixtures/facade-index.json` (API2's shape, §"The facade index");
  - `sdks/ruby/gen/loams-descriptor.binpb` and `sdks/kotlin/gen/loams-descriptor.binpb`, regenerated from **every SDK package** (all of `PROTO_PACKAGES`).

**Steps:**
1. Annotate every method of §2.2–§2.5 with `option (loams.options.v1.facade) = {…}`, using the names, `module` overrides (`namespaces`), `retry_safe` and `pagination` given in those tables. `QueryService/Search` carries two entries: `{module: "search", name: "query"}` and `{module: "vector", name: "search"}`. `GraphAdminService` methods go into module `graph` with the names of §2.5.
2. Typed filters on the document RPCs (additive, wire-compatible). Add `Query where = 9`, or the next free number, to `ScrollDocumentsRequest`, `CountDocumentsRequest`, `DeleteByFilterRequest` and `PatchByFilterRequest`.
   - The handler maps it through `connect_query_filters.rs`, the same path `SearchRequest.filter` takes.
   - Setting both `filter` and `where` is `invalid_argument` with reason `filter_conflict`, registered in `docs/api/reasons.md`.
   - Tests: `scroll_where_matches_filter_value_rpc`, `count_where_matches_filter_value_rpc`, `where_and_filter_conflict_refused_rpc` in `crates/loams/tests/connect_documents.rs`.
3. Write the `lang=index` renderer and commit `facade-index.json`. Test: `golden_index` in `crates/loams-facade-gen/tests`.
4. Regenerate the four generated facades and both descriptor sets. The drift steps in the TS, Rust, Python and Go CI jobs must pass. Add a drift check for `facade-index.json` and the descriptor sets to the `sdk-typescript` job, which already runs `gen.sh`.
5. `buf lint` and `buf breaking` must pass. Options are additive.

**Acceptance:** `cargo test -j 4 -p loams-facade-gen`; `cargo test -j 4 -p loams --test connect_documents`; every `sdk-*` job green with no hand edits outside the regenerated files.

### P0b — `loams.sql.v1` (API2 Task 1, with parameters)

API2 Task 1 as written, plus three points:
- `QueryRequest.params` (`repeated google.protobuf.Value`) binds `$1…$n` through DataFusion's parameter values. A count mismatch is `invalid_argument`, reason `sql_param_count`.
- Annotate `query` and `query_arrow` per §2.5.
- Record the `app_sql` scenario (P0d's format).

Tests: API2 Task 1's list, plus `query_params_bind_rpc` and `query_param_count_mismatch_refused_rpc`.

### P0c — `loams.stream.v1` on the main port (API2 Task 2), and `subscribe`'s server half

API2 Task 2 as written (including `ListStreams`), plus three points:
- `FetchRequest.max_wait_ms` (optional, capped at 30 000), a long poll that answers as soon as a record past `offset` is durable, or empty at the deadline. Implement it only if the log already exposes an append notification; otherwise leave the field reserved and record a ruling.
- Annotate per §2.5.
- Record the `app_streams` scenario.

Tests: API2 Task 2's list, plus `fetch_long_poll_returns_on_append` if `max_wait_ms` is implemented. API2 Task 3 (`loams.link.v1`) is not needed for parity and is not part of P0c.

### P0d — The app conformance corpus and runner

**Files:**
- Modify:
  - `sdks/conformance/record-fixtures.mjs` (a scenario mode);
  - `sdks/conformance/verify-corpus.mjs`;
  - `sdks/conformance/check-languages.mjs` (`--suite app`);
  - `sdks/conformance/run.sh` and `run-all.sh`;
  - `sdks/fixtures/manifest.json` and `sdks/fixtures/README.md`;
  - `docs/sdk/fixtures.md`.
- Create:
  - `sdks/fixtures/recorded/app/*.json`;
  - `sdks/fixtures/builders/*.json`;
  - `sdks/fixtures/tokens.json`;
  - `sdks/fixtures/app-scenario.md` (the steps below, in prose, for agents).

**The app scenario.** A notes app, recorded from `loams dev --listen 127.0.0.1:0 --no-live` with an API key from `admin.keys.create`. It has five sub-scenarios, so that each can be required on its own; P0d records the first three, and P0b and P0c record theirs. Each step is selected by `loams-fixture-step: <n>` (the fixture server already does this). The data is fixed: five notes with `title`, `body`, `tags` (keyword), `n` (i64) and a 4-dimensional `embedding`.

| Sub-scenario | Steps |
|---|---|
| `app_collections` | 1. `instance.get_instance` (feature detection) 2. `namespaces.create("app")`, then repeat → `already_exists` 3. `collections.create("notes", Schema…)` 4. `collections.get` 5. `collections.list` with `page_size=1`, through `list_all` (2 pages, with a second collection `scratch`) 6. `documents.upsert` of 5 notes with an idempotency key, then the same key again → the same answer (R3) 7. `documents.get` with `at_least` = the token from step 6 (read-your-writes, and the session store mode) 8. `documents.patch`, then `documents.delete` of one id 9. `documents.count` with `where` 10. `documents.scroll_all` with `limit=2` (3 pages) 11. `collections.get("missing")` → `not_found` 12. `collections.drop("scratch")` |
| `app_search` | 1. `search.query`: text `match` on `body`, `filter` `tags eq "policy"`, `highlight body`, `select [title]`, `limit 2`, sorted by score then `pk` 2. the next page through `search_after` (`query_all`) 3. `vector.search` on `embedding` 4. hybrid text plus vector with `rrf(k=60)` 5. the same with `dbsf` 6. the same with `weighted([0.7, 0.3])` 7. `weighted` with the wrong number of weights → `invalid_argument` with a field violation |
| `app_graph` | 1. `graph.create("notes_graph")` (skipped with a recorded `feature_not_in_variant` if the build lacks `graph`) 2. `graph.query` GQL `INSERT` of note and tag vertices and `TAGGED` edges, with `parameters`, keyed 3. `graph.query` `MATCH … RETURN` with `read_only` 4. `search.query` with `expand(graph="notes_graph", from_field="id", hops=1)` 5. `graph.delete` |
| `app_sql` (P0b) | 1. `sql.query("SELECT tags, count(*) AS c FROM notes WHERE n > $1 GROUP BY tags ORDER BY tags", [1])` 2. a param count mismatch → `invalid_argument` |
| `app_streams` (P0c) | 1. `streams.create("events", partitions=1)` 2. `streams.produce` of 3 JSON records with `producer_id` and `producer_seq` 3. the same `producer_seq` again → deduplicated 4. `streams.fetch(offset=0)` 5. `streams.subscribe(from_offset=1)` yields 2 records, then the iterator is cancelled 6. `streams.list` |

**Volatile fields** (masked by `--drift`): tokens, `read_token`, `consistency_token`, ULIDs and ids the server generates, timestamps and `performance`. Scores are **not** volatile; a change to them is a ranking change and must be re-recorded on purpose.

**The builder corpus** (`sdks/fixtures/builders/*.json`). Each case has:
- `name`;
- `spec`: the builder calls, in the neutral notation of §2.4 and §2.6;
- `expect`: the request's proto3 JSON.

The cases:
- `schema_notes`;
- `filter_term`, `filter_bool_nested`, `filter_range_open`, `filter_ids`;
- `search_text_filter_highlight_select`, `search_vector_ann_params`;
- `search_hybrid_rrf`, `search_hybrid_dbsf`, `search_hybrid_weighted`;
- `search_after_page`, `search_expand`.

`verify-corpus.mjs` checks that every `expect` decodes as the named message (with protobuf-es) and that each `search_*` case's `expect` equals the request body of the matching recorded app step, so the builders and the scenario cannot drift apart.

**The token corpus** (`sdks/fixtures/tokens.json`): parse, print and merge cases from `crates/loams-collection/src/token.rs`:
- disjoint partitions;
- the same partition, keeping the max;
- normalisation order;
- malformed tokens, which must raise `invalid_argument` client-side.

`crates/loams-collection` gets a test, `token_corpus_matches`, that runs the same file, so the server and the SDKs agree.

**The runner.** `check-languages.mjs --check <lang> --suite app` fails unless `<lang>`'s report has passed every `app_*` fixture that exists in the manifest, plus the builder and token corpora. App fixtures are `required: false` in `manifest.json` until S1 flips them, so a language that has not finished cannot break another language's job.

**Acceptance:** `node sdks/conformance/verify-corpus.mjs`; `sdks/conformance/run-all.sh`; `sdks/conformance/run.sh --drift` (after one `loams dev` build, `-j 4`).

### P0e — Runtime contract R4 amendment and R11–R14

**Files:**
- Modify `docs/sdk/runtime-contract.md`:
  - R4: the `v1:` encoding, `merge`, and the session store rules of §2.1;
  - new clauses R11 (default namespace), R12 (timeouts), R13 (`write_batch`) and R14 (`subscribe`). Each names the test that pins it: `<lang>_default_namespace`, `<lang>_timeout_deadline`, `<lang>_write_batch_chunks`, `<lang>_streams_subscribe_resumes`.
- Modify `docs/api/reasons.md`: `client_deadline_exceeded` and `batch_partially_applied` (client-side reasons, marked as such), plus `filter_conflict` and `sql_param_count` if P0a and P0b have not added them yet.
- Modify the reason registries that the CI checks per SDK (`check-sdk-jobs.py` and the per-language reason lists): only the new rows.

**Acceptance:** the docs lint and the reason-registry tests pass.

### L1–L13 — One task per language

**Every language task has the same shape** (substitute `<lang>` and the paths of §1.2).

**Parts and their tests.** A test that does not exist yet must be added under exactly this name (the `<lang>_` prefix and the snake_case name are what `check-languages.mjs` looks for in the language's report):

| Part | Deliverable | Tests (all in the language's own test suite) |
|---|---|---|
| A | R4 merge plus the session store; R11 default namespace; R12 timeouts; R13 `write_batch`; R14 `subscribe` (against a stub `Fetch`) | `<lang>_consistency_token_merge` (the token corpus), `<lang>_session_read_your_writes`, `<lang>_default_namespace`, `<lang>_timeout_deadline`, `<lang>_write_batch_chunks`, `<lang>_streams_subscribe_resumes` |
| A | The `Schema`, `Filter` and `Search` builders; the collection handle; `scroll_all`; `query_all` | `<lang>_builders_match_corpus` (every builder case), `<lang>_collection_handle_delegates`, `<lang>_search_query_all_requires_tiebreak` |
| B | The modules `namespaces`, `collections`, `documents`, `search`, `vector` and `graph` (generated, descriptor-driven, or hand-written against the index); `auth` and `admin.keys` where §1.2 says they are missing | `<lang>_facade_matches_index` (hand-written and descriptor-driven facades only), `<lang>_app_collections`, `<lang>_app_search`, `<lang>_app_graph` |
| C | The modules `sql` and `streams` | `<lang>_app_sql`, `<lang>_app_streams` |
| D | The README "Build an app" section (the scenario as a runnable example under `sdks/<lang>/examples/notes`), and the module table in `sdks/README.md` (one row, this language's only) | The example compiles or type-checks in CI; no server is needed |

**CI** (one hunk in `.github/workflows/ci.yml`, inside this language's own `sdk-<lang>` job only):
- Add `node sdks/conformance/check-languages.mjs --check <lang> --suite app` after the existing conformance step.
- TypeScript also runs the app suite against the live server in its existing `run.sh typescript` step.
- Python, Go and Rust add `./sdks/conformance/run.sh <lang>` to the **nightly** workflow, not to PR CI (API2-Q2).

**Acceptance for every language:**
1. The language's `sdk-<lang>` CI job is green, with the `--suite app` step passing. All five sub-scenarios, the builder corpus and the token corpus pass through the **public facade**, not the stubs.
2. The existing required fixtures still pass at 100%.
3. No hand-written RPC path, retry class or pagination field disagrees with `facade-index.json`.
4. `CI required` is green on the PR.

**Per-language specifics:**

| Task | Language | Files to add or change (beyond §1.2's paths) | Notes |
|---|---|---|---|
| **L1** | TypeScript | `packages/client/src/{builders/{schema,filter,search}.ts,handle.ts,runtime/{consistency,timeout,batch,subscribe}.ts}`, `loams.ts` (module getters for the new facade modules), `test/app-*.test.ts`, `test/builders.test.ts`, `test/tokens.test.ts` | **The reference.** It must also run in browsers (R10): no Node-only API in `builders/` or `runtime/`. `query_arrow` behind an optional `apache-arrow` peer dependency |
| **L2** | Python | `src/loams/{builders.py,handle.py,_runtime/{consistency,timeout,batch,subscribe}.py}`, the same in `aio.py`, `tests/test_app_*.py` | Sync and async parity is part of acceptance (each test runs both clients). `mypy --strict`. `query_arrow` behind the `loams[arrow]` extra |
| **L3** | Go | `builders.go`, `handle.go`, `batch.go`, `subscribe.go`, `timeout.go`, and `modules.go` (wire the generated `auth`, `admin`, app modules); `app_test.go`, `builders_test.go` | Context-first; `subscribe` returns `*Subscription` with `Next`, `Record`, `Err` and `Offset`; `go vet` |
| **L4** | Rust | `src/{builders/,handle.rs,batch.rs,subscribe.rs,timeout.rs}`, `tests/app.rs`, `tests/builders.rs` | `subscribe` is a `futures::Stream`. Build with `-j 4` (shared target; see the memory note on target collisions) |
| **L5** | C# | `src/Loams/{Builders/,CollectionHandle.cs,Batch.cs,Subscribe.cs,Timeout.cs}`, `Modules.cs`, `Gen/Collection.cs`, `Gen/Graph.cs`, `Gen/Sql.cs`, `Gen/Stream.cs`; `tests/` | `IAsyncEnumerable` for `subscribe`; `CancellationToken` on every call |
| **L6** | C++ | `include/loams/{builders.hpp,handle.hpp,batch.hpp,subscribe.hpp}`, `src/…`, `bindings.cpp`, `facade.hpp`; `tests/` | C++17; `Result<T>`; `subscribe` returns a pull reader with `cancel()` |
| **L7** | Swift | `Sources/Loams/{Builders/,CollectionHandle.swift,Batch.swift,Subscribe.swift}`, `Modules.swift`, `Messages.swift`; `Tests/` | `AsyncThrowingStream`; `Sendable` builders |
| **L8** | Java | `src/dev/loams/{builders/,CollectionHandle.java,Batch.java,Subscription.java}`, `Modules.java`, `Catalogue.java`; tests | Java 17; `Iterator` plus `AutoCloseable` for `subscribe` |
| **L9** | Kotlin | `src/main/kotlin/dev/loams/{Builders.kt,SearchDsl.kt,CollectionHandle.kt,Batch.kt,Subscribe.kt}`, typed wrappers per app module; tests | Modules come from the descriptor set regenerated in P0a; the wrappers add types. `Flow` for `subscribe` |
| **L10** | Ruby | `lib/loams/{builders.rb,collection_handle.rb,batch.rb,subscribe.rb,timeout.rb}`; `test/` | Modules come from the descriptor set (P0a). Both transports: the app fixtures carry no `transport:grpc-only`, so the Connect-unary transport must pass all except `graph.query_stream`, which is marked grpc-only |
| **L11** | PHP | `src/{Modules/,Builders/,CollectionHandle.php,Batch.php,Subscribe.php,Timeout.php}`; `tests/` | Write the module layer from `facade-index.json` (a generated PHP array is allowed; commit it and drift-check it). PSR-18 Connect-unary must pass, as for Ruby |
| **L12** | Dart | `lib/src/{modules/,builders/,collection_handle.dart,batch.dart,subscribe.dart}`; `test/` | `Stream<Record>`; the module layer as for PHP |
| **L13** | Objective-C | `objc/include/LMS*.h` plus `.m` for the modules and builders, the Swift shim in `Sources/LoamsObjC`; tests in the existing SwiftPM suite | Block callbacks and cancel tokens. If the conformance run cannot pass, label it "community" per Q611 and record that as a ruling; do not drop the task |

### S1 — Flip the app fixtures to required

After L1–L13 are all green:
- set `required: true` on every `app_*` fixture in `manifest.json`;
- remove `--suite app` from the jobs (the plain check now covers it);
- update `sdks/README.md` (the module matrix) and the plans index status.

**Test:** `check-languages.mjs --check <lang>` passes for all thirteen languages.

---

## 6. Rulings made in writing this plan (owner-approved 2026-10-19, SDKP-10)

| # | Question | Default | Why |
|---|---|---|---|
| SDKP-1 | Durable in the SDKs | No module (API2-Q3 stands). The README points to the Resonate SDKs | One durable API |
| SDKP-2 | Subscribe: a server stream or client polling | Client loop over unary `Fetch`, with an optional long poll | Works on Connect-unary (Ruby, PHP) and through every proxy; no new streaming RPC to keep resumable |
| SDKP-3 | Filters on the document RPCs | An additive typed `Query where` beside the `Value filter` | One `Filter` builder for all RPCs, with no per-language IR-to-JSON converter |
| SDKP-4 | Hand-written facades for Swift, Kotlin, Java and C# (Q604 says generate) | Hand-write now, checked by the index; generating them stays API2 Task 14 | Keeps SDKP off the generator's critical path; the index test makes the later switch mechanical |
| SDKP-5 | Gating | `--suite app` per language job until all thirteen pass, then S1 | No language's red blocks another's PR |
| SDKP-6 | Links | Second tier: in the facade when API2 Task 3 lands, not in the scenario | Not needed to build an app |
| SDKP-7 | `search` and `vector` | Two facade entries on one RPC (Q613) | Already ruled |

**Owner questions:** both were answered on 2026-10-19 (SDKP-8 and SDKP-9 below), so none is open.

## Rulings made during execution

| # | Ruling | Date and source |
|---|---|---|
| SDKP-8 | **Durable stays out of the SDKs.** API2-Q3 stands: no `loams.durable.v1` and no `durable` module, and each README points to that language's Resonate SDK (SDKP-1) | Owner, 2026-10-19 |
| SDKP-9 | **Objective-C is labelled "community" if it cannot pass the app suite.** L13 is not dropped: it ships the generated sources and the SwiftPM target, and records that label as a ruling here (Q611) | Owner, 2026-10-19 |
| SDKP-10 | **The defaults SDKP-1 to SDKP-7 stand as written.** They are no longer defaults awaiting review | Owner, 2026-10-19 |
| SDKP-11 | **R11 and R4's request rules apply to the app packages only:** `loams.collection.v1`, `loams.graph.v1`, `loams.sql.v1` and `loams.stream.v1`. A `namespace` field elsewhere is not a Loams namespace: `loams.live.v1` `CreateAppRequest.namespace` is free text naming an app's environment, and is never filled from the client default nor refused for lack of one. The same set decides which requests get `consistency.at_least` from the session | L1–L4 (Go agent's finding), coordinator, 2026-10-19 |
| SDKP-12 (P0a-1) | **`where` is a `google.protobuf.Struct` holding a `Query`'s proto3 JSON, not a typed `Query`.** `query.proto` imports `document.proto` (for `DocumentId`, `Consistency`), so `document.proto` cannot import `Query` back, and moving `Query` to its own file breaks `buf breaking` (FILE) and every generated import path. The server decodes `where` with the generated `Query` mapping and lowers it through `connect_query_filters`, the path `SearchRequest.filter` takes, so the meaning is the typed filter's; the field numbers are 9 (Scroll), 5 (Count), 8 (DeleteByFilter) and 9 (PatchByFilter). Both `filter` and `where` set is `invalid_argument`, reason `filter_conflict`. The SDK `Filter` builders already write that JSON | P0a, 2026-10-19 |
| SDKP-13 (P0a-2) | **`facade-index.json`'s `protoRev` is the `proto_rev` the SDKs declare (`v1`), not a git SHA**, so the file is a function of `proto/` alone and the drift check is reproducible. `retry` is read from the protos: `GraphAdminService/UpdateGraph` and `DeleteGraph` take an `idempotency_key`, so they are `keyed` rather than §2.5's `never` | P0a, 2026-10-19 |
| SDKP-14 (P0a-3) | **`NamespaceService`'s `ModuleOptions.name` is `namespaces`** rather than a per-method `FacadeOptions.module`, so the module keeps its own summary; the facade surface is §2.2's. A module may span services (`graph` is `GraphAdminService` and `GraphService`): the generators register every call's service and import each type from its own package (`DeleteGraph` answers `loams.operations.v1.Operation`) | P0a, 2026-10-19 |
| SDKP-15 (P0a-4) | **Engine fixes the SDKs depend on, landed with P0a.** A retriever without `k` (or `k: 0`) takes `max(10, offset + limit)` on REST and Connect, so the builder corpus needs no `k`. A primary-key `search_after` value sent back as a whole double (every Connect number is one) is the key it names, and an integer past 2^53 is answered as `{"uint": "…"}` / `{"int": "…"}` and read back the same, so `query_all` pages on numeric ids | C++/Ruby/PHP/Dart/ObjC agent's finding, coordinator, 2026-10-19 |
| SDKP-16 (P0d-1) | **The builder and token corpora were written with L1** (TypeScript, the reference) before P0d started, in P0d's names and format (`sdks/fixtures/builders/*.json` with `name`, `builder`, `message`, `spec`, `expect`; the neutral notation in that directory's README; `sdks/fixtures/tokens.json` from `crates/loams-collection/src/token.rs`'s cases). TypeScript, Python, Go and Rust pass all twelve cases | L1, 2026-10-19 |
| SDKP-17 (P0e-1) | **The session's token is used only for collection and SQL reads.** A graph's token names one graph's stream and a Graph read refuses any other, so a graph read never takes the session's token as `at_least`; a graph caller passes its own write's token explicitly. The session keeps one merged token per package, `merge` taking the max per `(stream, partition)` within it. R4 in `docs/sdk/runtime-contract.md` says so | C#/Java/Kotlin/Swift and C++/Ruby/PHP/Dart/ObjC agents' findings, coordinator, 2026-10-19 |
| SDKP-18 | **The search builder sets `k = max(10, offset + limit)` when a retriever has none**, the same default the engine now applies (SDKP-15), so a request built in any SDK means the same thing on any server version that has the fix | C#/Java/Kotlin/Swift agent, coordinator, 2026-10-19 |
| SDKP-19 | **`Schema.timestamp` maps to the engine's `date` kind**, the schema JSON's spelling for an RFC 3339 timestamp | C#/Java/Kotlin/Swift agent, coordinator, 2026-10-19 |
| SDKP-20 | **Kotlin's `streams.subscribe` returns an `Iterator`, not a `Flow`**, so it needs no coroutine dependency; §3's table is amended for Kotlin | C#/Java/Kotlin/Swift agent, coordinator, 2026-10-19 |
| SDKP-21 | **`--stream-grpc-listen` (the old tonic `StreamService`) stays until API2 Task 2** retires it; P0c serves `loams.stream.v1` on the main port beside it | C#/Java/Kotlin/Swift agent, coordinator, 2026-10-19 |
| SDKP-22 (P0d-2) | **The app scenario is recorded without an API key and from the default build.** `loams dev` registers no `KeyService` on the main port yet, so the scenario runs with auth off (keys stay covered by the admin fixtures); and the default build has no `graph` feature, so `app_graph` records the refusals (`feature_not_in_variant`, and `invalid_argument` for a search's expand stage), which an SDK must surface as typed errors. A graph-feature recording is a later re-record under the same steps. Every keyed request carries a fixed `idempotencyKey` (`app-…`) so a replay's body matches; scrolling four notes two at a time is two pages, and `search_after_page`'s `expect` is the recorded `sort_values` | P0d, 2026-10-19 |
| SDKP-23 (P0e-2) | **The five client-side reasons are registry rows** (`namespace_required`, `invalid_consistency_token`, `search_after_needs_tiebreak`, `client_deadline_exceeded`, `batch_partially_applied`), marked client-side in `docs/api/reasons.md` and listed as unproducible in `error.json`, so every SDK's typed error carries them like a server reason. `batch_partially_applied` is registered under `aborted`; the failed chunk's own error is its cause | coordinator, 2026-10-19 |
| SDKP-24 (C-1) | **Part C's facade and fixtures land before every language reaches Part C.** `loams.sql.v1` and `loams.stream.v1` get their `FacadeOptions` (`sql.query`, `sql.query_arrow`; `streams.create` (retry-safe: idempotent by name), `get`, `list` (paged), `produce` (not retried), `fetch`, `produce_events`, `fetch_events`) and `app_sql` and `app_streams` are recorded (`--source app --scenario …`) in one PR. The two Part C recordings carry `part: "C"`, and `check-languages --suite app` holds a language to one only once its suite names `<lang>_app_sql` / `<lang>_app_streams` or its report ran it, so the thirteen app suites stay green until each language's Part C PR; S1 requires them all regardless. A message whose leaf another facade package also has (`loams.sql.v1.QueryRequest` beside `loams.live.v1.QueryRequest`) is imported by the TS and Python facades under a package-prefixed alias (`SqlV1QueryRequest`) | Part C, 2026-10-20 |
| SDKP-25 | **`streams.produce` takes an optional record key in every SDK**, as `app_streams` produces keyed records (`Record.key`, bytes). The generated TS, Python, Go and Rust facades take the full `ProduceRequest`, so they carry it already | Lead, 2026-10-20 |
