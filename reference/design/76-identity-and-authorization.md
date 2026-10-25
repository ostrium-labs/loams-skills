# 76 — Identity and Authorization: Zitadel and OpenFGA

> **Amendment (approved 2026-10-23) by [§78](78-loams-cloud-on-tidb-x.md) §25, D1750: the hosted Loams Cloud try-out uses WorkOS AuthKit, not Zitadel.** This amends Q1201's default ("self-hosted Zitadel at `auth.loams.dev`, not Clerk or WorkOS") and D1615 **for the hosted try-out only**: sign-up, sign-in (Google, GitHub, Microsoft, work-email codes; D1751), email verification and invitations run on WorkOS's free tier, and `auth.loams.dev` is not used for the try-out. Self-hosted and company installs keep everything in this document (Zitadel and OpenFGA, or the built-in single-org mode). The try-out's admin role is a WorkOS organisation role plus Loams' allow-list and a TOTP step-up, still checked in `loams-authz` as `platform:loams-cloud#cloud_admin` (D1746, approved 2026-10-23).

> **Amendment (approved 2026-10-23) by [§78](78-loams-cloud-on-tidb-x.md)** (Loams Cloud on TiDB X; D1700–D1735). D1615's list of what Ostrium hosts loses `console.loams.dev` (retired, D1705) and gains the Loams Cloud control plane (D1701). Devices are machine users of kind `device` under the user's personal org (D1714, D1725); prod installs keep their own issuer.

Status: **Approved** · 2026-10-21 (written and approved the same day). D1600, the owner's ruling of 2026-10-21 that Loams cuts over from Authentik to Zitadel completely, with no migration path, no compatibility period and no dual running because there are no users, came first. The owner then approved the whole design with every default: D1601–D1621 are approved and Q1200–Q1214 are answered with their defaults.

Source: the owner's draft "Loam Identity & Authorization Plan: Zitadel + OpenFGA" (draft v1, pasted 2026-10-21, kept at `~/Documents/Ostriumlabs/pitch/research/identity-zitadel-openfga-owner-plan.md`), with the instruction "add this". This document carries the draft into the repository's design format and renames "Loam" to "Loams". It also does three more things:

- It **reconciles the draft with what exists**: Authentik (§38), the principals and tokens of §19, the `Authorizer` and outbox of §18 (D66, D67), the OpenFGA fragment `deploy/openfga/graph.fga`, the engine's auth layer (`crates/loams/src/auth/`), the light compose default, BYOC (§70, §71) and the edge (§69).
- It **fixes three outdated assumptions in the draft**:
  - Loams is fully open source (D1119). There are no closed `loam-cloud` or `loam-platform` layers; the former private repository was merged into this one.
  - Ostrium Labs offers no services (D1323).
  - The "cloud tier" means only the hosted console (`console.loams.dev`) and the auth service (`auth.loams.dev`) that Ostrium runs. Customers' data planes are theirs (BYOC, D1378).
- It **checks the draft's technical claims against primary sources** (§19). Where they could not be checked, it says so.

**Nothing in this document is built.** It is designed only. §2 says exactly what exists today.

**What it changes:**

- **Supersedes** (D1600): every decision that makes Authentik Loams' identity provider. That covers D404, D447, D448, D452 and D458 outright, and the Authentik parts of D431, D449, D450, D451, D455, D457, D459, D486, D584, D596, D-SF-4 and D1507. It also supersedes §38 §4 (Authentik), §38 §5's Authentik row, and plan MT1. §13 lists what gets removed.
- **Amends** (approved 2026-10-21):
  - D449: people's tokens are verified directly, and the exchange is kept for agents and workloads (D1606).
  - D66: one `Authorizer` trait, in its own crate (D1610).
  - D67: Lakekeeper gets its own OpenFGA store, which answers Q21 (D1604).
  - CP1 Task 3's type list in `core.fga` (D1603).
  - §19 §4 and §5: principals map onto Zitadel (D1605, D1607).
  - §41 §3–§6: Zitadel orgs instead of Authentik groups (D1612).
  - §64 §2.3 and §10.2: Zitadel instead of Authentik. This also resolves the "two `Authorizer` traits" noted there (D1610).
  - §74 D1507 and Q1158: Zitadel-issued tokens instead of Authentik-issued tokens.
- **Leaves unchanged:**
  - D1308, D1304 and D1372: workload-identity exchange, edge credentials, and Cloudflare Access on Cloudflare BYOC.
  - D188: Biscuit for sandboxes.
  - D1133: meter integrity.
  - D100: the `_audit` stream.
  - D450: the single binary keeps built-in sign-in, now named "built-in single-org mode".

**IA0 (2026-10-21)** settled the (verify) items below except Clerk and WorkOS federation, and found seven refinements, F1–F7, in [the IA0 report](../plans/ia0-spike-report.md) §6. **The owner approved all seven on 2026-10-21 ("approve all"). They are D1622–D1628**, which amend D1605, D1607, D1608, D1611, D1613 and D1617 and the sections they touch:

- F1 is D1622: `identity-sync` reads Zitadel's event list from a stored position; webhooks only wake it.
- F3 is D1623: membership comes from user grants on the Loams project, not Zitadel org members.
- F5 is D1624: a check on every request that the principal is not disabled.
- F6 is D1625: one decision cache of 5 s or less.
- F4 is D1626: OpenFGA on SQLite only for single-user installs.
- F2 is D1627: Zitadel's outbound deny list is narrowed only for the sync webhook.
- F7 is D1628: the principal kind comes from Loams' link table.

Markers, as in §64–§75:

- **(verify)**: not checked against a primary source. The plan task that depends on it checks it first.
- **(estimate)**: judged or computed, not measured.
- **(checked 2026-10-21)**: read in a primary source on that date. §19 gives the source.

**Numbering.** This document reserves **D1600–D1639** and **Q1200–Q1219**. D1600–D1628 and Q1200–Q1214 are used. D1583–D1599 are left free for work in flight. D1570–D1579 and Q1188–Q1189 are held by §75.

Plan stub: [IA1](../plans/2026-10-21-ia1-identity-authz.md) (track IA). Phase IA0 is specified task by task; IA1–IA6 are outlined.

---

## 1. Summary

**Zitadel answers "who". OpenFGA answers "what can they do". Loams' Rust code enforces it, and Loams keeps the audit.** People and machine identities live in Zitadel. Fine-grained permissions are OpenFGA relationship tuples over a fixed hierarchy: org → project → environment → resource. Application code checks only `can_*` relations, through one Rust `Authorizer` trait. A single-user local install runs neither service: the same trait is answered by a built-in single-org authorizer.

| # | Decision (short) | Status |
|---|---|---|
| D1600 | **Full cutover from Authentik to Zitadel.** There are no users, so there is no migration path, no compatibility period and no dual running. Every Authentik service, config file, Helm value, doc and code reference is removed (§13). Supersedes D404, D447, D448, D452, D458, the Authentik parts of D431, D449–D451, D455, D457, D459, D486, D584, D596, D-SF-4 and D1507, plus plan MT1 | Approved (owner, 2026-10-21) |
| D1601 | **Division of labour** (draft §1): Zitadel owns authentication and identity lifecycle (users, orgs, membership, machine users). OpenFGA owns fine-grained authorization. Loams' Rust code enforces through `Authorizer`. Loams owns audit. **No resource-level permission is modelled in Zitadel roles**; Zitadel roles stay coarse (may sign in to the console, console admin) | Approved (owner, 2026-10-21) |
| D1602 | **Licences** (§14): Zitadel is AGPL-3.0-only from v3, with `proto/` under Apache-2.0 and the login app and client packages under MIT (checked 2026-10-21). It runs **unmodified, as a separate service**, never linked, copied or vendored, and fixes go upstream only. This is the same rule as PgDog (D318) and §22's note on Zitadel. OpenFGA is Apache-2.0 (CNCF). Counsel reviews the boundary (Q1212) | Approved (owner, 2026-10-21) |
| D1603 | **One Loams OpenFGA model, extended, not duplicated.** The draft's model (§6.1) becomes `deploy/openfga/core.fga`, replacing CP1 Task 3's planned type list. It adds `service_account` and `platform` and the meter relations (§6.2). Product modules (`graph.fga` today) hang off `environment`. Code checks only `can_*` relations | Approved (owner, 2026-10-21) |
| D1604 | **Lakekeeper keeps its own OpenFGA store** on the same OpenFGA server. Its model already defines `project`, `namespace`, `user` and `role` types (checked 2026-10-21), which collide with Loams' types. Amends D67's "one shared store" and answers Q21 | Approved (owner, 2026-10-21) |
| D1605 | **Principal mapping** (§5): a human user is `user:<id>`, a machine user is `agent:<id>` or `service_account:<id>` (two types, never `user:svc-…`), a Zitadel org is `org:<id>` 1:1 with the Loams org, and a team is `team:<id>`, owned by Loams. **Never an email** as an id | Approved (owner, 2026-10-21); amended by D1628 (the principal kind comes from the link table), approved 2026-10-21 |
| D1606 | **Tokens.** Listeners verify **Zitadel JWTs** for people directly (JWKS, cached), as the engine's `oidc` mode already does for any issuer. Loams' token endpoint stays for **agents and workloads** (federation D1308, user delegation, vending, §19 §5.2) and for environment-bound tokens at the edge (D1304). Amends D449: the people's exchange is no longer required | Approved (owner, 2026-10-21) |
| D1607 | **Agents are Zitadel machine users with explicit grants** (draft §9). Each agent has an `owner` user and gets project or environment grants directly; it never inherits from its owner. The model never lets `agent` hold `operator`, `billing_admin` or `meter_admin` (D553, D1133). Session limits come from token claims plus an OpenFGA condition (Q1204). Revocation deletes the agent's tuples (fast) and disables the machine user (authoritative) | Approved (owner, 2026-10-21); amended by D1624 (a disabled agent is refused on every request), approved 2026-10-21 |
| D1608 | **`identity-sync`** (draft §7): a Rust bridge from Zitadel to OpenFGA. Actions v2 event executions are the primary feed and Zitadel's management API is used for repair. Writes are idempotent, with an outbox and retry, applied by `(aggregate, sequence)`. A nightly reconciliation emits a drift metric, and drift > 0 alerts | Approved (owner, 2026-10-21); amended by D1622 (the event list is the primary feed) and D1623 (membership from user grants), approved 2026-10-21 |
| D1609 | **Resource tuples through the transactional outbox** (D66): the resource row and the outbox row commit together. A worker writes the parent-link tuple. A resource is `pending` until its tuple is written, then `active`, so nothing is visible before it is protected | Approved (owner, 2026-10-21) |
| D1610 | **One `Authorizer` trait** in a new crate `loams-authz`: `check`, `batch_check` and `list_objects` (D66's `filter_visible`). Implementations: `OpenFgaAuthorizer`, `CachedAuthorizer`, `BuiltinAuthorizer`, `AllowAllAuthorizer` (loopback only) and `RecordingAuthorizer`. It replaces `crates/loams/src/auth/authorizer.rs`'s role-only trait and the per-product `authz.rs` checks, and resolves §64's "two `Authorizer` traits" | Approved (owner, 2026-10-21) |
| D1611 | **Enforcement** (draft §8): Axum middleware verifies the JWT and resolves the principal. Each route declares its `can_*` relation and object, and anything undeclared is **denied**. Lists use `ListObjects` when small and an org filter plus `BatchCheck` when large. A short-TTL decision cache (≤ 5 s, D66) is keyed with the model id and invalidated from `ReadChanges`. `HIGHER_CONSISTENCY` is used, and the cache bypassed, for destructive, admin and credential-issuing calls | Approved (owner, 2026-10-21); amended by D1624 (disabled-principal check) and D1625 (one cache, ≤ 5 s), approved 2026-10-21 |
| D1612 | **Tenancy in OpenFGA** (draft §6): one store per install. A multi-org control plane (§41, CP1) uses one store with `org` at the root of every object path, and an automated test proves no tuple links two orgs. A store per tenant is used only for an org that asks for hard isolation (Q1202) | Approved (owner, 2026-10-21) |
| D1613 | **The light default turns nothing on.** A single-user local install runs built-in single-org mode (the principal store, sessions, `BuiltinAuthorizer` evaluating model v1's semantics in process). Profile `identity` adds Zitadel. Profile `authz` adds OpenFGA (SQLite in compose, Postgres in Kubernetes). A differential test keeps the built-in and OpenFGA answers equal | Approved (owner, 2026-10-21); amended by D1626 (SQLite only for single-user installs), approved 2026-10-21 |
| D1614 | **BYOC on Cloudflare runs neither service in phase 0** (no Postgres there, D1371). People authenticate through Cloudflare Access with the customer's identity provider (D1372), authorization is `BuiltinAuthorizer` over Durable Object SQLite, and the data plane uses `lk_` keys. On AWS, Google Cloud and Azure, the compose profiles and Helm values apply | Approved (owner, 2026-10-21) |
| D1615 | **What Ostrium hosts:** Zitadel at `auth.loams.dev` for `console.loams.dev` and for installs that choose it as their issuer. There is no Clerk, no WorkOS and no Zitadel Cloud. Social logins are Zitadel identity providers. Ostrium runs no OpenFGA for customer data: each install authorizes its own data plane | Approved (owner, 2026-10-21) |
| D1616 | **Audit** (draft §10): Zitadel events, OpenFGA `ReadChanges` and Loams' decision log (allows and **denials**) go into the `_audit` stream (D100). Where the House or Lakekeeper runs, they also land as one Iceberg table family | Approved (owner, 2026-10-21) |
| D1617 | **Deployment** (draft §11): each service has its own Postgres database. Zitadel's admin console and management APIs are never public. Only its OIDC and login endpoints are public, on the auth hostname. OpenFGA is internal only. The model lives in Git and CI applies it. Versions are pinned, and model upgrades are tested against a snapshot of real tuples | Approved (owner, 2026-10-21); amended by D1627 (the outbound deny list), approved 2026-10-21 |
| D1618 | **Agents in sandboxes keep Loams tokens.** A sandboxed agent never holds a Zitadel key. agentd (D1453) and the supervisor vend 15-minute Loams tokens whose `sub` is the agent's Zitadel machine-user id, and Biscuit stays inside the runtime (D188) | Approved (owner, 2026-10-21) |
| D1619 | **Proto changes are additive.** `SignInKind` gains IdP-neutral values and its Authentik values are deprecated. `OidcProviderKind` gains `ZITADEL`, and its `AUTHENTIK` value stays because it names a customer's upstream IdP. No `buf breaking` exception is needed (Q1209) | Approved (owner, 2026-10-21) |
| D1620 | **Track IA**, phases IA0–IA6 (draft §13): the spike, console login, the authorization core, the sync bridge, agents, audit and hardening, and an optional Rust-native backend. The Authentik removal is IA1 Task 1, not a migration | Approved (owner, 2026-10-21) |
| D1621 | **Honest labels:** identity and authorization are "designed" until IA2 exits, "preview" after it, and never described as shipped before their gates. Docs say which authorizer an install runs | Approved (owner, 2026-10-21) |
| D1622 | **`identity-sync` reads Zitadel's event list from a stored position** (IA0 F1). Its primary feed is Zitadel's event listing API (`ListEvents`, filtered by the event types of §8.1, ascending), read from a position stored in the metastore: the last creation date read plus the `(aggregate, sequence)` pairs at that date, so a re-read dedupes. Actions v2 webhooks are **only a wake-up** that makes it read at once; it also reads every few seconds without one. The nightly reconciliation stays as the backstop. Amends D1608, whose primary feed was the webhooks: IA0 measured them at most once (a failed delivery is never retried, and the documented `Executions.MaxTtl` drops jobs after 5 minutes) and about 12 events per second in a burst | Approved (owner, 2026-10-21) |
| D1623 | **Loams membership comes from user grants on the Loams project** (IA0 F3). A Zitadel user grant (an "authorization") on the Loams project inside org O, with the coarse role keys `member`, `admin` or `owner`, maps to `org:O#member`, `#admin` or `#owner`: `user.grant.added` and `user.grant.changed` write, and `user.grant.removed` and `user.grant.cascade.removed` delete. **Zitadel org members are not mapped**: they are Zitadel's own managers (`ORG_OWNER`, `ORG_USER_MANAGER` and so on), and mapping them would make every Loams member a Zitadel administrator. Amends §8.1's table and D1608 | Approved (owner, 2026-10-21) |
| D1624 | **A disabled principal is refused on every request** (IA0 F5). A Zitadel JWT stays valid until `exp` after its user or machine user is deactivated, so the auth layer checks the principal store's status for every verified JWT and Loams token, not only at issuance. Revocation flips the status synchronously when Loams revokes, and from `identity-sync` when the change is made in Zitadel. IA0 measured about 1 s from deactivation to denial. Amends D1607 and D1611 | Approved (owner, 2026-10-21) |
| D1625 | **One decision cache, of 5 s or less** (IA0 F6). Loams' `CachedAuthorizer` (allows only, TTL ≤ 5 s) is the only cache, and OpenFGA runs with its check-query and iterator caches off; the two stacked would give up to 15 s of stale allows. An install that prefers OpenFGA's cache sets `--check-query-cache-ttl` to 5 s or less and turns `CachedAuthorizer` off, never both. `HIGHER_CONSISTENCY` calls bypass both, as D1611 says. Amends D1611 | Approved (owner, 2026-10-21) |
| D1626 | **OpenFGA on SQLite only for single-user installs** (IA0 F4). The `authz` profile uses SQLite for one user. A multi-user install, and any install that lists with `BatchCheck`, runs OpenFGA on Postgres (its own database, D1617). IA0 measured a 50-check `BatchCheck` at about 160 ms on SQLite and 9 ms on Postgres. On SQLite, lists use `ListObjects`. Amends D1613 | Approved (owner, 2026-10-21) |
| D1627 | **Zitadel's outbound deny list is narrowed only for the sync webhook** (IA0 F2). Zitadel's HTTP client refuses loopback, link-local and every private range by default, which blocks a webhook to `identity-sync` on the same network. Compose and the Helm values allow only the sync endpoint's network and keep the unspecified range `0.0.0.0/8` and the metadata address `169.254.169.254/32` blocked. Amends D1617 | Approved (owner, 2026-10-21) |
| D1628 | **The principal kind comes from Loams' link table** (IA0 F7). A Zitadel JWT does not say whether its subject is a person or a machine, so the kind (`user`, `agent`, `service_account`) comes from the principal store's link `(issuer, sub) → principal`. Loams writes the link when it creates an agent or service account, and `identity-sync` writes it on `user.machine.added`. A subject with no link is checked as `user:<sub>`, which holds none of an agent's grants, so a missing link fails closed. Amends D1605 | Approved (owner, 2026-10-21) |

## 2. Where Loams is today

| Area | Built (on `dev`, 2026-10-21) | Designed, not built |
|---|---|---|
| Authentication | `crates/loams/src/auth/`: `LOAMS_AUTH_MODE` = `apikey` · `oidc` (RS256 or EdDSA JWTs from `LOAMS_OIDC_ISSUER`, verified against that issuer's JWKS, admitted by a subject, email or group allowlist) · `tailscale` · `off` (loopback only). Console sessions (`ls_…`) and `lk_` keys from the principal store (`directory.rs`), and first-run setup | MT1's RFC 8693 exchange of IdP tokens for Loams tokens (D449). Loams' token endpoint with federation, delegation and vending (§19 §5.2, D1308 at ER2) |
| Authorization | `auth::authorizer::Authorizer::authorize(principal, need: Role)` with `OrgRoleAuthorizer`: org roles `owner` > `admin` > `member`. `crates/loams-graph/src/authz.rs` and `crates/loams-pg-control/src/authz.rs` answer their relations from the same org and project roles | D66's trait (`check`, `batch_check`, `filter_visible`, lifecycle hooks), D67's OpenFGA, CP1 Task 3's `core.fga` and `loams-tenancy`. **No OpenFGA server runs anywhere**: there is none in any compose file or chart |
| OpenFGA model | `deploy/openfga/graph.fga` only, a module naming `namespace`, `user`, `service_account` and `agent`, which it expects `core.fga` to define | `core.fga`, `postgres.fga`, `sqldb.fga`, `house.fga`, `live.fga`, `fga.mod` (CP1 Task 3) |
| IdP | Authentik 2026.8.3 at `auth.loams.dev` (`deploy/release/auth/`, `deploy/authentik/`), which the console, the site (Auth.js), Headscale and the apps sign in to | MT1 (Planned, no task started): blueprints guard, groups → team tuples |
| Console | `web/apps/console`: `tailscale` (default), `oidc` (code + PKCE at `https://auth.loams.dev`, client `loams-console`) or `apikey` | — |
| Audit | `_audit` stream (D100) for admin actions | Edge audit fields (D1318), Iceberg landing |

So the cutover touches running config and a few code paths, while most of this design replaces plans, not code.

## 3. Who does what

### 3.1 Division of labour (D1601; draft §1)

| Concern | Owner | Why |
|---|---|---|
| Authentication (who are you) | **Zitadel** | OIDC and SAML, passkeys, MFA, orgs, machine users, federation |
| Identity lifecycle (users, orgs, membership) | **Zitadel** | Source of truth for *who exists* and *which org they belong to* |
| Fine-grained authorization (can X do Y on Z) | **OpenFGA** | Zanzibar-style relationship tuples, which scale to deep hierarchies |
| Enforcement | **Loams' engine and control plane (Rust)** | Calls the authorizer through the `Authorizer` trait (D1610) |
| Audit | **Loams** (`_audit`, then Iceberg) | Merges Zitadel events, OpenFGA changes and Loams API decisions |

Rule of thumb from the draft: **Zitadel answers "who", OpenFGA answers "what can they do".** Do not model resource-level permissions in Zitadel roles.

"The gateway" in the draft means the auth layer that every Loams HTTP and gRPC surface already passes through (`crates/loams/src/auth/layer.rs`) plus the control plane's API (CP1). It is not `loams-gateway` (§57), the protocol gateway. That one routes Postgres by SNI and authenticates nothing itself (D1014); its later `http`, `llm` and `mcp` modules (D1022) use the same `Authorizer` crate (§9.4).

### 3.2 Zanzibar → OpenFGA (draft §2)

| Zanzibar concept | OpenFGA equivalent | Loams usage |
|---|---|---|
| Relation tuple `object#relation@user` | Tuple `(user, relation, object)` | `user:alice` is `admin` of `project:p1` |
| Namespace config | Authorization model (DSL, `type` blocks) | Versioned model, in Git, applied by CI (D1617) |
| Userset rewrite (`computed_userset`, `tuple_to_userset`) | `or`, `and`, `but not`, `X from Y` | Org admin implies project admin |
| Check | `Check`, `BatchCheck` | Every API request |
| Expand | `Expand` | Debugging, "why does X have access" |
| Read / Write | `Read`, `Write` | Sync bridge, resource lifecycle |
| Watch | `ReadChanges` | Cache invalidation, audit feed |
| Zookies (the new-enemy problem) | The `consistency` preference: `MINIMIZE_LATENCY` (the default) or `HIGHER_CONSISTENCY` on Check, Expand, ListObjects, ListUsers and Read (checked 2026-10-21) | `HIGHER_CONSISTENCY` for destructive and admin calls and credential issuance (D1611) |
| Leopard index (nested groups) | OpenFGA's own optimisations | Keep team nesting shallow anyway |

OpenFGA additions beyond the paper that Loams can use:

- **Conditions** are written in Google CEL in a schema 1.1 model. Their limits (checked 2026-10-21):
  - context stored with a tuple is limited to 32 KB;
  - a request is limited to 512 KB;
  - an expression's evaluation cost is capped at 100 by default.
- **Contextual tuples** (for example, an IP range, a time window or an agent's session scope).

## 4. Target architecture (draft §3)

```
 Browser / CLI / desktop / phone            Agent (outside a sandbox)        Agent in a sandbox
        │ OIDC code + PKCE (device code for CLI)  │ private-key JWT / client credentials   │ 15-min Loams token (D1618)
        ▼                                         ▼                                        │
   ┌───────────┐  JWT (sub, org id, coarse roles)                                          │
   │  Zitadel  │─────────────────────────────┐                                             │
   └─────┬─────┘                             ▼                                             ▼
         │ Actions v2 event        ┌──────────────────────────────┐   Check / BatchCheck / ListObjects
         │ executions (signed)     │ Loams auth layer (Axum)      │────────────────────────────┐
         ▼                         │ + Loams token endpoint       │                            ▼
  ┌───────────────┐                │   (federation, delegation,   │                    ┌──────────────┐
  │ identity-sync │                │    vending; D1308, §19 §5.2) │                    │   OpenFGA    │
  │    (Rust)     │                └──────────────┬───────────────┘                    │ (own store,  │
  └───────┬───────┘                               ▼                                    │  own DB)     │
          │ write/delete tuples      engine · control plane · gateway modules          └──────┬───────┘
          └────────────────────────► (resource create → outbox ── worker ── write tuple) ─────┘
```

Key properties, from the draft, with Loams' additions:

- The JWT carries identity and org context only. It is **not** the permission source for resources.
- Two writers feed OpenFGA. `identity-sync` writes people, agents and org membership. The control plane's outbox worker writes the resource hierarchy.
- Zitadel and OpenFGA use **separate Postgres databases**, so they have separate failure and backup domains. In compose, OpenFGA may use SQLite instead (D1613).
- A single-user install has neither box. The auth layer resolves sessions and keys from the principal store, and `BuiltinAuthorizer` answers the same `can_*` checks (§12.1).

## 5. Identity mapping (D1605; draft §4)

| Zitadel | OpenFGA object | Loams principal (§19 §5) | Notes |
|---|---|---|---|
| Human user `sub` | `user:<zitadel_user_id>` | `user` | Never the email as the id (Q1206 asks about the id space) |
| Machine user for an AI agent | `agent:<zitadel_user_id>` | `agent` | Its own type, so the model can limit agents (§10) |
| Machine user for a non-agent workload (CI job, batch, an app's backend) | `service_account:<zitadel_user_id>` | `service_account` | §19's third kind. The draft said "pick one convention"; Loams keeps two types because the meter (D1133) and the control plane (D553) treat agents differently from other machines |
| Organization | `org:<zitadel_org_id>` | the org | 1:1 with the Loams org. A single-org install still has one |
| Org member | `org:<id>#member@user:<id>` | membership | Written by `identity-sync` |
| Zitadel project roles | coarse only | — | For example, "may sign in to the console" or "console admin". Fine-grained roles go to OpenFGA |
| — | `team:<id>` | team | **Owned by Loams**, not Zitadel, which keeps it portable (§19 P4). Q1203 asks about SCIM |
| `lk_` API key (D65, §19 §5.5) | the key's `service_account` | `service_account` | Keys stay for SDKs and apps that cannot federate, and are never issued to agents (D295) |

**Reconciling with §19.** §19's three principal kinds (`user`, `service_account`, `agent`) and its rules carry over unchanged:

- agents hold no long-lived secrets;
- agent tokens are capped at 15 minutes by default and 1 hour at most;
- suspension revokes everything.

What changes is who keeps the identity record. Zitadel becomes the source of truth for *who exists*, and Loams' principal store keeps a link `(issuer, sub) → principal` and the Loams-only fields (agent policy, TTL cap, trust policies). **The link is the source of the principal's kind** (D1628, approved 2026-10-21): a JWT does not say whether its subject is a person or a machine, and a subject with no link is checked as `user:<sub>`, which fails closed.

**Built-in single-org mode.** Without Zitadel, the principal store issues ids in the same opaque format and the org is created at first-run setup (§19 §6). Turning on the `identity` profile later re-links people by verified email once, at first sign-in, and from then on by `sub`. With no users today, this matters only for future installs.

**The engine's `oidc` mode today matches members by email** (`directory.rs`, `member_by_email`). IA2 changes the match to `(iss, sub)` (D1605).

**Federation (draft §4).** The draft kept Clerk and WorkOS as upstream IdPs federated into Zitadel for the cloud tier. Federation is possible on paper (checked 2026-10-21):

- Zitadel accepts any OpenID Connect-compliant IdP through its generic OIDC template, and SAML and LDAP as well.
- Clerk acts as an OAuth 2.0 and OpenID Connect server for registered OAuth applications (the `openid` scope, ID tokens).
- WorkOS Connect publishes `/.well-known/openid-configuration`.

Neither combination has been tested **(verify)**. Loams does not use Clerk (retired, Q436) or WorkOS (D486). So D1615 recommends neither for `auth.loams.dev`, and Q1201 records the choice. A self-hoster's own IdP (Okta, Entra ID, Google Workspace, Keycloak, Authentik, GitHub) federates into their Zitadel the same way.

## 6. The authorization model (D1603; draft §5)

### 6.1 The draft's starting point, verbatim

```
model
  schema 1.1

type user

type agent
  relations
    define owner: [user]

type team
  relations
    define member: [user, team#member]

type org
  relations
    define owner: [user]
    define admin: [user, team#member] or owner
    define member: [user, team#member] or admin
    define billing: [user] or owner

type project
  relations
    define org: [org]
    define admin: [user, team#member] or admin from org
    define editor: [user, team#member, agent] or admin
    define viewer: [user, team#member, agent] or editor

type environment
  relations
    define project: [project]
    define deployer: [user, team#member, agent] or admin from project
    define editor: [user, team#member, agent] or deployer or editor from project
    define viewer: [user, team#member, agent] or editor or viewer from project

type deployment
  relations
    define environment: [environment]
    define can_view: viewer from environment
    define can_operate: editor from environment
    define can_destroy: deployer from environment

type database
  relations
    define environment: [environment]
    define can_read: viewer from environment
    define can_write: editor from environment
    define can_admin: deployer from environment
```

The draft's design notes stand:

- The hierarchy is `org → project → environment → resource`, and permissions flow down through `X from Y`.
- Depth is fixed and small (four levels), with no arbitrarily nested folders.
- **`can_*` relations are the only thing application code checks.** Roles (`admin`, `editor`, `viewer`, `deployer`) are what gets assigned, so role semantics change without touching code.
- **Conditions** come later, for agent sessions.

### 6.2 What Loams changes in it

| Change | Why |
|---|---|
| **`environment` is a Loams namespace.** The object id is the namespace id: `environment:<namespace_id>` | §19 P2: an environment is exactly one namespace; CP-R7 keys every product record by `NamespaceId`. Q1207 asks about the type name |
| **`service_account`** is assignable wherever `agent` is (`editor`, `viewer`, `deployer`), and gets `define project: [project]` | §19's third principal; `lk_` keys resolve to it (§5) |
| **`agent`** gets `define project: [project]` beside `owner` | §19 P5: an agent is registered in a project |
| **`team`** gets `define org: [org]` | The cross-org test (D1612) needs every team rooted in an org |
| **`platform`**, a new root type (one object per install): `operator`, `limits_writer`, `meter_admin`, each `[user]` only (`limits_writer` also takes `service_account`) | D66 maps Lakekeeper's `server` to `platform`; CP1 Task 3 planned these relations on a type it called `deployment`. That name is taken here by the draft's application deployments, so the root is `platform`. `org` gets `define platform: [platform]` |
| **`org.billing`** becomes **`billing_admin`** (`[user] or owner`), and `billing_viewer` (`[user, team#member] or billing_admin`) is added | D1133 already names these relations for Loams Meter. Neither admits `agent` or `service_account`, so "agents cannot hold `billing_admin` or `meter_admin`" is a property of the model's types, tested by `agent_cannot_write_grants` |
| **`database`** gains `define can_connect: viewer from environment` | D711: Postgres sign-in checks `connect`. A database's owner-only DDL stays `can_admin` |
| **Product modules hang off `environment`.** `graph.fga`'s `define namespace: [namespace]` becomes `define environment: [environment]`, and its `reader`, `writer` and `admin` take `viewer`, `editor` and `deployer from environment`. Postgres, SQL, House and Live modules follow CP1 Task 3's file list | Extend the model, don't duplicate it. The modules join through `fga.mod` (schema 1.2 modular models) (checked in IA0, 2026-10-21: the CLI v0.8.1 validates, tests and writes `fga.mod` to OpenFGA v1.22.0) |
| **An agent-session condition** (IA4), below | Q1204: a grant made for one agent session expires with it |

The condition, in OpenFGA's DSL with a CEL expression:

```
condition agent_session(session_id: string, allowed_session: string, now: timestamp, expires: timestamp) {
  session_id == allowed_session && now < expires
}
```

A grant written to an agent for one session uses the type restriction `[agent with agent_session]`, and standing grants stay unconditional.

The model files live in `deploy/openfga/` and are applied by CI (`fga model write` with the OpenFGA CLI, Apache-2.0). Every change goes through a PR with model tests (`fga model test`). Code never names a role, only `can_*` relations (D1603).

## 7. Tenancy in OpenFGA (D1612; draft §6)

| Option | Pros | Cons | Use for |
|---|---|---|---|
| **Single store, org-rooted objects** | One model to migrate, cheap, cross-tenant operations tooling | A bug in the model or the checks affects everyone | **A multi-org control plane** (§41, CP1) |
| Store per tenant | Hard isolation, per-tenant deletion | Model migrations × N, connection and operations overhead | An org that asks for hard isolation (Q1202) |

The draft's plan stands: one store with `org` at the root of every object path. Every Check must be traceable to an org, and an automated test proves that no tuple links objects in different orgs.

**What "cloud tier" means now.** The draft put the single shared store in "the cloud tier". But Ostrium hosts no customer data plane (D1378), so there are no customer resources in a hosted store to protect. In practice:

- **Each install has its own store**, inside the customer's account, and the install's control plane writes it.
- **A multi-org control plane** (an agency's or holding company's own CP1 hub, managing many orgs on its clusters) uses the single org-rooted store.
- **An air-gapped customer** runs its own install, and so its own store.

**OSS tier:** one org, the same model, and the same code path (the org always exists). In built-in mode, the same semantics are evaluated in process (§12.1).

## 8. The sync bridge and resource tuples (D1608, D1609; draft §7)

### 8.1 `identity-sync`

**Purpose:** keep OpenFGA's tuples consistent with Zitadel's identity state.

**Inputs** (amended by D1622, approved 2026-10-21). The draft said to pick one primary and keep the other for repair. IA0 showed that the webhooks cannot be the primary, so:

1. **Zitadel's event list** (primary). `identity-sync` reads `ListEvents` for §8.1's event types in ascending order from a position stored in the metastore (the last creation date read, plus the `(aggregate, sequence)` pairs at that date so a re-read dedupes). It reads every few seconds, and at once when a webhook wakes it. IA0 read every event in order in 32 ms, including the three the webhooks lost.
2. **Zitadel Actions v2 event executions** (a wake-up only). An execution's condition can be one event, a group of events or all events. Its target is a `Webhook`, a `Call` or an `Async` endpoint. Each call carries a `ZITADEL-Signature` header, an HMAC over the body and a timestamp with a per-target signing key (checked 2026-10-21). The docs do not promise ordering or redelivery. **IA0 measured it (2026-10-21): delivery is in order with the default single worker, but at most once: a failed call is never retried, the documented `Executions.MaxTtl` drops jobs after 5 minutes, and a burst drains at about 12 events per second.** So a webhook only tells `identity-sync` to read now; its body is not applied. The endpoint still verifies `ZITADEL-Signature`. Zitadel's outbound deny list allows only that endpoint's network (D1627).
3. **The management API listing** (repair and reconciliation).

**Event → tuple mapping** (amended by D1623, approved 2026-10-21: Zitadel's org members are its own managers, so Loams membership comes from user grants on the Loams project):

| Zitadel event | Tuple action |
|---|---|
| User grant added or changed (`user.grant.added`, `user.grant.changed`) on the Loams project in org O, role keys `member`, `admin` or `owner` | Write `org:O#member@user:U`, `#admin` or `#owner` to match the role keys, and delete the roles the grant no longer has |
| User grant removed (`user.grant.removed`, `user.grant.cascade.removed`) | Delete the same tuples |
| Org member added or removed (`org.member.*`) | **Nothing.** These are Zitadel's managers; their rights stay in Zitadel |
| Machine user added (`user.machine.added`) | Write the principal link with kind `agent` or `service_account` (D1628) |
| User deactivated or removed | Mark the principal disabled (D1624), then delete all tuples for `user:U` (read by user, delete in batches of at most 100, OpenFGA's default `max-tuples-per-write`, checked 2026-10-21) |
| Machine user deactivated (an agent) | Mark the principal disabled, so every request with its token is refused (D1624), then delete all tuples for `agent:A`; Loams' token endpoint refuses new tokens for it at once (§10) |
| Org removed | Delete the `org:O` subtree **through Loams' control plane** (which knows the resources), not blindly |

**Rules (draft):**

- Writes are **idempotent**: a duplicate write and a missing delete both count as success.
- The sync side has an **outbox and retry**. Events are applied by `(aggregate, sequence)` (sequences are per aggregate: the user grant, the user), and an event older than the last one applied for the same key is skipped.
- A **nightly reconciliation** lists the user grants on the Loams project in each org, `Read`s the tuples from OpenFGA, diffs them, repairs the drift and emits a drift metric. Drift > 0 is an alert.
- **Deprovisioning is the dangerous direction.** The target is that a removal shows in OpenFGA within seconds, with reconciliation as the backstop.

**Where it runs** (Q1213):

- A single install runs it as a task of the `loams` binary's control-plane role (the draft's "sidecar"), with its state in the metastore.
- In Kubernetes it is a separate Deployment.
- It verifies `ZITADEL-Signature` on a wake-up call before acting on it.

### 8.2 Resource tuples (control plane)

- Creating a project, environment, deployment or database writes its parent-link tuple (`environment:E#project@project:P`).
- A **transactional outbox** in the Loams metastore puts the resource row and the outbox row in one transaction (D66, CP1 Task 4's `OutboxRow`). A worker writes the tuple to OpenFGA.
- A resource starts `pending` and flips to `active` after its tuple write succeeds, so nothing is visible before it is protected.
- In built-in mode there is no tuple to write. The resource row is the relationship, and `pending` flips to `active` in the same transaction.

## 9. Enforcement in Rust (D1610, D1611; draft §8)

### 9.1 The trait

The draft's trait:

```rust
#[async_trait]
pub trait Authorizer: Send + Sync {
    async fn check(&self, req: CheckRequest) -> Result<Decision>;
    async fn batch_check(&self, reqs: Vec<CheckRequest>) -> Result<Vec<Decision>>;
    async fn list_objects(&self, user: &Subject, relation: &str, ty: &str) -> Result<Vec<ObjectId>>;
}
```

Loams adopts it, in a new crate `crates/loams-authz`, as **the** authorizer. Its value types carry what §19 and §69 already need:

- **`Subject`** is `{ kind: User | ServiceAccount | Agent, id, org, act: Vec<Subject>, session: Option<SessionId> }`. `act` is the RFC 8693 actor chain, and every decision record carries it.
- **`CheckRequest`** is `{ subject, relation, object, context, contextual_tuples, consistency }`, where `consistency` is `MinimizeLatency` or `HigherConsistency`.
- **`Decision`** is `{ allowed, model_id, cached }`, which is enough for the decision log (§11).

| Implementation | Use |
|---|---|
| `OpenFgaAuthorizer` | `openfga-client` 0.6.1 (Apache-2.0, gRPC; the client D66 chose, checked on crates.io 2026-10-21) |
| `CachedAuthorizer` | Wraps any authorizer: a foyer cache through `loams-cache`, allow decisions only, TTL ≤ 5 s (D66), keyed by `(model_id, subject, relation, object)`, invalidated from `ReadChanges` |
| `BuiltinAuthorizer` | Single-org mode: evaluates model v1's semantics from the principal store in process (replaces `OrgRoleAuthorizer`) |
| `AllowAllAuthorizer` | Developer mode only, refused unless every listener is on loopback (as `LOAMS_AUTH_MODE=off` is today) |
| `RecordingAuthorizer` | Tests and audit: records every request and decision |

This mirrors the metastore-trait pattern, so the backend can be swapped later, including for a Rust-native engine (IA6). It replaces three things:

- `crates/loams/src/auth/authorizer.rs`'s `Authorizer::authorize(principal, need: Role)`;
- the relation checks in `crates/loams-graph/src/authz.rs` and `crates/loams-pg-control/src/authz.rs`;
- CP1 Task 3's planned `check(&Principal, relation, &ObjectRef)`.

So there is one trait, and §64's "two `Authorizer` traits, two principal vocabularies" is resolved. D66's lifecycle hooks (`on_created`, `on_deleted`) become outbox rows (§8.2) rather than trait methods.

### 9.2 Middleware

The auth layer (`crates/loams/src/auth/layer.rs`, extended) runs four steps:

1. It **verifies the JWT** against the issuer's JWKS, cached. Zitadel's ID tokens, and its access tokens when the application is set to JWT access tokens, carry the client and project ids in `aud`, and the org in `urn:zitadel:iam:user:resourceowner:id` (checked 2026-10-21). Sessions, `lk_` keys and Loams tokens resolve as they do today.
2. It **extracts `sub` and the org** into a `Subject`, takes the kind from the principal link (D1628), and **refuses a disabled principal** even when its token's signature is valid (D1624).
3. It finds **the route's declared `can_*` relation and object**, either from a per-RPC table generated from proto options or from an annotation on REST routes.
4. It calls **`Check`**. **Deny by default:** a route with no declaration answers `PERMISSION_DENIED`, and a CI test fails if any served RPC lacks one.

### 9.3 Lists, caching and consistency

**List endpoints:**

- Use `ListObjects` where result sets are small, and on OpenFGA's SQLite datastore (D1626). OpenFGA's defaults stop it at 1000 results or 3 s (`listObjects-max-results`, `listObjects-deadline`, checked 2026-10-21 at v1.22.0).
- For large listings, filter in the database by org and then `BatchCheck`. A `BatchCheck` takes up to 50 checks per request by default (`max-checks-per-batch-check`), each with a `correlation_id` of up to 36 alphanumeric or hyphen characters. OpenFGA advises parallel `Check` calls below about 10.
- Alternatively, keep a materialised access index.

**Caching and consistency:**

- Allow decisions get a TTL of 5 s or less, in **one** cache (D1625, approved 2026-10-21): Loams' `CachedAuthorizer`, with OpenFGA's check-query and iterator caches off. An install may use OpenFGA's cache instead (`--check-query-cache-ttl` ≤ 5 s, `CachedAuthorizer` off), never both.
- Destructive and admin calls and credential issuance bypass the cache and ask with `HIGHER_CONSISTENCY`. So does the principal's first read after its own write (D66).

### 9.4 Which surfaces enforce

| Surface | Credential in | Check |
|---|---|---|
| Native, Connect and REST API, console API (main port) | Zitadel JWT, session, `lk_`, Loams token | Per-RPC `can_*` |
| Control plane (CP1 hub) | Zitadel JWT, Loams token, mTLS plus Biscuit for services (Q542) | Per-RPC, on `platform`, `org`, `cluster`, `rollout` |
| Postgres sign-in (D711) | SCRAM, or a Loams token by credential exchange | `can_connect` on `database` |
| Edge listener (§69) | Environment-bound credentials only (D1304), unchanged | Upstream, in the engine, as above |
| `loams-gateway` `http`, `llm`, `mcp` modules (D1022) | Loams keys or tokens | Through `loams-authz` when those modules are built |
| Graph, Live, House, Git, Jobs | As the main port | Their modules' `can_*` (D750, D772) |

## 10. Agents and machine identities (D1607, D1618; draft §9)

- **Each agent is a Zitadel machine user**, mapped to `agent:<id>`. Outside a sandbox it authenticates with **private-key JWT** (recommended by Zitadel) or **client credentials**. Personal access tokens exist for machine users but are not used (checked 2026-10-21).
- **An agent has an `owner` (a user).** Access is **granted explicitly** on projects or environments (`editor`, `deployer`) and is **never inherited from its owner** by default. In the model, `owner` confers nothing on the agent.
- **Short-lived tokens, narrow scopes.** A per-session limit is a contextual tuple or the `agent_session` condition (session id, expiry, allowed environment).
- **Agents in sandboxes never hold a Zitadel key** (D1618). agentd's rotating token file (D1453, `LOAMS_TOKEN_FILE`, 15 minutes) and the supervisor's vending (§19 §5.2 flow 3) issue **Loams tokens** whose `sub` is the agent's machine-user id. Biscuit stays inside the runtime (D188). The Jailer PDP (§61, D1163) still decides what a sandboxed command may do. OpenFGA decides what the agent may reach.
- **MCP clients acting for a user** (Claude Code, Codex) use §19 §5.2 flow 2, user delegation with `act`. The token's subject is the user, its actor is the agent, and it never exceeds either one's rights.
- **Every agent action carries `(agent_id, owner_id, session_id)`** in the audit record.
- **Revocation** deletes the agent's tuples in OpenFGA (fast) and disables the machine user in Zitadel (authoritative). Loams' token endpoint stops vending for a disabled agent at once, through the principal store's change feed (§19 §5.4). **The auth layer also refuses the disabled agent's tokens that are already issued** (D1624, approved 2026-10-21): Zitadel's JWTs stay valid until `exp`, so the principal store's status is checked on every request. IA0 measured about 1 s from deactivation in Zitadel to denial.
- **Marketplace agents** (§73) are listings. When installed, each install's agent becomes one machine user in the installing org, with no grants until someone grants them.

## 11. Audit (D1616; draft §10)

Three feeds go into one record shape in the `_audit` stream (D100) and, where the House or Lakekeeper runs, land as one **Iceberg table family**:

1. **The Zitadel event stream:** logins, MFA and membership changes, from the same Actions v2 executions as §8.1 and from Zitadel's event listing for backfill.
2. **OpenFGA `ReadChanges`:** tuple writes and deletes, meaning who got access and when.
3. **Loams' API decision log:** `(timestamp, subject, act, relation, object, decision, model_id, request_id)`, plus `(agent_id, owner_id, session_id)` for agents and `via=edge` fields from D1318.

**Denials are logged as well as allows.** Denials are rate-limited per subject, as failed authentications already are (D100). The decision log also feeds the planned metrics-based optimiser. A light install keeps only the `_audit` stream.

## 12. Deployment (D1613–D1617; draft §11)

### 12.1 The light default: built-in single-org mode

A single-user local install starts with `docker compose up -d loams tailscale` on the light stack (0.1.1, PR #973). **It turns on neither Zitadel nor OpenFGA.**

- **Authentication** is what exists today:
  - `tailscale` mode by default;
  - built-in sessions with first-run setup, password and TOTP (§19 P7, D450);
  - `lk_` keys;
  - or `oidc` against any issuer, including `auth.loams.dev`.
- **Authorization** is `BuiltinAuthorizer`. It holds the same org → project → environment roles in the principal store and answers the same `can_*` relations in process.
- **A differential test** (`builtin_matches_openfga`) runs one check matrix against `BuiltinAuthorizer` and against `OpenFgaAuthorizer` loaded with model v1, and must agree on every row. This keeps "same model, same code path" true.

Two compose profiles turn the services on:

| Profile | Adds | Storage |
|---|---|---|
| `identity` | Zitadel (API) and Zitadel Login, the Next.js login app shipped as its own component (checked 2026-10-21), plus `identity-sync` in the `loams` process | `zitadel-db`, its own Postgres 14–18 container (Zitadel supports PostgreSQL 14 to 18, checked 2026-10-21) |
| `authz` | OpenFGA | SQLite in a volume, **for a single-user install only** (D1626, approved 2026-10-21); a multi-user install, or one that lists with `BatchCheck`, uses Postgres. OpenFGA v1.22 supports `memory`, `postgres`, `mysql` and `sqlite` datastores (checked 2026-10-21) |

**Why not OpenFGA on by default with SQLite?** It is one more container and one more hop on every request for a single user who owns everything. Q1208 records this.

### 12.2 Kubernetes (operator, Helm)

| Component | Kubernetes |
|---|---|
| Zitadel | The upstream chart `zitadel/zitadel-charts` (Apache-2.0, `zitadel-10.4.0` stable on 2026-10-09; an 11.0 line is in beta), referenced by an Argo CD `Application`, with Postgres on CloudNativePG (D230). Two or more replicas for HA |
| OpenFGA | The upstream chart `openfga/helm-charts` (Apache-2.0, `openfga-0.3.16`), two or more replicas behind an internal Service, Postgres on CloudNativePG, with read replicas when measured to need them |
| `identity-sync` | A Deployment of the `loams` image in the sync role |

These replace MT3's Authentik waves: Zitadel's database in wave 1, then Zitadel and OpenFGA in wave 2, then the model-apply Job before `Loams` in wave 4.

### 12.3 What Ostrium hosts (D1615)

Ostrium runs `loams.dev`, `console.loams.dev` and `auth.loams.dev` and nothing else (D1317, D1378).

- `auth.loams.dev` becomes Zitadel, on the same kind of host Authentik used: compose, its own Postgres, a TLS front (Caddy, as `deploy/release/auth/` has today, or Cloudflare Tunnel), and nightly backups.
- **Public:** the OIDC endpoints and the login UI.
- **Never public:** Zitadel's admin console and management APIs, which are reachable on the tailnet only, carrying D596's rule over to Zitadel.
- Social logins (GitHub, Google) are Zitadel identity providers.
- There is no Clerk, no WorkOS and no Zitadel Cloud (Q1201).
- The site (`ostrium-labs/loams-site`, Auth.js) changes its provider to Zitadel in that repository.

An install that trusts `auth.loams.dev` sets `LOAMS_OIDC_ISSUER=https://auth.loams.dev` and the Zitadel project id as audience (checked in IA0, 2026-10-21: people's access tokens carry the client and project ids, an agent's carries the project id when it asks for scope `urn:zitadel:iam:org:project:id:<project id>:aud`). It still decides membership and authorization itself, so Ostrium never sees the install's tuples.

### 12.4 BYOC (D1614)

| Target | Identity | Authorization |
|---|---|---|
| VM tier (AWS, Google Cloud, Azure; §70) | The compose profiles, or `auth.loams.dev`, or the customer's IdP as `LOAMS_OIDC_ISSUER` | `BuiltinAuthorizer`, or OpenFGA with `authz` |
| Container services (Fargate, Cloud Run, Container Apps) | As the VM tier; Zitadel only where the customer's managed Postgres is available | As the VM tier |
| Kubernetes tier | §12.2 | §12.2 |
| **Cloudflare phase 0** | **No Zitadel.** It needs a long-running server and Postgres, and Loams Postgres stays out of Cloudflare BYOC (D1371). The console and admin routes sit behind Cloudflare Access with the customer's IdP, and the Worker checks `Cf-Access-Jwt-Assertion` (D1372, D1306). That IdP may itself be a Zitadel, theirs or `auth.loams.dev` | **No OpenFGA.** `BuiltinAuthorizer` over the Durable Object's SQLite (D1368); data-plane calls carry environment-bound `lk_` keys |
| Vercel | Hosts no data plane (D1373); apps use Vercel OIDC exchange (D1308) | — |

### 12.5 Operating rules (D1617)

- Both services are **internal only**, except Zitadel's OIDC and login endpoints on the auth hostname. The console and the Loams listeners are the only other public entry points.
- Backups and point-in-time recovery cover both Postgres databases. The OpenFGA model is also stored in Git and applied by CI.
- **Pin versions.** Images are pinned by digest at a release at least 14 days old. On 2026-10-21 that means Zitadel v4.19.4 (2026-10-01), not v4.19.5 (2026-10-15), and OpenFGA v1.22.0 (2026-10-06).
- Test OpenFGA model upgrades against a snapshot of real tuples.
- **Zitadel's outbound deny list** (`HTTPClient.DenyList`) is narrowed only enough for the `identity-sync` webhook's network; `0.0.0.0/8` and `169.254.169.254/32` stay blocked (D1627, approved 2026-10-21).
- OpenFGA runs with its check caches off when `CachedAuthorizer` is on (D1625).

## 13. The Authentik cutover (D1600, approved)

**The ruling:** "full cutover from Authentik to Zitadel. There are no users, so there's no migration path, no compatibility period and no dual running." So:

- **No migration.** No user, group, session or blueprint is exported from Authentik or imported into Zitadel. Zitadel starts empty and is configured from Git.
- **No compatibility period.** No release ships both IdPs or lets an install choose Authentik.
- **No dual running.** `auth.loams.dev` switches from Authentik to Zitadel in one step, and Authentik's VM services, Postgres and backups are deleted after the switch. A short sign-in outage at the switch is accepted.
- **The removal is a task, not a migration.** It is IA1 Task 1 in the plan, with a gate: afterwards, `git grep -il authentik` matches only the historical records listed at the end of this section.

**What gets removed:**

| Area | Paths |
|---|---|
| Hosted auth service | `deploy/release/auth/` (Authentik compose, Caddyfile, blueprints `loams-console.yaml` and `loams-confidential-client.yaml`, `setup.sh`, README, `.env.example`), replaced by a Zitadel bundle at the same path |
| Authentik deploy kit | `deploy/authentik/` entirely: README, `blueprints/loams-01…05`, `branding/`, `compose/` (with backup scripts and the CLI tunnel), `helm/` (values, CNPG cluster, cloudflared, backup CronJob, namespace, secrets example) and `kustomization.yaml` |
| Tailnet | `deploy/tailnet/authentik-tailnet/`; `deploy/tailnet/tailscale/k8s/tailscale-authentik.yaml` and its `kustomization.yaml` entry; Headscale's OIDC block (`deploy/tailnet/headscale/config/config.yaml`, compose and k8s secrets examples) pointed at Zitadel; `tag:authentik` in `deploy/tailnet/policy/policy.hujson` becomes `tag:zitadel` |
| Release config | `deploy/release/.env.example`'s OIDC comments ("Authentik puts the client id in `aud`") |
| Docs | `docs/platform/deploy/authentik.md` (replaced by `zitadel.md`), `headscale.md`, `tailnet.md`, `docs/platform/README.md`, `docs/platform/design/04-…` and `05-…`, `docs/guides/studio/remote-agentd.md`, `docs/monorepo.md`, `plugins/docs/auth-setup.md`, `plugins/README.md`, `plugins/SECURITY.md`, `platform/README.md`, `apps/mobile/README.md` and `apps/mobile/docs/*`, `crates/loams-sidebar-browser/README.md` |
| Code | `plugins/core/host/src/auth/*` (`AuthentikOidcClient` becomes an IdP-neutral client) and its tests and fake IdP; `crates/loams-apps-mock` (`oauth.rs`, `seed.rs`, `store.rs`, services, tests); `crates/loams-agentd-link/src/auth/oidc.rs` and `auth/mod.rs`; `crates/loams-agentd-connect/src/oidc.rs`; `crates/loams-agentd-proto/src/brand.rs`; `crates/loams-sidebar-browser/src/*`; `crates/loams/src/auth/oidc.rs` (comments); `crates/loams-live/tests/identity.rs`; Android `AuthentikSignIn.kt` and its callers, iOS `WebAuth.swift`, `AppModel.swift` and `WelcomeView.swift`; the Go mobile mock; `web/plugins/org/src/settings.tsx`; SDK token-source docs (Go, Rust, Python, Swift, TypeScript) |
| Protos and generated code | `proto/loams/instance/v1/instance.proto` (`SIGN_IN_KIND_AUTHENTIK*` deprecated, IdP-neutral values added, comments), `proto/loams/auth/v1/auth.proto` (`OIDC_PROVIDER_KIND_ZITADEL` added), `api/console/openapi.json`; regenerated SDK code, the mobile and agentd copies of `instance.proto`, and `sdks/fixtures/recorded/` (D1619) |
| Plans | MT1 marked superseded. MT3's Authentik waves and `Application`, NET1's Authentik roster sync, CP1's and MT4's Authentik mentions, and the plan rows that name Authentik are pointed at IA1 |
| Notices | `NOTICE`'s Authentik entry (the recovery-flow blueprint attribution) is removed with the blueprint |
| Other repositories | `ostrium-labs/loams-site`: Auth.js's provider becomes Zitadel (a PR there). The running `auth.loams.dev` VM is decommissioned by the owner after the switch |

**What stays as a historical record:** the decision log rows (marked superseded), §38 (with its amendment note), `CHANGELOG.md`'s past entries, and design §22's licence comparison. `OIDC_PROVIDER_KIND_AUTHENTIK` also stays, because it names a customer's own upstream IdP, which can still federate into Zitadel or be used directly as `LOAMS_OIDC_ISSUER`.

## 14. Licences (D1602; draft §12, not legal advice)

| Component | Licence (checked 2026-10-21) | How Loams uses it |
|---|---|---|
| Zitadel server (`zitadel/zitadel`) | **AGPL-3.0-only** since v3.0.0 (released 2025-05-02; Zitadel's v3 announcement covered the licence change and dropping CockroachDB). `LICENSING.md` gives `proto/` and `apps/docs/` Apache-2.0, and `apps/login/`, `packages/zitadel-client/` and `packages/zitadel-proto/` MIT. A commercial licence is offered. Latest v4.19.5 (2026-10-15) | **Unmodified official images, as a separate service**, reached only over OIDC and its APIs. No Zitadel code is copied, linked or vendored, and fixes go upstream (Zitadel accepts contributions under Apache-2.0). The same rule as PgDog (D318). §22 already noted that Zitadel is "allowed as a separate, unmodified service" |
| Zitadel SDKs | `zitadel-go` and `zitadel-node` are Apache-2.0. The community Rust crate `zitadel` (`smartive/zitadel-rust`) is `MIT OR Apache-2.0` (checked in IA0: its `Cargo.toml` and two licence files; GitHub shows NOASSERTION because there are two), but it is not needed | Loams uses `openidconnect` 4 and `jsonwebtoken` 11 (already in the tree) and Zitadel's Apache-2.0 protos if it needs the management API |
| Zitadel Helm chart | Apache-2.0 | Referenced, not vendored |
| OpenFGA server, CLI, chart, API protos | Apache-2.0 (CNCF). Server v1.22.0 (2026-10-06) | Unmodified service |
| `openfga-client` (Rust) | Apache-2.0, 0.6.1 | Linked (D66) |

**The draft's reasoning stands.** Running unmodified Zitadel as a separate service and talking to it over OIDC and APIs keeps Loams' Apache-2.0 code separate. If Loams modified Zitadel and offered it as a service, it would have to publish those changes. Loams does not modify it.

**What changed since the draft:** there are no closed layers to protect, so the draft's "before the closed-source `loam-cloud`/`loam-platform` layers ship" no longer applies. The lawyer check stays, re-aimed: **counsel reviews the AGPL boundary before Zitadel ships in a Loams release or runs at `auth.loams.dev`** (Q1212). The commercial licence stays as the fallback.

## 15. Phases (D1620; draft §13)

| Phase | Deliverable | Exit criteria |
|---|---|---|
| **IA0. Spike (3–5 days)** | Zitadel and OpenFGA in compose, one org, JWT → `Check` from a tiny Axum service; the facts marked (verify) checked | End-to-end allow and deny work; a spike report |
| **IA1. Console login and the cutover** | Authentik removed (§13); Zitadel at `auth.loams.dev` and in the `identity` profile; org and user model; console, CLI (device code), desktop and phone sign-in; social IdPs in Zitadel | Login, MFA, passkey and org switch work; `git grep -il authentik` is clean |
| **IA2. Authorization core** | Model v1 in Git; `loams-authz` with the trait and its implementations; the middleware; deny by default; `BuiltinAuthorizer` and the differential test; the `authz` profile | Every control-plane and engine route is behind a `can_*` check |
| **IA3. Sync bridge** | `identity-sync`; the outbox for resource tuples; the nightly reconciler | A member removal shows within seconds; the drift metric is 0 |
| **IA4. Agents** | Machine users, the `agent` type, session-scoped conditions, Loams-token vending with machine-user `sub` | An agent touches only granted environments; revocation is tested |
| **IA5. Audit and hardening** | Unified audit tables; the cross-tenant tuple test; a load test; cache and consistency tuning | Check p99 target met (Q1214); no cross-org tuples |
| **IA6. Optional** | A Rust-native `Authorizer` backend and/or a Rust IdP behind the same traits | Only if OpenFGA or Zitadel becomes a bottleneck or a licence issue |

## 16. Risks (draft §14, with additions)

| Risk | Mitigation |
|---|---|
| Tuple drift between Zitadel and OpenFGA | Idempotent sync, outbox, nightly reconciliation, drift alert |
| Revocation lag from caches | Short TTL, `ReadChanges` invalidation, `HIGHER_CONSISTENCY` for admin calls |
| Model sprawl | Code checks only `can_*`; model changes go through a PR with `fga model test` |
| Cross-tenant leak | Org-rooted objects, an automated cross-org tuple scan, a per-route org assertion |
| AGPL exposure | Zitadel unmodified and separate; counsel review (Q1212); commercial licence as the fallback |
| Latency from the extra hop | OpenFGA co-located, `BatchCheck`, the decision cache; built-in mode has no hop |
| *Added:* the built-in and OpenFGA authorizers disagree | The differential test (§12.1) runs on every PR that touches either authorizer or the model |
| *Added:* Actions v2 delivers out of order or drops events | Sequence-aware apply, the outbox, the nightly reconciliation; IA0 checks Zitadel's delivery guarantees |
| *Added:* Zitadel's footprint (API, login, Postgres) on small installs | Off by default (D1613); IA0 measured the `identity` profile idle at about 243 MB of memory and 0.7 GB of images, and `authz` at 30 MB |
| *Added:* the cutover leaves a dangling reference | IA1 Task 1's `git grep` gate; the cross-repo site PR |

## 17. Contradictions with earlier decisions, and how they are resolved

| # | Earlier | This document | Resolution |
|---|---|---|---|
| 1 | D404, D447, D448, D452, D458: Authentik is the IdP, configured by blueprints, guarded against Enterprise | Zitadel | **Superseded by D1600** |
| 2 | D449: people's IdP tokens are exchanged for Loams tokens, and listeners verify only Loams tokens | People's Zitadel JWTs are verified directly; the exchange remains for agents, workloads and the edge | Amended, approved 2026-10-21 (D1606, Q1205). The engine's `oidc` mode already verifies IdP JWTs directly |
| 3 | D450: the single binary keeps built-in sign-in | Kept, named built-in single-org mode | Unchanged (D1613) |
| 4 | D451: MT1 is the identity half of D111 | IA1–IA3 are | Authentik part superseded (D1600); the rest moves to IA |
| 5 | D66: trait with `filter_visible` and lifecycle hooks, in `loams-common` | `check`, `batch_check`, `list_objects` in `loams-authz`; hooks become outbox rows | Amended, approved 2026-10-21 (D1610) |
| 6 | D67: Loams and Lakekeeper share one store | Separate stores | Amended, approved 2026-10-21 (D1604); answers Q21 |
| 7 | CP1 Task 3: `core.fga` types `deployment` (root), `org`, `namespace`, `cluster`, `rollout` | `platform` (root), `org`, `team`, `project`, `environment` (= namespace), the draft's `deployment` (an application deployment), `database`; `cluster` and `rollout` stay | Amended, approved 2026-10-21 (D1603) |
| 8 | §41 §2.2, §5: Loams orgs are mapped from Authentik groups | Zitadel orgs 1:1 | Amended, approved 2026-10-21 (D1605, D1612) |
| 9 | §64 §2.3: "Loams keeps Authentik and adds no IdP"; §10.2 two traits | Zitadel replaces Authentik (still one IdP); one trait | Authentik part superseded (D1600); trait by D1610 |
| 10 | D1507 (§74): Temporal SDKs use Authentik-issued tokens; Q1158 Airflow FAB OAuth against Authentik | Zitadel-issued tokens; FAB OAuth against Zitadel | Superseded in its Authentik part (D1600) |
| 11 | D431, D486 (§37): desktop and phone sign in at Authentik | The same OIDC flows at Zitadel | Authentik part superseded (D1600) |
| 12 | D584, D596 (§43): Headscale OIDC at Authentik; Authentik admin tailnet-only | Headscale OIDC at Zitadel; Zitadel admin tailnet-only | Authentik part superseded (D1600) |
| 13 | D-SF-4 (§39): SSO through Authentik, proxy outposts for apps without OIDC | Zitadel for apps with OIDC; apps without OIDC lose the outpost and keep their own sign-in | Authentik part superseded (D1600); §39's agent half is already superseded (D1420) |
| 14 | Q442: hosted identity "Clerk or Authentik", decided in `loams-platform` | Zitadel at `auth.loams.dev`, decided here | Superseded (D1600, D1615) |
| 15 | Draft §6: a single shared store "for the cloud tier" | One store per install; the shared org-rooted store for multi-org control planes | Reconciled with D1378 (D1612) |
| 16 | Draft §11: both services internal only | Zitadel's OIDC and login endpoints are public on the auth hostname | Browsers must reach them (D1617) |

## 18. Open questions (answered 2026-10-21)

The owner answered every question below with its default on 2026-10-21.

| # | Question | Default | Owner | Needed by |
|---|---|---|---|---|
| Q1200 | ~~Does the OSS tier need SAML or SCIM in v1, or are OIDC and passkeys enough? (draft Q1)~~ | ~~OIDC and passkeys. SAML IdPs can federate into Zitadel at no cost to Loams, so a company with SAML signs in through Zitadel. SCIM provisioning is revisited after IA3~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA1 |
| Q1201 | ~~Cloud tier: federate Clerk or WorkOS into Zitadel, or run Zitadel Cloud? (draft Q2)~~ | ~~Neither. Self-hosted Zitadel at `auth.loams.dev`, with GitHub and Google as Zitadel IdPs. Clerk is retired (Q436) and WorkOS unused (D486); federation is possible on paper (§5)~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA1 |
| Q1202 | ~~A single OpenFGA store or a store per tenant for enterprise and air-gapped customers? (draft Q3)~~ | ~~One store per install. In a multi-org control plane, one org-rooted store, plus a per-tenant store for an org that asks. Air-gapped customers run their own install anyway~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA2 |
| Q1203 | ~~Should teams live in Loams only, or also sync to Zitadel for SCIM provisioning? (draft Q4)~~ | ~~Loams only; revisit with Q1200's SCIM decision~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA3 |
| Q1204 | ~~Where do agent session limits live: OpenFGA conditions, token claims, or both? (draft Q5)~~ | ~~Both. Token claims (`exp`, a session id, one environment) bound every token, and the `agent_session` condition binds session-scoped grants, so a leaked token and a stale grant both expire~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA4 |
| Q1205 | ~~Do listeners accept people's Zitadel JWTs directly, or only Loams tokens after an RFC 8693 exchange (D449)?~~ | ~~Directly (D1606), as the `oidc` mode does today. The edge still takes only environment-bound credentials (D1304)~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA2 |
| Q1206 | ~~Tuple subject ids: Zitadel's ids (draft), or Loams principal ids linked to `(iss, sub)`?~~ | ~~Zitadel's ids, as drafted, since Zitadel is the source of truth for who exists. Built-in mode issues ids in the same format, and a link table maps other issuers~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA2 |
| Q1207 | ~~The OpenFGA type for an environment: `environment` (draft) or `namespace` (`graph.fga`)?~~ | ~~`environment`, with the namespace id as its object id; `graph.fga` follows in IA2~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA2 |
| Q1208 | ~~In the light default, the built-in authorizer, or OpenFGA with SQLite on by default?~~ | ~~Built-in (D1613), with OpenFGA behind the `authz` profile~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA2 |
| Q1209 | ~~Protos: rename `SIGN_IN_KIND_AUTHENTIK*`, or add neutral values and deprecate?~~ | ~~Add `SIGN_IN_KIND_OIDC` and `SIGN_IN_KIND_OIDC_DEVICE_CODE` and deprecate the Authentik values, which `buf breaking` (FILE rules) accepts; add `OIDC_PROVIDER_KIND_ZITADEL`; keep `OIDC_PROVIDER_KIND_AUTHENTIK`~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA1 Task 1 |
| Q1210 | ~~Zitadel's Postgres: its own container or CNPG cluster, or Loams Postgres?~~ | ~~Its own now (compose container, CNPG in Kubernetes); Loams Postgres later, as Q447 ruled for Authentik~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA1 |
| Q1211 | ~~Login UI: Zitadel Login unmodified with branding, or a Loams-built login on Zitadel's session APIs?~~ | ~~Zitadel Login unmodified, branded through settings; a Loams login only if branding falls short (`apps/login` is MIT, so either is allowed)~~ **Answered 2026-10-21 by the owner: the default.** | Owner | IA1 |
| Q1212 | ~~Counsel reviews the AGPL boundary of running Zitadel unmodified, as the draft asked. When?~~ | ~~Before IA1 switches `auth.loams.dev` and before a release ships the `identity` profile~~ **Answered 2026-10-21 by the owner: the default.** | Owner (action) | IA1 |
| Q1213 | ~~Where does `identity-sync` run?~~ | ~~A task in the `loams` binary's control-plane role on single installs; its own Deployment in Kubernetes~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA3 |
| Q1214 | ~~The Check p99 target for IA5's exit~~ | ~~p99 ≤ 10 ms for a cached or co-located `Check` and ≤ 25 ms uncached under the IA5 load profile **(estimate)**; measured in IA0 and revised there~~ **Answered 2026-10-21 by the owner: the default.** | Eng | IA5 |

## 19. Verified facts and sources

Checked on 2026-10-21 unless a date is given.

- **Zitadel licence and versions:**
  - `github.com/zitadel/zitadel`: `LICENSE` is AGPL-3.0, and `LICENSING.md` gives the Apache-2.0 and MIT exceptions quoted in §14 and says contributions are accepted under Apache-2.0.
  - Releases: v3.0.0 (2025-05-02), v4.0.0 (2025-07-31), v4.19.4 (2026-10-01), v4.19.5 (2026-10-15).
  - Zitadel blog, "Zitadel v3: AGPL License, Streamlined Releases, and Platform Updates" (zitadel.com/blog/zitadel-v3-announcement) and "Moving to AGPL 3.0" (zitadel.com/blog/apache-to-agpl); licence FAQ (zitadel.com/license-faq).
  - `zitadel/zitadel-go` and `zitadel/zitadel-node` are Apache-2.0. `zitadel/zitadel-charts` is Apache-2.0, `zitadel-10.4.0` (2026-10-09). `smartive/zitadel-rust` shows "NOASSERTION" in GitHub's licence API.
- **Zitadel features:**
  - Actions v2 (zitadel.com/docs/concepts/features/actions_v2 and /docs/guides/integrate/actions/usage): targets and executions; `Webhook`, `Call` and `Async` targets; request, response, function and event conditions (one event, a group, or all); the `ZITADEL-Signature` HMAC header. Ordering and redelivery are not documented.
  - Database (/docs/self-hosting/manage/database): PostgreSQL 14 to 18.
  - Compose (/docs/self-hosting/deploy/compose): Traefik, the Zitadel API (Go), Zitadel Login (Next.js), PostgreSQL.
  - Token exchange (/docs/guides/integrate/token-exchange): RFC 8693 with impersonation and an `act` claim. Third-party JWTs are not accepted as subject tokens, which is why workload federation stays on Loams' token endpoint.
  - Service users (/docs/guides/integrate/serviceusers): private-key JWT (recommended), client credentials, and personal access tokens for machine users.
  - Claims (/docs/apis/openidoauth/claims): `aud` holds client and project ids, present in access tokens only when they are JWTs; `urn:zitadel:iam:user:resourceowner:id` and `urn:zitadel:iam:org:project:roles`.
  - Identity providers (/docs/guides/integrate/identity-providers/introduction): generic OIDC, SAML, LDAP, and named templates.
- **OpenFGA:**
  - `github.com/openfga/openfga` v1.22.0 (2026-10-06), Apache-2.0.
  - `pkg/server/config/config.go` at v1.22.0: `DefaultListObjectsMaxResults = 1000`, `DefaultListObjectsDeadline = 3s`, `DefaultMaxChecksPerBatchCheck = 50`, `DefaultMaxTuplesPerWrite = 100`, `DefaultWriteContextByteLimit = 32KB`.
  - `cmd/run/flags.go`: `listObjects-max-results`, `listObjects-deadline`, `max-checks-per-batch-check`. `cmd/run/run.go`: datastore engines `memory`, `mysql`, `postgres`, `sqlite`.
  - `openfga/api` `openfga/v1/openfga_service_consistency.proto`: `ConsistencyPreference { UNSPECIFIED; MINIMIZE_LATENCY = 100; HIGHER_CONSISTENCY = 200 }`. `openfga_service.proto`: BatchCheck's `correlation_id` (alphanumeric or hyphens, at most 36 characters).
  - Docs: openfga.dev/docs/interacting/consistency, /docs/interacting/relationship-queries, /docs/modeling/conditions (CEL, 32 KB tuple context, 512 KB request, cost limit 100).
  - `openfga/cli` v0.8.2 and `openfga/helm-charts` `openfga-0.3.16`, Apache-2.0. crates.io `openfga-client` 0.6.1, Apache-2.0.
- **Lakekeeper:** `lakekeeper/lakekeeper` v0.14.1 (2026-10-16); `authz/openfga/v4.13/components/` holds `project.fga`, `namespace.fga`, `user.fga`, `role.fga`, `server.fga`, `warehouse.fga` and `lakekeeper_view.fga`.
- **Federation:**
  - Clerk: clerk.com/docs/guides/configure/auth-strategies/oauth/how-clerk-implements-oauth (OAuth applications, the `openid` scope, ID tokens, `/.well-known/oauth-authorization-server`).
  - WorkOS: workos.com/docs/reference/workos-connect/metadata/openid-configuration.
- **This repository:**
  - §18 (D64–D67), §19, §22 §4 (the Zitadel row), §37, §38, §39, §41, §43, §57, §61, §63 §19 (D1133), §64 §2.3 and §10.2, §69 (D1304, D1306, D1308, D1318), §70 (D1367, D1368, D1371, D1372, D1378), §73 (D1453), §74 (D1507, Q1158);
  - D100, D295, D318, D553, D711, D750, D1119, D1323;
  - `crates/loams/src/auth/` (`mod.rs`, `authorizer.rs`, `oidc.rs`, `directory.rs`), `deploy/openfga/graph.fga`, `deploy/release/compose.yaml`, `deploy/release/auth/`, `deploy/authentik/`, `docs/platform/deploy/authentik.md`, `web/apps/console/src/hosted/auth.ts`, `proto/loams/instance/v1/instance.proto`, `proto/loams/auth/v1/auth.proto`, `buf.yaml` (`breaking: FILE`);
  - plans MT1, MT3, NET1, CP1 (Task 3, Task 4).
