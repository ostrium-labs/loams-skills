# API2 — The Unified Connect API and the SDKs in Production Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Work task by task, test first. Each task lists the interfaces it must produce and the tests that must exist and pass before it is done. Where this plan gives exact names, paths, headers or defaults, use them verbatim. Where it gives a contract and named tests, write the code to that contract, and record any deviation in "Rulings made during execution" at the end of this file.
>
> **Status: Planned** (2026-10-10). Track API2, design [§44](../design/44-unified-api-and-sdks.md) (D600–D619, Q600–Q614). API2 **finishes and supersedes** the three 2026-10-02 plans, which stay as the record of what they delivered:
> - [API1](2026-10-02-api1-unified-connect.md): Tasks 0–4 are on `dev` (route map, options, one port, `loams.collection.v1` with Namespace, Collection, Document and Query services). Its Tasks 5–10 are API2 Tasks 1–12.
> - [SDK1](2026-10-02-sdk1-generation-pipeline.md): its TypeScript, Python, Go and Rust slices of Tasks 1–4 landed through SDK2 (generator, runtime contract R1–R10, 33-fixture corpus, drift and pin scripts). Its Tasks 0, 5–8 and the rest of 1–4 are API2 Tasks 13–18 and 24–26.
> - [SDK2](2026-10-02-sdk2-languages.md): all 13 languages have a runtime under `sdks/<lang>/` that passes the 28 required fixtures, but the corpus covers only `loams.instance.v1`, `loams.live.v1` and the AP0 mock packages. Making each language cover the whole API and ship is API2 Tasks 19–23.
>
> When a task of this plan lands, the matching row of API1, SDK1 or SDK2 gets a one-line pointer to it (Task 28). Do not edit those plans' rulings.
>
> **SDKP pointer (2026-10-19).** [SDKP](2026-10-19-sdk-parity.md) carries Tasks 1–2, Task 13 for `loams.collection.v1` and `loams.graph.v1`, Task 14 and Task 17 for the app modules, and the app half of Tasks 15 and 20–22, as its P0a–P0e and L1–L13.
>
> **SD1 pointer.** `loams.agentd.v1`, the agent daemon's Connect surface ([SD1](2026-10-13-sd1-studio-ade.md) Task 4), is outside the SDK catalogue (Q816): `loams-facade-gen`'s `NOT_SDK_PACKAGES` keeps it out of `PROTO_PACKAGES`, it is a §44 §7.2 row with no SDK module, and its reasons are a daemon-only table in `docs/api/reasons.md` (SD1 T4-2–T4-4).

## Owner rulings 2026-10-10 (defaults)

The owner asked for the best default on every open question ("do the best for others"). These are binding for execution; the tasks below already follow them. Task 0 records them as rulings and Task 28 copies them into the decision log. Only the items under "Open questions" at the end still need a human.

| # | Question | Decision | Reason |
|---|---|---|---|
| Q600 | Legacy REST shims or delete outright | **Delete outright.** No `--legacy-rest`, no `legacy.rs`, no `Deprecation`/`Sunset` headers; removed paths 404 like any unknown path, and `docs/api/migrate.md` is the only migration aid | Pre-1.0 with no external users; a shim is code written only to be deleted |
| Q601 | Confirm 13 languages | Keep all 13; Objective-C is the first to drop if upkeep is too heavy (Q611) | Each already passes the 28 fixtures; dropping later is cheap, adding back is not |
| Q602 | OAuth/OIDC/well-known/health stay HTTP | Yes (D602) | They are protocol endpoints that standard clients expect at fixed HTTP paths |
| Q603 | Reflection scope | On in `loams dev`; elsewhere off unless `--reflection`, which requires auth | Developer convenience without exposing the schema on production ports |
| Q604 | Generator versus hand-written facades | **Generate** TypeScript, Python, Go, Rust, Swift, Kotlin, Java and C#; **hand-write** Dart, Ruby, PHP, C++ and Objective-C, each checked by `<lang>_facade_matches_index` | Generation where users are most numerous; the index test keeps the hand-written five from drifting |
| Q605 | Publish to the Buf Schema Registry | No; local plugins plus remote-plugin pins only | No extra account or dependency for a pre-1.0 API |
| Q606 | Ruby/PHP Connect-unary fallback counts as "no REST" | Yes | Same protos and messages; it is the Connect protocol, not a bespoke REST |
| Q607 | `loams.dev/go` vanity path | Yes; `sdks/go/go.mod` already uses it, and the `go-import` meta tag is a follow-up issue in `loams-cloud` | Stable import path independent of the repository host |
| Q608 | `QueryArrow` IPC with a cap | Yes, 16 MiB, `result_too_large` above it pointing to Flight | Arrow for small results without making the unary path a bulk channel |
| Q609 | Licence of generated code; DCO on mirrors | Generated code is Apache-2.0, the repository's licence (`LICENSE`, `Cargo.toml`); mirrors are bot-written, so no DCO there | Same licence as the source it is generated from; DCO certifies human contributions |
| Q610 | C++: own vcpkg/Conan first or source only | Ship a vcpkg registry (a git repository) and a Conan recipe in-repo; publishing a hosted Conan remote stays behind `SDK_PUBLISH_CONAN` | Users get a package manager path; the hosting is the only account step |
| Q611 | Objective-C: keep or drop | Keep generated sources plus SwiftPM; marked "community" if it fails conformance | Cheap to keep while it passes; honest label if it does not |
| Q612 | CODEOWNERS on `reasons.md` | Yes, plus `proto/**` | Reasons and protos are the public contract |
| Q613 | `loams.vector` module or view | Both names, one RPC (`vector.search` and `search.query`) | Discoverability for vector users without a second RPC |
| Q614 | Scroll: stream or unary with cursor | Whatever Task 0 finds built (expected unary with a cursor) | The built shape already works on every transport, including Connect-unary |
| API2-Q1 | Auth before MT1 | **Ship the main port with `ApiKeyAuthenticator` and `DevLoopbackAuthenticator` (loopback-only); REST removal does not wait for MT1.** `Authenticator` is the hook: MT1 adds `JwtAuthenticator` and the HTTP protocol endpoints without touching handlers | Unblocks API2b; the trait keeps MT1 a drop-in |
| API2-Q2 | Wave 2/3 CI jobs as required checks | **Nightly and on tags, never in `required.needs` in this plan.** Making one PR-blocking is a later, separate ruling per language | Keeps PR CI fast; regressions still surface within a day |
| API2-Q3 | `loams.durable.v1` scope | **No `loams.durable.v1` now. The Resonate HTTP protocol is the only durable API.** Task 8 ships `OperationsService` only | No second control surface to keep in step with Resonate; revisit when a Loams-specific durable need appears |
| API2-Q4 | Idempotency window | 24 h cluster-wide, as PG2's ledger; only keyed calls pay the metastore write, and expired entries are swept by the existing metastore TTL | Covers every realistic client retry window, matches the other ledger |
| API2-Q5 | Compat gRPC auth on the conventional ports | Unchanged until MT1; only the main-port mount goes through `Authenticator`. Recorded as an accepted pre-MT1 risk in Task 27's threat model | D603: compat ports keep their wire and auth behaviour; the main port is the new surface |

**Goal:** one public API and thirteen shippable SDKs.
- Every application call is a Connect/gRPC RPC in `loams.<service>.v1` on the `loams` main port. The native REST under `/v1/namespaces/...`, the console OpenAPI `/api/v1` and `/internal/*` are gone, deleted outright with no shims (Q600).
- Gone too: `api/console/openapi.json`, `openapi-fetch` and `openapi-typescript`.
- The compatibility surfaces of §44 §6 are unchanged and green.
- Every stable package carries the facade annotations, and the SDKs expose it as `loams.<module>.<call>`.
- Every language passes the required fixtures of a corpus that covers every stable package and every runtime-contract clause (R1–R10), against `loams dev` and the fault mock.
- Each language publishes from CI (dry run until the owner's registry actions, §44 §13), with reference docs generated from the protos.

The exit is "Exit criteria for production" at the end of this plan, with the owning tasks.

**Architecture** (§44 §4, §7, §10):
- **One router.** `crates/loams/src/api/connect.rs` owns the catalogue (`CATALOGUE`), `GetInstance.services[]`, health and reflection. Each package's handlers live in their own `api/connect_<package>.rs` and call the same service traits the REST handlers call today. The REST handlers' logic moves; it is not rewritten.
- **Auth is a layer, not a handler concern.** One `Authenticator` on the main router (Task 6) turns `Authorization: Bearer` into a `Principal` in the request extensions. Handlers read the principal; none parses a header.
- **Compat gRPC on the main port** (D601) is routed by gRPC service-name prefix to the existing tonic servers. connect-rust does not serve them.
- **`loams.internal.v1`** is a second buf module (`proto-internal/`), generated into `loams-proto` behind a non-default feature and served only by `internal_router`.
- **The facade has one source.** The `loams.options.v1` annotations drive `protoc-gen-loams-facade`, which renders TypeScript, Python, Go and Rust. It also emits `sdks/fixtures/facade-index.json`, the module/call/retry-class/pagination index. A language with a hand-written facade (D730's fallback, Q604) is checked against that index by a test, so no SDK can drift from the protos silently.
- **The fixture corpus is the specification.** It is recorded from `loams dev`, plus `loams-apps-mock` and the fault server for what a real server cannot produce on demand. `sdks/fixtures/manifest.json` decides what is required.

**Tech Stack:**
- Rust 1.97 (workspace `rust-version`), edition 2024, workspace lints.
- `connectrpc` 0.9.1, `connectrpc-build` 0.9, `buffa` 0.9.2, `connectrpc-health`/`-reflection` 0.9.0 (as `docs/api/route-map.md` records); `tonic` 0.14 for the compat servers only; `arrow-flight` 58.4; `axum` 0.8.
- `buf` 1.73.0 and the remote plugins pinned in `scripts/sdk/pins.lock`.
- Per language, the versions in each `sdks/<lang>/DEPENDENCIES.md` or lockfile. Task 0 records the ones that have neither.
- Test tools: `sdks/conformance/run.sh` (recorded corpus plus live `loams dev`), `loams-apps-mock`, `connectrpc/conformance` (Task 23), `actionlint`.

**Spec:**
- [§44](../design/44-unified-api-and-sdks.md) (all), and D600–D619, Q600–Q614 in the [decision log](../design/13-decision-log.md).
- [§05](../design/05-query-engine.md) §4 (hybrid IR), §5 (consistency), §8 (Flight).
- [§19](../design/19-console-identity-and-agents.md) §5 (tokens, sessions), P9 (console API).
- [§30](../design/30-loams-cli.md) §9 (variants).
- [§37](../design/37-desktop-and-mobile-apps.md) §8 (apps on the same protos).
- [MT1](2026-10-02-mt1-authentik-identity.md) (identity; Task 7 TLS on the main port), [LV1](2026-10-08-lv1-live-production.md) (Live on the main port), [GR1](2026-10-08-gr1-graph-production.md) Task 5, [PG2](2026-10-08-pg2-postgres-production.md) Tasks 1, 9 and 57 (package rules and served-only reflection, R1.3, R1.4).
- `docs/api/route-map.md`, `docs/api/reasons.md`, `docs/sdk/runtime-contract.md`, `docs/sdk/fixtures.md`, `docs/release/publishing.md`; issue #254.

## Global Constraints

- **Worktree and branches.** Work in `~/Documents/Ostriumlabs/loams-wt/api2-unified-api-sdks`. Use one branch per milestone, `feat/api2a-surface`, `feat/api2b-rest-removal`, `feat/api2c-pipeline`, `feat/api2d-languages` and `feat/api2e-release`, each based on `dev`, with stacked PRs targeting `dev`. A language task in API2d may take its own branch, `feat/api2d-<lang>`. Use `git commit -s` (DCO). Commit areas: `proto`, `api`, `loams`, `console`, `desktop`, `sdk-<lang>`, `sdks`, `facade`, `ci`, `docs`.
- **Rust builds** use the shared target directory (`~/Documents/.cargo/config.toml`). Never set `CARGO_TARGET_DIR` and never build in `/tmp`. Run one cargo build at a time (jobs and linker from `~/.cargo/config.toml`). Build the touched crates (`cargo test -p loams --test connect_sql`), not the workspace. `sdks/rust` is its own workspace: test it there (`cd sdks/rust && cargo test`). `scripts/sdk/gen.sh` builds `loams-facade-gen` at `LOAMS_BUILD_JOBS` (default 4); do not run it next to another cargo build.
- **Compatibility surfaces are not touched** (D603). The Qdrant REST and gRPC, Elasticsearch, Postgres wire, Kafka gateway, Flight SQL, Resonate, MySQL, Git Smart HTTP and CloudEvents suites stay green on every PR (`qdrant-client`, `es-client` and the rest of `ci.yml`). A PR that changes their wire output is wrong.
- **Behaviour first, deletion last.** A REST route or OpenAPI path is deleted only in Task 11, and only after its RPC passes the same tests, ported by name with an `_rpc` suffix (API1's rule).
- **AP0 API rules** (§44 §8):
  - Every mutation that is not idempotent by construction has `idempotency_key` (API1 ruling 2.4 stands: create-by-name and set-state RPCs do not).
  - Reads are `NO_SIDE_EFFECTS`.
  - Errors are `loams.errors.v1.ErrorInfo` with a reason registered in `docs/api/reasons.md`. A reason is never renamed or removed within a major version.
  - Pagination is `page_size`/`page_token`/`next_page_token`.
  - Streams are server-streaming only (D420), with a cursor and a 15 s heartbeat.
  - Resource names are message fields, never URL paths.
- **`buf lint` (STANDARD) is clean on every package.** `buf breaking` (FILE) runs against `dev` until the first release tag and against the tag after it (Task 12). Packages with `ModuleOptions.unstable` are excluded through `buf.yaml`'s `breaking.ignore`, and the option and the ignore line must agree (Task 12 test).
- **Generated code is never hand-edited.** `scripts/sdk/drift.sh` fails on a diff in any committed output. A pin bump is its own PR with the regenerated output and its `pins.lock` row.
- **One required-fixture bar** (D617). `sdks/fixtures/manifest.json` is the only definition. A language skips a required fixture only when the fixture is `transport: grpc-only` and the language runs the Connect-unary fallback (D613).
- **Secrets.** Bearer tokens, API keys and refresh tokens never appear in a URL, a log line, an error message, an `ErrorInfo.metadata` value, a span attribute or a fixture. SDK `Debug`/`toString`/`repr` of a `TokenSource` prints `[redacted]`. Recorded fixtures carry the placeholder `Bearer <redacted>`.
- **No billing** (D220, D552). No RPC, field or SDK module named `meter`, `usage`, `invoice`, `credit`, `plan` or `price`.
- **Pins.** Exact versions for every new crate, package and plugin, at least 14 days old (registry date). Record each pin in the task's commit message and, for the generation toolchain, in `scripts/sdk/pins.lock`.
- **Do not publish** to any registry until the owner has done §44 §13's account actions for it. Until then every release job is a dry run (Task 25).

## Review Focus

1. **A retried mutation applies twice.** Paths: a keyed `WriteDocuments` retried on another gateway node; a `Produce` without a producer key auto-retried by an SDK; an SDK that mints a fresh key per attempt. Expected: never. Tests: Task 4 `keyed_write_replays_across_nodes`; Task 16 `produce_without_key_never_retried`; Task 20 `<lang>_retry_reuses_idempotency_key` for every language (Tasks 20–22).
2. **A token leaks** into a URL, a log, an error, a fixture or an SDK's debug output. Expected: never. Tests: Task 6 `bearer_never_logged`, `query_string_token_refused`; Task 15 `fixtures_carry_no_credentials`; Task 27 `sdk_token_source_debug_redacted` (every language).
3. **An unauthenticated call reaches data.** Paths: a Connect, gRPC or gRPC-Web call with no bearer on the main port; compat gRPC on the main port; `loams.internal.v1` on the public listener. Expected: refused with `unauthenticated` or not routed at all. Tests: Task 6 `every_served_rpc_requires_auth_except_allowlist`; Task 5 `compat_grpc_on_main_port_requires_auth`; Task 4 `internal_service_not_on_public_port`.
4. **A REST route survives Task 11.** Expected: no; a removed path is a 404 like any unknown path (Q600: no shims). Tests: Task 11 `no_native_rest_route_remains`, `removed_rest_paths_are_404`.
5. **An SDK's surface drifts from the protos**: a call with no annotation, a wrong retry class, or a missing module. Expected: CI fails. Tests: Task 14 `<lang>_facade_matches_index` (every hand-written facade); Task 13 `every_stable_service_has_module`, `every_stable_rpc_has_facade_call`.
6. **A stale or wrong consistency token is used silently.** Paths: the session store merges tokens from two namespaces; a token from another instance is accepted. Expected: an error, never a stale read. Tests: Task 16 `token_merge_is_per_stream_partition_max`, `foreign_instance_token_refused`.
7. **A compat surface's wire output changes.** Expected: never. Tests: the existing compat suites, plus Task 5 `qdrant_grpc_bytes_unchanged_on_main_port`.
8. **Reflection lists an unserved service, or is on in production.** Expected: no. Tests: Task 9 `reflection_lists_only_served_services`, `reflection_off_by_default_outside_dev`.
9. **A breaking change ships in a stable package.** Expected: CI fails. Tests: Task 12 `breaking_change_fails_ci`, `unstable_flag_and_buf_ignore_agree`.

---

## File structure

```
proto/loams/sql/v1/sql.proto                                      Task 1
proto/loams/stream/v1/stream.proto                                Task 2 (moved from crates/loams-stream-grpc/proto)
proto/loams/link/v1/link.proto                                    Task 3
proto-internal/loams/internal/v1/internal.proto, buf.yaml         Task 4 (second buf module, D607)
proto/loams/auth/v1/auth.proto                                    Task 6
proto/loams/admin/v1/{org,project,agent,key,audit}.proto          Task 7
proto/loams/errors/v1/errors.proto                                Task 16 (RetryInfo use, request id)
crates/loams-proto/{build.rs,src/lib.rs}                          Tasks 1–8 (package list; `internal` feature)
crates/loams-stream-grpc/                                         Task 2 (tonic service kept as the CloudEvents gRPC binding only)
crates/loams/src/api/
  connect.rs                                                      Tasks 1–9 (CATALOGUE rows, registration, reflection)
  connect_sql.rs connect_streams.rs connect_links.rs              Tasks 1–3
  connect_internal.rs                                             Task 4
  connect_compat.rs                                               Task 5 (service-name routing to tonic)
  auth.rs connect_auth.rs connect_admin.rs                        Tasks 6–7
  connect_operations.rs                                           Task 8 (no connect_durable.rs: API2-Q3)
  idempotency_store.rs                                            Task 4 (cluster ledger; replaces connect_idempotency.rs's map)
  mod.rs collections.rs query.rs sql.rs streams.rs events.rs hot.rs internal.rs   Task 11 (REST deleted)
crates/loams/tests/connect_{sql,streams,links,internal,compat,auth,admin,operations}.rs
crates/loams/tests/{route_map.rs,curl_examples.rs}
crates/loams-console-mock/                                        Task 7 (Connect, not OpenAPI)
crates/loams-apps-mock/                                           Task 15 (fault endpoints on Connect)
crates/loams-facade-gen/src/{index.rs,lib.rs}                     Tasks 13–14
api/console/openapi.json                                          Task 11 (deleted)
web/apps/console/src/api/                                         Task 10 (Connect clients)
web/plugins/data-studio/src/client.ts                             Task 10
web/packages/proto/                                               Tasks 1–8 (regenerated @loams/proto)
apps/desktop-electron/src/main/protocol/local-shim.ts            Task 10
sdks/fixtures/{manifest.json,index.json,faults.json,facade-index.json,recorded/}   Tasks 13–15
sdks/conformance/                                                 Tasks 15, 23
sdks/templates/<lang>/template.env                                Task 14 (all 13)
sdks/<lang>/                                                      Tasks 17–22, 24
docs/api/{reasons.md,route-map.md,migrate.md,curl.md,reference/,snippets/}   Tasks 1–12, 26
docs/sdk/{runtime-contract.md,fixtures.md,toolchain-2026-10.md,versioning.md}   Tasks 0, 16, 24
docs/release/publishing.md                                        Task 25
scripts/sdk/{gen.sh,drift.sh,check-pins.sh,pins.lock,release-dry-run.sh}       Tasks 13–14, 25
.github/workflows/{ci.yml,sdk-release.yml,sdk-mirror.yml,api-protos.yml}        Tasks 12, 19, 23, 25
.github/CODEOWNERS                                                Task 12 (reasons.md, Q612)
```

## Shared contracts (all tasks use these names)

### The package catalogue (`CATALOGUE` in `crates/loams/src/api/connect.rs`)

`GetInstance.services[]` reports this table, with `available` decided per build and per runtime. A package with no proto has no row (the existing rule).

| Package | Services | Module(s) (`ModuleOptions.name`) | `unstable` | Served by | Task |
|---|---|---|---|---|---|
| `loams.instance.v1` | `InstanceService` | `instance` | no | always | exists |
| `loams.collection.v1` | `NamespaceService`, `CollectionService`, `DocumentService`, `QueryService` | `collections`, `documents`, `search` + `vector` | no | always | exists; facade calls Task 13 |
| `loams.sql.v1` | `SqlService` | `sql` | no | always | 1 |
| `loams.stream.v1` | `StreamService` | `streams` | no | always | 2 |
| `loams.link.v1` | `LinkService` | `links` | no | always | 3 |
| `loams.auth.v1` | `AuthService` | `auth` | no | always | 6 |
| `loams.admin.v1` | `OrgService`, `ProjectService`, `AgentService`, `KeyService`, `AuditService` | `admin.org`, `admin.projects`, `admin.agents`, `admin.keys`, `admin.audit` | no | always | 7 |
| `loams.operations.v1` | `OperationsService` | `operations` | no | always | 8 |
| `loams.live.v1` | `LiveService` | `live`, `tables` | yes | feature `live` (LV1) | 9 (wiring only) |
| `loams.graph.v1` | `GraphAdminService`, `GraphService` | `graph` | yes (GR1e) | feature `graph` | exists |
| `loams.postgres.v1` | `PostgresService` | `postgres` | yes (PG2 GA) | feature `postgres` (PG2 Task 9) | 9 (wiring only) |
| `loams.approvals.v1`, `loams.devices.v1`, `loams.notifications.v1` | AP0 | `approvals`, `devices`, `notifications` | no | `loams-apps-mock`; the server when AP1 serves them | no change |
| `loams.internal.v1` | `InternalService` | none (no SDK) | n/a | `internal_router` only | 4 |

Not in this table, and never in a public proto or SDK: the meter protocol, `loams.durable.v1` (API2-Q3: Resonate is the durable API), `loams.live.worker.v1` (LV1, internal), connector admin.

### Proto rules (Tasks 1–8 write the protos; this is the contract)

```proto
// Every new service:
service SqlService {
  option (loams.options.v1.module) = { name: "sql" summary: "SQL over collections and streams." };
  rpc Query(QueryRequest) returns (QueryResponse) {
    option idempotency_level = NO_SIDE_EFFECTS;      // a read: retry-safe, Connect GET allowed
    option (loams.options.v1.facade) = { name: "query" };
  }
  rpc QueryArrow(QueryArrowRequest) returns (QueryArrowResponse) {   // Q608; Task 1
    option idempotency_level = NO_SIDE_EFFECTS;
    option (loams.options.v1.facade) = { name: "query_arrow" };
  }
}
// A mutation that is not idempotent by construction: `string idempotency_key = 15;`
//   (field 15 in every package from API2 on; DocumentService's existing field 5 stays).
// A list RPC: `int32 page_size = 1; string page_token = 2;` in, `string next_page_token` out,
//   and `facade = { pagination: "<items>:next_page_token" }`.
// A read that takes consistency: `loams.collection.v1.Consistency consistency = <n>;`
// A write that produces state: `string consistency_token = <n>;` in the response, and the
//   same value in the `loams-consistency-token` response header.
```

### Headers (exact names)

| Header | Direction | Meaning |
|---|---|---|
| `authorization: Bearer <token>` | request | the only credential carrier (D608); a token in the query string is refused (Task 6) |
| `loams-consistency-token` | both | the write's token (response); the read's `at_least` (request, when the message field is empty) |
| `loams-request-id` | response | a ULID per call, also in `ErrorInfo.metadata["request_id"]` (Task 16) |
| `Loams-Hot` | request | the existing hot-tier hint (unchanged) |
| `loams-test-fault`, `loams-test-fault-after` | request | fault injection, **fixture server and `loams-apps-mock` only**; the real server ignores them (Task 15 test) |

### Rust seams (`crates/loams`)

```rust
/// Main-port authentication (Task 6). One implementation per deployment kind.
#[async_trait]
pub trait Authenticator: Send + Sync + 'static {
    /// `Ok(None)` only for an RPC on the unauthenticated allowlist.
    async fn authenticate(&self, rpc: &RpcPath, headers: &HeaderMap) -> Result<Option<Principal>, AuthError>;
}
pub struct Principal { pub subject: String, pub org: String, pub kind: PrincipalKind, pub scopes: Vec<String> }
pub enum PrincipalKind { User, Agent, ServiceAccount, ApiKey, DevLoopback } // API2-Q1: ApiKey and DevLoopback ship first; User/Agent/ServiceAccount arrive with MT1's JwtAuthenticator
pub enum AuthError { Missing, Malformed, Expired, Revoked, Unknown } // -> unauthenticated + reason

/// The unauthenticated allowlist (Task 6), the only RPCs answered without a principal.
pub const PUBLIC_RPCS: &[&str] = &[
    "/loams.instance.v1.InstanceService/GetInstance",
    "/loams.auth.v1.AuthService/ListProviders",
    "/loams.auth.v1.AuthService/CompleteSetup",   // first-run only; loopback or setup token
    "/grpc.health.v1.Health/Check",
    "/grpc.health.v1.Health/Watch",
];

/// The cluster-wide idempotency ledger (Task 4). Replaces the per-process map.
#[async_trait]
pub trait IdempotencyStore: Send + Sync + 'static {
    /// Takes the key's gate; returns the stored answer if the same fingerprint already completed.
    async fn begin(&self, scope: &IdemScope, key: &str, fingerprint: [u8; 32]) -> Result<IdemBegin, IdemError>;
    async fn complete(&self, ticket: IdemTicket, answer: Bytes) -> Result<(), IdemError>;
}
pub enum IdemBegin { Fresh(IdemTicket), Replay(Bytes), FingerprintMismatch }
// Window: 24 h. Backed by the metastore (`loams-meta`; TiKV when `--meta tikv://`).
```

### The facade index (`sdks/fixtures/facade-index.json`, Task 13)

Generated by `protoc-gen-loams-facade lang=index`, committed, and drift-checked. One entry per facade call:

```json
{ "protoRev": "<git sha of proto/>",
  "calls": [ { "module": "vector", "call": "search", "rpc": "loams.collection.v1.QueryService/Search",
               "retry": "safe|keyed|never", "idempotencyKeyField": "", "pagination": null,
               "stream": false, "unstable": false } ] }
```

`retry` follows D610: `safe` for `NO_SIDE_EFFECTS`/`IDEMPOTENT` or `retry_safe: true`; `keyed` when the request has `idempotency_key`; `never` otherwise. A hand-written facade's test (Task 14) loads this file and asserts its module/call set and retry classes are equal to it.

### The runtime contract

`docs/sdk/runtime-contract.md` clauses R1–R10 are the contract. API2 changes only what Task 16 says: R2 gains `RetryInfo`, R4 gains the token encoding and the session store, and R8 gains `request_id`. A clause number is never reused.

---

## Execution order

1. Task 0.
2. **API2a** (Tasks 1–9). Tasks 1–5 and 8 are independent of each other. Task 6 does not wait for MT1 (API2-Q1: API keys and dev-loopback first). Task 7 depends on Task 6. Task 9 runs last, after LV1's main-port task and PG2 Task 9 if they have merged, and again when either merges.
3. **API2b** (Tasks 10–12) after Tasks 6 and 7, whether or not MT1 has merged (API2-Q1). Task 11 after Task 10.
4. **API2c** (Tasks 13–18) can start at once, beside API2a. Task 13 needs each package's proto, so it lands per package. Task 15's recordings follow the packages they record.
5. **API2d** (Tasks 19–23). Task 19 at once. Tasks 20–22 after Tasks 13–16. Task 23 after Task 9.
6. **API2e** (Tasks 24–28) after API2d's wave 1 (Task 20). Task 28 last.

---

### Task 0: Reconcile with the code as built

**Files:** this plan's "Rulings made during execution" only.

Steps:
1. Answer each of the following and record the answer, with file paths and the `dev` commit, as a ruling:
   - Is this plan's "as built" still true? Check API1 Task 4's status: `QueryService/Search` in `crates/loams/src/api/connect_query.rs`, and whether `ScrollDocuments` is unary-with-cursor. If so, record it as the answer to Q614 unless the owner has ruled otherwise.
   - Which REST routes exist on `dev` today (`crates/loams/tests/route_map.rs`'s inventory)? `docs/api/route-map.md` has 27 `/v1` pairs plus `list_streams` and `list_links`, which §44 §5.1 omits. Tasks 2 and 3 add `ListStreams` and `ListLinks` for them.
   - Has MT1 merged anything? Look for a token endpoint (`/oauth/token`), sessions, a principal store and main-port TLS (MT1 Task 7). Either way (API2-Q1), Task 6 ships `ApiKeyAuthenticator` (keys in the metastore, hashed with argon2id) and `DevLoopbackAuthenticator` (`loams dev` on loopback only); `JwtAuthenticator` and the HTTP protocol endpoints are added only if MT1's issuer has merged, otherwise MT1 adds them through the `Authenticator` hook.
   - Has LV1's "Live on the main port" task merged (`LiveAbsent` replaced)? Has PG2 Task 9 merged (`PostgresService` registered, `Reflector::with_services`)? Record which wiring Task 9 still owns.
   - Which durable control surface exists in `crates/loams-durable`? Record it for reference only: API2-Q3 rules that the Resonate HTTP protocol stays the only durable API, so Task 8 adds no durable RPC.
   - Where is the MCP server (§44 §6 lists it on 8083)? On 2026-10-10 no crate under `crates/` serves MCP. Record that MCP stays out of scope here (M1.6).
   - Which SDKs have a generated facade and which a hand-written one? On 2026-10-10: TypeScript, Python and Rust are generated. Go generates `gen/facade` and hand-writes `modules.go` (D730). The other nine are hand-written. Record the list, and record which `sdks/<lang>` have a CI job: TypeScript, Rust, Python, Go, C++, C# and Swift do; Java, Kotlin, Ruby, PHP, Dart and Objective-C do not.
   - Naming drift. `sdks/README.md` says the Rust crate is `loams-sdk` and the Go module is `github.com/ostrium-labs/loams/sdks/go`. `sdks/rust/Cargo.toml` says `loams` and `sdks/go/go.mod` says `loams.dev/go`. Record that the manifests win, and add the README fix to Task 28.
   - Which `sdks/fixtures/manifest.json` notes are stale? For example, R4's "No RPC carries a `consistency_token`" was true before API1 Task 3 and is not now. List them for Task 15.
   - Q600–Q614 and API2-Q1–Q5: record the "Owner rulings 2026-10-10 (defaults)" table as rulings, plus any later owner answer that overrides one.
2. Write `docs/sdk/toolchain-2026-10.md` (SDK1 Task 0): one row per language with the Connect or gRPC library, its latest release and date, the remote plugin and its version, the registry, and whether it has trusted publishing. Apply D612's 12-month rule and note any language that changes column. Test: `toolchain_table_has_row_per_language` (a node test in `sdks/conformance/check-languages.mjs`).
3. Commit `docs(api2): task 0 rulings and the toolchain table`.

## API2a — The API surface, complete (Tasks 1–9)

### Task 1: `loams.sql.v1`

**Files:** create `proto/loams/sql/v1/sql.proto`, `crates/loams/src/api/connect_sql.rs` and `crates/loams/tests/connect_sql.rs`. Modify `crates/loams-proto/build.rs` (package list), `connect.rs` (catalogue row) and `docs/api/{reasons.md,route-map.md}`.

**Interfaces:**
- `SqlService/Query`: `namespace`, `sql`, `params` (`repeated google.protobuf.Value`), `consistency`, `max_rows` (default 10 000, cap 100 000). It answers `columns[]` (name, Arrow type string), `rows[]` (`google.protobuf.ListValue`), `truncated` and `consistency_token`. It calls the same function `api/sql.rs::sql` calls.
- `SqlService/QueryArrow` (Q608: yes). It answers `bytes arrow_ipc` (an Arrow IPC stream), with a cap of `--sql-arrow-max-bytes` (default 16 MiB). Over the cap it refuses with `resource_exhausted` and reason `result_too_large`, and `metadata.use = "flight"`.
- Flight SQL is unchanged.

Tests:
- Each test of `it/` that covers `POST /v1/namespaces/{ns}/sql`, ported with an `_rpc` suffix.
- `query_rows_match_rest`: the same SQL gives equal rows through REST and RPC while both exist.
- `query_arrow_roundtrips`: the IPC bytes decode with `arrow-ipc` to the same batches.
- `query_arrow_over_cap_refused`
- `query_is_get_cacheable`: a Connect `GET` with `encoding=json` works (`NO_SIDE_EFFECTS`).
- `flight_sql_still_serves`: the existing Flight test, unchanged, still passes.

Steps: tests (FAIL: package missing) → proto → generate (`@loams/proto` too) → handler → PASS → route-map rows → commit `feat(api): loams.sql.v1`.

### Task 2: `loams.stream.v1` on connect-rust

**Files:** move `crates/loams-stream-grpc/proto/loams/stream/v1/stream.proto` to `proto/loams/stream/v1/stream.proto`, keeping `Produce` and `ProduceCloudEvents` wire-compatible. Create `crates/loams/src/api/connect_streams.rs` and `crates/loams/tests/connect_streams.rs`. Modify `crates/loams-stream-grpc` so that it keeps only the CloudEvents gRPC binding (`io.cloudevents.v1`), which is compat.

**Interfaces:**
- `CreateStream` (idempotent by name), `DescribeStream`, `ListStreams` (paged), `Produce` (`producer_id`, `producer_seq`; never auto-retried without them), `Fetch` (offset, `max_records`, `max_bytes`; unary), `ProduceCloudEvents`, `FetchCloudEvents`.
- Records are `bytes`.
- The tonic `StreamService` on `--stream-grpc-listen` is removed, and the flag prints `--stream-grpc-listen is removed; StreamService is served on --listen`.

Tests:
- Ported REST stream tests.
- `stream_grpc_events.rs`, re-targeted at the main port and passing.
- `produce_without_key_is_not_retried_by_server`: two identical unkeyed produces append twice.
- `produce_with_producer_seq_dedupes`
- `fetch_paginates_by_offset`
- `cloudevents_http_binding_unchanged`: compat.
- `proto_wire_compatible_with_tonic_era`: messages encoded by the old generated types decode with the new ones.

Commit `feat(api): loams.stream.v1 on the main port`.

### Task 3: `loams.link.v1`

**Files:** `proto/loams/link/v1/link.proto`, `crates/loams/src/api/connect_links.rs` and `crates/loams/tests/connect_links.rs`.

**Interfaces:** `CreateLink` (idempotent by name; the link spec of `api/mod.rs::create_link` as a message, not a `Struct`, unless Task 0 finds it open-ended), `DescribeLink` and `ListLinks` (paged).

Tests: the ported REST link tests, plus `create_link_repeat_is_safe` and `list_links_paginates`.

Commit `feat(api): loams.link.v1`.

### Task 4: `loams.internal.v1` and the cluster-wide idempotency ledger

**Files:** create `proto-internal/buf.yaml` and `proto-internal/loams/internal/v1/internal.proto`. Modify `crates/loams-proto` (feature `internal`, off by default; `crates/loams` enables it). Create `crates/loams/src/api/connect_internal.rs` and `crates/loams/src/api/idempotency_store.rs`. Modify `api/internal.rs` (deleted at the end of this task), `cluster.rs` (callers) and `connect_documents.rs` (switch to the store). Tests: `crates/loams/tests/connect_internal.rs`, plus the cluster suite (`tests/cluster.rs`).

**Interfaces:**
- `InternalService`: `ForwardRead`, `NodeStats`, `HotStatus` and `HotWarm`, mapped one to one from `HOT_STATUS_PATH`, `HOT_WARM_PATH`, `NODE_STATS_PATH` and the forwarded-read route.
- It is served only by `internal_router`, and it keeps today's cluster-token check (mTLS when MT adds it).
- The metastore Raft transport moves to `loams.internal.v1` only if openraft's wire types allow it (API1 rulings 0.1–0.4). Otherwise it stays and is recorded.
- `IdempotencyStore` as in the shared contracts, with a 24 h window. It is backed by `loams-meta`, and by TiKV under `--meta tikv://`. `WriteDocuments` uses it. The per-process `Ledger` stays only as the store's in-flight gate.

Tests:
- `cluster_tests_use_internal_rpc`: `tests/cluster.rs` passes with `/internal/*` deleted.
- `internal_service_not_on_public_port`: every `/loams.internal.v1.*` path on `--listen` is a 404 (not routed), and the descriptor is not in public reflection.
- `internal_requires_cluster_token`
- `keyed_write_replays_across_nodes`: two gateways; the retry lands on the second and replays the first answer.
- `same_key_other_fingerprint_is_fresh_write` (the existing rule, kept).
- `ledger_window_expires_at_24h` (with a mock clock).
- `internal_module_excluded_from_sdk_generation`: no `sdks/**` output names `loams.internal`.

Commit `feat(api): loams.internal.v1 on the cluster listener; cluster-wide idempotency`.

### Task 5: Compat gRPC on the main port (D601)

**Files:** `crates/loams/src/api/connect_compat.rs` and `crates/loams/tests/connect_compat.rs`. Modify `server.rs` to build the tonic `Routes` for Qdrant gRPC and Flight SQL once, and to mount them on both their own port and the main router.

**Interfaces:**
- Routed by path prefix on the main router: `/qdrant.` goes to Qdrant's tonic server and `/arrow.flight.protocol.` goes to Flight. This happens before the connect router; the Connect paths cannot collide (`/loams.`).
- It uses gRPC only. gRPC-Web is not offered for compat.
- The main-port mount passes through the `Authenticator` (Task 6). The conventional ports keep their current auth until MT1 (API2-Q5), recorded as an accepted risk in Task 27.

Tests:
- `qdrant_grpc_client_works_on_main_port`: the Qdrant Rust client from `tests/qdrant`.
- `flight_client_works_on_main_port`
- `qdrant_grpc_bytes_unchanged_on_main_port`: the same request on both ports gives byte-equal responses.
- `compat_grpc_on_main_port_requires_auth`
- `conventional_ports_unchanged`: the existing suites.

Commit `feat(api): Qdrant gRPC and Flight SQL on the main port`.

### Task 6: Authentication on the main port, and `loams.auth.v1`

**Files:** `crates/loams/src/api/{auth.rs,connect_auth.rs}`, `proto/loams/auth/v1/auth.proto` and `crates/loams/tests/connect_auth.rs`. Modify `connect.rs` (`WhoAmI` answers the principal), `server.rs` (the layer) and `main.rs` (`--auth`).

**Interfaces:**
- `Authenticator`, `Principal` and `PUBLIC_RPCS` as in the shared contracts.
- Implementations:
  - `ApiKeyAuthenticator`: keys `lk_<26-char ULID>_<32 base62>`, stored as argon2id hashes in the metastore under the org.
  - `DevLoopbackAuthenticator`: only when `loams dev` listens on loopback. It yields `PrincipalKind::DevLoopback` and is refused at startup on any other address.
  - `JwtAuthenticator`: verifies the instance's own access tokens against its JWKS (D447, D449), when MT1 provides the issuer. Not required for this task to land (API2-Q1).
- `--auth {api-key,jwt,dev}`: `dev` is the default for `loams dev` (loopback only), and `api-key` otherwise; `api-key,jwt` becomes the default once MT1's issuer is configured. `--auth jwt` without an issuer refuses to start with a clear message.
- The `Authenticator` trait is the only hook MT1 needs: no handler, router or test in this task assumes which implementations exist.
- `AuthService`: `ListProviders`, `CompleteSetup`, `GetSession`, `CreateSession`, `DeleteSession` and `DecideConsent` (§44 §5.2). Sessions are cookie or bearer. `CreateSession` sets an `HttpOnly; Secure; SameSite=Lax` cookie on the Connect response.
- **Protocol endpoints that stay HTTP** (D602): `/oauth/token`, `/oauth/authorize`, `/.well-known/{oauth-protected-resource,oauth-authorization-server,jwks.json}` and `/auth/oidc/{provider}/{start,callback}`. They are served only when MT1's issuer is present. Otherwise they are absent, not stubbed.
- Errors are `unauthenticated` with one of the reasons `token_missing`, `token_malformed`, `token_expired` (exists), `token_revoked` and `credential_in_query_string`, and the `WWW-Authenticate: Bearer` header on HTTP.

Tests:
- `every_served_rpc_requires_auth_except_allowlist`: walks `CATALOGUE` and the descriptor set, calls every served RPC with no credential over Connect, gRPC and gRPC-Web, and asserts `unauthenticated` except for `PUBLIC_RPCS`.
- `api_key_round_trip`
- `expired_jwt_gets_token_expired`: the reason that R1's refresh keys on.
- `query_string_token_refused`: `?access_token=` and `?token=` are refused with `credential_in_query_string`.
- `bearer_never_logged`: captures `tracing` output and error bodies across the suite, and asserts the token bytes are absent.
- `dev_loopback_refused_on_public_address`
- `session_cookie_and_bearer_both_work`
- `oidc_start_remains_http_redirect`: only with MT1; `#[ignore]` with the reason otherwise.
- `jwt_without_issuer_refused_at_startup`
- `authenticator_is_swappable`: the suite passes with a test `Authenticator` injected, proving MT1 can add one without handler changes.

Commit `feat(api): main-port authentication and loams.auth.v1`.

### Task 7: `loams.admin.v1`, and the console mock on Connect

**Files:** `proto/loams/admin/v1/{org,project,agent,key,audit}.proto`, `crates/loams/src/api/connect_admin.rs` and `crates/loams/tests/connect_admin.rs`. Modify `crates/loams-console-mock` to serve the same services over Connect, removing its `/api/v1` routes in Task 11 (not here).

**Interfaces:**
- The RPCs of §44 §5.2's table. `KeyService/CreateKey` returns the key once; a replay returns `secret_already_issued` (the PG2 reason).
- `AuditService/WatchAuditEvents` streams a snapshot cursor, then events, then a heartbeat every 15 s.
- Authorization: every admin RPC checks the principal's org and role through an `Authorizer` trait. It is OpenFGA when MT1 lands; until then it is `OrgRoleAuthorizer` (owner/admin/member from the metastore).

Tests:
- Each console-mock route test, ported with `_rpc`.
- `create_key_returns_secret_once`
- `member_cannot_create_key`: `permission_denied`, reason `role_required`.
- `watch_audit_resumes_from_cursor`
- `mock_and_server_answer_alike`: the console-mock and the server give equal responses (up to volatile fields) on a scripted session, so the console can test against either.

Commit `feat(api): loams.admin.v1`.

### Task 8: Operations on the main port

**Files:** `crates/loams/src/api/connect_operations.rs` and tests `crates/loams/tests/connect_operations.rs`. No `connect_durable.rs` and no `proto/loams/durable/` (API2-Q3).

**Interfaces:**
- `OperationsService` (`GetOperation`, `ListOperations`, `WatchOperations`, `CancelOperation`) is served on the main port over one `OperationStore` that graph and postgres write to. The `loams.operations.v1` row goes into `CATALOGUE`.
- **No `loams.durable.v1`** (API2-Q3). The Resonate HTTP protocol stays the only durable API and is unchanged (compat). SDK users reach durable functions through the Resonate SDKs; a Loams durable package is a future plan under this plan's rules.

Tests:
- `operations_list_includes_graph_and_postgres_ops` (the features on).
- `watch_operations_snapshot_then_changes`
- `cancel_is_idempotent`
- `resonate_protocol_unchanged`: the existing durable suite.
- `no_durable_package_served`: `GetInstance.services[]` and reflection list no `loams.durable.*`.

Commit `feat(api): operations on the main port`.

### Task 9: Catalogue completeness, served-only reflection and variants

**Files:** `crates/loams/src/api/connect.rs` and `crates/loams/tests/connect_api.rs`.

**Interfaces:**
- `CATALOGUE` holds every row of the shared table.
- Reflection uses `Reflector::with_services(served_services())` (PG2 R1.4), so an unserved service is never listed. Reflection stays on in `loams dev` and off elsewhere unless `--reflection` is passed (Q603, API1 ruling 1.4).
- `--reflection` is added, and requires authentication.
- Live and Postgres: if LV1's and PG2 Task 9's wiring has merged, only the catalogue and test rows change here. If not, this task leaves `LiveAbsent` and the Postgres absent stub in place and records it.
- `VARIANT` follows CLI2's `release/variants.toml` once it exists (API1 ruling 1.5).

Tests:
- `reflection_lists_only_served_services`
- `reflection_off_by_default_outside_dev`
- `catalogue_matches_design_and_protos`: extends `route_map.rs`'s design guard. Every proto package under `proto/` that has a service has a row, and every row has a proto.
- `unavailable_service_reports_reason`: for every row whose feature is off.
- `get_instance_reports_unstable`

Commit `feat(api): catalogue complete, served-only reflection`.

## API2b — Removing REST and OpenAPI (Tasks 10–12)

### Task 10: The console, data studio and the desktop on Connect

**Files:**
- `web/apps/console/src/api/{client.ts,use.ts}`, the pages that call it (`shell.tsx`, `session.tsx`, `pages/{home,org,project,environment,agents,auth}.tsx`) and `runtime-config.ts`.
- `web/apps/console/package.json`: remove `openapi-fetch`, `openapi-typescript` and the `gen:api` script.
- Delete `web/apps/console/src/api/schema.d.ts`.
- `web/plugins/data-studio/src/client.ts`.
- `apps/desktop-electron/src/main/protocol/local-shim.ts`: it answers the Connect paths `InstanceService/GetInstance` and `AuthService/GetSession` instead of `/api/v1/instance` and `/api/v1/session`.

**Interfaces:** one `createLoamsTransport(runtimeConfig)` in `web/apps/console/src/api/client.ts`, over `@connectrpc/connect-web`, with the clients from `@loams/proto`. Data studio uses `QueryService`, `SqlService` and `CollectionService` through the same transport.

Tests:
- The console's existing tests, re-targeted, run in `web` and `web-browser`.
- `console_has_no_openapi_dependency`: `package.json` and the lockfile have no `openapi-*`.
- `data_studio_uses_connect`: no `/v1/namespaces` string under `web/`.
- `desktop_local_shim_answers_connect`: in `apps/desktop-electron`'s test suite.

Commit `feat(console): Connect clients replace the OpenAPI client`.

### Task 11: Delete the native REST and the OpenAPI (no shims)

**Files:**
- Delete the REST handlers in `crates/loams/src/api/{mod.rs,collections.rs,query.rs,sql.rs,streams.rs,events.rs,hot.rs}`, keeping the helpers the Connect handlers use, moved next to them. `/health` and `/ready` stay.
- Delete `api/console/openapi.json` and the console mock's `/api/v1` routes.
- Create `docs/api/migrate.md` (generated from the route map by `scripts/api/migrate-page.sh`). No `legacy.rs` (Q600).
- Amend the status lines of M1.6 and AP1a, one line each.

**Interfaces:**
- Delete outright (Q600). There is no `--legacy-rest` flag and no shim: a removed path is answered by the framework's unknown-path 404, with the JSON error body.
- `docs/api/migrate.md` maps every removed route to its RPC, and the release notes link it.

Tests:
- `no_native_rest_route_remains`: the router inventory equals the protocol endpoints of D602 plus the Connect and compat paths.
- `route_map_covers_every_route`: still green, every row now "removed" or "protocol".
- `removed_rest_paths_are_404`: every removed route of the route map answers 404, never data.
- `no_legacy_rest_flag`: `--legacy-rest` is not a recognised flag.
- `migrate_page_covers_every_removed_route`
- `framework_rejections_use_the_json_error_body`: still holds for unknown paths.
- `compat_suites_green`
- The console's tests.

Commit `feat(api): remove the native REST and the console OpenAPI`.

### Task 12: Contract gates

**Files:** `.github/workflows/api-protos.yml` (or a job in `ci.yml`, as `app-protos` is), `docs/api/curl.md`, `crates/loams/tests/curl_examples.rs`, `.github/CODEOWNERS` and `buf.yaml`.

**Interfaces:**
- `buf lint` on `proto/` and `proto-internal/`.
- `buf breaking` against `dev` until a tag `v*` exists, then against the latest tag. It covers every package not listed as `unstable`.
- CODEOWNERS on `docs/api/reasons.md` and `proto/**` (Q612).

Tests:
- `breaking_change_fails_ci`: a fixture proto pair under `crates/loams-proto/tests/breaking/`.
- `unstable_flag_and_buf_ignore_agree`: every `ModuleOptions.unstable` package is in `breaking.ignore`, and nothing else is.
- `curl_examples_run`: runs every block of `docs/api/curl.md` against `loams dev`.
- `reasons_are_snake_case_and_unique` (exists).
- `no_reason_removed_since_last_tag`

Commit `ci(api): breaking-change, curl and reason gates`.

## API2c — The pipeline, complete (Tasks 13–18)

### Task 13: Facade annotations on every stable package, and the facade index

**Files:** the protos of `loams.collection.v1`, `sql`, `stream`, `link`, `auth`, `admin` and `operations`. Also `crates/loams-facade-gen/src/{index.rs,lib.rs}`, `sdks/fixtures/facade-index.json`, every `sdks/<lang>/buf.gen*.yaml` package map, and the regenerated `facade.ts`, `facade.py`, `facade.rs` and Go `gen/facade`.

**Interfaces:**
- Each stable RPC carries `FacadeOptions`. API1 ruling 2.5's blocker (stubs before options) is resolved by adding the package map entries and the stubs in the same PR.
- `QueryService/Search` carries two entries: `{module:"vector" name:"search"}` and `{module:"search" name:"query"}` (Q613: both).
- List RPCs carry `pagination`.
- `lang=index` renders `facade-index.json`.

Tests:
- `every_stable_service_has_module`
- `every_stable_rpc_has_facade_call`
- `vector_and_search_share_one_rpc` (exists)
- `retry_class_follows_idempotency_level` (exists)
- `pagination_iterator_for_list_rpcs`
- `golden_index`
- `drift.sh` clean for all four generated languages.

Commit `feat(facade): every stable package annotated; the facade index`.

### Task 14: The other nine languages against the index (Q604)

**Files:** `sdks/templates/<lang>/template.env` for Swift, Kotlin, Java, C#, Dart, Ruby, PHP, C++ and Objective-C; `crates/loams-facade-gen/src/{swift,kotlin,java,csharp}.rs`; and a facade-index test in each hand-written SDK's test tree.

**Interfaces:**
- Q604 (ruled): Swift, Kotlin, Java and C# are **generated** (a renderer in `loams-facade-gen`, `golden_<lang>`); Dart, Ruby, PHP, C++ and Objective-C are **hand-written against the index** (D730's fallback).
- Go's `modules.go` is replaced by a generated file when `go.rs` renders modules, which is part of this task.

Tests:
- `<lang>_facade_matches_index`, for every hand-written language: the SDK's module/call set and per-call retry classes equal `facade-index.json`'s.
- `golden_<lang>`, for every generated language.
- `go_modules_generated`: `modules.go` carries the generated header, and `drift.sh` covers it.

Commit `feat(facade): nine languages checked against the index` (one commit per language is fine).

### Task 15: The fixture corpus covers the API

**Files:** `sdks/fixtures/{manifest.json,index.json,faults.json,recorded/}`, `sdks/conformance/{record-fixtures.mjs,faults.mjs,fixture-server.mjs,verify-corpus.mjs}`, `crates/loams-apps-mock` (fault RPCs on Connect) and `docs/sdk/fixtures.md`.

**Interfaces:**
- Recordings from `loams dev`, in the four encodings (Connect JSON, Connect proto, gRPC-Web, gRPC-Web JSON), for every stable package: namespaces and collections; documents (keyed write and replay; consistency token returned); search (dense, text, hybrid, filters); SQL; streams (produce/fetch); links; admin (with a dev principal); operations.
- Faults: retryable `unavailable` with `RetryInfo`, `resource_exhausted` with `RetryInfo`, `deadline_exceeded` before the response starts, a mid-stream disconnect after *n* frames, and `token_expired` followed by a good call. These make `mock_injects_retryable_errors` real.
- The stale manifest notes from Task 0 are rewritten. R2, R4 and R6 become `covered: true`.

Tests:
- `fixtures_pass_against_real_server`: a Rust test runs every non-fault fixture against `loams dev`.
- `mock_injects_retryable_errors`
- `every_clause_has_a_fixture`: every clause R1–R10 has at least one required fixture.
- `fixtures_carry_no_credentials`: no recorded header or body holds a bearer, key or cookie value.
- `real_server_ignores_test_fault_header`
- `verify-corpus.mjs --drift` clean.

Commit `test(sdks): the corpus covers every stable package and every clause`.

### Task 16: The runtime contract, v1

**Files:** `docs/sdk/runtime-contract.md`, `proto/loams/errors/v1/errors.proto` (documentation of the `google.rpc.RetryInfo` detail and of `request_id` in `metadata`), `crates/loams/src/api/connect_errors.rs` (emit them), and the consistency-token encoder in `crates/loams/src/api/connect_documents.rs` / `connect_query.rs`.

**Interfaces:**
- **R2.** The server attaches `google.rpc.RetryInfo` to `unavailable` and `resource_exhausted` when it knows a delay. SDKs honour it up to 30 s.
- **R4.**
  - The token is `v1:<base64url(postcard(TokenV1{instance_id, entries: [(stream_id, partition, offset)]}))>`.
  - `ConsistencyToken::merge` takes the maximum offset per `(stream_id, partition)`, and a token from another `instance_id` is refused with `invalid_argument`, reason `consistency_token_foreign`.
  - The session store is off by default (`session_consistency`).
- **R8.** Every error carries `metadata.request_id`, which equals the `loams-request-id` header.
- **R10.** Unchanged.

Tests:
- `token_merge_is_per_stream_partition_max`: a property test, server side, with the same vectors under `sdks/fixtures/tokens.json`.
- `foreign_instance_token_refused`
- `retry_info_present_on_unavailable`
- `request_id_header_matches_metadata`
- `produce_without_key_never_retried`: a fixture with `retry: never` that every SDK must pass.

Commit `feat(api): RetryInfo, request ids and the v1 consistency token`.

### Task 17: Typed query builders

**Files:** `sdks/{typescript/packages/client,python/src/loams,go,rust/src,java,kotlin,swift,csharp}/…/query_builder.*` and a shared fixture set `sdks/fixtures/builders/`.

**Interfaces:**
- Each builder is a thin layer over the generated `SearchRequest`: `dense(field, vector)`, `sparse(field, indices, values)`, `text(field, query)`, `filter(...)`, `fuse(rrf|weighted)`, `expand(graph)`, `limit`, and `consistency`.
- Each builder's output must equal the fixture's request JSON byte for byte, after canonical JSON.
- Dart, Ruby, PHP, C++ and Objective-C expose the generated messages, with documented examples (§44 §7.6).

Tests: `<lang>_builder_matches_fixtures` for each of the eight languages.

Commit `feat(sdks): typed hybrid-query builders` (one per language is fine).

### Task 18: Bulk data

**Files:** a `bulk` module in Python, Go, Java, C++, Rust and C#, and a `write_batch` helper in every language. Fixtures `sdks/fixtures/bulk_*`.

**Interfaces:**
- `write_batch` chunks `WriteDocuments`: 10 000 rows or 4 MiB, whichever comes first, one chunk in flight, and each chunk keyed (`<batch key>/<chunk index>`) so a retry replays.
- `bulk` uses Flight SQL `DoPut`/`DoGet` on the main port (Task 5) with the same bearer.
- `query_arrow` exists only where an Arrow IPC reader is available (TypeScript with `apache-arrow`, plus the six Flight languages).

Tests:
- `chunking_whole_or_nothing`
- `chunk_retry_replays_not_rewrites`
- `<lang>_bulk_put_get_roundtrip`: the six Flight languages.
- `query_arrow_matches_rows`

Commit `feat(sdks): write_batch and bulk`.

## API2d — Every language to production (Tasks 19–23)

### Task 19: A CI job for every language

**Files:** `.github/workflows/ci.yml` (jobs `sdk-java`, `sdk-kotlin`, `sdk-ruby`, `sdk-php`, `sdk-dart` and `sdk-objc`; the `changes` filter; `required.needs`).

**Interfaces:**
- Each job builds `loams` once and downloads it (the `sdk-typescript` pattern), runs `sdks/conformance/run.sh <lang>`, and checks the report against `required.mjs`.
- Wave 1 jobs run on every PR that touches `proto/**`, `sdks/<lang>/**`, `sdks/fixtures/**` or `crates/loams/src/api/**`, and are in `required.needs`.
- Waves 2 and 3 run nightly and on tags only, and are never in `required.needs` in this plan (API2-Q2). A nightly failure opens or updates one issue per language. Promoting a language to PR-blocking is a later, separate ruling.
- A PR that touches only `sdks/<lang>/**` of a wave 2 or 3 language runs that language's job on the PR as an advisory (non-required) check.
- The PHP job runs twice: with `ext-grpc`, and without it (Connect-unary).
- Ruby likewise.

Tests:
- `workflow_lint`: `actionlint` clean.
- `every_sdk_dir_has_a_job`: a script that fails when an `sdks/<lang>` directory has no `sdk-<lang>` job.
- `only_wave1_is_required`: `required.needs` holds exactly the wave 1 jobs.

Commit `ci(sdks): a conformance job for every language`.

### Task 20: Wave 1 (TypeScript, Python, Go, Rust) on the whole API

**Files:** `sdks/{typescript,python,go,rust}`, with their READMEs and examples.

**Interfaces:**
- Every module in `facade-index.json`.
- Python: sync and async clients, `mypy --strict`, and extras `flight`, `arrow` and `polars`. It replaces M1.6's `loams-client`.
- Go: context-first, with `*Stream` helpers.
- Rust: an async `Stream`, and no `tonic`.
- TypeScript: Node ≥ 22, browsers, Deno and Bun (Deno no longer claimed or tested since D1790, 2026-10-23). `@loams/live` re-exports.
- Every `TokenSource` of D608 exists. `OidcExchange` and `WorkloadIdentity` are tested against MT1's token endpoint when it exists, and against the fault server's `token_expired` script otherwise.

Tests (per language, named `<lang>_…`):
- `conformance_all_required_fixtures`
- `retry_reuses_idempotency_key`
- `error_reason_mapping`
- `stream_resume_with_cursor`
- `token_source_refresh`
- `pagination_iterator`
- `session_consistency_read_your_writes`
- `request_id_on_error`

Commit `feat(sdk-<lang>): the whole API` (one per language).

### Task 21: Wave 2 (Swift, Kotlin, Java, C#)

**Files:** `sdks/{swift,kotlin,java,csharp}`.

**Interfaces:**
- Everything Task 20 lists, in each language's idiom (§44 §7.1).
- Swift: `AsyncSequence`, iOS 15+ / macOS 12+ / Linux.
- Kotlin: `Flow`, JVM and Android. It shares generation with AP2's `buf.gen.kotlin.yaml`, which this task folds into `sdks/kotlin/`.
- Java: Java 17+, blocking and async, `Iterator` streams, and `grpc-netty-shaded` as an optional artifact.
- C#: net8.0, `IAsyncEnumerable`, `AddLoams` DI, and gRPC-Web for Blazor WASM.
- `buf.gen.swift.yaml` and `buf.gen.kotlin.yaml` at the root move under `sdks/<lang>/` (SDK1 Task 1's fold).

Tests: the eight `<lang>_…` tests of Task 20, for each language.

Commit `feat(sdk-<lang>): the whole API` (one per language).

### Task 22: Wave 3 (Dart, Ruby, PHP, C++, Objective-C)

**Files:** `sdks/{dart,ruby,php,cpp,objc}`.

**Interfaces:**
- Task 20's list, with these per-language details:
  - Dart: the Connect library if Task 0's check passes, otherwise `grpc`.
  - Ruby and PHP: both transports (D613). `transport: grpc-only` fixtures are skipped on unary.
  - C++: C++17 and a CMake config package; a vcpkg registry (git) and an in-repo Conan recipe (Q610). Uploading to a hosted Conan remote is gated by `SDK_PUBLISH_CONAN` (Task 25).
  - Objective-C: generated sources and a SwiftPM target, and no CocoaPods (D614). If it fails conformance (Q611), the matrix row says "community".
- `sdks/php/vendor/` stays untracked (it is not in git on 2026-10-10) and is built by Composer in CI from `composer.lock`.

Tests: the eight `<lang>_…` tests, plus `ruby_unary_transport_conformance` and `php_unary_transport_conformance`.

Commit `feat(sdk-<lang>): the whole API` (one per language).

### Task 23: Protocol conformance

**Files:** `sdks/conformance/connect-conformance/` (config and runner), and a nightly CI job `connect-conformance`.

**Interfaces:**
- `connectrpc/conformance`, at a version pinned in `pins.lock`, runs in server mode against the main port's handler, and in client mode for every Connect SDK (TypeScript, Python, Go, Rust, Swift, Kotlin, Dart if Connect).
- Known failures go in an allowlist file, one line per case with a reason. The allowlist may only shrink.

Tests:
- `server_conformance_passes_or_allowlisted`
- `<lang>_client_conformance_passes_or_allowlisted`
- `allowlist_only_shrinks`

Commit `test(api): Connect protocol conformance`.

## API2e — Release (Tasks 24–28)

### Task 24: Versioning

**Files:** `docs/sdk/versioning.md` and `sdks/<lang>/LOAMS_PROTO_REV` (one line: the `proto/` tree's git sha), plus the R9 check in each runtime.

**Interfaces:**
- Each SDK declares `LOAMS_PROTO_REV` and the supported `api_versions` range.
- At first call, `GetInstance` is compared: a minor mismatch is a warning and a major mismatch is a `failed_precondition` error with reason `api_version_unsupported`.
- SDKs stay `0.y.z` while the server is pre-1.0.
- Each SDK keeps its own changelog under `sdks/<lang>/CHANGELOG.md`.

Tests:
- `<lang>_version_check` (all 13).
- `proto_rev_matches_tree`: CI fails when `LOAMS_PROTO_REV` is not the sha the generated code came from.

Commit `feat(sdks): proto revision and version checks`.

### Task 25: Release workflows and mirrors

**Files:** `.github/workflows/{sdk-release.yml,sdk-mirror.yml}`, `scripts/sdk/release-dry-run.sh` and `docs/release/publishing.md`.

**Interfaces:**
- Tags are `sdk-<lang>-v<semver>`, signed.
- There is one job per registry: npm, PyPI, crates.io, RubyGems, NuGet and pub.dev use OIDC trusted publishing; Maven Central uses a Portal token and a GPG key; Packagist uses a webhook from the mirror; Go and SwiftPM use mirror tags; vcpkg uses its own git registry and Conan its own remote (Q610).
- Every job runs `--dry-run` on PRs, and runs for real only when `vars.SDK_PUBLISH_<REGISTRY> == 'true'`. The owner sets that variable after §44 §13's action.
- Generated code is Apache-2.0 (Q609); each package's licence field says so.
- `sdk-mirror.yml` subtree-splits `sdks/go`, `sdks/swift` and `sdks/php` into `ostrium-labs/loams-{go,swift,php}`. The mirrors reject human pushes (a branch rule, recorded in the doc).
- `publishing.md`'s table is rewritten. Its "D400 defers Java", "no Go module" and "no Python SDK" rows are no longer true.

Tests:
- `workflow_lint`
- `dry_run_all_languages`: a PR run of `sdk-release.yml` in dry-run mode, green.
- `no_long_lived_token_where_oidc_exists`: a script check of the workflow's secrets usage.

Commit `ci(sdks): release and mirror workflows`.

### Task 26: Reference docs and snippets

**Files:** `docs/api/reference/` (generated), `docs/api/snippets/` (from the fixtures) and `buf.gen.docs.yaml`.

**Interfaces:**
- The doc plugin is pinned in `pins.lock`. It writes one Markdown page per package.
- Snippets are rendered from the fixtures for curl, Python, TypeScript and Go, and are used by the README quickstarts.
- The `loams-cloud` site rendering is a follow-up issue in that repository, not here.

Tests:
- `every_rpc_documented`: every public RPC has a non-empty comment.
- `snippets_match_fixtures`
- `docs_snippets_compile`: for TypeScript, Python, Go and Rust, without a server.

Commit `docs(api): generated reference and tested snippets`.

### Task 27: Security review

**Files:** `docs/security/api-and-sdks-threat-model.md`, plus the tests below in their packages.

**Interfaces:** a threat model covering:
- the main port: auth (API keys and dev-loopback before MT1, API2-Q1), reflection, gRPC-Web CORS, compat mounts and `loams.internal.v1` exposure;
- the conventional compat ports' pre-MT1 auth, as an owner-accepted risk (API2-Q5);
- the SDKs: token storage, redaction, TLS defaults (verify on; `http://` only to loopback unless `allow_insecure` is set), retry amplification and idempotency;
- release: OIDC scopes, mirror write rights, signing.

Tests:
- `sdk_token_source_debug_redacted` (all 13).
- `sdk_refuses_plain_http_to_non_loopback`: every language.
- `cors_allows_only_configured_origins`
- `retry_budget_caps_amplification`: every SDK retries at most 3 times per call, and the client-wide budget (10% of calls over 10 s) holds.
- Every high or critical finding is closed or recorded as an owner-accepted risk.

Commit `docs(security): API and SDK threat model, and its tests`.

### Task 28: Docs, plan statuses and the decision log

**Files:**
- `sdks/README.md` (the naming fix from Task 0).
- The language matrix page `docs/sdk/languages.md` (SDK2 Task 13).
- `CONTRIBUTING.md` ("adding a community SDK").
- The status lines of API1, SDK1 and SDK2, each with a pointer to API2.
- `docs/plans/README.md`.
- In `docs/design/13-decision-log.md`, the "Owner rulings 2026-10-10 (defaults)" for Q600–Q614 and API2-Q1–Q5 (the latter under new IDs), each dated 2026-10-10, plus any later override.

Tests:
- `docs` (existing CI job): links resolve.
- `languages_page_matches_manifest`: every language's row matches its conformance report.

Commit `docs(api2): status, matrix, and decisions`.

---

## Exit criteria for production (§44, with the owning tasks)

- [ ] **One API:** every application call is an RPC on the main port, the catalogue is complete, reflection is served-only, and every unavailable package refuses with a reason: Tasks 1–3, 6–9.
- [ ] **No bespoke REST:** the native REST, the console OpenAPI and `/internal/*` are deleted, the console, data studio and desktop use Connect, and no shim exists (Q600): Tasks 4, 10, 11.
- [ ] **Compat unchanged:** every compat suite is green, and Qdrant gRPC and Flight are also on the main port: Task 5.
- [ ] **Auth:** every served RPC needs a principal except the allowlist, with no token in a URL or a log: Task 6.
- [ ] **Idempotency** holds across gateway nodes: Task 4.
- [ ] **Contract gates:** `buf lint`, `buf breaking` against the tag, `unstable` agreement, reasons never removed, and the curl examples run: Task 12.
- [ ] **Facade from one source:** every stable RPC is annotated, and every SDK matches the index: Tasks 13, 14.
- [ ] **Corpus:** every stable package and every clause R1–R10 has a required fixture, and faults are injected: Task 15.
- [ ] **Runtime contract v1:** RetryInfo, request ids and the v1 token: Task 16.
- [ ] **Thirteen languages:** each passes 100% of the required fixtures in a CI job (wave 1 PR-required; waves 2 and 3 nightly, API2-Q2), with Connect protocol conformance for the Connect SDKs: Tasks 19–23.
- [ ] **Builders and bulk** where §44 §7.5–§7.6 say so: Tasks 17, 18.
- [ ] **Release:** version checks, dry-run publishing of every language, mirrors, and signed tags: Tasks 24, 25.
- [ ] **Docs:** the reference is generated, the snippets are tested, and there is a migration page: Tasks 11, 26, 28.
- [ ] **Security:** the threat model, the tests, and no open high or critical finding: Task 27.

## Self-review

- **Spec coverage.**

  | §44 section | Task(s) |
  |---|---|
  | §4 One API, one port | 5, 6, 9 |
  | §5 Removal and migration | 1–4, 7, 10, 11 |
  | §6 Compat surfaces | 5, (all: constraint) |
  | §7.1–§7.3 SDK shape, catalogue, annotations | 13, 14, 20–22 |
  | §7.4 Cross-cutting behaviour | 4, 6, 15, 16 |
  | §7.5 Arrow and bulk | 1, 18 |
  | §7.6 Builders | 17 |
  | §8 Services to add | 1–4, 6–8 |
  | §9 Language matrix | 0, 14, 19–22 |
  | §10 Generation, conformance, docs | 12–15, 23, 24, 26 |
  | §11 Publishing | 25 |
  | §12 Risks | 14 (risk 6), 19 (1), 22 (8, 10), 23 (2, 3) |
  | §13 Owner actions | 25 (gated by `SDK_PUBLISH_*`) |

- **Types.** `Authenticator`, `Principal`, `PUBLIC_RPCS`, `IdempotencyStore`, the facade index and the header table are defined once, in the shared contracts.
- **Review Focus.** Items 1–9 each name an owning test (Tasks 4, 16, 20–22, 6, 15, 27, 5, 11, 14, 13, 9, 12).
- **Not in this plan.** MCP (no server exists; M1.6), `loams.durable.v1` (API2-Q3), `loams.jobs.v1`, `loams.flow.v1`, `loams.git.v1`, `loams.systemone.v1`, `loams.collab/bot/factory.v1` and `loams.console.v1`. Each of those plans adds its package under this plan's rules (a proto, a catalogue row, the facade annotations and fixtures), and Task 13's tests catch an omission.

## Open questions

Every design question is ruled in "Owner rulings 2026-10-10 (defaults)" at the top. What is left needs the owner personally, because it involves accounts, money or legal standing. None blocks a code task: each only gates a real publish, and every release job stays a dry run until then (Task 25).

| # | Owner action | Why a human | Gates |
|---|---|---|---|
| API2-H1 | Create or reserve the registry accounts and namespaces of §44 §13 (npm `@loams`, PyPI, crates.io, RubyGems, NuGet, pub.dev, Maven Central with a GPG key, Packagist), configure OIDC trusted publishing, then set each `vars.SDK_PUBLISH_<REGISTRY>` | External accounts in the company's name | Task 25 real publishes |
| API2-H2 | Host a Conan remote (and decide whether to pay for one) for Q610's C++ packages, or leave C++ at the vcpkg git registry plus the in-repo recipe | External account, possible cost | `SDK_PUBLISH_CONAN` |
| API2-H3 | Create the mirror repositories `ostrium-labs/loams-{go,swift,php}` with the bot-only push rule, and serve the `loams.dev/go` `go-import` meta tag (Q607) from the `loams-cloud` site | GitHub org and domain administration | Task 25 mirrors, Go module path |
| API2-H4 | Confirm Q609 (generated code under Apache-2.0, no DCO on bot-written mirrors) with whoever handles the company's legal review | Legal | First real publish |

## Rulings made during execution

(Task 0 and later tasks append here.)

### Task 0: reconciliation with `dev` at `b555fe734866fcd6c7bb74b072dbd92b8c940139` (2026-10-13)

| # | Task | Ruling | Why / evidence |
|---|---|---|---|
| 0.1 | 0 | **"As built" holds for API1 Task 4, and Q614 is unary with a cursor.** `QueryService/Search` is served by `crates/loams/src/api/connect_query.rs` (`register`, `search`), is defined in `proto/loams/collection/v1/query.proto:174`, is in `CATALOGUE` under `loams.collection.v1` (`crates/loams/src/api/connect.rs`) and is covered by `crates/loams/tests/connect_query.rs`. `DocumentService/ScrollDocuments` (`proto/loams/collection/v1/document.proto:168`) is unary and `NO_SIDE_EFFECTS`, with `after`/`page_token` (one `DocumentId` cursor) and `limit`. **Q614 is answered: unary with a cursor.** A server-streaming scroll, if one is ever wanted, is a separate additive RPC, as the proto comment says. **Finding for Task 12:** the response's cursor field is `next` (a `DocumentId`), not AIP-158 `next_page_token`, and the request's size field is `limit`, not `page_size`. Task 12's AP0 lint either adds the AIP names as aliases (additively, as `page_token` was added) or registers an exception. API1's own status line (`docs/plans/2026-10-02-api1-unified-connect.md:5`, "Tasks 0–1 done") is stale; Task 28 updates it | API1 Task 4 commits on `dev` (`0f9acf70` and earlier); the owner table's Q614 defers to what is built |
| 0.2 | 0 | **REST inventory: 29 `/v1` method/path pairs, not "27 plus `list_streams` and `list_links`".** `crates/loams/tests/route_map.rs:35` pins `APP_ROUTE_COUNT = 29`: the 27 that `api::router` registers (AP1e Task 19 added the stream and link listings to the 25) plus `hot` and `warm` from `hot::routes()`. `docs/api/route-map.md` rows 32–60 list the same 29, `GET /v1/namespaces/{ns}/streams` and `GET /v1/namespaces/{ns}/links` among them. Not counted as app routes: `/health`, `/ready`, the four node routes `/internal/v1/{reads,node/stats,hot/status,hot/warm}` (`api/internal.rs`) and the nine Raft/meta routes of `crates/loams-meta/src/rpc.rs`, all of which go to `loams.internal.v1` (Task 4). The console's `/api/v1/*` is served only by `loams-apps-mock`/`loams-console-mock` (Task 10). Tasks 2 and 3 still add `ListStreams` and `ListLinks`; Task 11's `no_native_rest_route_remains` takes its list from these 29 | `route_map.rs`, `docs/api/route-map.md`; the plan text's "27 pairs plus" double-counts the two listings |
| 0.3 | 0 | **MT1 has merged nothing (`docs/plans/2026-10-02-mt1-authentik-identity.md` status "Planned"), but engine authentication has: `crates/loams/src/auth/` (PR #436: `349b6e32`, `b371ce89`, `270e7f29`, 2026-10-11).** As built: the `Authenticator` hook and a `Principal { subject, kind, display_name, email }` in the request extensions (`auth/mod.rs`, `auth/layer.rs`); `ApiKeyAuthenticator` with argon2id hashes, but read from `LOAMS_API_KEYS_FILE`, **not the metastore** (`auth/api_key.rs`); `OidcAuthenticator`, which verifies RS256/EdDSA JWTs from external issuers through JWKS, with an allowlist (`auth/oidc.rs`); a `tailscale` mode on its own loopback listener (`auth/tailscale.rs`); and `LOAMS_AUTH_MODE=off`, which yields `PrincipalKind::DevLoopback` and refuses to start unless every listener is loopback. `MAIN_PUBLIC_PATHS` (`auth/layer.rs:44`) allows health, ready and `GetInstance`. No token endpoint (`/oauth/token` exists only in `loams-apps-mock` and `loams-console-mock`), no sessions, no principal store and no main-port TLS (`--*-tls-*` flags exist only for the SQL gate, `main.rs:278`). **Ruling: Task 6 extends `crates/loams/src/auth/` and does not write a second one.** What Task 6 still owns: moving API keys into the metastore (the file stays as a bootstrap source), `every_served_rpc_requires_auth_except_allowlist` against `MAIN_PUBLIC_PATHS`, `bearer_never_logged` and `query_string_token_refused`. The external-issuer `OidcAuthenticator` is kept and is not the MT1 `JwtAuthenticator`: MT1's issuer has not merged, so MT1 adds its issuer and the HTTP protocol endpoints through the hook (API2-Q1). `loams pg-control` has its own `Authenticator`/`DevLoopbackAuthenticator` seam in `crates/loams-pg-control/src/authz.rs` (PG2 R9.2), which Task 6 leaves alone. **Deviation to note for API2-Q5:** the as-built `Auth` also guards Qdrant REST and gRPC, the Elasticsearch gateway and Flight SQL (`auth/mod.rs` header), so compat auth on the conventional ports is no longer "unchanged until MT1": it uses the same bearer/API-key check. Task 27's threat model records that state instead of the pre-#436 one | `git log origin/dev -- crates/loams/src/auth`; `server.rs:1955–1960` |
| 0.4 | 0 | **LV1's "Live on the main port" has not merged.** `LiveAbsent` is still registered (`connect.rs:327`, `:480`) and `loams.live.v1` is `available: false, unstable: true`. **PG2 Task 9 has merged, but not as the plan assumed:** `PostgresService` is served by the separate `loams pg-control` role on its own `--listen` (`crates/loams/src/pg_control.rs`, handlers in `api/connect_postgres.rs`; PG2 R9.1, `f19892e1`), not on the engine's main port, and it is not in `CATALOGUE`. Served-only reflection (`Reflector::with_services`, PG2 R1.4) is on `dev` (`connect.rs:456`, `install_reflection`). **Task 9 still owns:** the `CATALOGUE` rows and registration of every API2 package (sql, stream, link, auth, admin, operations); replacing `LiveAbsent` if LV1's task has not done so when Task 9 runs; advertising `loams.postgres.v1` in `GetInstance` only through PG2 Task 57 (`loams dev --postgres`, PG2 Task 56); and `reflection_off_by_default_outside_dev` (Q603). Graph (`loams.graph.v1`, GR1 Task 5) and House (`loams.house.v1`, HS1 Task 7) are already in `CATALOGUE` | `connect.rs`; PG2 rulings R9.1–R9.9 |
| 0.5 | 0 | **Durable surface, for reference only:** `crates/loams-durable` embeds the Resonate server and serves its HTTP protocol on a loopback listener, `127.0.0.1:8001` by default (D138), with in-process calls through `DurableServer::process` and Loams's own functions on the Resonate Rust SDK over `InProcNetwork` (D141). The stores are SQLite, TiKV and MySQL. It has no Connect or gRPC service. API2-Q3 stands: no `loams.durable.v1`, and Task 8 ships `OperationsService` only | `crates/loams-durable/src/lib.rs` |
| 0.6 | 0 | **MCP stays out of scope (M1.6).** No crate under `crates/` serves MCP on 8083 or over HTTP. The only MCP server is `crates/loams-agentd-mcp`, a stdio JSON-RPC tool server for agent harnesses that proxies to `loams-agentd`'s localhost IPC (`src/lib.rs`, `src/jsonrpc.rs::serve_stdio`). It is not §44 §6's "MCP (8083, stateless streamable HTTP)" compat surface | `grep -rl 8083 crates/` finds nothing |
| 0.7 | 0 | **Facades as built.** Generated by `crates/loams-facade-gen` (renderers `typescript.rs`, `python.rs`, `go.rs` and `rust.rs` only; `scripts/sdk/gen.sh` cases `typescript`, `python`, `go` and `rust`): TypeScript (`packages/client/src/gen/facade.ts`), Python (`src/loams/_gen/facade.py`), Rust (`src/facade.rs`), and Go, which generates `gen/facade` and hand-writes `modules.go` (D730). **Hand-written, nine:** Swift (`Sources/Loams/Facade.swift`, which says "the Swift renderer does not exist"), Kotlin, Java (`src/dev/loams/facade/Facade.java`), C# (`src/Loams/Facade.cs`), Dart, Ruby, PHP, C++ and Objective-C. Q604 moves Swift, Kotlin, Java and C# to generated: each needs a new renderer (Task 14). **CI jobs on `dev`** (`.github/workflows/ci.yml`): `sdk-typescript`, `sdk-rust`, `sdk-python`, `sdk-swift`, `sdk-go`, `sdk-cpp` and `sdk-csharp`. Java, Kotlin, Ruby, PHP, Dart and Objective-C have none. **PR #466** (`sdk/wave23-ci`, open, not merged) adds `sdk-java`, `sdk-kotlin`, `sdk-ruby`, `sdk-php`, `sdk-dart` and `sdk-objc`. They run on a PR that touches their paths, on pushes to `dev` and `main`, and nightly, and none is in `required.needs`. That fits API2-Q2's "never required"; the "on tags" half is Task 25's to add if #466 does not. Separately, **D855** (2026-10-10) disabled GitHub Actions workflows: until they are re-enabled, every job here, #466's included, is run locally by the author. `docs/release/preview.md` (`9274cd70`) ships seven SDKs in `v0.1.0-preview.1` and lists the other six as coming soon | `ls crates/loams-facade-gen/src`; `gh pr view 466` |
| 0.8 | 0 | **Naming drift: the manifests win.** The Rust crate is `loams` (`sdks/rust/Cargo.toml:51`) and the Go module is `loams.dev/go` (`sdks/go/go.mod:1`). `sdks/README.md:13–14` (`loams-sdk`, `github.com/ostrium-labs/loams/sdks/go`) is wrong and is fixed in Task 28, whose file list already names it | §44 §9 rows 3–4 agree with the manifests |
| 0.9 | 0 | **Stale notes in `sdks/fixtures/manifest.json`, for Task 15.** (a) **R4**: "No RPC carries a `consistency_token` … arrive with API1 Tasks 2–4, and the token's encoding is not in the protos yet." This is false now. `WriteDocumentsResponse.token`, `ScrollDocuments`/`GetDocuments` `read_token`, `Consistency.at_least` and the `loams-consistency-token` header are all on `loams dev` (`proto/loams/collection/v1/document.proto:83–84, 209–210, 336–338`), so an R4 fixture is recordable. (b) **R6**: "No paged RPC exists on any server … the end-to-end half arrives with API1 Task 2." That is false too: `ListCollections` and `ListVersions` take `page_size`/`page_token` and answer `next_page_token` (`collection.proto:195–208, 253–260`), so R6 is recordable from `loams dev` and `mock_status_list_is_not_paged` no longer needs to stay optional for want of a paged RPC. (c) **R2** is still accurate: `RetryInfo` is on no proto (`grep -r RetryInfo proto/` is empty) and `mock_injects_retryable_errors` is unbuilt. It changes in Task 16. (d) The about text and `recordedFrom` stand. `instance_who_am_i_*` were re-recorded after #436 (`76c5787c`), and their notes are null | `sdks/fixtures/manifest.json` `clauses.R2/R4/R6.note` |
| 0.10 | 0 | **The owner rulings of 2026-10-10 are execution rulings, unchanged.** Each row of "Owner rulings 2026-10-10 (defaults)" above is a ruling of this plan as written. Q600: delete REST outright, no shims. Q601: keep all 13 languages, Objective-C first to drop. Q602: protocol endpoints stay HTTP (D602). Q603: reflection on in `loams dev`, elsewhere only with `--reflection` and auth. Q604: generate eight facades and hand-write five under `<lang>_facade_matches_index`. Q605: no BSR publish. Q606: Connect-unary counts as no REST. Q607: `loams.dev/go`. Q608: `QueryArrow` capped at 16 MiB, then `result_too_large`. Q609: Apache-2.0 generated code, no DCO on bot mirrors. Q610: vcpkg git registry plus an in-repo Conan recipe, with a hosted remote behind `SDK_PUBLISH_CONAN`. Q611: keep Objective-C, labelled "community" if it fails. Q612: CODEOWNERS on `reasons.md` and `proto/**`. Q613: `vector.search` and `search.query` as one RPC. **Q614: unary with a cursor (ruling 0.1).** API2-Q1: `ApiKeyAuthenticator` plus dev-loopback before MT1, with REST removal not waiting (see ruling 0.3 for what already exists). API2-Q2: wave 2/3 CI never in `required.needs`. API2-Q3: no `loams.durable.v1`. API2-Q4: a 24 h cluster-wide idempotency window. API2-Q5: compat auth unchanged until MT1, amended in fact by ruling 0.3 because #436 already guards the compat HTTP/gRPC ports with the same `Auth`. No later owner answer overrides any row: `git log origin/dev --since=2026-10-10` has none, and the decision log has none after D848. The decision log already carries the table as **D848** (`docs/design/13-decision-log.md:814`, "Answers Q600–Q614 and Q761", approved), and Q601 points at it (`:1176`). Task 28 adds the API2-Q1–Q5 rows under their own IDs and links D848 rather than duplicating it | D848; D855 |
| 0.11 | 0 | **Toolchain table:** `docs/sdk/toolchain-2026-10.md`, with `toolchain_table_has_row_per_language` in `sdks/conformance/check-languages.mjs` (`--toolchain`, or `node --test sdks/conformance/check-languages.mjs`). Under D612's 12-month rule (cutoff 2025-10-13), every Connect library has released inside the window, so **no language moves to gRPC**. **Dart changes column** from "Connect if maintained, else gRPC" to **Connect** (`connect-dart` 2.0.0, 2026-08-31); the conformance half is decided in Task 21. C++ stays as D639 left it (Connect and gRPC-Web over HTTP, no grpc++). As built, only TypeScript, Python, Go and Rust link their Connect library: the other nine speak the wire over a hand-written HTTP transport, and Tasks 20–22 decide whether each adopts the library in the table. `connect-go` v2.0.0 (2026-10-07) is not pinnable before 2026-10-21. Pin drift found while filling the table goes to Tasks 13 and 17: Python's `connectrpc/python` is `v0.12.1` in `pins.lock`, `v0.9.0` in `buf.gen.proto.yaml` and at most `v0.10.1` in `bufbuild/plugins`; Go's `buf.gen.go.yaml` still pins `v1.36.9`/`v1.19.2` against `v1.36.11`/`v1.21.0` elsewhere | Web lookups of 2026-10-13, cited in the document. Unverified: BSR-side plugin versions, Maven Central OIDC |

### API2 Task 19 (2026-10-13)

- **`only_wave1_is_required` checks a superset, pending the owner.** The plan says `required.needs` holds exactly the wave 1 jobs (`sdk-typescript`, `sdk-python`, `sdk-go`, `sdk-rust`). On `sdk/wave23-ci` it also holds `sdk-cpp`, `sdk-csharp` and `sdk-swift`, which were made PR-blocking before API2-Q2 was ruled, and are wave 3, 2 and 2 under D614. This task did not remove them: dropping a required check is an owner decision. `scripts/ci/check-sdk-jobs.py` therefore asserts that every wave 1 job is required and that no wave 2 or 3 job is, except those three (its `GRANDFATHERED` set). **Owner to decide:** either keep C++, C# and Swift required (and amend API2-Q2 to say so), or take them out of `required.needs` and empty `GRANDFATHERED`, after which the check is the plan's "exactly wave 1".
- **`every_sdk_dir_has_a_job` is `scripts/ci/check-sdk-jobs.py`**, a Python script with a `--self-test` in the style of the other `scripts/ci` guards, run by the unconditional `no-metering` job (light, no path filter, so a new `sdks/<lang>` without a job fails the PR that adds it). It also fails on an `sdk-<lang>` job with no directory. `conformance`, `fixtures` and `templates` are not languages. It reads `ci.yml` line by line rather than with PyYAML so that it needs nothing installed.
- **No ext-grpc / Connect-unary matrix for PHP and Ruby (D613 not yet applicable).** Neither SDK has a native gRPC transport today: `sdks/php` requires only `php >= 8.2` (no `grpc/grpc`, no `ext-grpc`) and `src/Transport.php` is an HTTP client; `sdks/ruby/Gemfile` has `google-protobuf` but no `grpc` gem, and `lib/loams/invoker.rb` speaks Connect and gRPC-Web over HTTP. Running each job "with and without ext-grpc" would run the same tests twice. The jobs stay single; when Task 22 adds the gRPC transport (with `ext-grpc` / the `grpc` gem as an optional dependency), the job gains the two-way matrix in the same commit.
- **Nightly failure issues.** Every wave 2 and 3 job (`sdk-swift`, `sdk-cpp`, `sdk-csharp`, `sdk-java`, `sdk-kotlin`, `sdk-ruby`, `sdk-php`, `sdk-dart`, `sdk-objc`) ends with a step that runs only on `schedule` and on failure: `scripts/ci/sdk-nightly-issue.sh <lang>` comments on the open issue titled `Nightly CI: sdk-<lang> fails`, or opens it. Only those jobs have `issues: write`, with the workflow's `GITHUB_TOKEN`; no label is applied, because the repository's label set has none for SDKs. `check-sdk-jobs.py` fails when a wave 2 or 3 job lacks the step or the permission.
- **Triggers not narrowed in this task.** The plan says waves 2 and 3 run nightly and on tags only (plus an advisory PR run for `sdks/<lang>/**`). PR #466's jobs also run on pushes to `dev` and `main` and on PRs that touch `proto/**`, `sdks/fixtures/**`, `sdks/conformance/**`, `docs/sdk/**` or `.github/workflows/**`. They are advisory, so this costs runner minutes but blocks nothing; narrowing them is left for review.
- **`workflow_lint`.** actionlint 1.7.7 (release binary, checksum-verified) with shellcheck 0.10.0 reports nothing on the lines this task added. It still reports, on lines it did not touch: the custom runner label `blacksmith-8vcpu-ubuntu-2404` (needs an `.github/actionlint.yaml` listing it) and two shellcheck infos (SC2086, SC2017) in existing `run:` blocks. actionlint is not yet a CI job.

### API2 Tasks 6/7 (mock half) (2026-10-13)

The proto and mock half of Tasks 6 and 7: `loams.auth.v1` and `loams.admin.v1` are defined, generated for TypeScript (`@loams/proto`), Rust (`loams-proto`, `loams-apps-mock`) and Python, and served over Connect, gRPC and gRPC-Web by `loams-apps-mock` on 8084 from the console seed, beside the REST routes they replace. The console (`web/apps/console`) can move to them in Task 10. Rulings 6/7.1–6/7.13.

| # | Ruling | Why / evidence |
|---|---|---|
| 6/7.1 | **The Connect half is served by `loams-apps-mock` (`crates/loams-apps-mock/src/admin/`), not by `loams-console-mock`.** `loams-console-mock` keeps the contract, the seed and the route list, and gains `CONNECT_RPCS` (the RPC of every contract operation) and `CONNECT_ONLY_RPCS`. The plan's "Modify `crates/loams-console-mock` to serve the same services over Connect" is met by the process that serves that crate's seed: `loams-apps-mock` already mounts it over REST on the listener the console's Vite proxy points at, and has the connect-rust stack and codegen; `loams-console-mock` has no server since the unified mock (its `httpmock` binary is gone) and no proto build. `serve` resolves the console seed **once** and hands the same value to the REST routes (`Console::from_seed`) and to `ConsoleApi`, so a route and its RPC answer the same record with the same times | `crates/loams-console-mock/Cargo.toml`, `crates/loams-apps-mock/src/{console.rs,lib.rs}` |
| 6/7.2 | **Every service carries `ModuleOptions` and every RPC `FacadeOptions`, as `loams.instance.v1` does; the SDK package maps follow.** Modules `auth`, `admin.org`, `admin.projects`, `admin.agents`, `admin.keys`, `admin.audit` (the shared table). Facade names are camelCase like `instance.getInstance`, short within a module (`admin.projects.list`, `admin.keys.create`, `admin.agents.listTokens`), and every list declares its AIP-158 pagination. A `facade` option makes the generator emit calls, so the package maps of `sdks/{typescript,python,go,rust}/buf.gen.yaml` and the facade generator's tests name both packages (`@loams/proto/auth` and `@loams/proto/admin`, a barrel over the six files; `loams.auth.v1.auth_pb2` and `loams.admin.v1.org_pb2`; `loams.dev/go/gen/loams/{auth,admin}/v1`; `loams_proto::loams::{auth,admin}::v1`), `crates/loams-proto/build.rs` compiles both packages (as its header always said Tasks 5–8 would), the Python stubs gain both packages (`sdks/python/buf.gen.proto.yaml`, `.pyi` and `__init__` as `tools/bootstrap_facade.py` writes them), and the committed TypeScript, Python and Rust facades and the Go, Python and Rust goldens are regenerated. The Rust SDK's table pins (`binding.rs`, `client.rs`, `streams.rs`) and `@loams/client`'s `a_paged_call_from_the_corpus_yields_every_item` (which took "the first paged call" to be `ListDeployments`) are updated to the larger table. Go's facade stays uncommitted (D730, Q604) and its stub template does not bind the two packages yet: Task 20 | `scripts/sdk/drift.sh` clean; `cargo test -p loams-facade-gen`; `sdks/rust` `cargo test` |
| 6/7.3 | **`loams.admin.v1` is six files: the plan's `org`, `project`, `agent`, `key`, `audit`, plus `common.proto`.** `SubjectRef`, `ResourceRef`, `User`, `OrgRole` and `ProjectRole` are used by every service file, and `org.proto`'s `GetTeamResponse` needs `ProjectRole`, so putting them in any one service's file makes the files import each other in a cycle. `loams.auth.v1` is one file, `auth.proto`, and imports `common.proto` and `org.proto` for its `Session` (`User`, `Org`, `OrgRole`) | `proto/loams/admin/v1/common.proto` header |
| 6/7.4 | **§44 §5.2's `ProjectService/GetEnvironmentUsage` is `GetEnvironmentActivity` (response `GetEnvironmentActivityResponse`, rows `ActivityDay`).** The plan's global constraint forbids an RPC, field or SDK module named `usage` (D220, D552). The answer is an environment's queries, writes and storage per day, an operational read; the REST route's name is unchanged until Task 11 deletes it. The contract's `limits` (`null` in every seed record, no schema) has no proto field; a limit model is added later as a field | `proto/loams/admin/v1/project.proto`; `docs/api/route-map.md` row updated |
| 6/7.5 | **Shapes.** Every RPC has its own request and response (buf STANDARD, no `buf.yaml` exception), and a resource comes back wrapped (`GetOrgResponse.org`), so the console unwraps one field; `GetTeam` answers the REST `TeamDetail`'s three fields at the top level, and lists answer the REST list object plus `next_page_token`. Resource names are request fields (`project` is a project's slug, `environment` an environment's slug, `agent` an agent's id). `UpdateOrg` and `UpdateAgent` take the resource plus a `google.protobuf.FieldMask` (`name`, `require_two_factor`, `allowed_domains`; `description`, `policy`, `max_token_ttl_s`, `limits`) in place of the PATCH body. `GrantAccess` and `CreateAgent` take a `SubjectRef` of which only `kind` and `id` are read. The audit list pages with `page_size`/`page_token` in place of `limit`/`before`, keeps the `project` and `actor` filters, and is newest first | `proto/loams/admin/v1/*.proto` |
| 6/7.6 | **Types.** The contract's enums are proto enums (`OrgRole`, `ProjectRole`, `SubjectKind`, `AgentStatus`, `TokenFlow`, `AuditOutcome`, `OidcProviderKind`, `ConsentDecision`) except `Action`, which is `repeated string` (`PolicyRule.actions`, `AgentToken.scopes`, `ApiKey.scopes`) because it is an open, OAuth-scope-like vocabulary (`collections:read`, `mcp:tools`) that a new engine permission must extend without a proto change; `ResourceRef.kind` is a string for the same reason. A nullable string of the contract is an empty string ("none"); a nullable time is an unset `Timestamp` ("never"); a `null` `PolicyRule.collections` (every collection) is an empty list. Counts are `int32`, sizes and traffic `int64`. `Instance.sign_in` moves to `AuthService/ListProviders` (password, totp, passkeys, OIDC providers); the rest of `GET /api/v1/instance` is `InstanceService/GetInstance`, which already exists (AP0). The mock's `GetInstance` keeps its AP0 seed (`Loams (mock)`), so it is the one console route whose REST and RPC answers differ; Task 10 reads the instance name from `GetInstance` | `crates/loams-apps-mock/src/seed.rs`; `mock_rest_and_connect_answer_alike` compares `ListProviders` with `sign_in` |
| 6/7.7 | **Retry classes.** Reads are `NO_SIDE_EFFECTS`. Set-state mutations are `IDEMPOTENT`: `UpdateOrg`, `RemoveMember`, `AddTeamMember`, `GrantAccess` (replaces the subject's role), `UpdateAgent`, `SuspendAgent`, `ResumeAgent`, `RevokeKey`, `RevokeToken`, `DeleteSession`. Create-by-name RPCs take no key (API1 ruling 2.4): `CreateInvitation` (by email), `CreateTeam`, `CreateProject` and `CreateEnvironment` (by slug), `CreateAgent`, `CreateTrustPolicy`, `CreateServiceAccount` (by name); a retry answers `already_exists`. **`KeyService/CreateKey` is the one keyed RPC** (`idempotency_key = 15`, required): key names are not unique, and its secret is answered once. `CompleteSetup` (first run only, a second is `failed_precondition`), `CreateSession` and `DecideConsent` (credential exchanges, each answer is new and harmless to repeat by hand) take no key and are `retry: manual` in every SDK | facade-gen derives the classes; the Rust SDK pins `keyed == [CreateKey, Mutate]` |
| 6/7.8 | **A replayed `CreateKey` is `failed_precondition`, reason `secret_already_issued`, PG2's reason**, never the secret again; `docs/api/reasons.md`'s row now names `CreateKey` beside PG2's three RPCs. No new reason is registered: the mock raises `not_found`, `invalid_argument`, `unauthenticated`, `internal` and `secret_already_issued`, all registered. `role_required` (`member_cannot_create_key`) is registered with the server half (6/7.12) | `docs/api/reasons.md` |
| 6/7.9 | **Sessions.** The session cookie is `loams_session`, set by `CreateSession` and `CompleteSetup` with `Path=/; HttpOnly; Secure; SameSite=Lax` and cleared by `DeleteSession` (`Max-Age=0`); the CSRF token keeps the REST console's header, `X-CSRF-Token`, on every cookie-authenticated call that is not a GET. **`CreateSession` is unauthenticated too**: the plan's `PUBLIC_RPCS` lists `ListProviders` and `CompleteSetup` only, but a person without a session gets one by calling it. The server half adds it to the allowlist (`auth/layer.rs`'s `MAIN_PUBLIC_PATHS`, ruling 0.3) | `proto/loams/auth/v1/auth.proto` header |
| 6/7.10 | **The mock's semantics are the REST mock's**, so the two answer alike. A create answers the seed's example object and the next list does not include it; update, suspend and resume answer the changed record without keeping it. On top: `CreateKey`'s once-only secret (the mock keeps the idempotency keys it answered, never the secret); an audit journal in which every Connect mutation records one event (actor: the seed session's user; ids `evt_c0001` on), which `ListAuditEvents` lists above the seed's and `WatchAuditEvents` streams; and `GetSession`, which is `unauthenticated` under `--signed-out` unless the call carries the session cookie or a bearer. No admin RPC checks a role, as no REST route does. Lists page 100 by default and at most 1 000, with an offset token; the watch's snapshot holds 50 by default, its cursors are `a<seq>`, and a cursor the journal no longer holds gets a fresh snapshot with `snapshot_reset` | `crates/loams-apps-mock/src/admin/{mod,journal}.rs`; README "What is real" row |
| 6/7.11 | **Tests.** `crates/loams-apps-mock/tests/console_rpc.rs`: the console route tests ported with `_rpc` (`every_operation_has_a_mock_and_every_mock_an_operation_rpc`, `every_mocked_body_matches_its_response_schema_rpc` (every seed route and its RPC answer the same body, 90+ routes), `the_server_answers_from_the_seed_rpc`, `console_rest_is_served_beside_the_app_protos_rpc` (Connect, gRPC and gRPC-Web), `console_query_constraints_pick_the_right_answer_rpc`, `console_signed_out_answers_401_on_session_rpc`); `mock_rest_and_connect_answer_alike` on a scripted console session (sign-in, every page's reads, key create, suspend, resume, token revoke, consent, sign-out, then the audit log is the REST log with the four Connect events on top); `create_key_returns_secret_once`; `watch_audit_resumes_from_cursor`; and `normalize_tells_records_apart`, the negative control for the comparison (keys without case or underscores, enums without their prefix, numbers as strings, defaults dropped). `seed_times_are_resolved` and `the_validator_rejects_bodies_that_break_the_schema` test the seed and the test's own validator, not a route, so they have no port. `loams-console-mock`'s `connect_rpcs_cover_the_contract_and_agree_with_the_route_map` holds `CONNECT_RPCS` to the contract, to the RPCs declared in `proto/`, and to `docs/api/route-map.md`. `mock_rest_and_connect_answer_alike` is the mock half of the plan's `mock_and_server_answer_alike`; the server half compares the server with this mock | `cargo test -p loams-apps-mock -p loams-console-mock` |
| 6/7.12 | **Deferred to the server half (a follow-up; not in this change).** `crates/loams/src/api/connect_auth.rs` and `connect_admin.rs` and their registration on the main port; the `CATALOGUE` rows for `loams.auth.v1` and `loams.admin.v1` (Task 9 owns the catalogue); the `Authorizer` trait with `OrgRoleAuthorizer` and `member_cannot_create_key` (`role_required`); API keys in the metastore with `CreateKey` writing argon2id hashes for `ApiKeyAuthenticator` (ruling 0.3); the session store, cookie issuing and `session_cookie_and_bearer_both_work`; `every_served_rpc_requires_auth_except_allowlist`, `bearer_never_logged`, `query_string_token_refused` and the rest of Task 6's list; `mock_and_server_answer_alike`. The server builds `loams-proto` with both packages already, but registers neither, and served-only reflection (`Reflector::with_services`) keeps them out of reflection until it does | ruling 0.3, 0.4 |
| 6/7.13 | **The RPC↔REST table** (also `CONNECT_RPCS` in `crates/loams-console-mock/src/lib.rs` and the console section of `docs/api/route-map.md`). Kept HTTP (D602): `GET /api/v1/auth/oidc/{provider}/start`, `POST /api/v1/oauth/token`, `GET /api/v1/oauth/authorize`, `/.well-known/{oauth-protected-resource,oauth-authorization-server,jwks.json}`. Connect-only: `AuthService/ListProviders`, `AuditService/WatchAuditEvents` | see the table below |

| REST (`/api/v1`) | RPC |
|---|---|
| `GET /instance` | `loams.instance.v1.InstanceService/GetInstance` (exists); `sign_in` is `loams.auth.v1.AuthService/ListProviders` |
| `POST /setup` | `loams.auth.v1.AuthService/CompleteSetup` |
| `GET /session` | `loams.auth.v1.AuthService/GetSession` |
| `POST /session` | `loams.auth.v1.AuthService/CreateSession` |
| `DELETE /session` | `loams.auth.v1.AuthService/DeleteSession` |
| `POST /oauth/consent` | `loams.auth.v1.AuthService/DecideConsent` |
| `GET /org` | `loams.admin.v1.OrgService/GetOrg` |
| `PATCH /org` | `OrgService/UpdateOrg` |
| `GET /org/members` | `OrgService/ListMembers` |
| `DELETE /org/members/{user}` | `OrgService/RemoveMember` |
| `GET /org/invitations` | `OrgService/ListInvitations` |
| `POST /org/invitations` | `OrgService/CreateInvitation` |
| `GET /teams` | `OrgService/ListTeams` |
| `POST /teams` | `OrgService/CreateTeam` |
| `GET /teams/{team}` | `OrgService/GetTeam` |
| `POST /teams/{team}/members` | `OrgService/AddTeamMember` |
| `GET /projects` | `ProjectService/ListProjects` |
| `POST /projects` | `ProjectService/CreateProject` |
| `GET /projects/{project}` | `ProjectService/GetProject` |
| `GET /projects/{project}/access` | `ProjectService/ListAccess` |
| `POST /projects/{project}/access` | `ProjectService/GrantAccess` |
| `GET /projects/{project}/environments` | `ProjectService/ListEnvironments` |
| `POST /projects/{project}/environments` | `ProjectService/CreateEnvironment` |
| `GET /projects/{project}/environments/{environment}` | `ProjectService/GetEnvironment` |
| `GET /projects/{project}/environments/{environment}/usage` | `ProjectService/GetEnvironmentActivity` (6/7.4) |
| `GET /projects/{project}/agents` | `AgentService/ListAgents` |
| `POST /projects/{project}/agents` | `AgentService/CreateAgent` |
| `GET /agents/{agent}` | `AgentService/GetAgent` |
| `PATCH /agents/{agent}` | `AgentService/UpdateAgent` |
| `POST /agents/{agent}/suspend` | `AgentService/SuspendAgent` |
| `POST /agents/{agent}/resume` | `AgentService/ResumeAgent` |
| `GET /agents/{agent}/tokens` | `AgentService/ListAgentTokens` |
| `GET /agents/{agent}/trust-policies` | `AgentService/ListTrustPolicies` |
| `POST /agents/{agent}/trust-policies` | `AgentService/CreateTrustPolicy` |
| `GET /projects/{project}/service-accounts` | `AgentService/ListServiceAccounts` |
| `POST /projects/{project}/service-accounts` | `AgentService/CreateServiceAccount` |
| `GET /projects/{project}/environments/{environment}/keys` | `KeyService/ListKeys` |
| `POST /projects/{project}/environments/{environment}/keys` | `KeyService/CreateKey` |
| `DELETE /keys/{key}` | `KeyService/RevokeKey` |
| `DELETE /tokens/{jti}` | `KeyService/RevokeToken` |
| `GET /audit` | `AuditService/ListAuditEvents`; new: `AuditService/WatchAuditEvents` |

### API2 Tasks 6/7 (server half) (2026-10-13)

The server half of Tasks 6 and 7, stacked on the mock half (#515): `loams.auth.v1` and `loams.admin.v1` served from the engine's main port (`crates/loams/src/api/{connect_auth,connect_admin}.rs`), with their `CATALOGUE` rows, over a principal store in the metastore (`crates/loams/src/auth/directory.rs`) and an `Authorizer` (`crates/loams/src/auth/authorizer.rs`). Ruling 0.3's `crates/loams/src/auth/` is extended, not duplicated. Rulings S6/7.1–S6/7.12.

| # | Ruling | Evidence |
|---|---|---|
| S6/7.1 | **The principal store is one JSON document behind a metastore pointer, not new `MetaStore` methods.** The org, members (with argon2id password hashes), invitations, teams, projects with grants and environments, agents with trust policies, service accounts, API keys (argon2id hashes), session digests and the audit log are one `Doc`, stored as an object under `system/directory/` in the bucket and named by the manifest pointer `principal-directory` of the reserved namespace `_system`. A write puts a new object and compare-and-swaps the pointer, redoing the change on the newer document after a lost race (the graph catalog's pattern, `loams-graph/src/catalog.rs`); a read re-parses only when the pointer's version moved, which is one local metastore read per authenticated call. Semantic `MetaStore` methods would have meant a raft command, a TiKV encoding and conformance cases for a single-org, low-write record; a later plan may move it. The proto messages are stored through their proto3 JSON mapping, so the document grows with the protos. Bounded: the newest 1 000 audit events and 10 000 revoked token ids; expired sessions are dropped on the next write. `_system` shows up in `ListNamespaces` (leftover) | `auth/directory.rs` header; `updates_persist_and_a_lost_race_is_redone` |
| S6/7.2 | **API keys: the metastore's and the file's, one authenticator path.** `KeyService/CreateKey` writes `lk_<ULID>_<32 base62>` as its argon2id PHC string (`api_key::generate`, `hash_key`, and one process-wide argon2 semaphore shared with the key file, `verify_hash`). `Auth::authenticate` asks the principal store first: it decides the keys it holds (revoked: `token_revoked`; expired: `token_expired`; wrong secret: `api_key_invalid`) and the sessions, and a key it does not hold falls through to `LOAMS_API_KEYS_FILE`, which stays the operator's bootstrap source. Keys the API made are accepted in every mode with authentication on, `oidc` alone included. `last_used_at` is not written per call (it would be a metastore write per request) and stays unset | `api_key_round_trip`; `stored_keys_verify_and_revoke` |
| S6/7.3 | **Sessions.** A session is `ls_<43 base62>` (~256 bits), stored as its SHA-256 with a CSRF token and a 12-hour expiry, answered once in `loams_session` (`Path=/; Max-Age=43200; HttpOnly; Secure; SameSite=Lax`) and usable as a bearer. The layer reads the cookie only when no `Authorization`/`api-key` credential is present, and a cookie-authenticated call that is not a `GET` must carry `X-CSRF-Token` (`csrf_token_invalid`, `permission_denied`), except `GetSession`, which is how the console learns the token. With `LOAMS_AUTH_MODE=off` a valid session still names its person (so the console's sign-in works in `loams dev`) and anything else is `DevLoopback`. An unknown or signed-out session is `token_revoked`, an expired one `token_expired`. Cookies cross-origin (CORS with credentials) are not enabled: the console reaches the engine same-origin or through its proxy (leftover for Task 10) | `session_cookie_and_bearer_both_work`; `the_session_cookie_needs_csrf_except_on_get_session` |
| S6/7.4 | **The allowlist is `PUBLIC_RPCS` (in `auth/mod.rs`): the plan's five, ruling 6/7.9's `CreateSession`, and `DeleteSession`.** `DeleteSession` is public because the proto says signing out without a session is not an error, and it only ends the session whose token it carries. `MAIN_PUBLIC_PATHS` is `PUBLIC_RPCS` plus `/health` and `/ready` (`main_public_paths_are_the_public_rpcs_and_the_probes`). A credential in the query string is refused before the allowlist is consulted, so even a public RPC refuses `?token=` | `every_served_rpc_requires_auth_except_allowlist`; `query_string_token_refused` |
| S6/7.5 | **The layer's Connect refusal now carries the `ErrorInfo`.** Besides `loams-auth-reason` and the message prefix, a Connect refusal has `details: [loams.errors.v1.ErrorInfo{reason}]`, so an SDK branches on `token_missing` like on any reason. gRPC refusals keep `grpc-status` 16/7/14 and the message (no `grpc-status-details-bin`). Every reason the layer can raise is registered in `docs/api/reasons.md` (`token_missing` … `auth_unavailable`), with the new `role_required`, `setup_required`, `token_revoked` and `csrf_token_invalid` | `docs/api/reasons.md`; `reasons_are_snake_case_and_unique` |
| S6/7.6 | **`--auth {api-key,jwt,dev}` is `LOAMS_AUTH_MODE` (ruling 0.3), not a new flag.** `apikey` is the plan's `api-key`, `jwt` is accepted as `oidc`, and `off` is `dev` (loopback only). `jwt_without_issuer_refused_at_startup` and `dev_loopback_refused_on_public_address` test those through the env/config path. No `JwtAuthenticator` for the instance's own tokens exists before MT1, so `expired_jwt_gets_token_expired` is covered by `tests/auth.rs`'s `oidc_tokens_are_verified_against_the_issuer` ("expired" case, external issuers) and by `sessions_expire_end_and_check_csrf` for sessions; `oidc_start_remains_http_redirect` is `#[ignore]` with that reason | `tests/connect_auth.rs` |
| S6/7.7 | **The `Authenticator` hook takes extra authenticators from config.** `AuthConfig.extra_authenticators` (an `Authenticators` list) is appended after the mode's own and turns authentication on; `authenticator_is_swappable` injects a test authenticator and the handlers serve its principals unchanged. The plan's `authenticate(rpc, headers)` signature is not adopted: ruling 0.3's `authenticate(&Credential)` stays, since the layer already owns the allowlist and the header parsing | `tests/connect_auth.rs`; `console_support::TestAuthenticator` |
| S6/7.8 | **`OrgRoleAuthorizer`: how a principal gets an org role.** A session: its person's role. An API key from `CreateKey`: the role stored with it (its creator's when it acts as its creator, `member` for a service account's). A key from `LOAMS_API_KEYS_FILE` and `DevLoopback`: `owner` (the operator made them). An OIDC, tailnet or injected user: the member with their email; else a pending invitation for that email is accepted on their first admin call (`invitation.accept` audit event, the invitation's role and teams); else they are not in the org (`role_required`). Reads need `member`, every mutation `admin`, inviting or removing an owner `owner`; the last owner cannot be removed (`failed_precondition`). Before setup every admin RPC is `failed_precondition`, `setup_required`. Only admin RPCs consult the authorizer: data-plane access is unchanged (any admitted principal) until MT1's OpenFGA | `member_cannot_create_key`; `setup_and_membership_rules_rpc` |
| S6/7.9 | **First-run setup.** `CompleteSetup` runs once (`failed_precondition` after). With authentication off it needs no token (every listener is loopback, which stands for "from loopback": the main port does not record peer addresses); with it on it needs `LOAMS_SETUP_TOKEN` (`SetupGate::Token`, constant-time compare, `permission_denied` otherwise), and with no token set it is refused with a message naming the variable (`SetupGate::Closed`). The engine does not print a generated token (that would put a credential in the log). Passwords are at least 8 characters, hashed with argon2id; `CreateSession` checks a wrong email against a dummy hash so both cost the same. TOTP and passkeys are not offered (`ListProviders` answers `password` only, and no OIDC provider while the start endpoints are absent) | `connect_auth.rs` |
| S6/7.10 | **What the server answers before other plans land.** `DecideConsent` denies (redirect with `error=access_denied`, as the mock) but `ALLOW` is `failed_precondition`: minting a code needs MT1's issuer. `ListAgentTokens` is empty and `RevokeToken` accepts any `jti` onto the revocation list (MT1's issuer mints the tokens). `EnvironmentStats` are zeros and `GetEnvironmentActivity` answers the 30-day period with an empty series until metering reports per-environment counters. An environment's `namespace` is `<project>-<environment>`, not created until data is written there | `connect_admin.rs` header |
| S6/7.11 | **`mock_and_server_answer_alike` compares shapes, not values.** The mock answers seed records and its creates answer the seed's example object without keeping it (ruling 6/7.10), while the server keeps what it is told, so equal values are impossible by design. The test runs one scripted console session (setup, session, every page's reads, creates, team and grant changes, agent update/suspend/resume, trust policy, service account, key create/replay/missing key, list, revoke, token revoke, an unknown project, a consent denial, the audit list, sign-out) against `loams-apps-mock` and the server, ids taken from each target's own lists, and asserts at every step the same outcome (`ok`, or the same code and `ErrorInfo` reason), the same top-level message and non-empty string fields, the same cookie attributes, and that the server's audit trail for the script equals the mock's Connect events (16 actions, in order). The server also records `org.setup`, `session.create`, `session.delete` and `invitation.accept`, outside the mock's vocabulary. Two differences remain by design: the server requires an 8-character password in `CompleteSetup` (the mock a non-empty one), and `ALLOW` consent (S6/7.10) is not in the script | `tests/connect_admin.rs` |
| S6/7.12 | **Task 7's tests.** The console-mock route tests are about the seed, so their `_rpc` ports are the mock's (6/7.11); on the server the script of S6/7.11 exercises every admin RPC but `RemoveMember` and `WatchAuditEvents`, which `setup_and_membership_rules_rpc` and `watch_audit_resumes_from_cursor` cover (snapshot then appends, a resumed cursor replays what it missed without a snapshot, an unknown cursor resets with `snapshot_reset`, heartbeats every 15 s). `create_key_returns_secret_once` and `member_cannot_create_key` are the plan's. `futures` becomes a plain dependency of `crates/loams` (the watch stream) and `loams-apps-mock` a dev-dependency | `cargo test -p loams --test connect_admin --test connect_auth` |
| S6/7.13 | **`GetSession` answers every admitted caller, and `GetInstance.setup_required` is set (rebase onto Task 10, 2026-10-13).** Task 10's console gates on `GetSession` wherever `loams.auth.v1` is served (`@loams/plugin-identity`: `unauthenticated` is `signed-out`, which sends the visitor to sign-in), and the hosted console signs in elsewhere: an OIDC token, an API key, or the tailnet, whose identity it reads with a credential-less Connect GET of `GetSession` (ruling 10.4). So a caller without a console session gets a `Session` naming them, read-only: an OIDC or tailnet user is the member with their email (else themselves, with no role); a `CreateKey` key is its creator (or its service account) with the key's role; a key-file key and the developer on loopback are `owner`. It has no CSRF token and no expiry; an invitation is still accepted on the first admin call, not here. `setup_required` is true while there is no org and setup can run (authentication off, or `LOAMS_SETUP_TOKEN` set), so the console opens on setup; with setup closed it stays false rather than open a page nobody can complete. The desktop's local shim clears it (it answers "This computer" itself, ruling 10.2) | `get_session_names_a_bearer_caller`, `setup_is_not_required_when_it_cannot_run`, `dev_loopback_session_is_the_owner`; `desktop_local_shim_answers_connect` |

### API2 Task 10 (2026-10-13)

The console, data studio and the desktop on Connect. Done with AP1a Task 5 (its rulings E13–E18 cover the pages, the routing and the session gate), in two stacked PRs: `console/connect-pages` (the pages, the OpenAPI client removed) and `console/connect-desktop` (the desktop, data studio and hosted half).

| # | Ruling | Why / evidence |
|---|---|---|
| 10.1 | **No `createLoamsTransport(runtimeConfig)` in `web/apps/console/src/api/client.ts`.** The console already has one Connect transport, `@loams/platform-web`'s (`createConnectTransport` over the runtime config's server, cookie session or bearer), provided to every plugin as `transport` and to Connect Query by console-host's `ServerStateProvider`; `src/api/` is deleted with the React console. The cookie session's CSRF token (ruling 6/7.9) reaches that transport through `platform.setCsrfToken` | `web/packages/platform-web/src/index.ts`; `csrf_token_rides_on_mutations_only` |
| 10.2 | **The desktop's local shim answers `AuthService/GetSession` itself and amends the engine's `InstanceService/GetInstance` rather than replacing it.** The engine serves `GetInstance` (AP0) with the `api_versions` the console gates on; the shim forwards it and adds the `local` and `desktop` features and `loams.auth.v1` (which the shim serves for the local edition). `GetSession` is the person at this computer in the org "This computer", `ListProviders` says there is no sign-in, and every other `loams.auth.v1` or `loams.admin.v1` RPC is `unimplemented` ("This needs a Loams control plane"). JSON and binary, as the request asks. The REST answers for `/api/v1/instance` and `/api/v1/session` are gone; the window loads `loams-app://console/ui/` | `apps/desktop-electron/src/main/protocol/local-shim.ts`; `desktop_local_shim_answers_connect` |
| 10.3 | **`data_studio_uses_connect` allows three named `/v1/namespaces` routes until their Connect services exist.** Data Studio's collections, documents and search are `CollectionService`, `DocumentService` and `QueryService` already; its SQL tab stays on `POST /v1/namespaces/{ns}/sql` until `loams.sql.v1` (Task 1) is generated, and the Streams and Overview pages' streams and links until `loams.stream.v1` (Task 2). The test fails on any other file under `web/` that names the path, and on a second REST call in Data Studio. Task 11 cannot delete those routes before Tasks 1 and 2 land | `web/plugins/data-studio/test/connect.test.ts` |
| 10.4 | **The hosted console's tailnet identity check is `AuthService/GetSession` as a Connect GET** (`?connect=v1&encoding=json&message={}`): a simple cross-origin request with no preflight and no credentials, like the `GET /api/v1/session` it replaces. It reads `session.user.email`, else the name | `web/apps/console/src/hosted/engine.ts` |
| 10.5 | **Against a server that does not serve `loams.auth.v1` and `loams.admin.v1` (today's engine, until Task 7's server half), the console shows only the app-proto pages**: its own pages gate on `api_versions` (AP1a Ruling 6) and the session has no sign-in gate. The engine's REST console routes stay until Task 11 | AP1a E13 |
