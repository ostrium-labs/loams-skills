# 44 — The Unified Connect API and the Multi-Language SDKs

Status: **Proposed** · 2026-10-02. The direction is the owner's, from 2026-10-02:

> "remove the REST API; [make] a unified gRPC API for all services in a single `loams`; dependencies are prebuilt and added to it; all can be accessed as functions in `loams.vector` etc.; also add support for Ruby, PHP, etc.: all the languages gRPC supports generating clients for."

This document turns that into decisions **D600–D619** and open questions **Q600–Q614** (recorded in the [decision log](13-decision-log.md)). Everything past the quotation (the service catalogue, the migration table, the facade generator, the language matrix) is a **proposal** until the owner confirms it; the owner's "do suggested for all" (2026-10-02) means the defaults below stand unless they say otherwise. **No code is written by this document.** Plans: [API1](../plans/2026-10-02-api1-unified-connect.md), [SDK1](../plans/2026-10-02-sdk1-generation-pipeline.md), [SDK2](../plans/2026-10-02-sdk2-languages.md).

**Numbering.** Two ID spaces, kept apart on purpose. **Plain numeric IDs:** as of this decision (2026-10-02, folded into the [decision log](13-decision-log.md) by #319 on 2026-10-03) the log allocates **D1–D619 and Q1–Q614**, and those top numbers are this document's own rows, not another document's. Earlier in the range, **D460–D466 are the seven Elasticsearch gateway decisions of 2026-09-24 and stay exactly as they are** — this document adds nothing inside that block. Blocks reserved by other documents, in plain numeric form: §37 D480–D499 (the native desktop), and D500–D512 with Q500–Q511 (§18.14, the Tauri web bridge); §38 D440–D459 with Q440–Q459 (Q440–Q454 used, Q455–Q459 still unused); §39 Q460–Q479; §40 D520–D539 with Q520–Q539; §41 D540–D559 with Q540–Q559; §42 D560–D579 (D560–D578 used) and Q560–Q579 (Q560–Q572 used); §43 D580–D599 and Q580–Q599 (Q580–Q597 used). D467–D479, D513–D519 and Q512–Q519 are unassigned. **Prefixed IDs:** §39's twenty decisions are not numeric at all — they are the separate block `D-SF-1`–`D-SF-20`, and they overlap no numeric range above. **D600–D619 and Q600–Q614 are this document's**, and no other document reserves them: the highest block reserved anywhere else is §43's D580–D599 and Q580–Q599, so the two blocks do not collide. Both the log's top numbers and that reserved ceiling move with every merge, so re-read the log before reserving a block above this one; "renumber at merge if taken" applies if a later document reserves D600–D619 or Q600–Q614 first.

## 1. Summary

1. **One API.** Every application call to Loams is a Connect/gRPC RPC in a `loams.<service>.v1` package, served by the single `loams` binary on one port. Connect serves gRPC, gRPC-Web and the Connect protocol on that port; a Connect unary call is an HTTP `POST` with a JSON body, so `curl` works without a bespoke REST layer (§4).
2. **The bespoke REST APIs go.** The native REST under `/v1/namespaces/...` (M1.2), the console's OpenAPI `/api/v1/...` (§19 P9, `api/console/openapi.json`), the internal node-to-node HTTP and any other hand-written JSON endpoint are replaced by RPCs, with a migration table (§5) and one release of deprecation shims.
3. **The compatibility surfaces stay.** The Qdrant REST and gRPC APIs, the Elasticsearch REST API, Postgres wire, the Kafka gateway, Flight SQL, the Resonate HTTP protocol, MySQL (Vitess, WeSQL), Git Smart HTTP and MCP exist so **unmodified third-party clients work**. Removing them would break the drop-in promise (§6). They are adapters over the same internal services.
4. **One SDK shape in every language.** One client object with namespaced modules: `loams.vector.search(...)`, `loams.search`, `loams.sql`, `loams.streams`, `loams.tables`, `loams.graph`, `loams.live`, `loams.durable`, `loams.jobs`, `loams.repos`, `loams.systemone`, `loams.bot`, `loams.admin` and the rest (§7). The module/method surface is **generated from annotations on the protos**, so 13 languages cannot drift (D606).
5. **Every language gRPC officially supports** (13, §9): C++, C#, Dart, Go, Java, Kotlin, Node/TypeScript, Objective-C, PHP, Python, Ruby, Rust, Swift. Connect where an official Connect library exists; gRPC elsewhere. This **supersedes the earlier "Java SDKs deferred" ruling** for generated clients (D612).
6. **One toolchain, one pipeline.** `buf generate` with remote plugins per language, a thin runtime per language, a conformance suite per SDK against a shared test server, and publishing from CI with trusted publishing where the registry has it (§10, §11, issue #254).

## 2. Goals and non-goals

**Goals.** One contract; the same call in any language; curl-able; streaming where it exists; typed errors; safe retries; consistency tokens carried automatically; bulk Arrow through Flight; breaking changes caught by `buf breaking`; reference docs generated from the protos.

**Non-goals.**
- No new compatibility surface and no removal of one (D603).
- No client- or bidi-streaming in the application API (D420's rule stands: browsers and URLSession cannot do it; half-duplex works through every proxy). Bulk upload that needs it goes through Flight `DoPut` (gRPC only) or chunked unary writes (§7.5).
- No hand-written REST SDKs. M1.6's Python and TypeScript REST SDKs are retired by this work (D604, pre-release, no users to migrate).
- No billing/metering RPCs here: the meter protocol belongs to the private `loams-platform` (D220, D552) and is **not** in the open protos or SDKs.

## 3. What exists today (checked 2026-10-02 on `dev`)

| Surface | Where | Protocol |
|---|---|---|
| Native REST, 22 routes under `/v1/namespaces/{ns}/...` plus `/health` and `/ready` | `crates/loams/src/api/{mod,collections,query,sql,streams,events,hot}.rs` | axum JSON |
| Internal node HTTP: forwarded reads, node stats, hot status and warm | `crates/loams/src/api/internal.rs` | axum JSON |
| `loams.stream.v1` `StreamService` (`Produce`, `ProduceCloudEvents`) | `crates/loams-stream-grpc/proto` | gRPC (tonic-era, to move to connect-rust, D128) |
| `loams.live.v1` `LiveService` | `proto/loams/live/v1` | Connect |
| `loams.instance/devices/approvals/operations/notifications/errors.v1` | `proto/loams/*` (AP0) | Connect |
| Console `/api/v1` OpenAPI (31 paths incl. OAuth and well-known) | `api/console/openapi.json` | REST |
| SDKs: `@loams/live` | `sdks/typescript/packages/live` | Connect (protobuf-es) |
| SDKs planned: `loams-client` (Python), `@loams/client` (TS), both REST (M1.6, #198) | `docs/plans/2026-09-24-m1.6-sdks-mcp.md` | REST |
| Designed, not yet on `dev`: `loams.jobs.v1` (§26), `loams.flow.v1` (§32), `loams.repos.v1` (§36 §19.10; protos from GT1 Task 12), `loams.collab.v1`/`loams.bot.v1`/`loams.factory.v1` (§39), `loams.systemone.v1` (§40, `SystemOneService`), `loams.console.v1` (`PluginService`, AP1a) | design docs | Connect (planned) |

D101 (OpenAPI for the native REST, then a Go SDK) and M1.6 Ruling 1 ("the SDKs target only the native REST API") are **superseded** (D604); the fixture corpus idea survives as the conformance corpus (§10.4).

## 4. One API, one port (D600, D601)

**Ports.** The Loams API listens on `--listen` (default `0.0.0.0:8080`, `127.0.0.1:8080` for `loams dev`). It serves, on that single port and HTTP/2 or HTTP/1.1:

- the Connect protocol (unary over `POST`, `GET` for `NO_SIDE_EFFECTS` reads; server streams over HTTP/1.1 or 2);
- gRPC and gRPC-Web (connect-rust serves all three; D128, §37 §8.3);
- `grpc.health.v1.Health` and `grpc.reflection.v1` (reflection on in `loams dev`, off in production unless configured), so `grpcurl` and Postman work;
- the **protocol endpoints** that standards or browsers dictate and that are not an application API (D602): `/health`, `/ready` (Kubernetes probes), `/.well-known/oauth-protected-resource`, `/.well-known/oauth-authorization-server`, `/.well-known/jwks.json`, the OAuth `token`, `authorize` endpoints (RFC 6749/7636/8414/8693/9728) and the OIDC redirect start/callback. They are not "REST API": their shape is fixed by RFCs and browser navigation, and every OAuth client library expects them.

Compatibility surfaces keep **their conventional ports** (Qdrant 6333/6334, Elasticsearch 9200, Postgres 5432, Flight SQL 8082, MCP 8083) because unmodified clients default to them. Because Flight SQL and Qdrant's gRPC are gRPC services, they are **also mounted on the main port** by service name (`arrow.flight.protocol.FlightService`, `qdrant.*`), so a firewall-constrained deployment needs one port (D601). The REST-shaped compat surfaces (Qdrant REST, ES) cannot share the main port's paths without collisions and stay on their ports.

**The binary.** "A single `loams` with dependencies prebuilt" is §30 §9's variant matrix: `cli`, `standard`, `full`. Each variant serves every service in the catalogue that its engines need; a service whose engine is not in the variant answers `UNIMPLEMENTED` with `ErrorInfo.reason = feature_not_in_variant` and `metadata.variant`. The variant does not change the wire contract. `GetInstance` already reports `api_versions`; D600 adds `services[]` (package, version, available) so an SDK can feature-detect.

**curl.** With the Connect protocol, unary RPCs are plain HTTP:

```sh
# search (POST, JSON)
curl -s http://127.0.0.1:8080/loams.collection.v1.CollectionService/Search \
  -H 'content-type: application/json' -H "authorization: Bearer $LOAMS_API_KEY" \
  -d '{"namespace":"acme","collection":"docs","text":"retention policy","limit":5}'

# a read marked NO_SIDE_EFFECTS can be a cacheable GET
curl -s -G http://127.0.0.1:8080/loams.instance.v1.InstanceService/GetInstance \
  --data-urlencode 'encoding=json' --data-urlencode 'message={}'
```

Server-streaming RPCs use the Connect streaming envelope; `curl` can read them with `--no-buffer` but the docs point to `loams` CLI (`loams live watch`) and `grpcurl` for streams.

## 5. What is removed, and the migration table (D602, D605)

### 5.1 Native REST (M1.2) to `loams.*.v1`

Resource names move from URL paths into the request message (`namespace`, `collection`, `stream`, `partition`).

| Old route | New RPC | Notes |
|---|---|---|
| `POST /v1/namespaces` | `loams.collection.v1.NamespaceService/CreateNamespace` | |
| `POST /v1/namespaces/{ns}/collections` | `CollectionService/CreateCollection` | idempotent by name |
| `GET /v1/namespaces/{ns}/collections` | `CollectionService/ListCollections` | paginated (AIP-158) |
| `GET .../collections/{c}` | `CollectionService/GetCollection` | `NO_SIDE_EFFECTS`; aliases resolve |
| `DELETE .../collections/{c}` | `CollectionService/DropCollection` | `{dropped}` |
| `POST .../collections/{c}/fields` | `CollectionService/AddFields` | |
| `GET .../collections/{c}/versions` | `CollectionService/ListVersions` | |
| `POST .../collections/{c}/scan` | `CollectionService/Scan` | returns `ScanPlan` and the pin token |
| `POST /v1/namespaces/{ns}/aliases` | `CollectionService/UpdateAliases` | |
| `POST .../documents` | `DocumentService/WriteDocuments` | `idempotency_key`; returns the consistency token |
| `POST .../documents/get` | `DocumentService/GetDocuments` | |
| `POST .../documents/scroll` | `DocumentService/ScrollDocuments` | server-streaming or paginated; Task decides (API1 Task 4) |
| `POST .../documents/count` | `DocumentService/CountDocuments` | |
| `POST .../documents/delete_by_filter` | `DocumentService/DeleteByFilter` | |
| `POST .../documents/patch_by_filter` | `DocumentService/PatchByFilter` | |
| `PUT .../collections/{c}/hot`, `POST .../warm` | `CollectionService/SetHot`, `WarmCollection` | |
| `POST /v1/namespaces/{ns}/query` | `QueryService/Search` | hybrid IR (§05 §4); `loams.vector.search` and `loams.search` are facade names for it (§7.2) |
| `POST /v1/namespaces/{ns}/sql` | `loams.sql.v1.SqlService/Query` | results as rows; bulk Arrow through Flight (§7.5) |
| `POST /v1/namespaces/{ns}/streams`, `GET .../streams/{s}` | `loams.stream.v1.StreamService/CreateStream`, `DescribeStream` | |
| `POST/GET .../partitions/{p}/records` | `StreamService/Produce`, `Fetch` | `Produce` already exists on gRPC |
| `POST .../streams/{s}/events`, `GET .../partitions/{p}/events` | `StreamService/ProduceCloudEvents`, `FetchCloudEvents` | the CloudEvents HTTP binding stays on the CloudEvents ingest endpoint (compat, §6) |
| `POST /v1/namespaces/{ns}/links`, `GET .../links/{l}` | `loams.link.v1.LinkService/CreateLink`, `DescribeLink` | |
| `GET /health`, `GET /ready` | kept (probes) and `grpc.health.v1` | D602 |
| `/internal/*` (forwarded reads, node stats, hot status) | `loams.internal.v1` on the cluster listener only, never on the public port, keeping today's cluster-token authentication (mTLS when MT adds it); unauthenticated calls are refused | not in any SDK (D607) |

### 5.2 Console OpenAPI `/api/v1` to Connect (Q423 resolved)

| Old route | New RPC (service in `loams.admin.v1` unless noted) |
|---|---|
| `GET /instance` | `loams.instance.v1.InstanceService/GetInstance` (exists) |
| `POST /setup` | `AuthService/CompleteSetup` (first-run only, loopback or setup token) |
| `GET,POST,DELETE /session` | `AuthService/GetSession`, `CreateSession`, `DeleteSession` (cookie or bearer; `loams.auth.v1`) |
| `GET /auth/oidc/{provider}/start` | **kept** (browser redirect, D602); `AuthService/ListProviders` lists them |
| `GET,PATCH /org`; members; invitations | `OrgService/GetOrg`, `UpdateOrg`, `ListMembers`, `RemoveMember`, `ListInvitations`, `CreateInvitation` |
| teams | `OrgService/ListTeams`, `CreateTeam`, `GetTeam`, `AddTeamMember` |
| projects, access, environments, environment usage | `ProjectService/{List,Create,Get}Project`, `{List,Grant}Access`, `{List,Create,Get}Environment`, `GetEnvironmentUsage` |
| agents, suspend/resume, tokens, trust policies, service accounts | `AgentService/{List,Create,Get,Update}Agent`, `SuspendAgent`, `ResumeAgent`, `ListAgentTokens`, `{List,Create}TrustPolicy`, `{List,Create}ServiceAccount` |
| `DELETE /tokens/{jti}`, keys | `KeyService/RevokeToken`, `{List,Create,Revoke}Key` |
| `GET /audit` | `AuditService/ListAuditEvents`, `WatchAuditEvents` (stream) |
| `POST /oauth/consent` | `AuthService/DecideConsent` (console-internal) |
| `POST /oauth/token`, `GET /oauth/authorize`, well-known, JWKS | **kept**, protocol endpoints (D602) |

The console's `openapi-typescript` step is replaced by `@loams/proto` (AP0). `api/console/openapi.json` is deleted when API1 Task 9 lands.

### 5.3 Shims (D605)

Each removed route gets a **shim for one release**: the old path answers `308`/`410`-style with a JSON body `{"error":"moved","rpc":"<the route's RPC from the table above, e.g. loams.collection.v1.QueryService/Search>","docs":"https://loams.dev/docs/api/migrate"}` and the `Deprecation` and `Sunset` headers, behind `--legacy-rest` (off by default from the first release that has the Connect equivalents; the shim is removed in the next minor). Because the product is pre-release, shims may be dropped by owner decision (Q600): there are no external users of the M1.2 REST yet.

## 6. Compatibility surfaces that stay (D603)

State plainly: **these are not removed, and no change in this plan alters their wire format.**

| Surface | Why it stays | Adapter over |
|---|---|---|
| Qdrant REST and gRPC (6333/6334) | unmodified Qdrant clients and LangChain/LlamaIndex stores | `CollectionService`, `QueryService` |
| Elasticsearch 8 REST (9200) | elasticsearch-py, Beats, LangChain's ES store | same |
| Postgres wire (PG1, RT) | psql, ORMs, BI | collections as tables; Loams Postgres |
| Kafka gateway (§34) | Kafka clients | `StreamService` |
| Arrow Flight SQL (8082, also on the main port) | ADBC/JDBC drivers, BI, bulk ingest (D49) | SQL engine, collections, streams |
| Resonate HTTP protocol | unmodified Resonate SDKs (D1) | `loams-durable` |
| MySQL via Vitess/WeSQL (§29) | MySQL clients | router |
| Git Smart HTTP (GT2) | stock `git`, sccache, cargo mirror | Loams Git |
| MCP (8083, stateless streamable HTTP) | Claude Code, Codex and other agents (D111) | `QueryService` |
| CloudEvents HTTP/gRPC bindings | CloudEvents producers | `StreamService` |
| OAuth/OIDC/JWKS/health endpoints | RFCs and orchestrators (D602) | auth |

Rule: a compat surface may never reach storage directly; it calls the same internal service traits the Connect handlers call (overview §8). A change to a Connect service that breaks an adapter fails the adapter's conformance suite (the Qdrant, ES and ADBC clients already run in CI).

## 7. The SDK shape (D606)

### 7.1 One object, namespaced modules

```python
loams = Loams("https://acme.loams.dev", api_key=os.environ["LOAMS_API_KEY"])
hits = loams.vector.search(collection="docs", vector=v, limit=10, consistency=token)
loams.documents.write("docs", [{"_id": "1", "text": "..."}])    # returns a ConsistencyToken
for t in loams.live.watch(query): ...                            # server stream -> iterator
op = loams.durable.start("ingest", {"url": u}); op.wait()
```

```ts
const loams = new Loams({ endpoint, auth: tokenSource });
const res = await loams.vector.search({ collection: "docs", vector, limit: 10 });
for await (const t of loams.live.watch({ query })) {}
```

Language idiom rules (D606): module and method names are the proto names in the language's case (`snake_case` Python/Ruby/Rust/PHP-method-camel, `camelCase` TS/Java/Kotlin/Swift/Dart, `PascalCase` Go/C#/C++); streams are the language's native async iteration (Python iterators/async iterators, TS `AsyncIterable`, Go `*Stream` with `Receive()`, Kotlin `Flow`, Swift `AsyncSequence`, Dart `Stream`, Rust `Stream`, Java blocking `Iterator` plus a reactive adapter later, C# `IAsyncEnumerable`, Ruby `Enumerator`, PHP `Generator`, C++ reader, ObjC block callbacks).

### 7.2 The module catalogue (service to module)

| Module | Proto service(s) | Status |
|---|---|---|
| `loams.collections`, `loams.documents` | `loams.collection.v1` `CollectionService`, `DocumentService` | API1 |
| `loams.search`, `loams.vector` | `QueryService/Search`; `vector` is the dense/sparse-first facade (`search`, `upsert`, `delete`), `search` the text/hybrid/graph-expand facade; same RPC | API1 |
| `loams.sql` | `loams.sql.v1` | API1 |
| `loams.streams` | `loams.stream.v1` | API1 |
| `loams.links` | `loams.link.v1` | API1 |
| `loams.tables`, `loams.live`, `loams.live.admin` | `loams.live.v1`: `tables` wraps `Query`, `Mutate`; `live` wraps `Watch`, `ModifyQuerySet`; `live.admin` wraps `LiveAdminService` (`Deploy`, `ListDeployments`, `GetDeployment`, `ActivateDeployment`, `GetIndexStatus`, `TailLogs`; D689, LV1 Task 10). `loams.live.worker.v1` is internal: the frames between the server and its sandboxed function workers (LV1 Ruling 5), never served or in an SDK | exists (R1); the worker protocol LV1 Task 5 |
| `loams.graph` | `loams.graph.v1` (M3; §07, §48) | served with the `graph` feature (GR1 Task 5); `feature_not_in_variant` otherwise; unstable until GR1e |
| `loams.durable`, `loams.operations`, `loams.approvals` | durable engine control (start/signal/inspect) in `loams.durable.v1`; `loams.operations.v1`; `loams.approvals.v1` | partial (D1, AP0) |
| `loams.jobs` | `loams.jobs.v1` (§26) | when J1 lands |
| `loams.flow` | `loams.flow.v1` (§32) | when FL lands |
| `loams.house` | `loams.house.v1` `HouseService` (§49 §18) | served by `loams-fabric house`; the main port proxies `/loams.house.v1.*` to `[house] endpoint` and lists it as available while that endpoint is healthy, `house_not_configured` otherwise (HS1 Task 7); RPCs in HS1 Task 8; unstable until HS1 Task 38 |
| `loams.repos` | `loams.repos.v1` `RepoService`, `WorkspaceService`, `ObjectService`, `EnvService`: repositories, refs, workspaces, objects and history, sealed env (§36 §19.10, D834). The data path stays Git Smart HTTP (D603); the on-disk formats are crate-local in `loams-git`, never served (GT1 Ruling 20) | GT1: protos in Task 12; `RepoService` served with the `loams` feature `git` from Task 13, `feature_not_in_variant` otherwise; unstable until GT1 GA |
| `loams.systemone` | `loams.systemone.v1` `SystemOneService` (`Decide`, `DecideBatch`, `ListBackends`, `SelfTest`) | SO1 |
| `loams.collab`, `loams.bot`, `loams.factory` | `loams.collab.v1`, `loams.bot.v1`, `loams.factory.v1` (§39) | SF |
| `loams.instance`, `loams.devices`, `loams.notifications` | AP0 packages | exists |
| `loams.admin` (`.org`, `.projects`, `.agents`, `.keys`, `.audit`), `loams.auth` | `loams.admin.v1`, `loams.auth.v1` | API1 |
| `loams.console` | `loams.console.v1` `PluginService` | AP1a |
| `loams.postgres` | `loams.postgres.v1` `PostgresService`: projects, branches, endpoints, roles, databases, connect (§46 §4) | PG2 (protos in Task 1; served by `pg-control` behind the `loams` feature `postgres` from Task 9) |
| `loams.memory` | `loams.memory.v1` `MemoryAdminService`, `MemoryService`: stores, spaces, add, search, facts, forget, blocks, consolidation, watch (§52 §9.1, D884); `CodeMemoryService` joins it (MM1 Task 27) | MM1: protos in Task 1; served with the `loams` feature `memory` from Task 11, `memory_not_yet_available` per RPC until its task lands; no facade calls until Task 44; unstable until MM1 Task 56 |
| none: no SDK module (Q816) | `loams.agentd.v1` `StudioService`, `ProjectService`, `ThreadService`, `QueueService`, `HarnessService`, `GitService`, `DiffService`, `FileService`, `TerminalService`, `PreviewService`, `ReviewService`, `RunGroupService`, `ApprovalService`, `NotificationService`, `AttachmentService`: Loams Studio's daemon surface (§53 §6.1, D907). Served by `loams-agentd` (`loams-agentd-connect`), never by the Loams server, so it is in no `CATALOGUE` and no SDK; no method carries a `FacadeOptions` and the facade generator leaves the package out of `PROTO_PACKAGES` | SD1: protos in Task 4, the loopback listener in Task 5, the remote listener in Task 7; unstable until SD1 Task 32 |
| none: no SDK module | `loams.marketplace.v1` `Listing`: the marketplace listing's data model (§73 §3.6, D1455), messages only, equal to `schemas/marketplace/listing.v1.json`. Read by `loams-agentd` and the index tooling, never served by the Loams server, so it is in no `CATALOGUE`; the facade generator leaves it out of `PROTO_PACKAGES` (`NOT_SDK_PACKAGES`) | MK1 Task 1 |

Not in the public protos or SDKs: `loams.internal.v1`, the meter protocol (private), connector-specific admin.

### 7.3 Annotations drive the facade (D606)

`proto/loams/options/v1/options.proto` defines method and service options:

```proto
extend google.protobuf.ServiceOptions { optional ModuleOptions module = 50001; }
extend google.protobuf.MethodOptions  { optional FacadeOptions facade = 50002; }
message ModuleOptions { string name = 1; string summary = 2; }                  // "vector"
message FacadeOptions { string module = 1; string name = 2; bool retry_safe = 3; // overrides
                        string pagination = 4; }                                 // "items:next_page_token"
```

A buf plugin, `protoc-gen-loams-facade` (Rust, in `crates/loams-facade-gen`, run locally as a buf `local` plugin and through `buf generate`), reads the descriptors and renders one facade per language from templates (`sdks/templates/<lang>/`): module classes, method signatures delegating to the generated stubs, the retry class of each call, the paging iterator, the idempotency-key field. Only the **runtime** (transport, auth, retry, error mapping, token store, pagination helper) and the **typed builders** are hand-written, once per language (~600 lines for the runtime). This is the buy/generate-over-build choice: 13 hand-wrapped surfaces would diverge in a quarter. If the plugin proves too costly for a language, that language falls back to a hand-written facade checked by the same conformance suite (D606, Q604).

`loams.vector.search` and `loams.search` are two facade names for one RPC with different required-argument shapes (a `FacadeOptions` can name the same RPC twice via `facade` repeated; the RPC itself is `QueryService/Search`).

### 7.4 Cross-cutting behaviour, identical in every SDK (D608–D611)

**Auth (D608).** A `TokenSource` interface returns a bearer and its expiry. Built in: `ApiKey(key)`; `Static(token)`; `OidcExchange(subject_token_provider)`, which posts RFC 8693 token exchange to the instance's `/oauth/token` protocol endpoint (§19 §5.2) and caches/refreshes the Loams access token (a person signed in through the IdP gives its token; the gateway exchanges it, D447/D449. Under §76 the IdP is Zitadel (D1600) and people's Zitadel JWTs are verified directly, the exchange staying for agents and workloads, D1606); `WorkloadIdentity()` for agents (GitHub Actions, Kubernetes service account, cloud OIDC; §19 §5.2 flow 1); `Env()` reading `LOAMS_API_KEY`/`LOAMS_TOKEN`/`LOAMS_ENDPOINT`. A 401 with `ErrorInfo.reason = token_expired` triggers one refresh and one retry. Tokens travel in `Authorization: Bearer`; never in the query string. Mobile clients keep D434's pairing grant path (AP2/AP3), not this SDK.

**Consistency tokens (D609).** Writes return `consistency_token` (a `v1:` string) in the response message and in the response header `loams-consistency-token`. Reads accept `consistency` in the request (`STRONG` default, `EVENTUAL`, `AT_LEAST{token}`, as §05 §5). Each client holds a **session token store** (off by default, `session_consistency=True` turns it on) that merges returned tokens (max offset per stream and partition) and attaches the merged token to subsequent reads: read-your-writes without the caller threading tokens. Tokens are plain strings in the API and a `ConsistencyToken` value type in SDKs, with `merge`.

**Retries and backoff (D610).** The generated facade knows each RPC's retry class from the proto: `NO_SIDE_EFFECTS` and `IDEMPOTENT` retry on `UNAVAILABLE`, `DEADLINE_EXCEEDED` (only before a response started) and `RESOURCE_EXHAUSTED` carrying `RetryInfo`; mutating RPCs retry only when they carry an `idempotency_key`, which the SDK **generates (UUIDv7) once per logical call and reuses on every retry**; `StreamService/Produce` is never auto-retried without a producer key. Capped exponential backoff with full jitter: base 100 ms, ×2, cap 2 s, 3 retries (M1.6 Ruling 5's numbers), `RetryInfo.retry_delay` honoured up to 30 s. Server streams reconnect with the stream's cursor, skip the snapshot or accept `snapshot_reset` (§37 §8.3). Per-call and per-client overrides; `max_retries=0` disables.

**Errors (D611).** Errors map to a typed `LoamsError` hierarchy in each language, built from the Connect/gRPC code, the `google.rpc.Status` details and `loams.errors.v1.ErrorInfo`: `code` (the canonical code), `reason` (a stable `snake_case` string, the thing callers branch on), `message` (human, may change), `metadata`, `hint`, `retry_info`, `request_id`. Classes: `InvalidArgument`, `NotFound`, `AlreadyExists`, `PermissionDenied`, `Unauthenticated`, `FailedPrecondition`, `ResourceExhausted`, `Unavailable`, `DeadlineExceeded`, `Aborted`, `Internal`, `Unimplemented`, and a base `LoamsError`. `reason` values are registered in `docs/api/reasons.md` and enforced by a test (a reason may not be removed or renamed within a major version). `BadRequest` field violations surface as `error.field_violations`.

**Pagination.** AIP-158: `page_size`, `page_token` in; `next_page_token` out. The facade exposes both the raw page call and an iterator (`for x in loams.collections.list_all(...)`) that follows tokens.

**Streaming.** Server-streaming only (D420). Heartbeats every 15 s, cursors, and the reconnect policy above are part of the runtime.

### 7.5 Arrow and bulk data

Queries return rows over Connect (JSON or protobuf). **Bulk results and bulk ingest use Arrow Flight SQL** (§05 §8, D49), which is a gRPC service on the same port (§4). SDKs with a maintained Flight/ADBC client expose `loams.bulk`: **Python** (`adbc-driver-flightsql`, optional extra `loams[flight]`, as M1.6 Ruling 3), **Go, Java, C++, Rust, C#** (Arrow's Flight libraries/ADBC). **Node/TypeScript, Dart, Swift, Kotlin, PHP, Ruby, Objective-C** have no maintained Flight client; their `loams.documents.write_batch` chunks unary `WriteDocuments` (default 10 000 rows or 4 MiB per call, one chunk in flight, each whole or not at all, same as the Flight path) and `loams.sql.query_arrow` is absent. Arrow IPC bytes may be returned by `SqlService/QueryArrow` as a single `bytes` field up to a size cap for clients that can read IPC (TS with `apache-arrow`), Q608.

### 7.6 Hand-written typed builders

Generated message types are verbose for the hybrid query IR. Languages with a rich type system get a builder over the generated messages (as M1.6's typed builder): Python, TypeScript, Go, Rust, Java, Kotlin, Swift, C#. The others expose the generated messages directly in v1 plus documented JSON/proto examples. Builders are tested against the same fixture corpus as the facade (§10.4).

## 8. Services to add (summary of API1)

New protos: `loams.options.v1`, `loams.collection.v1` (Namespace, Collection, Document, Query), `loams.sql.v1`, `loams.link.v1`, extension of `loams.stream.v1` (create, describe, fetch, events), `loams.admin.v1`, `loams.auth.v1`, `loams.internal.v1` (cluster-only). All follow AP0's conventions: `NO_SIDE_EFFECTS` on reads, `idempotency_key` on mutations, `ErrorInfo.reason` errors, server-streaming only, cursors on every stream, `buf lint STANDARD`. `buf breaking` (`FILE`) is enforced from the first release tag for every package except those explicitly marked `unstable` (today `loams.live`; R1's rule).

## 9. The language matrix (D612–D614)

Verified 2026-10-02: gRPC's [languages page](https://grpc.io/docs/languages/) lists 13 official languages (C#/.NET, C++, Dart, Go, Java, Kotlin, Node, Objective-C, PHP, Python, Ruby, Rust, Swift; the page itself is dated 2021-08-11 but lists these). The [connectrpc.com](https://connectrpc.com/docs/introduction) introduction names Connect for Go, TypeScript/JavaScript (Web and Node), Swift (stable), Kotlin and Python (beta), with Dart and Rust as newer; the `github.com/connectrpc` organisation (2026-10-02) has `connect-go`, `connect-es`, `connect-swift`, `connect-kotlin`, `connect-dart`, `connect-python`, `connect-rust` and the shared `conformance` suite. There is no official Connect for Java, C#, C++, Ruby, PHP or Objective-C. Connect joined the CNCF ([Buf, 2026](https://buf.build/blog/connect-rpc-joins-cncf)). Status of each library is re-checked in SDK2 Task 0 of the language (a library that is not maintained at that date falls back to gRPC; rule below).

| # | Language | Client protocol | Library | `buf` plugins (remote) | Package | Registry | v1 status |
|---|---|---|---|---|---|---|---|
| 1 | TypeScript / JavaScript (Node, browsers, Bun; Deno dropped by D1790, [§79](79-void-and-vite-plus.md)) | **Connect** | `@connectrpc/connect` 2.x (`connect-es`), protobuf-es | `buf.build/bufbuild/es` | `@loams/client` (facade), `@loams/proto` (generated), `@loams/live` (kept, re-exports) | npm | **First** |
| 2 | Python | **Connect** | `connect-python` (beta), protobuf | `buf.build/protocolbuffers/python`, `buf.build/connectrpc/python` | `loams` | PyPI | **First** |
| 3 | Go | **Connect** | `connect-go` | `buf.build/protocolbuffers/go`, `buf.build/connectrpc/go` | module `loams.dev/go` (mirror repo `ostrium-labs/loams-go`) | Go module proxy | **First** |
| 4 | Rust | **Connect** | `connectrpc` crate (`connect-rust`) with buffa, the server's stack (D128) | `connectrpc-build` in `build.rs` (no buf remote) | `loams` | crates.io | **First** |
| 5 | Swift | **Connect** | `connect-swift` (stable) | `buf.build/apple/swift`, `buf.build/connectrpc/swift` | `Loams` | SwiftPM (mirror `ostrium-labs/loams-swift`) | Second |
| 6 | Kotlin (JVM, Android) | **Connect** | `connect-kotlin` (beta) | `buf.build/protocolbuffers/java` (lite), `buf.build/connectrpc/kotlin` | `dev.loams:loams-kotlin` | Maven Central | Second |
| 7 | Java | **gRPC** | `grpc-java` + `protobuf-java` | `buf.build/protocolbuffers/java`, `buf.build/grpc/java` | `dev.loams:loams` | Maven Central | Second |
| 8 | C# / .NET | **gRPC** | `Grpc.Net.Client` + `Google.Protobuf` (gRPC-Web in Blazor WASM through `Grpc.Net.Client.Web`) | `buf.build/protocolbuffers/csharp`, `buf.build/grpc/csharp` | `Loams` | NuGet | Second |
| 9 | Dart / Flutter | **Connect if `connect-dart` is maintained and passes conformance at Task 0; else gRPC** | `connect-dart` (`connectrpc` on pub.dev) or `grpc` (Google) | `buf.build/protocolbuffers/dart` + connect plugin or `buf.build/grpc/dart` | `loams` | pub.dev | Third |
| 10 | Ruby | **gRPC**, plus a Connect-unary HTTP transport for hosts without the native gem (D613) | `grpc` gem, `google-protobuf` | `buf.build/protocolbuffers/ruby`, `buf.build/grpc/ruby` | `loams` | RubyGems | Third |
| 11 | PHP | **gRPC**, plus a Connect-unary PSR-18 transport (D613) | `grpc/grpc` + `google/protobuf` (ext-grpc, ext-protobuf recommended) | `buf.build/protocolbuffers/php`, `buf.build/grpc/php` | `loams/loams` | Packagist (mirror `ostrium-labs/loams-php`) | Third |
| 12 | C++ | **gRPC** | `grpc++`, protobuf | `buf.build/protocolbuffers/cpp`, `buf.build/grpc/cpp` | `loams` | vcpkg overlay registry and Conan remote; source tarball always (Q610) | Third |
| 13 | Objective-C | **gRPC** | gRPC-ObjC (`gRPC-ProtoRPC`) | `buf.build/protocolbuffers/objc`, `buf.build/grpc/objc` | `Loams` (generated sources + podspec) | **Source and SwiftPM target only**; CocoaPods trunk goes read-only on 2026-12-02, so no new trunk publish (D614, Q611) | Third |

Rules (D612–D614):

- **D612 Connect-first, gRPC where no official Connect exists.** A library is "official" if it lives in the `connectrpc` organisation or is named on connectrpc.com. If a Connect library is unmaintained (no release in 12 months) or fails the conformance suite at Task 0, the language uses gRPC. **The "Java SDKs deferred" ruling (D101-era, M1.6/AP notes) is superseded for generated clients:** all 13 languages get a generated client. Hand-written ergonomic wrappers beyond the generated facade remain per the tiers in §7.6. This is a change in scope to flag to the owner (Q601).
- **D613 A transport fallback for Ruby and PHP.** Their gRPC client needs a native extension that shared hosting, serverless runtimes and some containers lack. The SDK also ships a **Connect-unary transport** (an HTTP POST of JSON over `Net::HTTP`/PSR-18) for unary RPCs, generated from the same protos. It is not a REST API: same contract, same messages. Streaming RPCs require the gRPC transport. (Not applied to C++/ObjC.)
- **D614 Waves.** First wave (TS, Python, Go, Rust) ships with API1 and covers the product's own consumers and the conformance corpus; second wave (Swift, Kotlin, Java, C#); third wave (Dart, Ruby, PHP, C++, Objective-C). A wave is a priority order, not a gate; the language issues are independent once SDK1 lands.

Also considered and not added: Elixir, Scala, Haskell and others have community gRPC libraries that are not on grpc.io's official list; `buf generate` makes a community SDK cheap, and the proto repo plus the facade generator is the contribution path (documented in `CONTRIBUTING.md`).

## 10. Generation, conformance and docs (D615–D617)

### 10.1 Pipeline

1. `proto/` is the single source. `buf.yaml` module with `lint: STANDARD` and `breaking: FILE`.
2. One `buf.gen.<lang>.yaml` per language (`sdks/<lang>/buf.gen.yaml`) using **remote plugins** (table above), so CI needs no local `protoc` plugin installs; the facade generator is a `local` plugin (a Rust binary built once per CI run). Existing `buf.gen.yaml` (live TypeScript), `buf.gen.apps.yaml`, `.kotlin.yaml` and `.swift.yaml` are folded into this layout by SDK1 Task 1; `@loams/proto` stays the committed TypeScript output, others are generated at build time or committed per the language's practice (Go and PHP mirrors must carry generated code in the repo; npm/PyPI/Maven/NuGet build it into the package).
3. `buf generate` output is **committed for the mirrored languages and checked for drift in CI** (the AP0 rule). For the rest, CI regenerates and builds.
4. Buf Schema Registry (BSR) use is not required; remote plugins are used from `buf.build/<owner>/<plugin>` at a **pinned plugin version** (`:v33.1`-style tags recorded in each `buf.gen.yaml`). Whether to publish the module to a BSR (generated SDK mirrors for free) is Q605.

### 10.2 Layout

```
sdks/
  templates/<lang>/        facade templates (D606)
  fixtures/                request/response corpus (JSON + proto), shared
  typescript/ python/ go/ rust/ swift/ kotlin/ java/ csharp/ dart/ ruby/ php/ cpp/ objc/
  live-typescript/         -> folded into typescript/ by SDK2 TS task (kept one release)
  conformance/             the test server launcher + runner scripts
```

### 10.3 Versioning (D616)

- **Protocol**: `buf breaking` (`FILE`) on every PR against the last release tag; a breaking change requires a new package version (`v2`) served alongside `v1` for one minor release. `unstable` packages are marked in `options.proto` and excluded.
- **SDKs**: semantic versioning **per SDK**, independent of the server's version; each SDK declares the proto tag it was generated from (`LOAMS_PROTO_REV`) and the minimum/maximum server `api_versions` it supports, checked at connect by `GetInstance` (a mismatch is a warning, a major mismatch an error). Pre-1.0 (`0.y.z`) while the server is pre-1.0; the first stable SDK release is with the server's 1.0.
- A server release never requires an SDK release and vice versa within the same API major.

### 10.4 Conformance (D617)

- **Shared fixtures** (`sdks/fixtures/`): ordered cases, each a request message (JSON), the expected response and error, the consistency/retry behaviours, and streaming scripts. Replaces M1.6's wire-fixture corpus; a Rust test runs every fixture against the real server.
- **Test server**: `loams dev --listen 127.0.0.1:0` with fixtures loaded (real engines, small), the primary target; `loams-apps-mock` (AP0) covers the app packages (instance, devices, approvals, operations, notifications) and fault injection (retryable `UNAVAILABLE`, `RetryInfo`, mid-stream disconnect, token expiry), which the real server cannot produce on demand. The runner exports `LOAMS_TEST_ENDPOINT`.
- **Per-SDK suite**: a language's tests run the fixture list through the **public facade** (not the generated stubs), asserting results, error `reason`/`code`, retry counts, idempotency-key reuse, token merge, pagination and streaming resume. A language is publishable only when it passes 100% of the required fixtures; fixtures marked `transport:grpc-only` skip on the Connect-unary fallback.
- **Protocol conformance**: Connect libraries are additionally run through `connectrpc/conformance` against the server's handler (client and server modes) once per release.
- CI cost: one job builds `loams` once and uploads it; each SDK job downloads it. Jobs for the second and third waves run on tag and nightly, not on every PR, until those jobs are made required checks for merging (paths filter on `proto/**` and `sdks/<lang>/**`).

### 10.5 Reference docs (D618)

`buf generate` with `buf.build/community/pseudomuto-doc` (or `protoc-gen-doc`) emits a Markdown API reference from comments; the `loams.dev` docs site (`loams-cloud`) renders it with per-RPC curl/Python/TS/Go snippets taken from the **fixture corpus** (so every example is tested). Each SDK has its own README with the same quickstart. Reasons are listed in `reasons.md`. The docs pipeline is a task in SDK1 (generation) and a follow-up issue in `loams-cloud` (rendering; owner decision on site layout).

## 11. Publishing (D615, issue #254)

All publishing is from GitHub Actions on a signed tag `sdk-<lang>-v<semver>`; the **owner accounts** are listed in §13.

| Registry | Publisher auth | Notes |
|---|---|---|
| crates.io | Trusted Publishing (OIDC) | crate `loams`; `loams-proto` split if the generated types are large |
| PyPI | Trusted Publishers (OIDC) | wheel is pure Python; extras `flight`, `arrow`, `polars` |
| npm | Trusted Publishing (OIDC) with provenance | scope `@loams` must be owned by the org |
| Go | git tag on the mirror repo; `proxy.golang.org` and `pkg.go.dev` pick it up | vanity path `loams.dev/go` needs `go-import` meta on `loams.dev` (`loams-cloud` task) |
| Maven Central | Central Portal user token + GPG signing key in Actions secrets (no OIDC) | namespace `dev.loams` requires DNS TXT verification of `loams.dev` |
| RubyGems | Trusted Publishing (OIDC) | gem `loams` |
| Packagist | webhook from the mirror repo (`ostrium-labs/loams-php`) on tag; no push | package `loams/loams` |
| NuGet | Trusted Publishing (OIDC, since 2025) | package id `Loams` (reserve prefix `Loams.*`) |
| pub.dev | automated publishing from GitHub Actions (OIDC) | package `loams`; tag pattern `loams-v{{version}}` |
| SwiftPM | git tag on the mirror repo `ostrium-labs/loams-swift` | no account |
| vcpkg / Conan | own overlay registry (git) / own remote first; submission to the central registries after v1 | Q610 |
| CocoaPods | **not published** (trunk read-only from 2026-12-02) | D614 |

Trusted-publishing availability is re-verified at each language's Task 0 (dated). The mirror repos are written only by the `sdk-mirror` workflow (a subtree split of `sdks/<lang>/` plus generated code) and reject human pushes.

## 12. Risks

| # | Risk | Mitigation |
|---|---|---|
| 1 | Thirteen SDKs multiply maintenance | One generator for the facade, shared fixtures, waves, and a rule that a language that fails conformance is marked "experimental" on the matrix |
| 2 | Connect libraries for Python (beta), Kotlin (beta), Dart (new) change | Pin versions; the fallback is gRPC; the conformance suite decides |
| 3 | connect-rust and buffa are young | Already the server stack (D128); server is the forcing function |
| 4 | Browsers and Unity-like runtimes cannot do HTTP/2 gRPC | Connect and gRPC-Web are served on the same port; TS/Swift/Kotlin/Dart use Connect |
| 5 | Removing the REST breaks curl-based habits and the M1.6 plan | Connect JSON is curl-able (§4); M1.6 is amended, not deleted; shims for one release |
| 6 | Facade generator is custom code | Small (templates over descriptors), fallback is hand wrappers for one language; reviewed in SDK1 Task 3 |
| 7 | Registry namespace squatting before launch | Reserve names first (owner action list, §13) |
| 8 | Ruby/PHP gRPC native extensions are painful to install | D613 Connect-unary fallback |
| 9 | `buf` remote plugins' availability and versions | Pin versions; CI caches; a local-plugin fallback script |
| 10 | CocoaPods trunk read-only from 2026-12-02 | Not a target; SwiftPM covers ObjC consumers via the Swift package, and ObjC ships generated sources |
| 11 | Two ways to say the same thing (`vector` and `search`) | One RPC; docs show both as views |
| 12 | Streaming with half-duplex only limits bulk upload | Flight `DoPut`, chunked unary (§7.5) |

## 13. Owner actions (proposed; none block design)

1. Accounts and namespaces: **RubyGems** (gem `loams`, enable trusted publishing), **Packagist** (vendor `loams`, link the `loams-php` mirror), **NuGet** (`Loams`, reserve ID prefix `Loams.`), **pub.dev** (publisher `loams.dev`, verified domain), plus confirming the existing: crates.io `loams`, PyPI `loams`, npm org `@loams`, Maven Central namespace `dev.loams` (DNS TXT on `loams.dev`; GPG key), GitHub orgs/repos for mirrors `loams-go`, `loams-swift`, `loams-php`.
2. `go-import` meta for `loams.dev/go` on the site.
3. Approve the supersession of "Java deferred" and the 13-language scope (Q601) and the removal of the M1.2 REST without shims (Q600).
4. Decide whether to use a Buf Schema Registry account (Q605).

## 14. Open questions

See [decision log](13-decision-log.md) (Q600–Q614). The ones that change work if answered differently: Q600 (shims), Q601 (all 13 languages), Q602 (is the OAuth/OIDC protocol surface acceptable as kept), Q604 (generator vs hand wrappers), Q606 (Connect-unary fallbacks count as "no REST"), Q608 (Arrow IPC over Connect).

## 15. Contradictions with earlier decisions, and how they are resolved

| Earlier | Now |
|---|---|
| D101: OpenAPI for native REST, Go SDK over REST (M2) | Superseded: no OpenAPI; Go SDK is Connect over the protos (D604) |
| M1.6 Ruling 1 (SDKs target REST), Ruling 4 (zero-dependency TS), Python `httpx` | Superseded for protobuf surfaces: the TS SDK depends on `@connectrpc/connect` and `@bufbuild/protobuf` (D128's M1.6 amendment becomes the rule); Python depends on `connect-python`; Flight extra unchanged |
| M1.6 `loams-client` (PyPI) and `@loams/client` (npm) as REST SDKs | Renamed/replaced: PyPI `loams`, npm `@loams/client` (Connect). Names were reserved, never published |
| §19 P9: console OpenAPI `/api/v1`, "stays REST" (§37 §8.1, Q423 proposed) | Moved to Connect (`loams.admin.v1`, `loams.auth.v1`); OAuth/OIDC/well-known stay HTTP (D602) |
| §37 §8.3 "Connect for all three apps" | Unchanged; the apps use the same protos |
| "Java SDKs deferred" | Superseded for generated clients (D612) |
| §30 MCP server on its own listener (D111) | Unchanged; MCP stays a compat surface (D603) |
| D420 server-streaming only | Unchanged; Flight is the only bidi/client-stream use and is a compat surface |

## 16. Sources

- gRPC supported languages: <https://grpc.io/docs/languages/> (fetched 2026-10-02).
- Connect: <https://connectrpc.com/docs/introduction>, <https://github.com/connectrpc> (2026-10-02), <https://buf.build/blog/connect-rpc-joins-cncf>; Dart on pub.dev (`connectrpc`, `connect_kit` 0.1.0 of 2026-09-04): verify which is the connectrpc-org package at SDK2 Task 0.
- NuGet Trusted Publishing: <https://learn.microsoft.com/nuget/nuget-org/trusted-publishing>.
- CocoaPods trunk read-only 2026-12-02: <https://blog.cocoapods.org> (as reported 2026-10-02).
- Internal: `proto/`, `crates/loams/src/api/`, `crates/loams-stream-grpc/proto`, `api/console/openapi.json`, §05 §4, §5, §8; §06; §19; §30 §9, §12; §37 §8; §39; §40 (branch `systemone-design`); M1.6 plan; D101, D128.
