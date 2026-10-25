# IA1 — Identity and Authorization: Zitadel and OpenFGA — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Work task by task, test first. Where this plan gives exact names, paths, flags or tests, use them verbatim. Record any deviation in "Rulings made during execution" at the end of this file.
>
> **Status: Approved** (owner, 2026-10-21, all defaults; stub). Track IA, design [§76](../design/76-identity-and-authorization.md) (D1600–D1628, Q1200–Q1214).
>
> **What is approved:** all of §76, on 2026-10-21: first D1600 (the full cutover from Authentik to Zitadel, with no migration, no compatibility period and no dual running), then every other decision and question with its default.
>
> **Coverage:** this plan specifies **IA0** (the owner's Phase 0, the spike) and **IA1 Task 1** task by task. IA1's other tasks and IA2–IA6 are outlined at milestone level and get full task specifications when IA0 has reported.
>
> **Supersedes** [MT1](2026-10-02-mt1-authentik-identity.md) (D1600).

**Goal (IA0):** prove, on one laptop, that a person signed in at Zitadel can be allowed or denied by OpenFGA through a small Rust service, and check every fact §76 marks **(verify)** before IA1 builds on it.

Exit, from §76 §15:

- end-to-end allow and deny work;
- a spike report (`docs/plans/ia0-spike.md`) with the measured footprint, the pins and the answers to §76's (verify) items.

**Architecture** (§76 §4, §12.1):

- **`conformance/identity/compose.yaml`**: the IA0 stack on loopback:
  - Zitadel (API) and Zitadel Login, pinned by digest;
  - `zitadel-db` (Postgres 17);
  - OpenFGA with the SQLite datastore;
  - a bootstrap job that creates one org, one human user, one machine user (an agent) and one OIDC application.
- **`deploy/openfga/core.fga`, `deploy/openfga/fga.mod`, `deploy/openfga/tests/*.fga.yaml`**: model v1 (§76 §6) and its model tests, run with the OpenFGA CLI (`fga model test`).
- **A spike service** (branch `ia0-spike`, **not merged**): an Axum app with one protected route per `can_*` relation. It verifies Zitadel JWTs with `openidconnect` 4 and `jsonwebtoken` 11 and calls OpenFGA with `openfga-client` 0.6.1. Only its report is merged.

**Tech stack:**

- **Rust:** 1.97, edition 2024, workspace lints. `axum`, `openidconnect` 4, `jsonwebtoken` 11, `openfga-client` 0.6.1 (Apache-2.0), `tokio`, `reqwest`, `serde`.
- **Docker Compose v2.** Images are pinned by digest at a release at least 14 days old. On 2026-10-21 that is Zitadel v4.19.4 (2026-10-01) and OpenFGA v1.22.0 (2026-10-06); Task 0 re-checks.
- **OpenFGA CLI** (`openfga/cli`, Apache-2.0) for `fga model test` and `fga model write`.

**Spec:**

- [§76](../design/76-identity-and-authorization.md), all of it.
- [§19](../design/19-console-identity-and-agents.md) §4–§6: principals, tokens and sign-in.
- [§18](../design/18-metastore-backends-and-router.md) §7: D66 and D67.
- `deploy/openfga/graph.fga`.
- `crates/loams/src/auth/`: what is built.

## Global Constraints

- **Worktrees and branches.** Work in `~/Documents/Ostriumlabs/loams-wt/ia0-<task>`, with one branch and one PR per task into `dev`. CodeRabbit reviews; merge on green. Commit with `git commit -s`.
- **Light Rust builds.** Build only the crate in hand (`cargo build -p <crate>`), never `--workspace` locally. Build output goes to the shared target set in `~/Documents/.cargo/config.toml`. Never set `CARGO_TARGET_DIR`, and never build in `/tmp`. Stop compose stacks while building.
- **Licences (§76 §14):**
  - Zitadel is AGPL-3.0-only: **run its official images unmodified**. Never copy, vendor or link Zitadel code, and never commit a patched image. A Zitadel bug goes upstream as an issue or PR.
  - The Zitadel and OpenFGA charts are Apache-2.0 and are referenced, not vendored.
  - Every new crate passes `cargo deny`.
- **No dependency enters `loams`'s default build in IA0.** The spike service stays on its branch.
- **Secrets.** Bootstrap passwords, keys and client secrets are generated at `up` time into a git-ignored `.env`. None is committed.
- **GitHub-hosted runners only.** The compose smoke runs nightly and on PRs that touch `conformance/identity/` or `deploy/openfga/`.

## Review Focus

1. **A check allows something the model does not grant.** Expected: never. Tests:
   - `deny_by_default_without_tuple`;
   - `agent_does_not_inherit_owner` (an agent whose owner is a project admin, with no grant of its own, is denied);
   - `agent_cannot_write_grants` (`billing_admin`, `meter_admin` and `operator` refuse an `agent` or `service_account` subject at model-write time).
2. **A tuple links two orgs.** Expected: never. Test: `no_cross_org_tuples` (a scan over a seeded store).
3. **A Zitadel token from the wrong audience or issuer is accepted.** Expected: rejected. Tests: `rejects_wrong_aud` and `rejects_wrong_iss` in the spike.
4. **A secret lands in Git.** Expected: never. Test: a CI check that `conformance/identity/.env` is ignored, plus a `gitleaks` pass on the PR.

## IA0 — The spike (Tasks 0–6)

### Task 0 — Rulings

Check each item and record the answers in "Rulings made during execution":

- **Pins.**
  - The Zitadel server image and the Zitadel Login image at the newest release at least 14 days old, with their digests.
  - The OpenFGA image and digest.
  - The Postgres image used by Zitadel's own compose example.
- **§76's (verify) items:**
  1. Actions v2 delivery: ordering, retries and redelivery for event executions, and the event names for org member added or removed, user deactivated or removed, and machine user deactivated.
  2. The `aud` value an engine should require for Zitadel access tokens (project id or client id), and how to make access tokens JWTs.
  3. OpenFGA modular models (`fga.mod`, schema 1.2) at v1.22: their status and CLI support.
  4. The licence of `smartive/zitadel-rust` (read its `LICENSE` file).
  5. Lakekeeper's setting for its OpenFGA store name (for D1604).
  6. Clerk and WorkOS as generic OIDC IdPs in Zitadel. Read the docs only; Q1201's default does not need it.
- **What is built:** whether `auth::authorizer`, `graph_auth.rs`, `loams-graph/src/authz.rs` and `loams-pg-control/src/authz.rs` changed since 2026-10-21, and whether CP1 Task 3 has started (`crates/loams-tenancy`, `deploy/openfga/core.fga`).

Files: this plan. Tests: none. Gate: the rulings are recorded.

### Task 1 — The IA0 compose stack

**What to write:**

- `conformance/identity/compose.yaml`, with services:
  - `zitadel` (API, `start-from-init`, masterkey from `.env`);
  - `zitadel-login`;
  - `zitadel-db`;
  - `openfga` (`--datastore-engine sqlite`, a named volume, `migrate` first);
  - `bootstrap`.
- `conformance/identity/bootstrap.sh`, which uses Zitadel's API with the first instance's machine-user key. It creates:
  - org `acme`;
  - human user `alice`;
  - machine user `agent-1`, with a private-key JWT key written to `.secrets/`;
  - an OIDC application `loams-console` (code + PKCE, JWT access tokens);
  - and writes `.env.ia0` with the issuer, the client id and the project id.
- `conformance/identity/README.md`.

Everything listens on `127.0.0.1` only.

**Footprint:** measure idle memory and CPU of each container after 5 minutes (`docker stats --no-stream`) and record them for the report.

**Tests:** `conformance/identity/smoke.sh` (bring up, wait for health, run the bootstrap, assert that `/.well-known/openid-configuration` and OpenFGA's `/healthz` answer, bring down) in a CI job `identity-smoke`.

Commit: `conformance: the IA0 Zitadel and OpenFGA compose stack (IA0 Task 1)`.

### Task 2 — Model v1 and its tests

**What to write:**

- `deploy/openfga/core.fga`: the draft's model with §76 §6.2's changes (`service_account`, `platform`, `team.org`, `billing_admin` and `billing_viewer`, `meter_admin`, `database.can_connect`, and the `agent_session` condition).
- `deploy/openfga/fga.mod`, listing `core.fga` and `graph.fga`.
- `graph.fga` moves from `namespace` to `environment` (Q1207's default). Its `reader`, `writer` and `admin` take `viewer`, `editor` and `deployer from environment`. Its header comment drops "Until MT1's OpenFGA authorizer lands" in favour of §76.
- `deploy/openfga/tests/core.fga.yaml`, with OpenFGA CLI model tests:
  - a seeded org, project, environment, deployment and database;
  - checks for every `can_*` relation and each role;
  - `agent_does_not_inherit_owner`;
  - `deny_by_default_without_tuple`;
  - `agent_session` allowed before and denied after `expires`.
- A second file with two orgs, for `no_cross_org_tuples` (a script over `fga tuple read` output).

**Tests:** a CI job `openfga-model` runs `fga model validate` and `fga model test`. A script test asserts that writing `billing_admin@agent:x` fails with a type-restriction error (`agent_cannot_write_grants`).

Commit: `openfga: model v1 (core.fga) with tests; graph.fga hangs off environment (IA0 Task 2)`.

### Task 3 — JWT → Check, end to end (spike, not merged)

On branch `ia0-spike`, write `spikes/ia0-authz/`, a standalone Cargo package outside the workspace, built with `--manifest-path`. It is an Axum service with:

- **JWT verification:** JWKS from the issuer's discovery document, cached for 5 minutes. It checks `iss`, `aud` (Task 0's value), `exp` with 60 s of skew, and the algorithm (RS256).
- **A `Subject`** from `sub` and `urn:zitadel:iam:user:resourceowner:id`.
- **Routes:**
  - `GET /deployments/:id` → `can_view`;
  - `POST /deployments/:id/restart` → `can_operate`;
  - `DELETE /deployments/:id` → `can_destroy`, with `HIGHER_CONSISTENCY`;
  - `GET /deployments` → `ListObjects`.
- **Deny by default:** an unknown route answers 403.

**Run:**

1. Alice, with no tuple, is denied.
2. Alice is granted `viewer` on the project and is allowed `can_view` but denied `can_destroy`.
3. `agent-1` (private-key JWT → token) is denied until granted `editor` on the environment.
4. Deleting the grant takes effect on the next `HIGHER_CONSISTENCY` check.

**Tests:** `rejects_wrong_aud`, `rejects_wrong_iss`, `allow_after_grant`, `deny_after_revoke` (integration tests against Task 1's stack).

Not merged. Task 6's report links the branch and quotes the results.

### Task 4 — Actions v2 into a tuple (spike, not merged)

On the same branch, add a `POST /zitadel/events` endpoint that verifies `ZITADEL-Signature` (the HMAC over the body and timestamp, with the target's signing key) and maps "org member added" and "org member removed" to OpenFGA writes and deletes.

Register a `Webhook` target and an event execution for those events. Then measure, over 100 add/remove pairs:

- the time from the Zitadel call to the tuple being present;
- out-of-order arrivals;
- missed events, compared against a management-API listing afterwards. This is the reconciler's diff in miniature.

**Tests:** `rejects_bad_signature`, `idempotent_duplicate_add`, `missing_delete_is_success`.

Not merged. Task 6 reports the numbers.

### Task 5 — Latency baseline

With Task 3's service, measure `Check` latency against OpenFGA on loopback:

- p50, p95 and p99 over 10 000 checks;
- for `MINIMIZE_LATENCY` and `HIGHER_CONSISTENCY`;
- single `Check` against a 50-item `BatchCheck`.

Use `oha` or a small Rust driver. The numbers seed Q1214's target.

Files: the report only.

### Task 6 — The IA0 report

Write `docs/plans/ia0-spike.md`:

- the pins and digests;
- the footprint table;
- the answers to Task 0's (verify) items;
- the allow and deny results;
- the Actions v2 delivery numbers;
- the latency table;
- a recommendation for each open question §76 lists for IA1–IA2 (Q1205–Q1211, Q1214);
- anything in §76 that the spike contradicts.

Update §76's (verify) markers that the report settles. Update this plan's "Rulings made during execution".

Gate: the owner reads the report before IA2 starts.

Commit: `docs: the IA0 identity and authorization spike report (IA0 Task 6)`.

## IA1 — Console login and the Authentik cutover

### Task 1 — Remove Authentik (D1600, approved; a removal, not a migration)

**No user, group, session or blueprint is exported or migrated.** Nothing reads Authentik's database, and no compatibility switch is added. This task can run before IA0 finishes, because its only dependency on Zitadel is the replacement bundle at `deploy/release/auth/`, which reuses Task 1 of IA0's compose stack.

**Remove, or replace with the Zitadel equivalent** (the inventory is §76 §13; re-run `git grep -il authentik` first and add anything new to this list):

1. **The hosted auth bundle.** Delete Authentik's compose, Caddyfile, blueprints (`loams-console.yaml`, `loams-confidential-client.yaml`), `setup.sh`, README and `.env.example` from `deploy/release/auth/`. In their place, add a Zitadel bundle in the same directory:
   - `compose.yaml` with Zitadel, Zitadel Login, Postgres and Caddy;
   - `setup.sh`, which creates the `loams-console`, `loams-desktop`, `loams-mobile` and `headscale` applications and the GitHub and Google IdPs through Zitadel's API;
   - `README.md`, keeping the "why" section rewritten for D1600;
   - `.env.example`.
2. **The Authentik deploy kit.** Delete `deploy/authentik/` entirely:
   - `README.md`;
   - `blueprints/loams-01-groups.yaml` through `loams-05-headscale.yaml`;
   - `branding/`;
   - `compose/` (`docker-compose.yml`, `docker-compose.cli-tunnel.yml`, `.env.example`, `backup/backup.sh`, `backup/restore.sh`);
   - `helm/` (`values.yaml`, `postgres-cnpg.yaml`, `cloudflared.yaml`, `backup-cronjob.yaml`, `namespace.yaml`, `secrets.example.yaml`);
   - `kustomization.yaml`.
3. **The tailnet.**
   - Delete `deploy/tailnet/authentik-tailnet/`.
   - Delete `deploy/tailnet/tailscale/k8s/tailscale-authentik.yaml` and its entry in `deploy/tailnet/tailscale/kustomization.yaml`.
   - Point Headscale's OIDC (`deploy/tailnet/headscale/config/config.yaml`, `compose/.env.example`, `compose/docker-compose.yml`, `k8s/secrets.example.yaml`) at Zitadel.
   - Rename `tag:authentik` to `tag:zitadel` in `deploy/tailnet/policy/policy.hujson` and `deploy/tailnet/tailscale/k8s/secrets.example.yaml`, and run the policy tests.
   - Update `deploy/tailnet/README.md`.
4. **Release config.** Rewrite `deploy/release/.env.example`'s `oidc` comments for Zitadel. The audience is Task 0's value.
5. **Protos and generated code** (D1619, Q1209):
   - In `proto/loams/instance/v1/instance.proto`, add `SIGN_IN_KIND_OIDC = 5` and `SIGN_IN_KIND_OIDC_DEVICE_CODE = 6`, mark `SIGN_IN_KIND_AUTHENTIK` and `SIGN_IN_KIND_AUTHENTIK_DEVICE_CODE` `[deprecated = true]`, and rewrite the comments without Authentik.
   - In `proto/loams/auth/v1/auth.proto`, add `OIDC_PROVIDER_KIND_ZITADEL = 8`.
   - Update `api/console/openapi.json`.
   - Copy the change into `apps/mobile/proto/loams/instance/v1/instance.proto` and `crates/loams-agentd-link/proto/loams/instance/v1/instance.proto`.
   - Regenerate every generated tree (`web/packages/proto`, `sdks/*/gen` and descriptors, `sdks/python/src/loams/*_pb2*`, `sdks/csharp/src/Loams/Gen`, `apps/mobile/*` generated code, `crates/loams-agentd-link/src/gen`, `apps/mobile/mock/gen`) and the recorded fixture `sdks/fixtures/recorded/apps-mock/mock_status_get_instance.json`.
   - `buf breaking` must pass with no exception.
6. **Code:**
   - `plugins/core/host/src/auth/*`: `AuthentikOidcClient` becomes `OidcClient`'s default implementation, with IdP-neutral comments, and `tests/auth.spec.ts` and `tests/fake-idp.ts` follow. `plugins/core/store/src/service.ts` follows too.
   - `crates/loams-apps-mock` (`oauth.rs`, `seed.rs`, `store.rs`, `services/mod.rs`, `main.rs`, `Cargo.toml`, `tests/mock.rs`): it serves `SIGN_IN_KIND_OIDC`.
   - `crates/loams-agentd-link/src/auth/{mod.rs,oidc.rs}` and `src/lib.rs`, `crates/loams-agentd-connect/src/oidc.rs`, `crates/loams-agentd-proto/src/brand.rs`, and `crates/loams-sidebar-browser/src/{lib.rs,navigate.rs,session.rs}` with its README.
   - `crates/loams/src/auth/oidc.rs`: comments only.
   - `crates/loams-live/tests/identity.rs`.
   - Android: `AuthentikSignIn.kt` becomes `OidcSignIn.kt`, and `SessionManager.kt`, `WelcomeScreen.kt`, `WelcomeViewModel.kt`, `Pkce.kt`, `PairingPayload.kt` and `TokenEndpoint.kt` follow.
   - iOS: `WebAuth.swift`, `AppModel.swift`, `WelcomeView.swift`, `TokenEndpoint.swift`.
   - The Go mobile mock (`internal/server/*.go`).
   - `web/plugins/org/src/settings.tsx`: add a Zitadel button and drop Authentik from the SAML hint.
   - SDK token-source docs: `sdks/go/token_source.go`, `sdks/rust/src/token.rs`, `sdks/python/src/loams/runtime/token_source.py`, `sdks/swift/Sources/Loams/TokenSource.swift`, `sdks/typescript/packages/client/src/runtime/token-source.ts`.
7. **Docs:**
   - Replace `docs/platform/deploy/authentik.md` with `zitadel.md`.
   - Update `docs/platform/deploy/headscale.md`, `tailnet.md`, `docs/platform/README.md`, `docs/platform/design/04-hosted-loams-cloud-on-cloudflare.md`, `05-hosted-software-factory-and-marketplace.md` and `docs/platform/design/README.md`.
   - Update `docs/guides/studio/remote-agentd.md`, `docs/monorepo.md`, `docs/open-issues-analysis.md`, `plugins/docs/auth-setup.md`, `plugins/docs/README.md`, `plugins/README.md`, `plugins/SECURITY.md`, `platform/README.md`, `apps/mobile/README.md`, `apps/mobile/docs/*.md`, `apps/mobile/mock/README.md` and `README.md`.
8. **Plans.**
   - Mark MT1 superseded by IA1.
   - Point at IA1 from MT3's Authentik waves and `Application` (MT3 Task 1), NET1's Authentik roster sync, and the Authentik mentions in CP1, MT4, SF1–SF5, SD1, JB1, GT1, CL1, API2, DD1, SQ1, PG2, LV1, HS1, AP1e and SO2.
9. **Notices.** Remove the Authentik entry from `NOTICE` (it attributed the deleted recovery-flow blueprint). Add a `CHANGELOG.md` entry; past entries stay as they are.
10. **Other repositories and operations** (owner or a separate PR):
    - `ostrium-labs/loams-site` changes its Auth.js provider to Zitadel.
    - After Zitadel answers at `auth.loams.dev`, the owner deletes the Authentik containers, their Postgres volume and backups, and the Cloudflare Tunnel route for Authentik's admin paths.
    - **No period with both running.**

**Gate:** `git grep -il authentik` matches only:

- `docs/design/13-decision-log.md`;
- the designs that record history: §19, §22, §37, §38, §39, §41, §43, §63, §64, §74, §76;
- superseded plans;
- `CHANGELOG.md`;
- the deprecated proto values and their generated code.

A CI script `scripts/docs/check-no-authentik.sh` enforces this allowlist.

**Tests:**

- The console's auth tests against the fake IdP, renamed.
- `loams-apps-mock`'s tests with `SIGN_IN_KIND_OIDC`.
- The mobile unit tests.
- `buf lint` and `buf breaking`.
- The tailnet policy tests.
- `scripts/docs/check-no-authentik.sh`.

Commit: one PR per numbered group where it keeps PRs small (for example, 1–4 deploy, 5 protos, 6 code, 7–9 docs). Each commit is titled `identity: remove Authentik (<group>) (IA1 Task 1, D1600)`.

### IA1 outline

- **Zitadel at `auth.loams.dev`** (D1615): the deploy runbook; backups; the admin console on the tailnet only (D596's rule); GitHub and Google IdPs; branding through settings (Q1211).
- **The `identity` compose profile** (D1613): Zitadel, Zitadel Login and `zitadel-db`, off by default.
- **Sign-in at Zitadel:**
  - console: code + PKCE;
  - `loams login`: device code;
  - desktop: loopback redirect;
  - phone: system browser.
- **The engine's `oidc` mode:** matches members by `(iss, sub)` instead of email (D1605), and reads the org from `urn:zitadel:iam:user:resourceowner:id`.
- **Exit:** login, MFA, passkey and org switch work.

## IA2 and IA4 — the agent path first (Tasks specified after IA0)

IA0 reported **go** ([report](ia0-spike-report.md)). The owner put agent authentication first (2026-10-21: "current agent auth is too complex and friction"), so these tasks run in this order: IA2 Tasks 1–3, IA2 Task 4 and IA4 Tasks 1–3 (IA4 Task 1 by a clean-room agent), then the rest of IA2. One branch and one PR per task, each green before merge. F1 and F3 of the report change IA3 only and wait for the owner.

### IA2 Task 1 — `crates/loams-authz`: the trait and four implementations

**What to write** (D1610, §9.1):

- `Subject { kind: SubjectKind (User | ServiceAccount | Agent), id, org: Option<String>, act: Vec<Subject>, session: Option<String> }`, with `Subject::fga_user()` → `user:<id>`, `agent:<id>` or `service_account:<id>`.
- `ObjectRef { ty, id }` (parsed from and shown as `type:id`), `Consistency` (`MinimizeLatency`, `HigherConsistency`), `CheckRequest { subject, relation, object, context, contextual_tuples, consistency }` and `Decision { allowed, model_id, cached }`.
- `#[async_trait] trait Authorizer { check, batch_check, list_objects }`; `batch_check` has a default that runs the checks concurrently.
- `OpenFgaAuthorizer` (`openfga-client` =0.6.1, gRPC): one client per consistency; `batch_check` in chunks of 50 (`max-checks-per-batch-check`); `list_objects`; idempotent `write` and `delete` for the outbox and sync callers.
- `CachedAuthorizer<A>`: allow decisions only, TTL clamped to ≤ 5 s (D66), keyed by `(model_id, user, relation, object)`; bypassed for `HigherConsistency` and for requests with context or contextual tuples; `invalidate` and `invalidate_all` for IA3's `ReadChanges` feed. It uses `moka` (in the tree): `loams-cache` is a byte-range cache over immutable objects, not a decision cache.
- `AllowAllAuthorizer`, built only by `AllowAllAuthorizer::loopback_only(&[SocketAddr])`, which refuses any non-loopback listener (as `LOAMS_AUTH_MODE=off` does).
- `RecordingAuthorizer<A>`: records every request and decision (tests and the decision log).

**Tests:** `cached_allows_only`, `cache_ttl_is_clamped`, `higher_consistency_bypasses_cache`, `context_bypasses_cache`, `allow_all_refuses_non_loopback`, `recording_records_denials`, `subject_fga_user`, `object_ref_round_trip`; and, when `LOAMS_AUTHZ_OPENFGA_GRPC` is set, `openfga_deny_by_default`, `openfga_allow_after_grant`, `openfga_batch_check_over_50`, `openfga_list_objects` against model v1 (CI: the Identity workflow starts the `authz` profile).

Commit: `authz: the loams-authz crate, its trait and implementations (IA2 Task 1, D1610)`.

### IA2 Task 2 — deny-by-default middleware

**What to write** (D1611, §9.2): `loams_authz::http`:

- `Need { relation, object, consistency }`, where `relation` must start with `can_` (D1603) and `consistency` is `HigherConsistency` for destructive, admin and credential-issuing routes.
- A route table: a function from `(method, path)` to `Declared::Check(Need) | Declared::Authenticated | Declared::Public`; anything it does not declare answers **403** with `no authorization is declared for this route`.
- An Axum/tower layer that reads the `Subject` the authentication layer put in the request's extensions (401 without one), checks the route's `Need`, and answers 403 on deny and 503 when the authorizer fails (never an allow).
- Every decision, allow and deny, is a `tracing` event on target `loams_authz::decision` with subject, `act`, relation, object, decision, model id and request id (D1616's decision log; denials rate-limited per subject later in IA5).

**Tests:** `undeclared_route_is_403`, `missing_subject_is_401`, `deny_is_403`, `allow_passes_through`, `authorizer_error_is_503`, `destructive_route_uses_higher_consistency`, `need_rejects_non_can_relation`, `decision_log_records_denials`.

Commit: `authz: the deny-by-default middleware (IA2 Task 2, D1611)`.

### IA2 Task 3 — `BuiltinAuthorizer` and `builtin_matches_openfga`

**What to write** (D1613, §12.1): `BuiltinAuthorizer` evaluates model v1 (`deploy/openfga/core.fga` and `graph.fga`) in process over a `TupleStore` (in memory now; the principal store in IA2 Task 4), including the `agent_session` condition and type restrictions on write.

**Tests:** `builtin_matches_openfga` runs every check and list in `deploy/openfga/core.fga.yaml` against `BuiltinAuthorizer` (always) and against OpenFGA (when `LOAMS_AUTHZ_OPENFGA_GRPC` is set), and both must match the file's assertions. `builtin_refuses_agent_billing_admin`.

Commit: `authz: BuiltinAuthorizer, with the differential test against OpenFGA (IA2 Task 3, D1613)`.

### IA2 Task 4 — the engine enforces through `loams-authz` (agent-facing surfaces first)

**What to write** (D1605, D1610, D1611; report F5, F7):

- **Subjects.** `crates/loams/src/auth` maps every admitted principal to a `loams_authz::Subject` and puts it in the request's extensions:
  - an OIDC JWT: the principal store's link `(iss, sub) → (kind, status)` (a new `PrincipalLink` record in `directory.rs`; no link means `user`). A `disabled` link is refused with `token_revoked` on every request, whatever the JWT's `exp`;
  - a session: `user:<user id>`; an `lk_` key: `service_account:<id>`; a tailnet user: their linked user; `LOAMS_AUTH_MODE=off`: no subject, and `AllowAllAuthorizer`.
- **Selection.** `LOAMS_AUTHZ` = `builtin` (default, D1613) | `openfga` (`LOAMS_AUTHZ_OPENFGA_GRPC`, `_STORE`, `_MODEL`, optional `_KEY`). Either is wrapped in `CachedAuthorizer` (5 s); `off` is accepted only with `LOAMS_AUTH_MODE=off` on loopback. In `builtin` mode the principal store is the tuple source.
- **Model.** `environment` gains `can_read: viewer`, `can_write: editor` and `can_admin: deployer`, for data-plane calls scoped to one namespace (collections, KV, Live, Durable), with model tests.
- **First surfaces.** `namespace_reach.rs`, `graph_auth.rs` and `durable.rs` ask `can_read`/`can_write`/`can_admin` on `environment:<namespace id>` (Graph on `graph:<ns>/<name>`) through the trait; the admin RPCs keep `OrgRoleAuthorizer` until IA2 Task 5 declares a `can_*` per RPC.

**Tests:** `agent_jwt_reaches_only_granted_namespace`, `disabled_link_refuses_live_jwt`, `session_maps_to_user`, `lk_key_maps_to_service_account`, `builtin_is_the_default`, `authz_off_refused_off_loopback`, and the existing auth tests unchanged.

### IA4 Task 1 — agents as Zitadel machine users

**Clean room.** `loams-identity` is written by an agent that has not read Zitadel's source, from Zitadel's public documentation and its Apache-2.0 protos only (the owner's rule; the IA0 report's provenance note). The IA0 report, the spike and its worktree are not inputs.

**What to write** (D1607, §10): `crates/loams-identity`, a client for Zitadel's public HTTP APIs (no Zitadel code): create a machine user with JWT access tokens, add and remove a private-key JWT key, deactivate and reactivate, and the principal link `(issuer, sub) → (kind, status)`. An agent is created with its `owner` and `project` tuples and no grant; a grant is one tuple on an environment, plain or `agent with agent_session`. Revocation deletes the agent's tuples and deactivates the machine user, and the link's status is checked per request (report F5).

**Tests:** against the IA0 stack when `LOAMS_IDENTITY_ZITADEL_URL` is set: `create_agent_has_no_grant`, `grant_reaches_only_that_environment`, `revoke_denies_within_seconds`, `deactivated_agent_gets_no_token`.

### IA4 Task 2 — the agent's token source

**What to write:** a `MachineKey` token source in the Rust SDK (`sdks/rust/src/token.rs`): read Zitadel's JSON key file (`LOAMS_AGENT_KEY_FILE`), sign a 5-minute RFC 7523 assertion, trade it for an access token with the project audience scope, refresh before `exp`. Then the TypeScript, Python and Go SDKs.

### IA4 Task 3 — agentd vends 15-minute tokens (D1618)

Outlined; specified after IA4 Task 1 lands.

## IA2–IA6 outline

- **IA2, the authorization core:**
  - `crates/loams-authz` with the trait and the `OpenFga`, `Cached`, `Builtin`, `AllowAll` and `Recording` implementations (D1610);
  - per-RPC `can_*` declarations and deny by default (D1611);
  - `BuiltinAuthorizer` replacing `OrgRoleAuthorizer`, with the differential test `builtin_matches_openfga`;
  - the `authz` profile; the Helm values for OpenFGA;
  - Lakekeeper on its own store (D1604);
  - `graph.fga`, `loams-graph` and `loams-pg-control` moved onto `loams-authz`.
  - The principal's kind from the link table `(issuer, sub) → principal`; a subject with no link is checked as `user:<sub>` (D1628). Test: `unlinked_subject_is_a_user`.
  - The middleware refuses a disabled principal on every request, whatever its token's signature says (D1624). Test: `disabled_principal_refused_with_valid_jwt`.
  - One decision cache: `CachedAuthorizer`, allows only, TTL ≤ 5 s, with OpenFGA's check-query and iterator caches off in the `authz` profile and the Helm values (D1625). Test: `cache_ttl_at_most_5s`.
  - The `authz` profile keeps SQLite for a single user; the Helm values and multi-user installs use Postgres, and on SQLite lists use `ListObjects` (D1626).
  - Exit: every control-plane and engine route sits behind a `can_*` check.
- **IA3, the sync bridge:** `identity-sync` (D1608, amended by D1622 and D1623), the resource-tuple outbox with `pending` → `active` (D1609), the nightly reconciler and its drift metric.
  - **Primary feed: Zitadel's event list** (`ListEvents`, §76 §8.1's event types, ascending) from a position stored in the metastore: the last creation date plus the `(aggregate, sequence)` pairs at it (D1622). It reads every few seconds and at once on a webhook. Tests: `resumes_from_stored_position`, `reread_dedupes_by_aggregate_sequence`, `lost_webhook_still_applied`.
  - **Actions v2 webhooks are a wake-up only:** the endpoint verifies `ZITADEL-Signature` and triggers a read; it never applies the body. Test: `webhook_body_not_applied`.
  - **Membership from user grants** on the Loams project: `user.grant.added|changed|removed|cascade.removed` with role keys `member`, `admin`, `owner` map to `org:O#member|admin|owner`; `org.member.*` (Zitadel's managers) map to nothing (D1623). Tests: `grant_roles_map_to_org_relations`, `org_manager_is_not_a_member`.
  - `user.deactivated|locked|removed` mark the principal disabled before deleting its tuples (D1624); `user.machine.added` writes the principal link (D1628).
  - Zitadel's `HTTPClient.DenyList` narrowed in compose and Helm to allow only the sync endpoint's network, keeping `0.0.0.0/8` and `169.254.169.254/32` blocked (D1627).
  - The reconciler diffs user grants (not org members) against OpenFGA.
  - Exit: a member removal shows within seconds, also when its webhook is lost, and drift is 0.
- **IA4, agents** (D1607, D1618, amended by D1624 and D1628): agents as machine users; the `agent` and `service_account` types in use; `agent_session` grants; agentd and supervisor vending with machine-user `sub`; revocation tests.
  - Creating an agent creates the machine user and its key, writes the principal link with kind `agent` (D1628) and the `owner` and `project` tuples, in one action.
  - Revocation from Loams deletes the tuples, deactivates the machine user and marks the principal disabled in one call, so the agent's issued tokens are refused at once (D1624). A deactivation made in Zitadel's console reaches the same state through IA3. Tests: `revoked_agent_old_token_refused`, `revoked_agent_new_token_refused`; target ≤ 2 s (IA0 measured about 1 s).
  - Exit: an agent touches only granted environments, and revocation is tested.
- **IA5, audit and hardening** (D1616): the three feeds in `_audit` and the Iceberg table family; the cross-org tuple scan in CI; a load test; cache and consistency tuning. Exit: Q1214's p99 target and no cross-org tuples.
- **IA6, optional:** a Rust-native `Authorizer` backend and/or a Rust IdP behind the same traits, only if OpenFGA or Zitadel becomes a bottleneck or a licence issue.

## Rulings made during execution

| # | Ruling | Task |
|---|---|---|
| R0.1 | The report is `docs/plans/ia0-spike-report.md`, not `ia0-spike.md` (the owner's brief, 2026-10-21) | 6 |
| R0.2 | Pins (14-day rule on 2026-10-21): Zitadel and Zitadel Login v4.19.4, OpenFGA v1.22.0, OpenFGA CLI v0.8.1 (v0.8.2 is 6 days old), Postgres 17.10-alpine (Zitadel's own compose pin), all by digest; the digests are in the report §4.1 | 0 |
| R0.3 | The spike code is merged with the report, as `spikes/ia0-authz`, a standalone package outside the workspace (`[workspace]` of its own, built with `--manifest-path`), so it still adds nothing to `loams`'s build (the owner's brief: "open it, with the spike code, as a PR") | 3 |
| R0.4 | The tree pins `jsonwebtoken` =10.4.0 and has no `openidconnect`; the spike uses 10.4.0 and its own 40-line discovery fetch instead of `jsonwebtoken` 11 and `openidconnect` 4 | 3 |
| R0.5 | `bootstrap.sh` runs on the host (curl and jq) instead of as a compose `bootstrap` service, and Zitadel writes the IAM-owner and login-client PATs into the git-ignored `.secrets/` at `start-from-init` | 1 |
| R0.6 | The model tests are `deploy/openfga/core.fga.yaml`, beside `fga.mod`: the CLI refuses a test file's `model_file` outside its own directory | 2 |
| R0.7 | The spike service runs in the compose network (profile `spike`): under rootless Podman a container cannot reach the host's loopback, and Zitadel's webhook must reach it | 4 |
| R0.8 | Answers to Task 0's (verify) items, and findings F1-F7 (Actions v2 is at-most-once; Zitadel org members are managers; SQLite serializes batches; JWTs outlive deactivation; caches stack; principal kind from the link table; the HTTP deny list): the report §5 and §6. F1 and F3 amend D1608 and §8.1 | 0-6 |
| R0.9 | The owner approved all seven findings on 2026-10-21 ("approve all"): D1622 (F1, the event list as primary feed), D1623 (F3, membership from user grants), D1624 (F5, the disabled-principal check), D1625 (F6, one cache ≤ 5 s), D1626 (F4, SQLite for one user only), D1627 (F2, the deny list), D1628 (F7, the kind from the link table). IA2, IA3 and IA4 above carry them | IA2–IA4 |
| T1.R1 | `deploy/release/auth/` keeps only a README. The Zitadel bundle (compose, `setup.sh`, `.env.example`) lands with IA1's `auth.loams.dev` work, on IA0 Task 1's pinned stack, rather than being written before IA0 has verified the images, the audience and the API calls. Until then an install signs in with built-in single-org mode (D1613: `tailscale`, console sessions with first-run setup, `lk_` keys) or `oidc` mode against any issuer | IA1 Task 1 |
| T1.R2 | `deploy/tailnet/tailscale/` is deleted whole: it held only Authentik's k3s tailnet node. Zitadel's node comes with its bundle. In `policy.hujson`, `tag:zitadel` takes `tag:authentik`'s place and the admin port becomes Zitadel's `8080` (was `9000`); policy lint and Headscale's checker pass | IA1 Task 1 |
| T1.R3 | Headscale keeps `allowed_groups: [loams-net-users]`. Zitadel asserts project roles, not `groups`, so the bundle must map the role into a `groups` claim for the `headscale` application; until it does, nobody signs in to Headscale through OIDC (pre-auth keys still work) | IA1 Task 1 |
| T1.R4 | The gate (`scripts/docs/check-no-authentik.sh`, list in `scripts/docs/no-authentik-allowlist.txt`) also accepts the file names `38-knative-authentik-gitops` and `2026-10-02-mt1-authentik-identity` in links anywhere, and its own name (`no-authentik`), which CI runs it by: renaming those two historical documents would break every link to them, including loams.dev's copies of the designs. The allow-list adds this plan, the gate's own files, `api/console/openapi.json` and `web/plugins/org/src/settings.tsx` (which keep `OIDC_PROVIDER_KIND_AUTHENTIK`, a customer's upstream IdP), and the generated code | IA1 Task 1 |
| T1.R5 | The mocks' fake IdP lives at `/mock/oidc/` and issues `mock-oidc-…` tokens (`loams-apps-mock` and the Go mobile mock); the seed serves `SIGN_IN_KIND_OIDC`, "Sign in with your organization". The live end-user test is `oidc_end_user_e2e` with `LOAMS_TEST_OIDC_ISSUER`, `_AUDIENCE` and `_TOKEN` | IA1 Task 1 |
| T1.R6 | `plugins/core/host`'s OIDC client is `HttpOidcClient`. The trailing-slash rule for `OIDC_ISSUER` (an Authentik quirk) is replaced by an exact, byte-for-byte match with the discovery document's `issuer` (OIDC Discovery §4.3), so Zitadel's slash-less issuer works and no mismatch is accepted | IA1 Task 1 |
| T1.R7 | Plan paths for Zitadel that IA1 has not fixed yet (MT3's `deploy/zitadel/values.yaml`, NET1's `deploy/zitadel/apps/loams-headscale.yaml`) are provisional and are settled by the IA1 task that writes them | IA1 Task 1 |
| T2.R1 | `loams_authz::http::segments` percent-decodes a path the way axum's `Path` extractor does, so a route table checks the object the handler acts on, and a segment that is not UTF-8, `.` or `..` makes the route undeclared (403). `Need::new` also refuses an object with an empty type or id, whitespace, `#`, or `:` in the type, so a path segment cannot name a userset | IA2 Task 2 |
| T3.R1 | One condition context for both backends: `CheckRequest::effective_context` sets `session_id` from the authenticated subject's session (over the caller's) and `now` from the server's clock when the caller gave none. A subject with a session is not cacheable (the cache key carries neither session nor time). A tuple may not store `session_id` or `now` (`Tuple::check_condition`): a stored value wins in OpenFGA, so it would unbind or never expire an `agent_session` grant. `BuiltinAuthorizer` refuses tuples that are not `type:id`, and wildcards, which model v1 does not declare | IA2 Task 3 |
| IA4.R1 | The crate is `crates/loams-zitadel`, not `crates/loams-identity`: that name is the clean-room identity provider's (`ostrium-labs/loams-identity`), and two crates of one name cannot both publish. It is a client of Zitadel's documented v2 user API and token endpoint, and takes the caller's `reqwest::Client`, so TLS is the process's | IA4 Task 1 |
| IA4.R2 | Written from Zitadel's public documentation and its Apache-2.0 protos (`proto/zitadel/user/v2`; Zitadel's `LICENSING.md` lists `proto/` as Apache-2.0) only. Neither the IA0 report nor the spike was read; the IA0 stack (`conformance/identity`) is used as a black box: its README, `up.sh`, and the names of the variables `.env.ia0` exports | IA4 Task 1 |
| IA4.R3 | An agent's id is its machine user's id (`agent:<sub>`). Zitadel generates the key pair by default and hands the key file out once (`AddedKey::key_file`); `add_key` also takes a public key, for a caller that keeps the private key itself. Keys last 90 days unless set. A failed step after the machine user exists deletes it again and disables its link | IA4 Task 1 |
| IA4.R4 | The tuple store and the link store are traits (`TupleStore` for OpenFGA; `LinkStore`, with `MemoryLinks`). Wiring the link store to the engine's principal store (`Doc.links`, IA2 Task 4) and the agent RPCs to `Agents` is IA4 Task 3's. `revoke` disables the link first (D1624), then deletes the grants it is given and deactivates the machine user, running every step and returning the first failure; the `owner` and `project` tuples stay (they grant nothing), so `reinstate` needs new grants | IA4 Task 1 |
| IA4.R5 | The Rust SDK's `MachineKey` is behind the feature `machine-key` (jsonwebtoken on its pure-Rust RSA backend), off by default so the SDK's default build gains no crypto; it takes the SDK's `FormPoster` for the token call, as `OidcExchange` does. The Go SDK's `MachineKey` uses the standard library only. The tests sign with the repository's existing test-only RSA key (Rust) or a key generated per test (Go). The TypeScript source (`machineKey`, WebCrypto, PKCS #1 or #8 keys) is in the same PR; so is Python's (`loams.runtime.machine_key`, sync and async), behind a new `agent` extra that adds `cryptography` 50.0.2 (pycparser held below 3.12 by the 14-day rule) | IA4 Task 2 |
| S.2026-10-24 | **Status, 2026-10-24.**<br>**Merged:** IA2 Task 2 (#1009); IA4 Task 2 (#1145: `MachineKey` in the Rust, Go, TypeScript and Python SDKs).<br>**Open, green or stacked:**<br>• IA2 Task 3: #1143, on `dev`.<br>• IA2 Task 4: #1154, draft, stacked on #1143.<br>• IA2 Task 5: #1169, draft, stacked on #1154.<br>• IA4 Task 1: #1155, every check green, including the live Zitadel job; it was re-running `CI required` after it was marked ready.<br>**Owner decision pending:** T4.R2.<br>**Blocked:** IA4 Task 3. Agentd's rotating token file (D1453) and a Loams token endpoint do not exist yet.<br>**Still to do:**<br>• `RemoveMember` should revoke the person's keys (T5.R1).<br>• IA2's `authz` compose profile and Helm values, with OpenFGA's caches off (D1625) and SQLite for one user (D1626).<br>• Lakekeeper's own store (D1604).<br>• Moving `loams-graph`, `loams-pg-control`, `/durable/` and Loams Git onto `loams-authz`.<br>• IA3, the sync bridge, which `LOAMS_AUTHZ=openfga` needs for its tuples.<br>• The engine wiring of `loams-zitadel`'s `LinkStore` and the agent RPCs.<br>**Review:** Greptile did not review any of the `review:greptile` PRs, probably because its credits ran out. The PRs were self-reviewed | status |

### Owner approvals (2026-10-24)

The owner approved the pending decisions below on 2026-10-24 ("approve all with your recommendations").

- **T4.R2 (IA2 Task 4): option 2.** Org members see only the projects they are granted. The project `viewer` relation in `deploy/openfga/core.fga` loses `or member from org`, the model tests (`deploy/openfga/core.fga.yaml`) and `BuiltinAuthorizer`'s relation table change with it, and the differential test `builtin_matches_openfga` checks both modes against the same expectations. The built-in projection already writes no `project#org` for members, so it needs no change. **Approved (owner, 2026-10-24).** The row T4.R2 itself is in #1154's copy of this table; mark it "Approved (owner, 2026-10-24), option 2" when that PR merges.
