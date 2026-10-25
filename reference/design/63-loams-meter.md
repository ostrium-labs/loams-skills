# 63 — Loams Meter: Open Usage Metering and Enforcement

> **Amendment (approved 2026-10-23) by [§78](78-loams-cloud-on-tidb-x.md)** (Loams Cloud on TiDB X; D1700–D1735). The Meter enforces Loams Cloud's free try-out plan as code with a hard stop; counters come from the control plane and TiDB, and devices' own reports are used only for warnings (D1721).

> **Amendment (approved 2026-10-21) by [§76](76-identity-and-authorization.md)** (identity and authorization: Zitadel and OpenFGA; owner, all defaults). This document's §19 relations (D1133) map onto the core model: `billing_admin` and `billing_viewer` on `org` (replacing the owner draft's `billing`) and `meter_admin` on the new `platform` root, none assignable to `agent` or `service_account` (D1603). The site signs in through Zitadel, not Authentik (D1600).

Status: **Approved** · 2026-10-19 (owner approved the design, all decisions and every default, 2026-10-19). Source: the owner's draft notes of 2026-10-19 (below, §2), adapted to the Loams architecture, and the 2026-10-19 open-source decision (D1119, owner: "Remove the open core. Everything is in OSS."), which moves metering, entitlements, billing exporters and the control-plane pieces into this repository under this design. Decisions **D1120–D1138** and questions **Q960–Q969** are in the [decision log](13-decision-log.md), all approved by the owner on 2026-10-19. Plan stub: [ME1](../plans/2026-10-19-me1-loams-meter.md). **This document writes no code.**

It touches these earlier documents (§20 lists every amendment):
- [§02](02-stream-engine.md) §7.4 (D270, the CloudEvents dedup ledger) and [§34](34-protocol-gateway-and-standards.md) §2–§3 (D364, the Loams CloudEvents profile; D365, the high-rate path);
- [§24](24-cpu-time-runtime.md) and [§27](27-usage-hooks.md) (usage hooks, D200–D202, D548, D549);
- [§41](41-multitenant-byoc-control-plane.md) §9–§12 (D541, D546–D552, D555) and plan [CP1](../plans/2026-10-10-cp1-control-plane-byoc.md) (the limits record, CPQ4, D850);
- [§49](49-loams-house-production.md) §6, §9 and §15 (lake tables, pipes, quotas and usage events; HS1 Task 22, `fabric/crates/loams-house/src/{quota,usage}.rs`);
- [§57](57-loams-gateway.md) §11–§12 (the gateway's observer and its API, LLM and MCP modules);
- [§59](59-console-foundation.md) D1080 (Settings → Billing).

Markers, as in §51 and §62:
- **(verify)** means the claim has not been checked against a primary source or running code. The task that depends on it checks it first.
- **(estimate)** means the number was judged, not measured.
- **(target)** is a number this document sets as a gate.

---

## 1. Summary

**Loams Meter** is one small Rust system, in this repository and Apache-2.0, that does two different jobs and keeps them apart:

1. **Metering** records what was used. Usage events are CloudEvents, compatible with OpenMeter's event format, appended to a Loams stream, deduplicated by `source` + `id`, and aggregated in the House into per-subject, per-meter windows (minute, hour, day). The House is the source of truth for what a tenant is billed. It is never on the request path.
2. **Enforcement** decides whether the next unit of work may run. Products check an in-process gate that reads only local counters, fed by quota **leases** that a counter owner hands out in chunks (about 1% of the limit). Counters are reconciled against the House every five minutes. A tenant with no payment method is hard-stopped at its limit; overage is opt-in, needs a card, and has a spend cap.

Entitlements follow OpenMeter's model: **meter → feature → entitlement → grant** (amount, priority, expiry, rollover), with the modes `hard_limit`, `soft_limit` and `overage_allowed`. OpenMeter, Stripe and Lago are **optional exporters**, never dependencies.

**What exists and is reused.** The House already counts quotas and emits `house.query`, `house.insert`, `house.storage` and `house.compaction` to an open `UsageObserver` (HS1 Task 22). The gateway has a `ConnectionObserver` and pg-control a `ComputeLifecycleObserver`. Streams accept CloudEvents with a `source` + `id` ledger (D270), and the high-rate path for usage events is already decided (D365). The House can pipe a stream into an Iceberg lake table exactly once with offsets in the snapshot (D769) and supports `ReplacingMergeTree` with `FINAL` on lake tables (D765). Loams Meter is mostly the glue between these, plus the entitlement engine, the lease protocol and the exporters.

| # | Decision | Status |
|---|---|---|
| D1120 | **Loams Meter is the one metering and enforcement system, open, in Rust, in this repository** (§5): crates `loams-meter-core` (types and a sans-I/O balance engine), `loams-meter-gate` (the in-process gate and emitter products link) and `loams-meter` (the `meter` role of `loams`, behind feature `meter`), plus exporters behind features. No runtime dependency on OpenMeter, Lago, Kafka or a ClickHouse server. The imported node agent becomes `loams-meter-agent` (§21). It replaces the private `loams-platform` metering (D548's record, its docs 01 and 06) | Approved (owner, 2026-10-19) |
| D1121 | **The usage event** (§6) is a CloudEvent in the Loams profile (D364) that OpenMeter's ingest accepts unchanged: `subject` required (the billing subject), `data` a JSON object (`application/json`, **amends D364's protobuf default** for `io.loams.dev.usage.*` types), `type` `io.loams.dev.usage.<product>.<name>.v<major>`, `id` derived from the work so a re-emission is a duplicate, `tenantid` set from the credential. One new extension, `loamslease` (the lease that admitted the work) | Approved (owner, 2026-10-19) |
| D1122 | **Storage and dedupe** (§7): every event lands in the system stream `_system/_meter` (32 partitions, keyed by `subject`). Loams' own producers use D365's batched plain produce; external producers use D270's ledger. Duplicates are removed three times: at the emitter (stable ids), in the counter owner (a recent-keys set) and in the House, where `_system/_meter` is a `ReplacingMergeTree` lake table keyed `(subject, type, source, id)` read with `FINAL` | Approved (owner, 2026-10-19) |
| D1123 | **Aggregation in the House** (§8): a close job writes closed minute, hour and day windows into `_meter.windows` (`ReplacingMergeTree(close_run)`), from `FINAL` reads of raw events; late events up to 24 h re-close their windows; billing periods close two hours after their end. **Only the House's windows are billed**; counters never are | Approved (owner, 2026-10-19) |
| D1124 | **The entitlement model** (§9) is OpenMeter's: customers own subjects; a meter feeds a feature; an entitlement (metered, boolean or static) gives a subject a feature for a usage period; grants carry amount, priority, effective time, expiry and rollover; burn-down by priority, then nearest expiry, then oldest. Modes `hard_limit`, `soft_limit` and `overage_allowed`. Money is integer micros plus an ISO 4217 code (D374) | Approved (owner, 2026-10-19) |
| D1125 | **The enforcement path** (§10): products call `Gate::check` or `Gate::reserve`/`settle` at their admission points; a hit reads only local, sharded counters (≤ 5 µs p99, target); a miss fetches a lease synchronously within a 50 ms deadline, after which the policy table decides. Hook points: House admission, the gateway's API, LLM and MCP modules, Postgres connect and compute wake, jobs enqueue, function invoke, Git and stream ingest | Approved (owner, 2026-10-19) |
| D1126 | **Multi-node leasing** (§11): the counter owner of a subject is the holder of its `_system/_meter` partition lease (fenced, §02 §7). It grants chunks of `clamp(1% of the allowance, min_chunk, remaining / (2 × holders))` with a 30 s TTL and a 10 s refresh; below a floor it switches to exact mode. After a failover the new owner runs Doorman's **learning mode** for one TTL. Overshoot is accepted and bounded (§11.4) | Approved (owner, 2026-10-19) |
| D1127 | **Reconciliation** (§12): every 5 minutes and at period close, each owner rebases its counters on the House's totals at the pipe's committed offsets and replays only the stream tail after them, so no event is counted twice or missed. Drift above 1% of an allowance raises an alert | Approved (owner, 2026-10-19) |
| D1128 | **Policy** (§13): no payment method → hard stop; overage only with a card, opt-in and under a spend cap; prepaid credits stop at zero; counters unavailable → free tenants fail closed, paid tenants fail open within their spend-cap headroom; thresholds at 80% and 100% | Approved (owner, 2026-10-19) |
| D1129 | **Threshold and lifecycle events go into the same stream** (§15): `io.loams.dev.meter.threshold.v1`, `…entitlement.reset.v1`, `…access.changed.v1`, with ids that make re-emission after a failover a duplicate; a notifier fans them out to webhooks, email and the console | Approved (owner, 2026-10-19) |
| D1130 | **Exporters are optional and one-way** (§16): Stripe (meter events of closed windows, `identifier` for idempotency), OpenMeter (raw events, unchanged), Lago (events by `transaction_id`, over HTTP only, AGPL never linked), a CloudEvents webhook, and the Iceberg tables themselves. Loams owns balances and access; a provider owns invoices and payment | Approved (owner, 2026-10-19) |
| D1131 | **APIs** (§17): Connect `loams.meter.v1` (`MeterService`, `IngestService`, `CustomerService`, `EntitlementService`, `PlanService`, internal `LeaseService`) with REST through the API conventions (AIP-158 pages, `idempotency_key`, `ErrorInfo.reason`), and an **OpenMeter-compatible REST subset** under `/api/v1/` so OpenMeter SDKs work. Hard stops answer `entitlement_exhausted` (HTTP 402) and keep 429 for rate limits | Approved (owner, 2026-10-19) |
| D1132 | **Console** (§18): Settings → Billing (D1080) gets Usage, Entitlements, Spend controls, Payment and Alerts pages, plus an 80%/100% banner; the operator view gets Meters, Plans, Customers and Meter health. **Answers Q546**: money may be shown | Approved (owner, 2026-10-19) |
| D1133 | **Integrity comes from authentication and from where usage is measured, not from secrecy** (§19): reserved `io.loams.dev.*` types only from service identities; `tenantid` and `subject` from the credential; CPU and bytes measured outside tenant code; billing only from the House; grants, plans and opt-ins written only by humans or the billing service, never by agent principals; every change audited. **Restates D541** for an open codebase | Approved (owner, 2026-10-19) |
| D1134 | **Entitlements are a second gate beside the limits record** (§20): D546's protective limits (rates, concurrency, daily volume) stay, with their counters approximate and engine-local; entitlements add billing-period allowances. CPQ4/D850's private House observer is superseded: `MeterSink` implements the open observers. The no-metering naming rules (JB1, CP1 Task 27, §52) are retired | Approved (owner, 2026-10-19) |
| D1135 | **Migration from `loams-platform`** (§21), surveyed from its code: its one crate, the node metering agent (cgroup reader, delta tracker, resolver, spool, stream sink), is kept and renamed `loams-meter-agent` in the root workspace, with its events moved to §6's types and its sink to D270's record layout and the subject partitioner; its designed but unbuilt aggregator is superseded by the House and the counter owners; the host-report design (doc 06) is kept for the T1 source; `platform/` is removed once empty | Approved (owner, 2026-10-19) |
| D1136 | **Plan ME1** (§23) in six milestones: ME1a events and emitters, ME1b House storage and windows, ME1c entitlements, counters and reconciliation, ME1d enforcement and policy, ME1e exporters, console and migration, ME1f user meters and provisioning | Approved (owner, 2026-10-19) |
| D1137 | **Meters, features and plans as code** (§22): YAML in the GitOps repository (D544) applied through `PlanService`; an OpenTofu provider later. Terraform is not a target, since its licence is BUSL 1.1; neither tool meters | Approved (owner, 2026-10-19) |
| D1138 | **User meters** (§9.6): the same engine, scoped to a user's namespace, lets Loams users meter and limit their own customers (OpenMeter's use case). After Loams' own billing works (ME1f) | Approved (owner, 2026-10-19) |

---

## 2. Why

**The owner's draft (2026-10-19, intent kept verbatim in substance).** A CI provider's free tier did not stop at its limit and invoiced the overage in arrears, with no card on file. The lesson: **metering and enforcement are different things, and enforcement is first-class.** A system that counts accurately and bills later has done the easy half. The hard half is refusing the next unit of work at the moment the allowance is gone, without a network call on every request and without stopping a paying customer because a counter store hiccuped.

The industry record agrees:
- Vercel: "Setting a spend amount does not stop usage on its own"; pausing is opt-in and checks run "every few minutes" ([Vercel spend management](https://vercel.com/docs/spend-management)). It later made pausing production deployments the default ([changelog, 2024-06-27](https://vercel.com/changelog/spend-management-now-pauses-production-deployments-by-default)).
- Firebase and Google Cloud: "Budget alerts do not pause services"; spend caps exist only for some services ([Firebase: avoid surprise bills](https://firebase.google.com/docs/projects/billing/avoid-surprise-bills)).
- Lago documents alerts and progressive billing that **notify or invoice** but do not block; halting usage is left to the integrator ([Lago alerts](https://getlago.com/docs/guide/alerts), [progressive billing](https://getlago.com/docs/guide/plans/progressive-billing)). Stripe's credits are applied at invoice finalisation, not in real time ([billing credits](https://docs.stripe.com/billing/subscriptions/usage-based/billing-credits)), and its usage alerts notify only ([alerts](https://docs.stripe.com/billing/subscriptions/usage-based/alerts)).

**Why build, not adopt.** The draft's reasoning holds after checking:
- **OpenMeter** (Apache-2.0, acquired by Kong on 2025-09-03 and kept open source) is the right *model* and *format*: CloudEvents ingest, meters with SUM/COUNT/UNIQUE_COUNT/AVG/MIN/MAX/LATEST, entitlements with grants and rollover ([repository](https://github.com/openmeterio/openmeter), [Kong announcement](https://konghq.com/blog/news/kong-acquires-openmeter)). Its runtime is Go services plus Kafka, ClickHouse and PostgreSQL, with Redis for dedupe ([README](https://github.com/openmeterio/openmeter)). Self-hosters of Loams would run a second data platform to count the first one. Loams already has a log (the WAL), an analytical store (the House on Iceberg) and a metastore.
- **Lago** is AGPL-3.0 ([repository](https://github.com/getlago/lago)), which D11 excludes as a dependency, and it does not enforce.
- **Stripe** meters are a billing rail with a 1,000 calls/s v1 limit and asynchronous processing ([recording usage](https://docs.stripe.com/billing/subscriptions/usage-based/recording-usage-api)); not an enforcement point, and not available to self-hosters who bill by contract.

So Loams keeps OpenMeter's event format and entitlement semantics, so that OpenMeter can be an exporter (or a replacement, for someone who prefers it), and implements the core on its own primitives.

**OpenTofu, not Terraform, for provisioning.** HashiCorp moved Terraform from MPL 2.0 to BUSL 1.1 on 2023-08-10 ([HashiCorp](https://www.hashicorp.com/en/blog/hashicorp-adopts-business-source-license), [LICENSE](https://raw.githubusercontent.com/hashicorp/terraform/main/LICENSE)); OpenTofu is the MPL-2.0 fork under the Linux Foundation ([announcement](https://www.linuxfoundation.org/press/announcing-opentofu), [LICENSE](https://raw.githubusercontent.com/opentofu/opentofu/main/LICENSE)). Neither meters anything. §22 covers meters and plans as code.

---

## 3. Goals and non-goals

### 3.1 Goals

1. **Enforcement is first-class.** Every billable resource has an admission point that can refuse work when an entitlement is exhausted, with a measured latency budget (§10) and a written failure policy (§13, §14).
2. **Billing-grade metering is exact and auditable.** Each event counts once; any invoice line can be traced to the windows and raw events that produced it.
3. **One event format** compatible with CloudEvents 1.0 and OpenMeter's ingest, so exporting is a pass-through.
4. **Built on Loams' primitives**: the stream WAL, the metastore's leases, the House on Iceberg, the gateway, the observers. No new stateful service for self-hosters.
5. **The hot path never waits on the House**, and only waits on the network on a cold miss.
6. **Self-hosters get the whole thing**: a single organisation can meter and limit its teams; an operator can run a multi-tenant Loams and bill tenants through Stripe, OpenMeter, Lago or a contract.
7. **Deterministic core.** The balance engine and the lease allocator are sans-I/O state machines that `loams-detsim` can simulate (§55's direction).

### 3.2 Non-goals

- **Invoicing, tax, dunning and payment collection.** A billing provider does these. Loams computes usage, balances and access, and exports.
- **A pricing engine with tiers, packages and proration.** Plans name allowances and a per-unit overage price (§9.5); richer price models are the provider's. Revisit after ME1e.
- **Rate limiting.** Requests per second and concurrency stay D546's protective limits (token buckets and semaphores in each product). Entitlements are about billing-period allowances.
- **Real-time exact global counters.** Enforcement accepts a small, bounded overshoot (§11.4) in exchange for no network call per request.
- **Replacing the Event Fabric.** External high-rate telemetry still goes to Iggy and Fluss (D331). Usage events are a Loams-internal, moderate-rate stream (§7.4).
- **Metering tenant code from inside the tenant.** No producer of billing-grade numbers runs in tenant code (§19).

---

## 4. Where Loams stands today (checked 2026-10-19 on `dev`)

| Piece | State | Path or decision |
|---|---|---|
| CloudEvents codec, `source` + `id` key (SHA-256(`source` ‖ 0x00 ‖ `id`)) | Built | `crates/loams-cloudevents`, D270 |
| Ledger dedup on `/events` and `ProduceCloudEvents` (claim, append, complete; 1 h window, at most 24 h) | Built | §02 §7.4; `MetaStore::claim_idempotency_keys` and friends in `crates/loams-common/src/meta/store.rs` |
| High-rate events bypass the ledger (batched plain produce, 50 ms or 1 MiB, bounded queue drops oldest) | Decided, not built | D365, §34 §3 |
| Loams CloudEvents profile (`type` prefix `io.loams.dev`, `tenantid` from the credential, `dataschema`) | Decided | D364, §34 §2 |
| House quotas per namespace (concurrency, per-minute, scan bytes/day, CPU s/day, storage, insert bytes/day, pipes) | Built | `fabric/crates/loams-house/src/quota.rs`, HS1 Task 22, D776 |
| House usage events (`house.query`, `.insert`, `.storage`, `.compaction`) to `UsageObserver`; CPU read from outside the worker | Built | `fabric/crates/loams-house/src/usage.rs`, R22.5, R22.6 |
| Stream → lake-table pipes, exactly once, offsets in the Iceberg snapshot summary | Designed in HS1c | §49 §9, D769 |
| `ReplacingMergeTree` on lake tables, `FINAL` merges at read time | Designed in HS1b | §49 §6.2, D765 |
| Gateway `ConnectionObserver` (open, close, bytes, duration) | Built | `crates/loams-gateway/src/observe.rs`, D1021 |
| pg-control `ComputeLifecycleObserver` (`Started`, `Resized`, `Suspended`, `Failed`) | Built | `crates/loams-pg-control/src/observer.rs`, D703 |
| `InvocationObserver` for runners | Designed, not built | §27 §3.7, D549 |
| The limits record, limits API, directory and `EnforcementSample` | Planned | §41 §9, CP1 Tasks 4–7 |
| The node metering agent (cgroup v2 reader, delta tracker, static resolver, spool, `_system/_meter` sink) | Built in the former private `loams-platform`, being imported under `platform/` (branch `import-platform`) | Platform doc 01 MB0, §21 |
| The usage aggregator, the per-invocation host report, the Stripe export | Designed in `loams-platform` (docs 01, 06), never built | §21 |

What D1119 changes: the integrity argument of D541 no longer needs the code to be private, so this design restates it (D1133), and the "counters are never usage" rule of D546 becomes "**counters are never billed**" (D1134).

---

## 5. Architecture

### 5.1 The picture

```
 producers (in-process)                    external producers
 ┌──────────────────────────────┐          (user meters, partner systems)
 │ House front   UsageObserver  │                 │ POST /api/v1/events
 │ Gateway       ConnectionObs. │                 │ (CloudEvents, D270 ledger)
 │ pg-control    ComputeLifecyc.│                 ▼
 │ Runners       InvocationObs. │          ┌──────────────┐
 │ Node agent    cgroups, spool │          │              │
 │ Jobs, Git, Live, streams     │          │ meter: ingest│
 │   └─ MeterSink ─┐            │          └──────┬───────┘
 │   Gate::check ◄─┼── leases ──┼──────┐          │
 └─────────────────┼────────────┘      │          │
                   │ D365 batched produce          │
                   ▼                    │          ▼
        ┌───────────────────────────────────────────────────┐
        │ stream _system/_meter (32 partitions,              │
        │ key = subject, WAL class express where available)  │
        └───────┬───────────────────────────────┬────────────┘
                │ named consumer                │ House pipe (LoamsStream,
                │ meter-counters                │ CloudEvents), exactly once
                ▼ (partition leases)            ▼
        ┌──────────────────────┐        ┌──────────────────────────────┐
        │ meter role:          │ rebase │ House (system namespace)     │
        │ counter owner per    │◄───────│ _meter.events  (Replacing)   │
        │ partition; lease     │ every  │ _meter.windows (Replacing)   │
        │ server; entitlement  │ 5 min  │ close job: minute/hour/day   │
        │ engine; thresholds   │        └──────────────┬───────────────┘
        └──────────┬───────────┘                       │ closed windows
                   │ threshold events → same stream    ▼
                   ▼                            ┌──────────────┐
            notifier: webhooks, email,          │ exporters:   │
            console                             │ Stripe, Open-│
                                                │ Meter, Lago, │
                                                │ webhook      │
                                                └──────────────┘
```

### 5.2 Components

| Component | Where | Does |
|---|---|---|
| `loams-meter-core` | `crates/loams-meter-core` | The types of §9; the balance engine (grants, burn-down, rollover, resets) and the lease allocator as sans-I/O machines; meter evaluation (JSONPath value and group-by extraction). No async, no I/O, no Arrow. Usable from the `fabric/` workspace by path |
| `loams-meter-agent` | `crates/loams-meter-agent` (moved from `platform/crates/loams-meter`, §21) | The node agent: a DaemonSet (or a service on a single host) that reads every sandbox's and runtime's cgroup from outside it and writes spooled usage events (§7.2) |
| `loams-meter-gate` | `crates/loams-meter-gate` | What products link: `Gate` (local counters, lease cache, `check`/`reserve`/`settle`), `MeterSink` (the D365 emitter) and adapters that implement `UsageObserver`, `ConnectionObserver`, `ComputeLifecycleObserver` and `InvocationObserver`. A `NoGate` default that allows everything and emits nothing, so a product without the meter behaves as today |
| `meter` role | `crates/loams-meter`, a role of `loams` behind feature `meter` | Ingest API, counter owners (named consumer `meter-counters` with partition leases), `LeaseService`, entitlement and plan APIs, reconciler, close-job scheduler, notifier, exporters |
| House tables | The House, system namespace, database `_meter` | Raw events, windows, period totals; the source of truth (§7, §8) |
| Exporters | `crates/loams-meter/src/export/{stripe,openmeter,lago,webhook}.rs`, features `export-stripe` and so on | Push closed windows or raw events to a provider (§16) |
| Console pages | `web/plugins/billing` (new) and the operator view | §18 |

**One role, scaled out.** The `meter` role runs in `loams dev` and `standalone` as part of the single process, and as N ≥ 2 instances in a cluster. Counter ownership follows the stream's partition leases, so adding instances spreads subjects without a separate coordinator.

**Without the House.** A stack that does not run `loams-fabric house` (a desktop, a minimal compose) runs the meter in **enforcement-only mode**: counters are checkpointed into the `ControlStore` at each reconcile tick instead of rebased on the House, and nothing is exported. Q964.

---

## 6. The usage event (D1121)

### 6.1 Attributes

A usage event is a CloudEvent 1.0 ([spec](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md)) in the Loams profile (§34 §2, D364), constrained so that OpenMeter's `POST /api/v1/events` accepts it unchanged. OpenMeter's `Event` schema requires `subject` (stricter than CloudEvents) and accepts only `application/json` data ([openapi.yaml](https://raw.githubusercontent.com/openmeterio/openmeter/main/api/openapi.yaml)).

| Attribute | Required | Rule |
|---|---|---|
| `specversion` | yes | `1.0` |
| `id` | yes | **Derived from the work**, not random, so a re-emission after a crash is a duplicate. Examples: `house.query` → `<query_id>/<lease_seq>`; `house.storage` → `<ns>/<db>/<table>/<utc_day>`; a Postgres compute tick → `<endpoint>/<compute_id>/<tick_n>`; an LLM call → the gateway's request id; the node agent → `<node>/<source>/<seq>`, with `seq` assigned before the event enters its spool and kept on every retry (platform PD2). External producers keep it stable across retries (D364) |
| `source` | yes | URI reference naming the producer instance: `loams://<cluster>/<role>/<node>` for Loams' producers; `loams-ns://<ns>/<client>` for tenant producers (§9.6). `source` + `id` is the dedup key; CloudEvents requires it to be unique per distinct event and lets consumers treat a repeat as a duplicate (spec §"id") |
| `type` | yes | `io.loams.dev.usage.<product>.<name>.v<major>` for Loams' meters (§6.3). Meters select on `type`, as in OpenMeter |
| `subject` | yes | **The billing subject**: `org_<id>` for Loams' own billing (the org is the billing unit, D65). For user meters, the user's customer key. Set from the credential for Loams' producers, never from tenant input |
| `time` | yes | RFC 3339 with nanoseconds (D364). For Loams' producers, the producer's clock at the end of the measured interval. External events must be within 35 days in the past and 5 minutes in the future (Stripe's acceptance window, [meter event](https://docs.stripe.com/api/billing/meter-event/create)) |
| `datacontenttype` | yes | `application/json` (amends D364's `application/protobuf` default for these types; §6.4) |
| `dataschema` | yes for Loams types | `urn:loams:json:<type>` naming a JSON Schema published in `docs/api/meter/` |
| `tenantid` | yes | `<org>/<namespace>` (D364), from the credential |
| `traceparent` | optional | W3C Trace Context, as D364 |
| `loamslease` | optional | **New extension.** The lease id that admitted the work (§11.3), so the counter owner can retire the lease's outstanding amount as the event arrives. 10 characters, within CloudEvents' naming rule. Exporters strip it |
| `data` | yes | A JSON object; the measured values are numbers (integers where the unit allows: bytes, milliseconds, tokens), dimensions are strings |

No `idempotencykey` or `sequence` extension: `source` + `id` is the key (D364), and order within a subject is the partition's order.

### 6.2 Example

```json
{
  "specversion": "1.0",
  "id": "q-7f3a9c/1",
  "source": "loams://prod-eu1/house/node-3",
  "type": "io.loams.dev.usage.house.query.v1",
  "subject": "org_42",
  "time": "2026-10-19T09:14:03.218734Z",
  "datacontenttype": "application/json",
  "dataschema": "urn:loams:json:io.loams.dev.usage.house.query.v1",
  "tenantid": "42/7",
  "loamslease": "l9k2m4q8x1",
  "data": {
    "namespace": "7",
    "cpu_ms": 1840,
    "bytes_scanned": 734003200,
    "rows_read": 1200000,
    "result_bytes": 52100,
    "worker_ms": 2210,
    "statements": 1,
    "code": 0
  }
}
```

### 6.3 Loams' event types (initial set)

| Type | From | Data (units) | Producer seam |
|---|---|---|---|
| `…usage.house.query.v1` | House front, per worker lease | `cpu_ms`, `bytes_scanned`, `rows_read`, `result_bytes`, `worker_ms`, `statements`, `code` | `UsageObserver` (built) |
| `…usage.house.insert.v1` | House front | `rows`, `bytes`, `database`, `table` | `UsageObserver` |
| `…usage.house.storage.v1` | House storage round, daily per table | `bytes`, `day` (gauge; §8.3) | `UsageObserver` |
| `…usage.house.compaction.v1` | House front | `cpu_ms`, `bytes_scanned`, `worker_ms` | `UsageObserver` (billed to the namespace by Q692's default) |
| `…usage.postgres.compute.v1` | pg-control, every 60 s per running compute and at suspend | `cu_ms` (compute units × ms), `endpoint`, `branch` | `ComputeLifecycleObserver` plus a ticker |
| `…usage.postgres.storage.v1` | pg-control, hourly per branch | `logical_bytes` (gauge) | new pg-control round |
| `…usage.gateway.connection.v1` | Gateway `pg` module, on close | `bytes_in`, `bytes_out`, `duration_ms` | `ConnectionObserver` (built) |
| `…usage.gateway.request.v1` | Gateway `http` module (§57 §12.1) | `route`, `bytes_in`, `bytes_out`, `status` | new, per request, batched |
| `…usage.llm.tokens.v1` | Gateway `llm` module (§57 §12.2) | `model`, `provider`, `input_tokens`, `output_tokens`, `cached_tokens` | new |
| `…usage.mcp.call.v1` | Gateway `mcp` module (§57 §12.3) | `server`, `tool`, `duration_ms` | new |
| `…usage.function.invocation.v1` | Runners | `cpu_ms`, `wall_ms`, `tier`, `runner`, `estimated` (D180: CPU time plus requests) | `InvocationObserver` (D549) |
| `…usage.jobs.run.v1` | Jobs engine | `cpu_ms`, `queue` | JB1 hooks |
| `…usage.stream.ingest.v1` | Log role, per namespace per flush | `logical_bytes`, `records` (D103) | D103's counters |
| `…usage.storage.logical.v1` | Worker role, hourly per namespace | `logical_bytes` (gauge, D103) | D103's counters |
| `…usage.git.storage.v1`, `…usage.git.transfer.v1` | Loams Git | bytes (D393) | Git hooks |
| `…usage.sandbox.session.v1` | Sandbox | `cpu_ms`, `wall_ms`, `memory_mb_ms` | `stats()` (D995) |
| `…usage.node.cpu.v1`, `…usage.node.memory.v1`, `…usage.node.network.v1` | The node agent (`loams-meter-agent`, §21), one event per (tenant, resource, window) delta | `cpu_usec`; `byte_seconds`, `peak_bytes`; `ingress_bytes`, `egress_bytes`; plus `resource`, `function`, `tier`, `window_start`, `window_end` | cgroup v2 read from outside the sandbox (built, platform MB0) |
| `…usage.function.fuel.v1` | wasmtime hosts, per invocation | `fuel`, `cpu_usec`, `estimated` | the host report (platform doc 06; not built, MB4) |

A product that is not in this list is metered by adding its type and a `MeterSink` call; nothing else changes.

### 6.4 Why JSON, not protobuf

D364 chose protobuf data for Loams-defined types on the high-rate path. For usage events JSON wins:
- OpenMeter's meters read values by JSONPath (`valueProperty`, `groupBy`) and accept only JSON data, so a protobuf event cannot be exported as-is.
- The House pipe's CloudEvents format reads `data` with ClickHouse's JSON functions; protobuf would need the descriptor in the worker.
- The rate is moderate (§7.4) and events are small (about 400 bytes as JSON, estimate), so the size cost is small after Parquet compression.

---

## 7. Storage and dedupe (D1122)

### 7.1 The stream

All usage events, from every product and every external producer, land in **one system stream**: `_meter` in the `_system` namespace (the namespace that already holds the migration ledger, D928, and is not a tenant's). The name is the one D180 gave the meter stream and the one the imported node agent already writes (§21).

| Property | Value | Why |
|---|---|---|
| Partitions | 32 (configurable at creation) | Spreads counter ownership over meter instances; at least 4 × the expected instance count |
| Key | `subject` (D270: `partitionkey`, else `subject`) | One subject's events are in one partition, in order, owned by one counter owner |
| WAL class | `express` where configured, else `standard` | Stream → counter lag sets how stale enforcement counters are (§10.3) |
| Retention | 14 days (at least twice the longest House pipe outage the operator tolerates) | The reconciler replays the tail after the House's committed offsets (§12) |
| Encoding | `kafka` (D270's record layout) | The House pipe and the consume path rebuild CloudEvents from `ce_` headers |

### 7.2 Three ingest paths

1. **Loams' own in-process producers** (House, gateway, pg-control, runners, jobs, log and worker roles) use `MeterSink` in its **best-effort** mode, which is D365's high-rate path: a bounded queue per process (65,536 events or 64 MiB), flushed every 50 ms or at 1 MiB as one `Produce` of records in the CloudEvents record layout, oldest batch dropped when full and counted (`loams_events_dropped_total{reason="queue_full"}`). No ledger. The producer's response never waits.
2. **The node agent** (`loams-meter-agent`, imported from `loams-platform`, §21) uses `MeterSink` in its **spooled** mode: every event is appended and fsynced to a size-bounded on-disk spool before the agent advances its cgroup baselines, and a forwarder drains the spool into the stream. A full spool delays usage instead of losing it, because the counters it reads are cumulative (platform doc 01 PD4, PD8). This is the path for CPU and memory of sandboxes and runtimes, where a gap would be billed revenue lost.
3. **External producers** (user meters, partner systems, a migrating OpenMeter client) use `POST /api/v1/events` or `IngestService.Ingest`, which go through D270's ledger (claim, append, complete) with the dedup window set to **24 hours** for this stream. Each tenant's ingest is rate limited (D546's model) because the ledger costs two metastore proposals per request (§02 §7.4).

**Dropped events.** A full queue means the stream was unavailable for a while. Dropping favours the tenant (under-billing), never double billing. Enforcement does not depend on the events arriving, since the gate's local counters already decremented (§10). A sustained drop rate above zero pages the operator (§14).

### 7.3 Three layers of dedupe

| Layer | Catches | How | Exact? |
|---|---|---|---|
| 1. Producer | Retries of one event | `id` derived from the work (§6.1); the emitter retries a batch with the same records | Yes, if the producer is honest |
| 2. Counter owner | Duplicates within minutes (emitter retries, D270 `CommitUnknown` re-appends) | A per-partition set of recent dedup keys (16-byte prefix of the SHA-256 key), 10 minutes or 2 M keys per partition, oldest evicted | Best effort; residue fixed at reconcile |
| 3. House | Every duplicate, forever | `_meter.events` is `ReplacingMergeTree` with `ORDER BY (subject, type, source, id)`; every billing read uses `FINAL` | Yes |

ClickHouse documents `ReplacingMergeTree` as "eventual correctness only" without `FINAL` ([docs](https://clickhouse.com/docs/engines/table-engines/mergetree-family/replacingmergetree)); OpenMeter avoids it by deduplicating before ClickHouse (its events table is a plain `MergeTree`, [event_query.go](https://raw.githubusercontent.com/openmeterio/openmeter/main/openmeter/streaming/clickhouse/event_query.go)). Loams can use `FINAL` cheaply because the close job (§8) reads only closed windows, by partition and primary-key prefix, never the whole table, and the House's lake tables merge at read time (D765).

`FINAL` deduplicates within a partition. The table is partitioned by `toYYYYMM(time)`, and duplicates carry the same `time` (it is part of the event), so a duplicate always lands in its original's partition.

### 7.4 Volume (estimate)

A busy cluster with 10,000 active namespaces: House queries at 2,000/s, LLM calls at 500/s, gateway requests batched to 1 event per namespace per second (5,000/s), compute ticks at 200/s. About **8,000 events/s, 3.2 MB/s raw, about 280 GB/day before Parquet compression**. That is a moderate stream for the WAL (§02 targets) and well below the Fabric's range; it does not need Iggy.

### 7.5 House tables

```sql
-- in the system namespace, database _meter
CREATE TABLE _meter.events_q (…) ENGINE = LoamsStream('_meter', 'CloudEvents', 'house-meter');

CREATE TABLE _meter.events (
    subject     String,
    type        LowCardinality(String),
    source      String,
    id          String,
    time        DateTime64(6, 'UTC'),
    tenantid    String,
    lease       String,                -- loamslease, '' when absent
    data        String,                -- the JSON text
    stream_partition UInt32,
    stream_offset    UInt64
) ENGINE = ReplacingMergeTree
PARTITION BY toYYYYMM(time)
ORDER BY (subject, type, source, id);

CREATE MATERIALIZED VIEW _meter.events_mv TO _meter.events AS
SELECT _ce_subject AS subject, _ce_type AS type, _ce_source AS source, _ce_id AS id,
       parseDateTime64BestEffort(_ce_time, 6) AS time, _ce_tenantid AS tenantid,
       ifNull(_ce_loamslease, '') AS lease, data, _partition AS stream_partition, _offset AS stream_offset
FROM _meter.events_q;
```

The pipe is exactly once into `_meter.events`: the consumed offsets per partition are in the same Iceberg snapshot as the rows (D769). The reconciler reads them back (§12). Column names of the pipe's CloudEvents format (`_ce_*`, `_partition`, `_offset`) are §32 §5.4's layout (verify at HS1 Task 16).

---

## 8. Aggregation (D1123)

### 8.1 Meters

A meter is OpenMeter's definition ([`MeterCreate`](https://raw.githubusercontent.com/openmeterio/openmeter/main/api/openapi.yaml)):

| Field | Rule |
|---|---|
| `slug` | `^[a-z0-9]+(?:_[a-z0-9]+)*$`, at most 64 characters |
| `event_type` | the `type` it selects |
| `aggregation` | `SUM`, `COUNT`, `UNIQUE_COUNT`, `AVG`, `MIN`, `MAX`, `LATEST` |
| `value_property` | JSONPath into `data` (ignored for `COUNT`; a string for `UNIQUE_COUNT`) |
| `group_by` | map of dimension name → JSONPath (at most 8 dimensions; Stripe allows 100 combinations per customer, [limits](https://docs.stripe.com/billing/subscriptions/usage-based/recording-usage-api)) |
| `event_from` | optional: ignore events before this time |

Built-in meters (the starting catalogue; operators add more): `house_cpu_ms`, `house_scan_bytes`, `house_insert_bytes`, `house_storage_bytes` (`LATEST` per day, §8.3), `pg_compute_cu_ms`, `pg_storage_bytes`, `llm_input_tokens` and `llm_output_tokens` (grouped by `model`), `api_requests`, `mcp_calls`, `function_cpu_ms`, `function_invocations`, `jobs_cpu_ms`, `stream_ingest_bytes`, `logical_storage_bytes`, `git_storage_bytes`, `sandbox_cpu_ms`. Q695 (the House billing unit) is answered by configuration: both `house_cpu_ms` and `house_scan_bytes` exist and the plan chooses.

### 8.2 Windows and the close job

```sql
CREATE TABLE _meter.windows (
    meter         LowCardinality(String),
    subject       String,
    window_size   Enum8('minute' = 1, 'hour' = 2, 'day' = 3),
    window_start  DateTime('UTC'),
    group_key     String,               -- canonical JSON of the group-by values
    value         Decimal(38, 9),
    events        UInt64,
    close_run     UInt64                 -- the version: a later run wins
) ENGINE = ReplacingMergeTree(close_run)
PARTITION BY toYYYYMM(window_start)
ORDER BY (meter, subject, window_size, window_start, group_key);
```

The meter role schedules a **close job** in the House every 60 s:
1. Minute windows that ended at least 2 minutes ago and are not yet closed, plus any minute window that received a late event (detected from rows with `time` in a closed window and a stream offset beyond the last run's), are computed from `_meter.events FINAL` and inserted with a new `close_run`.
2. Hour and day windows are rolled up from minute windows for `SUM`, `COUNT`, `MIN` and `MAX`. `UNIQUE_COUNT`, `AVG` and `LATEST` do not roll up and are recomputed from raw events for each window size.
3. **Lateness**: an event up to 24 h late re-closes its windows. An event later than that but before its billing period is exported (§8.4) re-closes too. After export it is written as an adjustment to the next period (Q963).

Because each window row is versioned and keyed, a close job that runs twice, or two close jobs racing after a failover, produce the same final rows.

### 8.3 Gauges

Storage-like meters (`house_storage_bytes`, `pg_storage_bytes`, `logical_storage_bytes`, `git_storage_bytes`) are gauges, not counters. Their meter is `LATEST` per hour, and the plan's feature turns them into **byte-hours** for billing (the sum of hourly `LATEST` values) or a ceiling for enforcement (the latest value against a `storage_bytes` allowance). Enforcement of storage keeps the House's existing behaviour: past the limit, writes are refused (`164 READONLY`) and reads continue (§49 §15).

### 8.4 Billing periods

`_meter.period_totals` holds, per (subject, feature, period), the value used to rate the period. A period closes **2 hours after its end** (one close-job lap plus the minute grace plus margin), and is then handed to the exporters (§16). Loams uses each customer's own anchor (the subscription's start) as OpenMeter's `usagePeriod.anchor` does.

### 8.5 Query API

`MeterService.QueryMeter` and `GET /api/v1/meters/{slug}/query` (OpenMeter's parameters: `from`, `to`, `windowSize` `MINUTE|HOUR|DAY|MONTH`, `windowTimeZone`, `subject`, `groupBy`, `filterGroupBy`) read closed windows from `_meter.windows FINAL` and, for the open edge of the range, aggregate raw events directly. Results include `as_of` (the latest closed minute) so a caller can tell closed from provisional values.

---

## 9. The entitlement model (D1124)

### 9.1 Concepts

| Concept | Meaning | OpenMeter counterpart |
|---|---|---|
| **Customer** | Who pays: an org for Loams' own billing; a user's customer for user meters. Holds payment state and billing-provider ids | `Customer` with `usageAttribution.subjectKeys` |
| **Subject** | The key events carry in `subject`; a customer has one or more | `subject` |
| **Meter** | Turns events into numbers (§8.1) | `Meter` |
| **Feature** | What a customer is entitled to; optionally backed by a meter, with a unit | `Feature` |
| **Entitlement** | A subject's right to a feature: `metered` (with a usage period and a mode), `boolean` (on or off) or `static` (a config value, such as a maximum branch count) | `Entitlement` (metered, boolean, static) |
| **Grant** | An amount added to a metered entitlement: priority, effective time, expiry, rollover, recurrence | `Grant` |
| **Balance** | What is left in the current period after burn-down | `EntitlementValue` (`hasAccess`, `balance`, `usage`, `overage`) |
| **Plan** | A named set of entitlement templates applied to a customer (§9.5) | `Plan` (OpenMeter's product catalogue, simplified) |

### 9.2 Rust types (`loams-meter-core`)

```rust
/// One usage event as the core sees it, after the CloudEvent is validated.
pub struct Event {
    pub id: String,
    pub source: String,
    pub r#type: String,
    pub subject: SubjectKey,
    pub time: UtcNanos,                // i64 ns UTC (D374)
    pub tenant: TenantId,              // <org>/<namespace>
    pub lease: Option<LeaseId>,
    pub data: serde_json::Map<String, serde_json::Value>,
}
impl Event {
    pub fn dedup_key(&self) -> [u8; 32];  // SHA-256(source ‖ 0x00 ‖ id), as loams-cloudevents
}

pub enum Aggregation { Sum, Count, UniqueCount, Avg, Min, Max, Latest }
pub enum WindowSize { Minute, Hour, Day, Month }

pub struct Meter {
    pub slug: Slug,                     // ^[a-z0-9]+(?:_[a-z0-9]+)*$, ≤ 64
    pub event_type: String,
    pub aggregation: Aggregation,
    pub value_property: Option<JsonPath>,
    pub group_by: BTreeMap<String, JsonPath>,
    pub event_from: Option<UtcNanos>,
}

pub struct Feature {
    pub key: FeatureKey,                // e.g. "house_cpu"
    pub meter: Option<Slug>,
    pub filter: BTreeMap<String, String>, // group-by values that select part of a meter
    pub unit: Unit,                     // ms, bytes, tokens, requests, byte_hours …
    pub archived: bool,
}

pub enum Mode {
    /// Refuse when the balance reaches zero.
    HardLimit,
    /// Allow past zero; notify; overage is not billed (a grace allowance).
    SoftLimit,
    /// Allow past zero; overage is billed at `overage_price`, up to `spend_cap`.
    OverageAllowed { overage_price: Money, spend_cap: Money },
}

pub enum EntitlementKind {
    Metered {
        usage_period: UsagePeriod,       // interval + anchor
        mode: Mode,
        issue_after_reset: Option<Amount>,
        preserve_overage_at_reset: bool,
    },
    Boolean,
    Static { config: serde_json::Value },
}

pub struct Entitlement {
    pub id: EntitlementId,
    pub subject: SubjectKey,
    pub feature: FeatureKey,
    pub kind: EntitlementKind,
    pub created_at: UtcNanos,
    pub source: EntitlementSource,       // Plan { plan, version } | Manual { by }
}

pub struct Grant {
    pub id: GrantId,
    pub entitlement: EntitlementId,
    pub amount: Amount,                  // Decimal, in the feature's unit
    pub priority: u8,                    // 1..=255, lower burns first
    pub effective_at: UtcNanos,          // ceiled to the minute
    pub expires: Option<Expiry>,         // { duration: Hour|Day|Week|Month|Year, count: 1..=1000 }
    pub rollover: Rollover,              // { max: Amount, min: Amount }
    pub recurrence: Option<UsagePeriod>,
    pub kind: GrantKind,                 // Included | Promotional | Prepaid { paid: Money }
    pub voided_at: Option<UtcNanos>,
}

pub struct Balance {
    pub entitlement: EntitlementId,
    pub period: Period,
    pub has_access: bool,
    pub balance: Amount,                 // ≥ 0
    pub usage: Amount,
    pub overage: Amount,
    pub total_available: Amount,
    pub as_of: AsOf,                     // Counter { owner_epoch, offset } | House { snapshot }
}

pub struct Money { pub micros: i64, pub currency: Currency } // ISO 4217 (D374)
```

`Amount` is a fixed-point decimal (`rust_decimal`, 28 significant digits), never a float.

### 9.3 Burn-down, rollover and reset

As OpenMeter ([grants](https://openmeter.io/docs/billing/entitlements/grant)):
- Usage burns grants in order: **lower priority first, then nearest expiry, then oldest created**. Stripe's credit order is the same with promotional before paid at equal priority ([credits](https://docs.stripe.com/billing/subscriptions/usage-based/billing-credits)); Loams adds that tie-break: `Promotional` before `Included` before `Prepaid`, so paid credit is spent last.
- At a reset, each grant's balance becomes `min(rollover.max, max(balance_before_reset, rollover.min))`. `issue_after_reset` sets the starting balance of each period. Expired or voided grants forfeit what is left.
- `preserve_overage_at_reset`: last period's overage is deducted from the new starting balance (for soft limits only; billed overage is not carried).

The engine is a pure function `balance(entitlement, grants, usage_segments, at) -> Balance`, where usage segments are per-window sums. It is property-tested against a brute-force replay and against OpenMeter's documented examples (OpenMeter's own implementation is a reference, not a dependency).

### 9.4 Prepaid credits

A prepaid purchase is a `Prepaid` grant with a money value. **Prepaid credits stop at zero**: an entitlement funded only by prepaid grants behaves as `HardLimit` when they are spent, whatever its mode, unless the customer has turned on **auto top-up** (a card charge of a set amount when the balance falls below a threshold; ME1e, Q965).

### 9.5 Plans

A plan is a versioned list of entitlement templates:

```yaml
# plans/free.yaml (in the GitOps repository, §22)
plan: free
version: 3
entitlements:
  - feature: house_cpu
    metered: { period: { interval: month }, mode: hard_limit, issue_after_reset: 36000000 }  # 10 CPU-hours
  - feature: llm_output_tokens
    metered: { period: { interval: month }, mode: hard_limit, issue_after_reset: 1000000 }
  - feature: pg_branches
    static: { max: 10 }
  - feature: byoc
    boolean: false
```

Assigning a plan to a customer creates its entitlements and their recurring grants; changing a plan version updates them at the next period, or at once with proration left to the billing provider. Plans hold allowances and per-unit overage prices only (§3.2).

### 9.6 User meters (D1138)

The same engine, scoped to a user's namespace: a Loams user defines meters, features and plans for **their own** customers, sends events to `POST /api/v1/events` with a namespace credential, and checks `GET /api/v1/subjects/{subject}/entitlements/{feature}/value` or links `loams-meter-gate` into their service. Their events go to a stream in their namespace and their House database, never to `_system/_meter`, and their usage of the meter itself is metered as ordinary stream ingest and House usage. ME1f.

---

## 10. The enforcement path (D1125)

### 10.1 The gate

```rust
pub trait Gate: Send + Sync + 'static {
    /// May `cost` units of `feature` be spent for `subject` now? Reads local state only on a hit.
    fn check(&self, subject: &SubjectKey, feature: &FeatureKey, cost: Amount) -> Decision;
    /// Hold `estimate` units for work whose cost is known only after it runs.
    fn reserve(&self, subject: &SubjectKey, feature: &FeatureKey, estimate: Amount) -> Result<Reservation, Refusal>;
}

pub struct Reservation { /* lease id, held amount */ }
impl Reservation {
    /// Spend `actual`, return the rest to the local chunk, emit the usage event with `loamslease`.
    pub fn settle(self, actual: Amount, event: UsageEventBuilder);
}

pub enum Decision { Allow { lease: LeaseId }, AllowOverage { lease: LeaseId }, Refuse(Refusal) }
pub struct Refusal { pub reason: RefusalReason, pub feature: FeatureKey, pub retry_after: Option<Duration> }
pub enum RefusalReason { EntitlementExhausted, PaymentMethodRequired, SpendCapReached, NotEntitled, MeterUnavailable }
```

**Local state.** Per (subject, feature): the active lease (remaining amount, expiry, mode, owner epoch) in a sharded map of atomics; a hit is a lookup and an atomic subtract. Leases refresh in the background when half the chunk is used or `refresh_after` passes. A refused check costs the same as an allowed one and never calls the network more than once per lease interval, so a refused tenant cannot load the owner.

### 10.2 Where the gate is called

| Product | Admission point | Call | Cost known |
|---|---|---|---|
| House | `Quotas::admit`, before the queue (§49 §15), after the protective quotas | `check(cost = 1 unit floor)` on `house_cpu` / `house_scan`; the statement that crosses the limit finishes, the next is refused (R22.1's rule) | After: settled by the `house.query` event |
| Gateway `http` (§57 §12.1) | Pingora `request_filter` | `check(api_requests, 1)` | Before |
| Gateway `llm` (§57 §12.2) | Before forwarding | `reserve(llm_output_tokens, max_tokens)` and `check(llm_input_tokens, estimated prompt tokens)`; `settle` with the provider's counts in the response | After, bounded by `max_tokens` |
| Gateway `mcp` (§57 §12.3) | On `tools/call` (`Mcp-Method`) | `check(mcp_calls, 1)` | Before |
| Gateway `pg` (§57) | On route resolution, before the splice | `check(pg_compute, 0)` (has access at all); a hard-stopped subject's new connection gets an error at startup | — |
| pg-control | Compute wake and resize | `reserve(pg_compute_cu_ms, 60 s × CU)`, settled by each compute tick | After, per minute |
| Jobs | Enqueue and lease (§26 §6.9) | `check(jobs_runs, 1)`; CPU via `reserve` per run | Both |
| Functions | Invoke | `check(function_invocations, 1)`, `reserve(function_cpu_ms, the function's CPU limit)` | After, bounded by the limit |
| Streams | Produce (log role) | `check(stream_ingest_bytes, batch bytes)` | Before |
| Git, sandbox | Push; session start and each minute | `check` / `reserve` | Both |

Each product keeps its own error mapping (§17.4). A product never calls the House or the meter's APIs on the request path; the gate is its only contact.

### 10.3 Latency budget

| Step | Budget | Kind |
|---|---|---|
| `check` / `reserve` / `settle` on a lease hit | ≤ 1 µs p50, ≤ 5 µs p99, no allocation | target |
| Event enqueue in `MeterSink` | ≤ 1 µs p99, never blocks | target |
| Background lease refresh | off the request path; one RPC per (subject, feature, node) per ≤ 10 s while active | target |
| **Cold miss** (no lease on this node): synchronous `AcquireLease` to the owner | ≤ 2 ms p50, ≤ 10 ms p99 within a region; **deadline 50 ms**, then §13's policy | target |
| Emitter flush | every 50 ms or 1 MiB (D365) | decided |
| Append to `_system/_meter` | `express` p99 20–50 ms, `standard` 400–600 ms (§02 §2) | published targets |
| Stream → counter owner | ≤ 1 s p99 on `express`, ≤ 2 s on `standard` | estimate |
| Threshold crossing → threshold event appended | ≤ 5 s p99 | target |
| Minute window visible in `_meter.windows` | ≤ 3 min after the minute ends | target |
| Counter rebase on the House | every 5 min | decided |

For comparison: Envoy's external rate-limit filter defaults to a 20 ms timeout and fails open ([rate_limit.proto](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/filters/http/ratelimit/v3/rate_limit.proto), verify the default); Google Service Control recommends caching `check` results for 60 s and failing open on server errors ([check](https://docs.cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/check), [allocateQuota](https://docs.cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/allocateQuota)); OpenMeter's edge access reads are "≤ 25 ms" and eventually consistent over seconds ([blog](https://openmeter.io/blog/launchweek-2-03-low-latency-usage-limit-enforcement)). Loams' in-process lease hit is three to four orders of magnitude cheaper than any of these, and only the cold miss pays a round trip.

---

## 11. Multi-node leasing (D1126)

### 11.1 Who owns a subject

The `meter` role consumes `_system/_meter` as the named consumer `meter-counters`, taking a **per-partition lease** in the metastore, which fences its offset commits (§02 §7). The holder of partition *p*'s lease is the **counter owner** for every subject that hashes to *p* (murmur2 of `subject`, as the stream's partitioner, so the gate computes the owner without asking). The lease's fencing epoch is the **owner epoch**; every lease grant carries it, and a gate drops grants from an older epoch.

### 11.2 The protocol

`LeaseService.AcquireLease{subject, feature, node, want, held: [LeaseReport]}` → `Lease{id, amount, mode, has_access, expires_at, refresh_after, owner_epoch, balance_hint}`.

- **TTL 30 s, refresh after 10 s** (Doorman's defaults are 60 s and 16 s, [design](https://github.com/youtube/doorman/blob/master/doc/design.md)); shorter here because the TTL bounds how long capacity is stranded on a dead node.
- Each refresh reports what the node holds and spent (`LeaseReport{lease, granted, spent}`), so the owner always knows outstanding capacity per node.
- **Chunk size** = `clamp(allowance × 1%, min_chunk(feature), remaining / (2 × active_holders))`. Early in a period a node gets 1% of the allowance per refresh; as the balance shrinks, chunks shrink so one node cannot strand the tail. `min_chunk` is per feature (1 request, 64 KiB, 1,000 tokens, 100 CPU-ms).
- **Exact mode.** When `remaining < min_chunk × active_holders`, the owner grants only what a single operation needs and only to one requester at a time; every operation in that tail pays the round trip. A subject with a tiny allowance (Q962) is in exact mode from the start.
- `has_access = false` with `amount = 0` is a refusal the gate caches until `refresh_after`, so refused traffic stays local.
- For `SoftLimit` and `OverageAllowed` subjects under their cap, the lease only counts; it never refuses below the cap.

### 11.3 Owner accounting

For each (subject, feature, period) the owner keeps:
- `counted`: usage from events read off the stream (dedup layer 2), rebased by §12;
- per active lease: `granted − retired`, where `retired` is the sum of events carrying that lease id (`loamslease`) — so the same unit is not counted once as usage and again as an outstanding lease;
- `available = balance(counted) − Σ outstanding(active leases)`.

A lease whose TTL has passed stops being outstanding after one more stream-lag allowance (5 s), and its later events count only as usage.

### 11.4 Overshoot, and why it is bounded

Leases never grant more than `available`, so gated, cost-known work cannot overshoot. Overshoot comes only from:

| Source | Bound |
|---|---|
| Work whose cost is known after it runs (House CPU and scan, function CPU) | One operation per concurrent slot per node; each is capped by the product's own per-operation limits (House `max_execution_time`, function CPU limit, LLM `max_tokens`) |
| A node that loses contact with its owner keeps spending its chunk until expiry | ≤ one chunk per node (≤ 1% of the allowance) |
| Owner failover (§11.5) | ≤ one chunk per node that refreshed with the old owner and is not yet re-reported |
| Fail-open for paid subjects (§13) | The customer's spend-cap headroom |
| Ungated producers (storage gauges) | Refused at the next write after the next storage round (§8.3) |

With N nodes holding leases, overshoot from leasing is at most about N% of the allowance, and in practice much less because chunks shrink near the end (§11.2). Doorman's simulations handed out about 96.6% of capacity with shortfall peaking at 106% ([design](https://github.com/youtube/doorman/blob/master/doc/design.md)); this is the same class of trade-off.

### 11.5 Failover

1. The partition lease moves to another meter instance (fenced epoch + 1).
2. The new owner **rebuilds counters** for the partition: House totals at the pipe's committed offset for *p*, plus a replay of the stream from that offset (§12). Target: under 30 s for 1 M subject-features per partition (estimate).
3. **Learning mode** for one TTL (30 s), as Doorman's servers do after a master election: the new owner treats each node's last possible chunk as outstanding until that node re-reports its `held` leases, and grants only from what is left.
4. Gates seeing the new epoch re-report and continue. Requests that miss during the rebuild wait up to the 50 ms deadline, then follow §13.

---

## 12. Reconciliation (D1127)

Every 5 minutes, and at every period close, each counter owner, per partition *p*:
1. reads the House's latest `_meter.events` snapshot and its committed pipe offset `O_p` (the snapshot summary, D769);
2. queries period-to-date usage per (subject, feature) from `_meter.events FINAL` restricted to `stream_partition = p` and `stream_offset ≤ O_p`, pinned to that snapshot;
3. sets `counted = house_total + tail`, where `tail` is the sum of events after `O_p`. The owner keeps per-feature sums in **offset buckets** (one bucket per 65,536 offsets), so the tail is the buckets after `O_p` plus a replay of the one straddling bucket;
4. records the drift `|counted_before − counted_after|` per subject and feature.

This counts each event exactly once, because the House side and the tail side are split at the same offset, and it removes any duplicate layer 2 missed. A House pipe that is behind only makes the tail longer (more replay), never wrong. A drift above **1% of an allowance** raises `loams_meter_reconcile_drift_ratio` and an operator alert.

Billing never reads counters. Enforcement never waits for reconciliation.

---

## 13. Policy (D1128)

| Situation | Free tier (no payment method) | Paid, no overage opt-in | Paid, overage opted in | Prepaid only | Contract (`invoice`) |
|---|---|---|---|---|---|
| Balance > 0 | Allow | Allow | Allow | Allow | Allow |
| Balance reaches 0 | **Hard stop** (`entitlement_exhausted`) | Hard stop | Allow; overage billed per unit | **Hard stop** (unless auto top-up) | Per contract: `soft_limit` by default |
| Overage reaches the spend cap | — | — | **Hard stop** (`spend_cap_reached`) | — | Operator alert |
| Turn on overage | Refused: `payment_method_required` | Allowed: card on file, opt-in by a billing admin, cap required (default 2× the plan's included value, Q961) | — | Needs a card | Operator sets |
| Counter owner unreachable past the 50 ms deadline, no lease | **Fail closed** (`meter_unavailable`, `Retry-After: 1`) | **Fail open** within the spend-cap headroom (or one plan period's included amount if no cap), counted locally and reconciled | Fail open within the cap | Fail closed | Fail open |
| Gate holds an unexpired lease | Use it | Use it | Use it | Use it | Use it |
| Stream unavailable (events cannot be appended) | Enforcement continues on leases; events queue, oldest dropped and counted | Same | Same | Same | Same |
| House unavailable | Enforcement continues; reconciliation and export pause; alert after 1 h | Same | Same | Same | Same |
| Thresholds | 80% and 100% events (§15) | Same, plus 80% and 100% of the spend cap | Same | Same, plus "credits at 20%" | Same |

Defaults are configuration in the `meter` role, overridable per plan. The payment state of a customer is `none`, `card` or `invoice`, set by the billing exporter from provider webhooks (Stripe `payment_method.attached`, `customer.updated`, a daily poll, Q967), or by an operator for contract customers.

**Hard stop means:** new work is refused; work already running finishes (House statements, LLM streams in progress); running Postgres computes are suspended at the next idle or after 15 minutes, never killed mid-transaction by the meter (Q968). Reads of existing data stay allowed so a customer can export (storage enforcement refuses only writes, §8.3).

---

## 14. Failure modes

| Failure | Detection | Effect | Response |
|---|---|---|---|
| Counter owner crashes | Partition lease expires | Leases on gates continue until TTL; cold misses wait, then policy | New owner rebuilds (§11.5); `loams_meter_owner_rebuild_seconds` |
| Two owners believe they own *p* | Impossible to commit: offset commits and grants are fenced by the partition lease epoch | Gates ignore grants with an older epoch | — |
| Network partition gate ↔ owner | Refresh fails | Gate spends its chunk until expiry, then policy | `loams_meter_gate_refresh_failures_total` |
| `_system/_meter` unavailable | `Produce` errors | `MeterSink` queues, drops oldest when full (under-bills), enforcement unaffected | Alert on any `loams_events_dropped_total{stream="_meter"}` |
| D270 ledger `CommitUnknown` on external ingest | §02 §7.4 | A retry after 2 min may append again | Layers 2 and 3 remove it |
| House pipe lags | `loams_house_pipe_lag_records{pipe="_meter"}` | Longer replay at reconcile; windows late | Alert at 15 min lag; exports wait |
| House down | Reconcile fails | Counters drift unchecked; exports wait | Alert after 1 h; enforcement unaffected |
| Close job fails or races | Run ledger in `_meter.close_runs` | Windows late; duplicate runs are harmless (versioned rows) | Retry; alert after 3 failures |
| Producer clock skew | `time` vs append time | Events in the wrong minute | Loams nodes run NTP (§10 operations); external events outside [−35 d, +5 min] are refused |
| Late events after export | Close job sees `time` in an exported period | Adjustment row in the next period | Q963 |
| Exporter provider down | HTTP errors | Cursor does not advance | Retries with idempotency keys (§16); alert after 6 h |
| Payment webhook missed | Daily poll mismatch | Payment state stale up to a day | Poll corrects; a stale `card` never bypasses the spend cap |
| Metastore or `ControlStore` down | Entitlement writes fail | Cached entitlements keep enforcing; no new grants | The last known entitlements stay in force (§41 §9.3's rule) |
| A gate bug allows everything | Reconcile drift, usage above allowance on a hard-limit subject | Over-use | `loams_meter_hard_limit_overrun_ratio` alert at 5% |
| A tenant forges usage events | Reserved types refused from tenant credentials (§19) | None | Audit |

---

## 15. Threshold and lifecycle events (D1129)

The counter owner appends these to **the same stream**, `_system/_meter`, so one consumer pattern serves usage and notifications and the House keeps a history:

| Type | When | `id` (re-emission is a duplicate) |
|---|---|---|
| `io.loams.dev.meter.threshold.v1` | Usage crosses a configured percentage of the period's total available (default 80 and 100), or of the spend cap | `<subject>/<feature>/<period>/<threshold>` |
| `io.loams.dev.meter.access.changed.v1` | `has_access` flips (hard stop starts or ends) | `<subject>/<feature>/<period>/<seq>` |
| `io.loams.dev.meter.entitlement.reset.v1` | A usage period resets | `<entitlement>/<period>` |
| `io.loams.dev.meter.grant.v1` | A grant is created, voided or expires | `<grant>/<state>` |

These types are excluded from every meter. A **notifier** in the `meter` role consumes them and delivers:
- webhooks (CloudEvents, structured mode, signed with the endpoint's secret, retried with backoff), configured per customer;
- email through the deployment's configured mailer (none on the desktop);
- console notifications and the shell banner (§18).

OpenMeter's equivalent is the `entitlements.balance.threshold` notification with `usage_percentage` values ([openapi.yaml](https://raw.githubusercontent.com/openmeterio/openmeter/main/api/openapi.yaml)); the OpenMeter exporter maps to it where the target uses OpenMeter's notifications.

---

## 16. Exporters (D1130)

Every exporter is optional, off by default, and one-way: it reads Loams' tables and writes to a provider. Loams never reads balances back from a provider; it reads only payment state.

| Exporter | Sends | Idempotency | Notes |
|---|---|---|---|
| **Stripe** (first, Q965) | For each closed **hour** of a billable feature, the **overage** of that hour, not the raw usage, as platform doc 01 §9.4 (PD7) specified: `overage(h) = max(0, cum(h) − included) − max(0, cum(h−1) − included)`, where `cum` is the period's usage through hour `h` from `_meter.windows` and `included` the balance the grants gave at the period start. One meter event per hour with `event_name` = the meter, `payload.stripe_customer_id`, `payload.value`, `timestamp` = the hour; an hour with no overage is sent only to overwrite an earlier non-zero revision with 0. Billing periods start on whole UTC hours (`billing_cycle_anchor` at the next whole hour, never reset on plan changes), so no hour straddles a period start; an off-hour `period_start` stops that customer's export and alerts | `identifier` = `loams:<meter>:<subject>:<hour>:<close_run>`; Stripe keeps identifiers unique for at least 24 h ([meter event](https://docs.stripe.com/api/billing/meter-event/create)). The Stripe meters are **hour pre-aggregated** (`event_time_window = hour`), where the latest event of an hour overwrites the earlier ones ([configure](https://docs.stripe.com/billing/subscriptions/usage-based/meters/configure)), so a re-closed hour is sent again under its new `close_run`; an export ledger in the `ControlStore` records what was sent per hour (platform PD7) | Aggregating per hour keeps far below the v1 limit of 1,000 calls/s and the 10,000 dimension combinations per meter per hour ([limits](https://docs.stripe.com/billing/subscriptions/usage-based/recording-usage-api)); the v2 meter event stream (session tokens, up to 10,000 events/s, verify the path) is used only if a deployment exceeds that. Overage prices are Stripe prices; included allowances are Loams grants. Legacy usage records are gone since API `2025-03-31.basil` ([changelog](https://docs.stripe.com/changelog/basil/2025-03-31/deprecate-legacy-usage-based-billing)) and are not used |
| **OpenMeter** | Raw usage events, filtered by type, unchanged (minus `loamslease`), `POST /api/v1/events` with `application/cloudevents-batch+json` | OpenMeter dedupes by `id` + `source` ([docs](https://openmeter.io/docs/metering/events/usage-events)); its window is configuration, 32 days in the example config and off by default ([config.example.yaml](https://raw.githubusercontent.com/openmeterio/openmeter/main/config.example.yaml)), so the exporter also keeps its own cursor | For operators who prefer OpenMeter's billing and catalogue (or Kong Konnect) |
| **Lago** | Events by window, `POST /api/v1/events/batch` with `transaction_id`, `external_subscription_id`, `code`, `timestamp`, `properties` ([events](https://getlago.com/docs/api-reference/events/usage)) | `transaction_id` = the Stripe identifier's form; Lago overwrites a duplicate on its ClickHouse pipeline | Over HTTP only: Lago is AGPL-3.0 and is never linked or shipped (D11) |
| **CloudEvents webhook** | Closed windows or period totals as CloudEvents | `source` + `id` | For in-house billing systems |
| **Iceberg** | Nothing to send: `_meter.windows` and `_meter.period_totals` are Iceberg tables any engine can read (D762) | — | Spark, Trino, DuckDB |

Each exporter's cursor (the last exported `close_run` per period) is in the `ControlStore`. An export is retried until the provider accepts it; the idempotency key makes retries safe.

---

## 17. APIs (D1131)

### 17.1 Connect: `loams.meter.v1`

| Service | Methods | Who |
|---|---|---|
| `MeterService` | `CreateMeter`, `GetMeter`, `ListMeters`, `UpdateMeter` (display fields only; a meter's semantics are immutable once used, as Stripe's), `ArchiveMeter`, `QueryMeter` (server-streaming for large ranges) | `meter_admin`; `QueryMeter` also `billing_viewer` for own subjects |
| `IngestService` | `Ingest` (a `CloudEventBatch`, D270's ledger) | Namespace or service credentials |
| `CustomerService` | `CreateCustomer`, `GetCustomer`, `ListCustomers`, `UpdateCustomer`, `SetPaymentState` (billing service only), `SetSpendControls` (overage opt-in, cap; `billing_admin`) | `meter_admin`, `billing_admin` |
| `PlanService` | `ApplyPlan` (from YAML, §22), `ListPlans`, `AssignPlan`, `UnassignPlan` | `meter_admin` |
| `EntitlementService` | `CreateEntitlement`, `ListEntitlements`, `GetEntitlementValue`, `CheckAccess` (batch), `CreateGrant`, `VoidGrant`, `ListGrants`, `ResetEntitlement`, `GetBalanceHistory` | Writes `meter_admin`; reads `billing_viewer` |
| `NotificationService` | `CreateRule`, `ListRules`, `CreateWebhook`, `RotateWebhookSecret`, `ListDeliveries` | `billing_admin` |
| `LeaseService` (internal) | `AcquireLease`, `ReturnLease` | Service identities of product roles only |
| `ExportService` | `ListExporters`, `GetExporterStatus`, `ReplayPeriod` | `meter_admin` |

AP0 conventions apply: `NO_SIDE_EFFECTS` reads, `idempotency_key` on every mutation, AIP-158 pages, `ErrorInfo.reason` (§49 §18.2).

### 17.2 REST

Every Connect method has its REST mapping under `/v1/meter/…` through the API's transcoding (§44). In addition, an **OpenMeter-compatible subset** is served under `/api/v1/` on the meter's listener, so OpenMeter SDKs and existing integrations work unchanged:

| Path | OpenMeter operation |
|---|---|
| `POST /api/v1/events` | `ingestEvents` (`application/cloudevents+json`, `application/cloudevents-batch+json`, `application/json`); 204 |
| `GET /api/v1/meters`, `GET /api/v1/meters/{slug}`, `GET /api/v1/meters/{slug}/query` | `listMeters`, `getMeter`, `queryMeter` |
| `GET /api/v1/subjects/{subject}/entitlements/{featureKey}/value` | `getEntitlementValue` |
| `GET/POST /api/v1/subjects/{subject}/entitlements/{id}/grants` | grants |

Compatibility is checked against OpenMeter's `api/openapi.yaml` at a pinned commit by a conformance test (ME1c). OpenMeter's v2 and billing APIs are out of scope.

### 17.3 Read-only views for other products

The limits record's `EnforcementSample` (CP1) gains an `entitlement` section (`feature`, `balance`, `has_access`, `as_of`) so the operator view shows one picture of protective limits and entitlements.

### 17.4 Errors on each surface

| Surface | Hard stop | Spend cap | Payment needed | Meter unavailable (free) |
|---|---|---|---|---|
| Connect / gRPC | `RESOURCE_EXHAUSTED`, reason `entitlement_exhausted` | `RESOURCE_EXHAUSTED`, `spend_cap_reached` | `FAILED_PRECONDITION`, `payment_method_required` | `UNAVAILABLE`, `meter_unavailable` |
| REST (API, LLM, MCP gateways) | **402** with the reason (Q960) | 402 | 402 | 503 with `Retry-After` |
| ClickHouse (House) | `202` for reads, `164` for writes, message starting `house_quota_exceeded: entitlement …` (§49 §15's convention) | same | same | `202` |
| Postgres (gateway, pg-control) | SQLSTATE `53400` (`configuration_limit_exceeded`) at connect, message names the feature | same | same | `53300` (`too_many_connections`)-style retry, verify the best fit |
| OpenAI- and Anthropic-compatible faces | HTTP 402 with the face's error shape and `type: "insufficient_quota"` (OpenAI's shape; verify) | same | same | 503 |

429 remains for D546's rate limits, so a client can tell "slow down" from "out of allowance".

---

## 18. Console (D1132)

The pages are a new plugin, `web/plugins/billing`, in the `oss` catalogue (D1119 removes the edition split for billing), shown when the deployment runs the `meter` role (`flags.features.meter`).

| Surface | Where (§59) | Shows |
|---|---|---|
| **Usage** | Settings → Billing → Usage (D1080) | Per feature: used, included, balance, overage, the period, a daily chart from `_meter.windows`; per namespace breakdown; `as_of` and "provisional" for the open window |
| **Entitlements** | Settings → Billing → Entitlements | The plan, each entitlement's mode, grants (with expiry and rollover), prepaid credit |
| **Spend controls** | Settings → Billing → Spend controls | Overage opt-in (disabled without a card, with the reason), the spend cap, auto top-up; changes need `billing_admin` and a confirmation, and are audited |
| **Payment** | Settings → Billing → Payment | Payment state and a link to the provider's hosted page (Stripe Customer Portal); Loams never handles card data |
| **Alerts** | Settings → Billing → Alerts | Threshold rules, webhook endpoints, delivery log |
| **Banner** | Shell top bar | At 80%: "<feature> is at 80% of this month's allowance"; at 100% with a hard stop: what stopped and how to resume (add a card, buy credit, wait for the reset date) |
| **Operator: Meters, Plans, Customers** | Operator view (D555) | Definitions, plan versions, customer search, manual grants (with a reason, audited) |
| **Operator: Meter health** | Operator view | Owner per partition and epoch, lease counts, reconcile drift, pipe lag, close-job runs, exporter cursors, dropped events |

Charts use the shared chart components; numbers in money are formatted from micros with the currency (D374). Q546 is answered: dashboards may show money, labelled with `as_of`.

---

## 19. Security, integrity and tenancy (D1133)

D541 held that figures that set what a tenant pays must not be open to manipulation by agents or tenants, and concluded the code must be private. The goal stands; the means change. Loams relies on these, all of which work with the source public (Kerckhoffs's principle):

1. **Who may produce what.** `io.loams.dev.usage.*` and `io.loams.dev.meter.*` types are accepted only from Loams service identities (the cluster's mTLS identities, as CP1's agent and hub use). A tenant or agent credential that sends them gets `PERMISSION_DENIED`. Tenant ingest goes to the tenant's own stream (§9.6), never `_system/_meter`.
2. **Who is billed.** `subject` and `tenantid` on Loams' events are set from the authenticated principal at the first Loams hop (D364's rule, extended to `subject`); a client-supplied value is replaced and counted.
3. **Where usage is measured.** CPU, bytes and tokens are measured outside tenant code: the House front reads the worker's cgroup (R22.6), the node agent reads every sandbox's cgroup (§21), the gateway counts bytes and reads token counts from the provider's response, never from the client. **Every billed meter has a source the tenant cannot reach** (platform PD63, PD67): where a shared host reports per-invocation splits (wasmtime, T0's workerd), the split is reconciled per window against the cgroup total, and the cgroup total wins. **Host reports are authenticated** (platform PD64, kept): when the host-report socket is built (ME1e), the node agent identifies each connecting host with `SO_PEERCRED`, gives it a random per-boot epoch and key, and accepts only reports carrying `HMAC-SHA-256(key, host_id ‖ epoch ‖ seq ‖ invocations)`; a replayed, foreign or out-of-order report is refused, and lost reports produce a `Gap` record instead of silence.
4. **What is billed.** Only `_meter.windows` from deduplicated raw events. Gaming a counter gains at most a chunk of allowance before the next reconcile; it cannot lower a bill (D546's argument, kept).
5. **Who may change entitlements.** OpenFGA relations on the deployment and org: `meter_admin` (meters, plans, grants, customers), `billing_admin` (spend controls, alerts, payment link), `billing_viewer` (read). **Agent principals cannot hold `meter_admin` or `billing_admin`** (test `agent_cannot_write_grants`, as CP1's `agent_cannot_write_limits`), and an agent principal's own spend is under a hard daily cap it cannot raise (platform PD71). `SetPaymentState` is callable only by the billing exporter's service identity.
6. **Audit.** Every grant, plan assignment, spend-control change, manual adjustment and payment-state change is an audit event in `_audit` (D100) with the principal and reason.
7. **Tamper evidence.** Raw events are append-only in the WAL and in Iceberg snapshots; each period's totals record the snapshot id and the close run they came from, so an invoice line can be recomputed from history.
8. **Privacy.** Usage events carry identifiers and numbers, never query text, prompts or document content. Raw events are kept 13 months and windows 7 years by default (Q963; verify the operator's tax obligations). Erasing an org deletes its raw events and keeps anonymised window totals only where the law requires invoices to be kept.
9. **Tenancy.** The `_system/_meter` stream and the `_meter` House database are in the `_system` namespace; tenants read their own usage only through `MeterService` and `EntitlementService`, filtered by their subjects. User meters (§9.6) live entirely in the user's namespace.

---

## 20. Relationship to earlier decisions (D1134)

| Earlier | Change |
|---|---|
| D541 (integrity) | **Restated** by D1133: integrity by authentication, measurement location and billing only from the House, with the code open |
| D546 (quota enforcement without metering) | **Amended**: the limits record and its approximate counters stay for protective limits; entitlements are a second gate (§10.2 calls the gate after the product's protective quota). "Counters are never usage" becomes "counters are never billed" |
| D547 (what stays open in observability) | **Superseded in part**: billing-grade usage is open; the metric families stay as they are |
| D548, D550, D552 (private metering, private commercial APIs, the guard) | Superseded by D1119; this design is where metering lives |
| D202, D551 (dependency runs one way) | **Kept as a build rule**: products depend on `loams-meter-gate`'s trait and its `NoGate`, never on the `meter` role; the role depends on products only through events |
| D364 (CloudEvents profile) | **Amended** for `io.loams.dev.usage.*`: `subject` required, JSON data, the `loamslease` extension |
| D365 (high-rate path) | Used as is; "the event table keyed on `(source, id)`" is the House table `_meter.events` |
| D103 (logical bytes) | Its units become meters (`stream_ingest_bytes`, `logical_storage_bytes`); "namespace statistics" stays the API name for the D547 view |
| D180 (CPU time plus requests) | Restored as meters `function_cpu_ms` and `function_invocations` |
| CPQ4 / D850 (House per-query events through a private observer) | **Superseded**: `MeterSink` implements the open `UsageObserver` |
| JB1 global constraint "no billing", CP1 Task 27 (field-name ban), §52's naming rule | **Retired**: fields may be named `usage`, `meter`, `credit` and so on |
| Q546 | Answered (D1132) |
| Q695 | Answered by configuration (§8.1) |
| Q692 | The default (the namespace pays for compaction) is kept as a meter the plan may price at zero |
| Q-RT-7, Q562 | Prices and start dates are plan configuration, not design; they stay with the owner |

---

## 21. Migration from `loams-platform` (D1135)

**What was surveyed.** The owner approved cloning `ostrium-labs/loams-platform` on 2026-10-19 (Q966). Its `main` (`0592cbf`; every one of its 9 other branches is merged into it, 39 commits from 2026-09-29 to 2026-10-02) was read for this section, and the repository is being imported into this monorepo with its full history on the branch `import-platform`, under `platform/` as a Cargo workspace of its own (like `fabric/`), relicensed Apache-2.0 by its sole author. The import's audit found no secrets, credentials, customer or personal data, and no copyleft dependency in the meter crate. The import renamed `loam-meter` to `loams-meter` and its event type to `dev.loams.meter.v1`, without changing behaviour, and left refactors to this design.

**What the code contains.** One crate, about 2,900 lines with tests: the **node metering agent** of platform doc 01's milestone MB0.

| Module | What it does | Fate |
|---|---|---|
| `cgroup` (`CgroupReader`) | Walks configured cgroup v2 subtrees, samples target cgroups (pod cgroups, `tenant-*.slice`, `wasm-host.scope`) for `cpu.stat`, `memory.current`, `memory.peak`, `cgroup.events`; never descends into a target (no double count); a lenient scan skips a vanished or unreadable cgroup | **Kept** as is |
| `delta` (`DeltaTracker`) | Cumulative counters into per-window deltas: a decrease is a reset, resources keyed by path and inode, first sight counts all of a new resource while warm, a cold start baselines at the current reading (an undercount, never a double bill), two-phase observe and commit, memory integrated as byte-seconds | **Kept**; these are the accuracy rules of §7.2's spooled path (platform PD8) |
| `resolver` (`TenantResolver`, `StaticResolver`) | Maps a cgroup to org, namespace, function and tier by prefix rules from JSON | **Kept**; the Kubernetes pod-watch resolver and final reads (MB2) are ME1e work |
| `event` (`MeterEvent`) | CloudEvents in JSON structured mode: type `dev.loams.meter.v1`, `subject` `<org>/<namespace>`, `id` `<node>/<source>/<seq>`, `data` with `org`, `namespace`, `meter` (`cpu_usec`, `memory_byte_seconds`, `memory_peak_bytes`, `wasm_fuel`, `requests`, `ingress_bytes`, `egress_bytes`, `durable_promises`, `durable_timers`), the delta `value`, the window and the resource; unattributed usage under the org `_platform` | **Changed** to §6: types `io.loams.dev.usage.node.{cpu,memory,network}.v1`, `subject` the billing subject and `tenantid` `<org>/<namespace>`, `dataschema`; the `id` scheme is kept. Requests and durable counts come from the gateway and the durable server, not the agent. `_platform` usage is kept, metered and never billed |
| `spool` (`Spool`) | Segment files of JSON lines, fsync per append, an `ack` file replaced by rename, a torn last line truncated on open, size-bounded (`SpoolFull` keeps usage in the counters) | **Kept** as `MeterSink`'s spooled mode (§7.2) |
| `agent` | The tick (scan, resolve, delta, spool, commit) with state saved by rename and spool replay after a crash, so nothing is emitted twice under a new id | **Kept** |
| `sink`, `stream` (`Sink`, `MemorySink`, `StreamSink`, `forward`) | Forwards the spool to `_system/_meter` over the HTTP produce route; key = org, partition `xxh3(org) % partitions`, value = the structured-mode JSON, header `ce-id` | **Changed**: records in D270's binary-mode layout (`loams-cloudevents::record::to_record`), key = `subject` and the stream's default partitioner (murmur2), so the House pipe reads `_ce_*` columns and a subject's events reach its counter owner (§11.1) |

**What the platform designed but never built** (doc 01's track MB and doc 06): the `loams-usage` aggregator with hourly rollups in the `ControlStore`, dedupe keys in TiKV and `GET /v1/orgs/{org}/usage` (MB1); the Kubernetes resolver and inotify final reads (MB2); the Envoy access-log receiver (MB3); the wasmtime host socket with `loams.meter.v1.HostReport`, `SandboxFinished` and the per-invocation reporter (MB4, doc 06 PD61–PD64); the Stripe billing export with Checkout, the Customer Portal and webhooks (MB5, PD7, PD10); Resonate counts and per-sandbox egress (MB6).

**How each platform decision lands here:**

| Platform | Here |
|---|---|
| PD1 node agent plus aggregator | The agent stays (renamed, below). The aggregator is **superseded** by the House windows (§8) and the counter owners (§11); it was never built |
| PD2 CloudEvents meter events, OpenMeter-compatible | Kept and refined by D1121 |
| PD3 sources per tier | Kept: cgroup for T0 and T2, the host report for T1 (ME1e), the gateway instead of Envoy ALS (§57 replaced Envoy, D1010–D1013), Resonate counts from the durable server |
| PD4 spool then `_system/_meter` | Kept: §7.2's spooled path and the same stream |
| PD5 hourly rollups in the `ControlStore` | **Superseded** by D1123 (House windows) |
| PD6 OpenMeter compatible but not adopted; Lago rejected | Kept (§2, §16) |
| PD7 Stripe hourly overage with an export ledger, PD10 Stripe directly at GA | Kept in the Stripe exporter (§16); the Clerk beta phase is gone (the site uses Auth.js with Authentik) |
| PD8 accuracy rules | Kept in the agent; the 2 h lateness matches §8.4 |
| PD61–PD64 host report, signed epoch | Kept as the design of the T1 source (ME1e), with the HMAC epoch in §19 |
| PD66 Lambda by billed duration | Kept as the default for the Lambda runner |
| PD67 a second source for every billed meter | Kept (§19) |
| PD69–PD72 commercial layer writes limits; agent spend caps | The plan YAML writes entitlements and protective limits (§9.5); agent spend caps in §19 |
| Doc 06 §4a and doc 07 §5: x402 agent payments through Cloudflare's Monetization Gateway (PD80–PD83) | **Out of scope** here; it stays a candidate in `docs/platform/`. A meter's HTTP 402 (§17.4) carries no x402 payment requirements |

**The migration:**

| Piece | Fate |
|---|---|
| `platform/crates/loams-meter` (the agent) | **Renamed `loams-meter-agent`** and moved into the root workspace (`crates/loams-meter-agent`, on the root's reqwest and base64 majors) in ME1a, so this design's role crate keeps the name `loams-meter` (Q966, answered 2026-10-19 by the owner: keep and rename to `loams-meter-agent`). Its event and sink changes above land in the same task |
| `dev.loams.meter.v1` events already in any `_system/_meter` stream | Translated by the `meter` role's ingest for one release into the per-quantity types, keeping `source` and `id`, so re-reads deduplicate |
| `loams.meter.v1` host-report messages (doc 06) | Built under `loams.meter.v1.host` when the T1 source is built; the package `loams.meter.v1` is this design's Connect API |
| `io.loams.dev.meter.usage.v1` (doc 06 PD65) | Never built; not carried over |
| `platform/` workspace | Empty once the agent moves; removed with the move |
| Platform docs 01, 06, 07 | Kept in `docs/platform/design/` as history, each with a note that §63 supersedes their metering and billing parts |

After the migration, metering lives in `crates/` and this design, and `platform/` holds no code.

---

## 22. Meters, features and plans as code (D1137)

Plans, features and meters are YAML in the deployment's GitOps repository (§41 §6: tenant onboarding is a Git commit), applied by `PlanService.ApplyPlan` from the reconciler that already applies tenant records. A change is a reviewed commit; the meter refuses an apply that changes the semantics of a meter with recorded events (a new meter slug is required, as Stripe's meters are immutable).

An **OpenTofu provider** (`loams_meter`, `loams_feature`, `loams_plan`, `loams_customer_plan`) comes after the API is stable (ME1f). It targets OpenTofu's registry and protocol; it may work with Terraform's, but Terraform is not tested or documented as a target, since its licence is BUSL 1.1 (§2). Neither tool meters; they only provision definitions.

---

## 23. Phases (D1136)

| Milestone | Delivers | Depends on |
|---|---|---|
| **ME1a** events and emitters | `loams-meter-core` types and event validation; the imported agent moved and renamed `loams-meter-agent`, with §6's event types and D270's record layout; `loams-meter-gate`'s `MeterSink` (D365) and `NoGate`; adapters for `UsageObserver`, `ConnectionObserver`, `ComputeLifecycleObserver`; `_system/_meter` created by `migrate` (§54); JSON Schemas in `docs/api/meter/` | D365's emitter (built here if not yet built) |
| **ME1b** storage and windows | House tables, pipe and MV; close job; period totals; `QueryMeter` and the OpenMeter query path | HS1b (lake tables, `ReplacingMergeTree`), HS1c (pipes) |
| **ME1c** entitlements, counters, reconciliation | The balance engine with property tests; `CustomerService`, `EntitlementService`, `PlanService`; counter owners on partition leases; reconcile; threshold events and the notifier; the OpenMeter conformance test | ME1a, ME1b; MT1 for real credentials |
| **ME1d** enforcement and policy | `LeaseService` and the lease allocator; `Gate::check/reserve/settle`; calls at House admission, pg-control, the gateway `pg` module, jobs, streams; policy table; error mapping per surface; detsim runs of failover and partition | ME1c; GW1 for the `pg` module; the gateway `http`, `llm`, `mcp` modules as they land |
| **ME1e** exporters, console, migration | Stripe exporter and payment-state webhooks; OpenMeter, Lago, webhook exporters; `web/plugins/billing`; operator pages; the T1 host-report source and the agent's Kubernetes resolver (platform doc 06, MB2); retire the naming rules | ME1c; CF1 shell |
| **ME1f** user meters and provisioning | Namespace-scoped meters for users; the OpenTofu provider | ME1e |

Gates (target): `check` p99 ≤ 5 µs under 64 threads; cold miss p99 ≤ 10 ms in a three-node test cluster; overshoot ≤ 2% of the allowance with 10 nodes and an owner killed mid-period (detsim and a real cluster); zero double-counted events across 10,000 injected duplicates and pipe restarts; reconcile drift ≤ 0.1% in steady state.

---

## 24. Effort and risk

**Effort (estimate):** ME1a 2 weeks, ME1b 3 weeks, ME1c 4 weeks, ME1d 4 weeks, ME1e 4 weeks, ME1f 3 weeks: about 20 engineer-weeks, much of it parallel after ME1c.

| Risk | Mitigation |
|---|---|
| House pipes and lake `ReplacingMergeTree` are not built yet (HS1b, HS1c Planned) | Enforcement-only mode (§5.2) runs without the House; ME1b waits for HS1 |
| `FINAL` cost grows with history | Close job reads only closed windows by partition and primary-key prefix; compaction merges duplicates in the background (D765) |
| Lease protocol bugs over- or under-grant | Sans-I/O allocator, detsim with node kills and partitions, a hard-limit overrun alarm (§14) |
| Per-request events from the gateway overload the stream | Batch per namespace per second for `api_requests`; per-request only for LLM and MCP |
| OpenMeter changes its API | Compatibility is pinned to a commit and tested; it is a subset |
| Billing errors are expensive | Billing only from the House, re-computable from history; manual adjustments audited; a dry-run export mode that writes what it would send |
| Stripe rate limits | Hourly aggregation (§16) |

---

## 25. Open questions (Q960–Q969)

**All answered on 2026-10-19 by the owner: the default.**

| # | Question | Default |
|---|---|---|
| Q960 | REST status for a hard stop: 402 Payment Required, or 429 like rate limits? | **402** with `ErrorInfo.reason`; 429 stays for rate limits, so clients can tell "out of allowance" from "slow down" |
| Q961 | Is a spend cap mandatory with overage, what is its default, and how long may a paid subject fail open when its counter owner is unreachable? | **Yes**, a cap is mandatory; default **2×** the plan's included value per period, which the customer may raise. A paid subject fails open **only within its spend-cap headroom** (or one period's included amount if it has no cap), counted locally and reconciled; never unbounded |
| Q962 | Lease defaults: chunk 1% of the allowance, TTL 30 s, refresh 10 s; and exact mode for tiny allowances (fewer than 10 chunks per active node)? | **Yes** to both, per feature overridable. In exact mode every operation asks the owner, so free tiers with small allowances pay a round trip only near their limit |
| Q963 | Lateness and retention: re-close windows for events up to 24 h late, also later ones before export, an adjustment in the next period after export, external events older than 35 days refused; raw events kept 13 months, windows and period totals 7 years? | **Yes** (verify the retention against the operator's jurisdiction) |
| Q964 | Without the House (desktop, minimal compose): enforcement-only mode with counters checkpointed in the `ControlStore`? | **Yes**; no export in that mode |
| Q965 | Which exporter first, and who owns credits (Loams grants or Stripe credit grants)? Auto top-up? | **Stripe first** (the owner's earlier choice for the beta: Stripe directly), then OpenMeter, then Lago. **Loams grants own credits**: enforcement needs them in real time, and Stripe credits apply only at invoice time. Auto top-up in ME1e, off by default |
| Q966 | The migration survey needs read access to `loams-platform`, and the crate name `loams-meter` collides with the platform's crate | **The owner grants read access or runs the survey** (nothing is cloned without asking). **This design takes the crate name**. **Answered 2026-10-19 by the owner: keep and rename to `loams-meter-agent`.** The clone was approved, §21 is written from the code, and the platform's crate (the node agent, not the reporter) is kept under that name instead of retired |
| Q967 | Source of payment state: provider webhooks plus a daily poll; contract customers set by an operator? | **Yes** |
| Q968 | At a hard stop, is running work killed? | **No**: new work is refused and statements and streams finish; Postgres computes are suspended at the next idle or after 15 minutes |
| Q969 | JSON data for usage events, amending D364's protobuf default? | **Yes** (§6.4) |

User meters (§9.6) are decided by D1138 and scheduled as ME1f, so they need no question.

## 26. Sources

Loams (all on `dev` at `034c8cb4`, read 2026-10-19):
- [§02](02-stream-engine.md) §3, §7, §7.4; [§32](32-loams-flow-fabric-house.md) §5; [§34](34-protocol-gateway-and-standards.md) §2–§3; [§41](41-multitenant-byoc-control-plane.md) §9–§12; [§49](49-loams-house-production.md) §5–§9, §15, §18; [§54](54-upgrades.md) D928; [§57](57-loams-gateway.md) §11–§12; [§59](59-console-foundation.md) D1080; [§24](24-cpu-time-runtime.md), [§26](26-jobs-api.md) §6.9, [§27](27-usage-hooks.md).
- Plans: [HS1](../plans/2026-10-08-hs1-house-production.md) Task 22 and rulings R22.1–R22.8; [CP1](../plans/2026-10-10-cp1-control-plane-byoc.md); [JB1](../plans/2026-10-10-jb1-jobs-and-usage.md); [GW1](../plans/2026-10-16-gw1-loams-gateway.md).
- `loams-platform` at `0592cbf` (cloned with the owner's approval 2026-10-19; imported on `import-platform`): `crates/loam-meter/src/{agent,cgroup,delta,event,resolver,sink,spool,stream}.rs`; docs `01-metering-and-billing.md` (PD1–PD10), `06-meter-record-and-usage-reporter.md` (PD61–PD68), `07-commercial-apis.md` (PD69–PD72, PD80–PD83); the import's pre-publication audit (2026-10-19).
- Code: `fabric/crates/loams-house/src/{quota,usage}.rs`; `crates/loams-cloudevents/src/{lib,record,partition}.rs`; `crates/loams-gateway/src/observe.rs`; `crates/loams-pg-control/src/observer.rs`; `crates/loams-common/src/meta/store.rs`.
- Decisions: D11, D65, D86, D98, D100, D103, D180, D202, D270, D331, D343, D364, D365, D374, D541, D546–D552, D555, D762–D769, D776, D850, D928, D1010–D1024, D1080, D1119 (the 2026-10-19 open-source decision).

External (read 2026-10-19):
- OpenMeter: [repository and README](https://github.com/openmeterio/openmeter); [OpenAPI](https://raw.githubusercontent.com/openmeterio/openmeter/main/api/openapi.yaml); [usage events](https://openmeter.io/docs/metering/events/usage-events); [entitlements](https://openmeter.io/docs/billing/entitlements/entitlement); [grants](https://openmeter.io/docs/billing/entitlements/grant); [ClickHouse post](https://openmeter.io/blog/how-openmeter-uses-clickhouse-for-usage-metering); [events table query](https://raw.githubusercontent.com/openmeterio/openmeter/main/openmeter/streaming/clickhouse/event_query.go); [Redis dedupe](https://raw.githubusercontent.com/openmeterio/openmeter/main/openmeter/dedupe/redisdedupe/redisdedupe.go); [config example](https://raw.githubusercontent.com/openmeterio/openmeter/main/config.example.yaml); [edge access](https://openmeter.io/blog/launchweek-2-03-low-latency-usage-limit-enforcement); [Kong acquisition](https://konghq.com/blog/news/kong-acquires-openmeter).
- CloudEvents 1.0.2: [spec](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md); [JSON format](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/formats/json-format.md); [HTTP binding](https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/bindings/http-protocol-binding.md); [partitioning](https://github.com/cloudevents/spec/blob/main/cloudevents/extensions/partitioning.md) and [sequence](https://github.com/cloudevents/spec/blob/main/cloudevents/extensions/sequence.md) extensions.
- Lago: [repository](https://github.com/getlago/lago); [events API](https://getlago.com/docs/api-reference/events/usage); [aggregation types](https://getlago.com/docs/guide/billable-metrics/aggregation-types/overview); [wallets](https://getlago.com/docs/guide/wallet-and-prepaid-credits); [progressive billing](https://getlago.com/docs/guide/plans/progressive-billing); [alerts](https://getlago.com/docs/guide/alerts); [entitlements](https://getlago.com/docs/guide/entitlements).
- Stripe: [create meter](https://docs.stripe.com/api/billing/meter/create); [configure meters](https://docs.stripe.com/billing/subscriptions/usage-based/meters/configure); [meter event](https://docs.stripe.com/api/billing/meter-event/create); [recording usage and limits](https://docs.stripe.com/billing/subscriptions/usage-based/recording-usage-api); [billing credits](https://docs.stripe.com/billing/subscriptions/usage-based/billing-credits); [usage alerts](https://docs.stripe.com/billing/subscriptions/usage-based/alerts); [legacy usage removed](https://docs.stripe.com/changelog/basil/2025-03-31/deprecate-legacy-usage-based-billing).
- Distributed quota: Doorman [repository](https://github.com/youtube/doorman), [design](https://github.com/youtube/doorman/blob/master/doc/design.md), [algorithms](https://github.com/youtube/doorman/blob/master/doc/algorithms.md); Envoy [ratelimit service](https://github.com/envoyproxy/ratelimit), [global rate limiting](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/other_features/global_rate_limiting), [local rate limit filter](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/local_rate_limit_filter), [rate limit filter API](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/filters/http/ratelimit/v3/rate_limit.proto); Google Service Control [allocateQuota](https://docs.cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/allocateQuota), [check](https://docs.cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/check), [report](https://docs.cloud.google.com/service-infrastructure/docs/service-control/reference/rest/v1/services/report); [Stripe rate limiters](https://stripe.com/blog/rate-limiters); [Cloudflare counting](https://blog.cloudflare.com/counting-things-a-lot-of-different-things/); [US10275283B2, low-latency distributed counters for quotas](https://patents.google.com/patent/US10275283); [GCRA](https://brandur.org/rate-limiting).
- Spend caps: [Vercel](https://vercel.com/docs/spend-management), [Vercel changelog](https://vercel.com/changelog/spend-management-now-pauses-production-deployments-by-default), [Firebase](https://firebase.google.com/docs/projects/billing/avoid-surprise-bills).
- Licences: [HashiCorp BUSL](https://www.hashicorp.com/en/blog/hashicorp-adopts-business-source-license), [Terraform LICENSE](https://raw.githubusercontent.com/hashicorp/terraform/main/LICENSE), [OpenTofu announcement](https://www.linuxfoundation.org/press/announcing-opentofu), [OpenTofu LICENSE](https://raw.githubusercontent.com/opentofu/opentofu/main/LICENSE).
- ClickHouse: [ReplacingMergeTree](https://clickhouse.com/docs/engines/table-engines/mergetree-family/replacingmergetree), [guide](https://clickhouse.com/docs/guides/replacing-merge-tree).
