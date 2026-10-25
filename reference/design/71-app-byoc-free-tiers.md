# 71 — App BYOC: Apps and Agents on Each Cloud's Free Tier

> **Amendment (approved 2026-10-23) by [§78](78-loams-cloud-on-tidb-x.md)** (Loams Cloud on TiDB X; D1700–D1735). Target manifests (`deploy/targets/<id>.toml`, D1707, D1735) reference this document's free-tier table for app ports instead of copying it, and `loams app deploy --target` reads the same target ids as `loams byoc deploy` and `loams promote` (D1706, D1724).

> **Amendment (approved 2026-10-23) by [§79](79-void-and-vite-plus.md)** (Void and Vite+; D1770–D1791 approved by the owner). Void becomes the recommended authoring framework, portable by default (D1392, D1778). For Void apps on Cloudflare, `void deploy` replaces `wrangler` (D1406, D1779). Bundling moves from esbuild to Vite+ (Rolldown). On AWS, Google Cloud, Azure and Vercel, §7's shims wrap Void's `node` build (D1780), because Void has no deploy adapter for them. New targets: `void-platform` (D1782) and a self-hosted image (D1781). Self-hosted and container targets (Cloud Run) run a Bun binary compiled ahead of time, while Lambda, Azure Functions and Vercel keep Node (D1791). Deno is not a target (D1790). The portable contract (D1391) and the ports (D1393) stand.

Status: **Approved** · 2026-10-21. The requirement is the owner's; the owner approved the design with every default on 2026-10-21.

Source: the owner's requirement of 2026-10-21, **"app BYOC must map onto each cloud's free tiers"** (the owner's example: AWS Lambda plus DynamoDB), given while [§70](70-byoc-clouds.md) was being written. §70 designs **data-plane BYOC**, Loams itself in a customer's account. This document designs **app BYOC**: the apps and agents that the Software Factory, Studio and the agents marketplace build, deployed into a customer's own account on serverless free tiers, with no always-on VM. It was split from §70 because together they were too long for one document; they share §70's principles (§70 §4) and the `loams` CLI.

Decisions **D1390–D1410** (D1411–D1419 kept free) and open questions **Q1095–Q1109** are declared in the [decision log](13-decision-log.md), in the range §70 reserved. Plan: [BY1](../plans/2026-10-21-by1-byoc-clouds.md), track AB (a stub; AB0 is listed task by task at milestone level).

This document builds on:
- [§70](70-byoc-clouds.md): the principles (D1351), the customer's account and keys, the AWS test account (§70 §12.2).
- [§24](24-cpu-time-runtime.md): Loams Functions: the `static`, `fetch` and `http-port` contracts (D181), the Workers framework presets (D171), the manifest `loams.toml` `[functions]` (D177), workerd's KV and R2 bindings mapped onto Loams (§24 §4.2), the `Runner` trait and its external runners (D375, §24 §16), and D170 ("not a Vercel clone").
- [§20](20-reactive-database-on-tikv.md) and [§45](45-loams-live-production.md): Live's queries, mutations, actions and scheduled functions, which run inside the engine on `loams-kv` (D691, D692).
- [§26](26-jobs-api.md): Loams Jobs on TiKV (D204–D216; Q97: no Postgres or DynamoDB jobs backend).
- [§68](68-open-protocol-compute.md): the protocol ladder (D1251), whose L0 rule ("back the unmodified product with Loams primitives") this document turns around: the app keeps open interfaces, and each cloud's own primitives back them.
- [§69](69-edge-reach.md): how an app on a serverless runtime reaches a Loams data plane: one edge door (D1302), Cloudflare Tunnel and Workers VPC (D1305), cloud private links (D1315), workload-identity exchange (D1308).
- [§63](63-loams-meter.md): the Gate (D1125), entitlements and thresholds (D1128, D1129).
- [§64](64-loams-infra-stack.md): workload trust policies and the Loams workload issuer (D1158).
- [§39](39-software-factory-and-loams-bot.md) and [SF4](../plans/2026-10-02-sf4-factory-loop.md): the factory loop's deploy stage (D-SF-13); [§53](53-studio-ade.md): Studio; `docs/marketplace/publishing.md`: the marketplace's package.

Markers, as in §64–§70: **(verify)**, **(estimate)**, **(unverified)**. Vendor facts were read on 2026-10-21 from the page linked beside each one; a date after a link is the page's own date, "read 2026-10-21" means the page shows none.

---

## 1. Summary

### 1.1 The problem

The Factory, Studio and the marketplace produce apps and agents: a webhook handler, a chat agent, a small CRUD app, a scheduled report. Today the only place Loams can run them is a Loams data plane (Loams Functions on workerd, designed in §24 and not built). That has two costs for the customer:
- **An always-on machine** (§70's VM tier, about $60–$82 a month, or a Cloudflare Container at about $46), for apps that may serve a few hundred requests a day.
- **Lock-in to Loams' runtime** for code that needs nothing Loams-specific.

Every cloud the owner named offers serverless compute and state with a free allowance that such apps never leave. The owner's requirement is that app BYOC lands there.

### 1.2 The answer

- **Two tiers, said plainly** (D1390): *app BYOC* runs an app on the customer's serverless free tier with that cloud's own state services; *data-plane BYOC* (§70) runs Loams. An app needs a data plane only when it uses a Loams feature (§4).
- **One portable app contract** (D1391): the Workers module shape (`export default { fetch, queue, scheduled }`, a WinterTC-style `fetch(Request) → Response`), declared in `loams.toml` under `[app]`. It is §24's `fetch` contract (D181) with two more entry points.
- **Four small ports** (D1393): `items` (keyed JSON documents with conditional writes and prefix ranges), `blobs`, `queue` and `schedule`, plus `secrets` and an optional `loams` binding. Each has an adapter per cloud onto that cloud's free-tier service.
- **Targets** (D1395):

  | Cloud | Compute | `items` | `blobs` | `queue` | `schedule` |
  |---|---|---|---|---|---|
  | **Cloudflare** | Workers | D1 | R2 | Queues | Cron Triggers |
  | **AWS** | Lambda with a Function URL | DynamoDB (provisioned 25 RCU / 25 WCU) | S3 | SQS | EventBridge Scheduler |
  | **Google Cloud** | Cloud Run (request-based billing) | Firestore (the default database) | GCS (a free US region) | Cloud Tasks | Cloud Scheduler |
  | **Azure** | Functions (Flex Consumption) | Cosmos DB free tier | Blob Storage | Queue Storage | Timer trigger |
  | **Vercel** | Vercel Functions | Upstash Redis or Neon (Marketplace free plans) | Vercel Blob | Vercel Queues | Vercel Cron |
  | **Loams** (any data plane) | Loams Functions (§24) | Live tables | The bucket | Loams Jobs | Durable schedules |

- **The same app moves between targets** without code changes, including onto a Loams data plane when it outgrows a free tier or needs Loams features (D1396).
- **Cost guards** (D1400–D1403): deploy within the free allowances by default; refuse a deploy whose declared limits could exceed them unless the user says `--allow-paid`; set the caps each cloud offers (fixed DynamoDB capacity, Lambda reserved concurrency, Cloud Run and Functions instance limits, budgets with actions); warn at 80% and 100% from the cloud's own usage figures; report into the customer's Loams Meter when they run one.
- **When an app needs Loams** (D1397): it gets a `loams` binding (`@loams/client`) that reaches the customer's data plane through §69's edge, with workload identity from ER2. Ostrium hosts no shared data plane (D1378).
- **Phases** (D1405): **AB0** Cloudflare and AWS first (Cloudflare's Free plan stops at its limits instead of billing; the owner's AWS account exists); **AB1** Google Cloud, Azure, Vercel, the Meter and the `loams` target; **AB2** the Factory, Studio and marketplace wired to it.

### 1.3 Decisions

| # | Decision | § | Status |
|---|---|---|---|
| D1390 | **App BYOC is a tier of its own:** apps and agents built with Loams deploy into the customer's account on serverless free tiers, with that cloud's own state services and no always-on VM; data-plane BYOC (§70) is separate and needed only for Loams features | 1, 4 | Approved (owner, 2026-10-21) |
| D1391 | **The portable contract is the Workers module shape:** `export default { fetch(request, env, ctx), queue(batch, env, ctx), scheduled(event, env, ctx) }`, TypeScript or JavaScript, standard `Request`/`Response`, declared in `loams.toml` `[app]`. It extends D181's `fetch` contract with two entry points; `static` assets ride along; `http-port` is not an app-BYOC contract | 5 | Approved (owner, 2026-10-21) |
| D1392 | **Frameworks are the developer's choice** (Hono recommended; Nitro, OpenNext, plain handlers) as long as the build output exports that shape; Loams writes a thin entry shim per target, not a framework | 5.3 | Approved (owner, 2026-10-21) |
| D1393 | **Four ports plus two bindings:** `items`, `blobs`, `queue`, `schedule`; `secrets`; and an optional `loams` client binding. Their semantics are the common floor of every target (§6), with documented limits; nothing richer is promised | 6 | Approved (owner, 2026-10-21) |
| D1394 | **`items` is a keyed-document store:** a partition key and a sort key, JSON values up to 256 KiB, `get`, `put` (unconditional, create-only or on a version), `delete` (optionally on a version), `list` by sort-key prefix within one partition (paged), optional TTL; no secondary indexes, no multi-item transactions | 6.1 | Approved (owner, 2026-10-21) |
| D1395 | **One target per cloud, chosen for its free tier:** Cloudflare Workers + D1 + R2 + Queues + Cron Triggers; Lambda + Function URL + DynamoDB (provisioned) + S3 + SQS + EventBridge Scheduler; Cloud Run + Firestore + GCS + Cloud Tasks + Cloud Scheduler; Azure Functions Flex Consumption + Cosmos DB free tier + Blob + Queue Storage + timer triggers; Vercel Functions + Marketplace free storage + Vercel Blob + Queues + Cron; and `loams` | 7 | Approved (owner, 2026-10-21) |
| D1396 | **The `loams` target:** the same app on a Loams data plane, as Loams Functions (§24) with `items` on Live tables, `blobs` on the bucket, `queue` on Loams Jobs and `schedule` on Durable schedules, so an app can leave a free tier without a rewrite | 7.7 | Approved (owner, 2026-10-21) |
| D1397 | **Loams features need a data plane:** hybrid retrieval, graph memory, Memory, Durable, Live's reactive queries and mutations, Loams Postgres, House and Git are reached through the `loams` binding over §69's edge (environment-bound keys now, workload identity from ER2). No free-tier imitation of them is built | 8 | Approved (owner, 2026-10-21) |
| D1398 | **The data plane an app uses is the customer's**, either one per app or one shared by many of the customer's apps (one namespace and environment each); Ostrium hosts none (D1378) | 8.2 | Approved (owner, 2026-10-21) |
| D1399 | **Live apps split at the engine:** a Live app's queries and mutations stay in the engine (they are transactions on Live's store); app BYOC hosts its HTTP front, actions, queue consumers and schedules, which call Live through the client | 8.3 | Approved (owner, 2026-10-21) |
| D1400 | **Free by default:** `loams app deploy` provisions each service on its free allowance (DynamoDB provisioned at the free capacity, Firestore's default database, Cosmos DB's free-tier account, a free GCS region) and refuses a deploy that the free tier cannot hold, unless `--allow-paid` | 9.1 | Approved (owner, 2026-10-21) |
| D1401 | **A dated free-tier table** (`sdks/typescript/packages/app/free-tiers.json`, generated from `docs/guides/app-byoc/free-tiers.md`) holds each allowance with its source link and the date it was read; a nightly job fails when a source page changes a number | 9.2 | Approved (owner, 2026-10-21) |
| D1402 | **Caps at deploy time:** where a cloud can cap spend, the deploy sets the cap (DynamoDB capacity without autoscaling, Lambda reserved concurrency, Cloud Run and Flex Consumption maximum instances, Cloudflare and Vercel free plans that stop at their limits); where it cannot, a budget with an alert, and an action where the cloud has one | 9.3 | Approved (owner, 2026-10-21) |
| D1403 | **Usage warnings:** a usage poller reads each target's own usage figures daily and warns at 80% and 100% of the free allowance; with a Loams data plane it sends them as usage events to the customer's Meter (§63) with the allowances as entitlements, so the Meter's thresholds, alerts and console apply | 9.4 | Approved (owner, 2026-10-21) |
| D1404 | **Deploy credentials are short-lived and the customer's:** the developer's own cloud CLI session for `loams app deploy`; for the Factory's deploy stage, federation from the Loams workload issuer (D1158) or CI OIDC to an AWS role, a Google workload identity pool or an Azure federated credential; scoped API tokens in Loams Secrets for Cloudflare and Vercel | 10 | Approved (owner, 2026-10-21) |
| D1405 | **Phases AB0–AB2:** AB0 Cloudflare and AWS (contract, ports, two adapters, the deploy CLI, cost guards v0, an example, a nightly within free tiers); AB1 Google Cloud, Azure, Vercel, the Meter link and the `loams` target; AB2 the Factory's deploy stage, Studio's deploy action and marketplace packages | 13 | Approved (owner, 2026-10-21) |
| D1406 | **App infrastructure is generated, not hand-written:** `loams app deploy` renders a small OpenTofu root module per app for AWS, Google Cloud and Azure (the official providers, state in the customer's own bucket), and uses `wrangler` for Cloudflare and the Vercel CLI for Vercel | 7.8 | Approved (owner, 2026-10-21) |
| D1407 | **Ports conformance:** one suite per port, run against each adapter on local emulators in PRs and on real free-tier accounts nightly | 12 | Approved (owner, 2026-10-21) |
| D1408 | **Not built:** a framework, a hosting service, a free-tier imitation of Loams features, multi-item transactions or secondary indexes in `items`, a portable workflow port in AB0, a Dapr sidecar, or deployment to any account but the customer's | 14 | Approved (owner, 2026-10-21) |
| D1409 | **Apps' own endpoints are the app's surface, not Loams':** a Function URL or Worker route is public when the app is a public web app; the never-public rule (D1311) governs the Loams data plane, which the app reaches only through the edge | 8.1 | Approved (owner, 2026-10-21) |
| D1410 | **Queue messages are CloudEvents** (D270) of at most 48 KiB; larger payloads go to `blobs` with the key in the event (claim check) | 6.3 | Approved (owner, 2026-10-21) |

### 1.4 Phases

| Phase | Content | Effort (estimate) |
|---|---|---|
| **AB0** | The contract and `loams.toml` `[app]`; `@loams/app` with the four ports; the Cloudflare and AWS adapters and entry shims; `loams app deploy\|destroy\|status\|usage` for both; cost guards v0 (the free-tier table, the refusal, caps, AWS Budgets); the example app (`web/examples/app-byoc-notes`); the ports suite on Miniflare and DynamoDB Local; a nightly on a Cloudflare Free account and the owner's AWS account | 5–7 engineer-weeks |
| **AB1** | Google Cloud, Azure and Vercel adapters and shims; the usage poller and the Meter link; the `loams` target; the `loams` binding with edge-reach options; Python handlers (Q1103) | 7–9 engineer-weeks |
| **AB2** | The Factory deploy stage (SF4) calling `loams app deploy`; a Studio "Deploy" action; marketplace packages that declare targets; a Live split-app template | 4–6 engineer-weeks |

## 2. Scope

**In scope:** what an app built by the Factory, Studio or a marketplace author looks like so that it runs on any of the five clouds' serverless free tiers or on Loams; the storage, queue and schedule interfaces and their adapters; deploying into the customer's account; staying within free tiers; reaching a Loams data plane when needed.

**Not in scope:**
- **Loams itself** in the customer's account: §70.
- **Frontend hosting features** (preview deploys, image optimisation, a CDN): the clouds have them, and D170 rules out a Vercel clone. Static assets deploy with the app; nothing more.
- **Long-running servers** (`http-port`, D181): they need a container service, which is §70's container tier or the customer's own.
- **A hosted marketplace or hosted Factory:** neither runs apps for customers.

## 3. Where things are today (`dev` at `f33d5c7d6`, 2026-10-21)

| Piece | State |
|---|---|
| Loams Functions (§24) | Designed (D170–D188, Proposed; D375 Approved). Not built: no runner crates. The manifest is `loams.toml` `[functions]`; contracts `static`, `fetch`, `http-port`; workerd runs Workers presets; `kvNamespace` and `r2Bucket` bindings map onto Live tables and the bucket (§24 §4.2). External runners (Lambda, Cloudflare, Knative) are "for burst capacity and BYOC accounts that want their own cloud bill" (§24 §16) |
| Live server functions | Partly built (`loams-kv` with `embedded`, `loams-live-js`). They run in QuickJS inside the engine; production means TiKV (D692) |
| Loams Jobs (§26) | Designed, not built; TiKV only (Q97) |
| App manifest for the Factory | None. The factory's own package (D-SF-20) has a manifest whose fields are undefined; deploy is stage 6 of `factory.run` (D-SF-13), a Forgejo merge plus an Argo CD rollout (SF4) |
| Studio | No deploy flow; previews are local dev servers (§53 §5.8) |
| Marketplace | `docs/marketplace/publishing.md` is a draft; the manifest schema is not published |
| Meter (§63) | Approved, not built; no external cloud-bill integration |
| `@loams/client` | Built; `transport.fetch` (#947) carries a Workers VPC binding's `fetch` (§69 §7.3) |

## 4. The trade-off: free tier or full Loams (D1390, D1397)

| An app needs | On the free tier | Needs a Loams data plane |
|---|---|---|
| HTTP endpoints, webhooks, an API for a UI | Yes | — |
| Records by key, small lists by prefix, optimistic updates | Yes (`items`) | — |
| Files, uploads, exports | Yes (`blobs`) | — |
| Background work, retries, fan-out | Yes (`queue`) | — |
| Cron jobs | Yes (`schedule`) | — |
| Secrets | Yes (the cloud's own) | — |
| Ad-hoc queries, joins, secondary indexes | Use the cloud's database directly (an escape hatch, §6.6), or Loams Postgres | Loams Postgres or Live |
| **Hybrid retrieval** (BM25 + vectors, Qdrant and ES APIs) | No | Yes |
| **Graph memory and Loams Memory** | No | Yes |
| **Durable execution** (Resonate promises that survive restarts) | No (each cloud's workflow service is an escape hatch, §6.6) | Yes |
| **Reactive queries and transactional mutations** (Live) | No | Yes (D1399) |
| **Loams Postgres** branches, **House** analytics, **Loams Git** | No | Yes |

What it costs:

| | Free-tier app | App + the customer's data plane |
|---|---|---|
| Monthly cost, small | $0 inside the allowances (§7); a few cents for S3 on a new AWS account | $0 for the app plus about $46 (Cloudflare Container) to about $82 (Azure VM) for the data plane (§70 §1.3) |
| Cold start | The cloud's (tens to hundreds of ms) | Same for the app; the data plane is always on |
| Portability | Any target in §7 | Same, plus the data plane's cloud |
| What it gives up | Loams features | Nothing |

The point of the split: **most apps the Factory builds should cost nothing to run**, and only the ones that use Loams' features pay for a data plane, which many apps can then share (D1398).

## 5. The portable app contract (D1391, D1392)

### 5.1 The shape

```ts
// src/index.ts
import type { AppEnv } from "@loams/app";

export default {
  async fetch(request: Request, env: AppEnv, ctx: ExecutionContext): Promise<Response> {
    const note = await env.items.get({ pk: "user#42", sk: "note#1" });
    return Response.json(note?.value ?? null);
  },
  async queue(batch: MessageBatch<CloudEvent>, env: AppEnv, ctx: ExecutionContext) {
    for (const msg of batch.messages) { /* … */ msg.ack(); }
  },
  async scheduled(event: ScheduledEvent, env: AppEnv, ctx: ExecutionContext) {
    await env.queue.send("reports", { type: "dev.example.report.due", data: { at: event.scheduledTime } });
  },
};
```

- **Why this shape:** it is Cloudflare Workers' module format, which workerd already runs (§24, D171), which Hono, Nitro and OpenNext already emit, and which is a thin wrapper over WinterTC's `fetch` (the Minimum Common API's first edition was adopted by Ecma in December 2025: [WinterTC](https://wintertc.org/), [Minimum Common API](https://min-common-api.proposal.wintertc.org/), draft of 2026-07-31). On Cloudflare and on Loams it runs as is; elsewhere a shim adapts each cloud's event to it (§7).
- **`ctx.waitUntil`** is supported everywhere (Lambda and Azure keep the invocation open; Cloud Run's request-based billing allows work until the response, so the shim awaits pending promises before responding there; verify per target in AB0 Task 0).
- **Static assets** under `public/` deploy with the app: Workers static assets, S3 behind the Function URL's handler (not CloudFront in AB0), a GCS bucket, Azure Blob, Vercel's static output.

### 5.2 The manifest

`loams.toml`, the file §24 already names, gains an `[app]` table beside `[functions]`:

```toml
[app]
name = "notes"
main = "dist/index.js"          # exports { fetch, queue, scheduled }
assets = "public"
compatibility_date = "2026-10-01"
memory_mb = 256                 # a hint; each target rounds to what it offers
timeout_s = 30

[app.items.notes]               # one items store, named "notes"
ttl_attribute = "expires_at"    # optional

[app.blobs.uploads]

[app.queues.reports]
max_batch = 10
max_retries = 3
dead_letter = true

[[app.schedules]]
cron = "0 7 * * *"              # UTC
name = "daily-report"

[app.secrets]
names = ["OPENAI_API_KEY"]

[app.loams]                     # optional: this app needs a Loams data plane (§8)
endpoint_env = "LOAMS_ENDPOINT"
namespace = "notes/production"
reach = "access"                # "workers-vpc", "access", "privatelink", "tailnet"

[app.limits]                    # the cost guard's inputs (§9)
requests_per_day = 20000
cpu_ms_per_request = 20
queue_messages_per_day = 2000
items_reads_per_day = 40000
items_writes_per_day = 5000
blob_gb = 1

[app.targets]                   # optional; all targets the ports allow by default
allow = ["cloudflare", "aws", "gcp", "azure", "vercel", "loams"]
```

The manifest declares names and limits only. Secrets never appear in it (as in §24 §5).

### 5.3 Frameworks (D1392)

- **Hono** (runs on Workers, Lambda, Vercel, Node and others; [hono.dev](https://hono.dev), 4.13.13, read 2026-10-21) is the recommended router and is what the Factory's templates use.
- **Nitro** apps use Nitro's own presets for the HTTP layer and `@loams/app` for state. Nitro 3 is in beta (2026-09-03) and dropped some presets that Nitro 2 has (`azure-functions`, `deno-deploy`) ([deploy](https://nitro.build/deploy), read 2026-10-21), so Loams does not depend on it.
- **Next.js** apps use OpenNext's adapters on AWS and Cloudflare and Vercel natively; OpenNext is moving onto Next.js's Deployment Adapters API, stable in Next.js 16.2 ([OpenNext](https://opennext.js.org/), read 2026-10-21). Next.js on Google Cloud and Azure is out of AB scope.
- **Prior art considered:** SST (v4 since 2026-02-24, on Pulumi, AWS- and Cloudflare-first; [docs](https://sst.dev/docs/), read 2026-10-21) shows typed resource links between functions and state; Dapr's components (state, pub/sub, bindings, jobs; [building blocks](https://docs.dapr.io/concepts/building-blocks-concept/), v1.18) show swappable adapters behind small interfaces, but need a sidecar that serverless free tiers cannot run; Ampt (AWS-only; its SDK was last released 2025-04-07, unverified whether still developed) shows a "just write the app" developer experience. `@loams/app` takes SST's typed bindings and Dapr's swappable components, without a sidecar or a hosted control plane.

## 6. The ports (D1393, D1394, D1410)

The ports are the common floor of every target. Each port's limits are the strictest of its adapters, so an app that runs on one target runs on all.

### 6.1 `items`

```ts
interface Items {
  get(key: Key, opts?: { consistent?: boolean }): Promise<Item | null>;
  put(key: Key, value: Json, opts?: { ifVersion?: string; ifAbsent?: boolean; ttl?: number }): Promise<{ version: string }>;
  delete(key: Key, opts?: { ifVersion?: string }): Promise<void>;
  list(pk: string, opts?: { prefix?: string; after?: string; limit?: number }): Promise<{ items: Item[]; next?: string }>;
}
type Key = { pk: string; sk: string };      // each at most 512 bytes
type Item = { key: Key; value: Json; version: string; expiresAt?: number };
```

- **Values** are JSON up to **256 KiB**, below DynamoDB's 400 KB item limit with the key and attributes (verify each target's item limit in AB0 Task 0).
- **Conditional writes** (`ifAbsent`, `ifVersion`) fail with `ConditionFailed`; the version is an opaque string the adapter derives (a numeric attribute on DynamoDB, an update time on Firestore, the `_etag` on Cosmos DB, a version column on D1).
- **`list`** reads one partition in sort-key order, by prefix, paged (at most 100 per page). There is no cross-partition query.
- **Consistency:** a `get` with `consistent: true` reads its own writes on every target; `list` may lag on DynamoDB (eventually consistent query by default, consistent on request). The adapter documents it.
- **TTL** is best effort, like every target's (DynamoDB deletes expired items within days); `get` hides expired items.

### 6.2 `blobs`

`put(key, body, { contentType, ifAbsent })`, `get(key, { range })`, `head`, `delete`, `list(prefix)`, `signedUrl(key, { method, expiresIn })`. Bodies up to 100 MiB in one `put` (a request-body limit on Workers' Free plan, research §2, unverified) and streamed.

### 6.3 `queue`

`send(queue, event | event[], { delaySeconds })`; the `queue` handler receives batches; each message is acked or retried with backoff up to `max_retries`, then dead-lettered.
- **At-least-once** delivery on every target; handlers must be idempotent (the CloudEvent `id` is the key).
- **Messages are CloudEvents** (D270, D1410), JSON, at most **48 KiB**: Azure Queue Storage holds 64 KiB and Functions' queue trigger expects base64, which leaves about 48 KiB (verify). Larger payloads go to `blobs`, with the blob key in the event.
- **Delay** up to 12 hours (Cloud Tasks and SQS differ; the floor is set by the strictest target in AB0 Task 0).

### 6.4 `schedule`

Cron expressions in UTC, at most one invocation per minute, declared in the manifest. Each target's limits apply: Cloudflare Free allows 5 Cron Triggers per account; Cloud Scheduler gives 3 free jobs per billing account (§7). The deploy refuses more than the target's free count (D1400).

### 6.5 `secrets` and `loams`

- `env.secrets.get(name)` or plain `env.NAME`: Worker secrets, Lambda environment from Secrets Manager or Parameter Store at cold start, Secret Manager, Key Vault references, Vercel sensitive variables.
- `env.loams` is a `LoamsClient` from `@loams/client`, configured from `[app.loams]` (§8).

### 6.6 Escape hatches

`env.native.<port>` exposes the underlying client (the D1 binding, the DynamoDB document client, the Firestore client, the Cosmos container, the Upstash client) for what the ports do not cover. An app that uses it is tied to that target, and the deploy marks it so (`targets = ["aws"]`). The same holds for each cloud's workflow service (Step Functions, Cloud Workflows, Durable Functions, Cloudflare Workflows, Vercel Workflow): usable natively, not ported in AB0 (D1408, Q1099).

## 7. Targets and their free tiers (D1395)

Allowances are per month unless marked per day, as read on 2026-10-21. They change; the free-tier table (D1401) is the source the CLI uses.

### 7.1 Cloudflare

| Port | Service | Free plan | Source |
|---|---|---|---|
| Compute | Workers | 100,000 requests a day; 10 ms CPU per invocation; 128 MB | [Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/), 2026-10-02; [limits](https://developers.cloudflare.com/workers/platform/limits/), 2026-10-12 |
| `items` | D1 | Per day: 5M rows read, 100,000 rows written; 5 GB in total | Workers pricing, 2026-10-02 |
| `blobs` | R2 | 10 GB-month, 1M Class A and 10M Class B operations; free egress | [R2 pricing](https://developers.cloudflare.com/r2/pricing/), 2026-10-01 |
| `queue` | Queues | 10,000 operations a day, 24 h retention (now on the Free plan) | [Queues pricing](https://developers.cloudflare.com/queues/platform/pricing/), 2026-04-21 |
| `schedule` | Cron Triggers | 5 per account | Workers pricing, 2026-10-02 |

- **Why D1 for `items`, not KV:** KV's Free plan allows 1,000 writes a day and is eventually consistent; D1 allows 100,000 rows written a day and supports conditional updates in SQL. Durable Objects' SQLite (100,000 requests a day on Free) is the alternative for strongly ordered per-key state and an escape hatch.
- **The guard is built in:** on the Free plan, requests over the limit fail instead of billing. The 10 ms CPU limit is the binding constraint for agents that parse large responses; such apps target Workers Paid ($5 a month) explicitly.

### 7.2 AWS

New accounts since 2025-07-15 get credits ($100, plus up to $100 for activities) and a set of **always-free** offers; the old 12-month offers are legacy ([AWS blog](https://aws.amazon.com/blogs/aws/aws-free-tier-update-new-customers-can-get-started-and-explore-aws-with-up-to-200-in-credits), 2025-07-15; [free-tier plans](https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/free-tier-plans.html), read 2026-10-21).

| Port | Service | Always free | Source |
|---|---|---|---|
| Compute | Lambda, arm64, Node.js | 1M requests and 400,000 GB-s; 100 GB of response streaming | [Lambda pricing](https://aws.amazon.com/lambda/pricing/), [free serverless](https://aws.amazon.com/free/serverless/), read 2026-10-21 |
| HTTP | **Function URL**, not API Gateway | Billed as Lambda invocations; API Gateway's free calls are a 12-month offer that new accounts get only as credits | [API Gateway pricing](https://aws.amazon.com/api-gateway/pricing/), read 2026-10-21 |
| `items` | DynamoDB, **provisioned** capacity | 25 GB, 25 RCU and 25 WCU (Standard table class), per Region and payer account. **On-demand requests are not free** | [DynamoDB pricing](https://aws.amazon.com/dynamodb/pricing/), read 2026-10-21 |
| `blobs` | S3 | **Not free for new accounts** (credits only); $0.023/GB-month | [S3 pricing](https://aws.amazon.com/s3/pricing/), read 2026-10-21 |
| `queue` | SQS | 1M requests | [SQS pricing](https://aws.amazon.com/sqs/pricing/), read 2026-10-21 |
| `schedule` | EventBridge Scheduler | 14M invocations (the page does not say always free or 12-month) | [EventBridge pricing](https://aws.amazon.com/eventbridge/pricing/), read 2026-10-21 |
| Logs | CloudWatch Logs | 5 GB across ingestion, archive and Insights | [CloudWatch pricing](https://aws.amazon.com/cloudwatch/pricing/), read 2026-10-21 |

- **One table per app** in DynamoDB, provisioned at a share of the account's free 25 RCU and 25 WCU with **autoscaling off**, which caps both throughput and cost. Several apps in one account split the 25 units; the CLI keeps a ledger in the account (a tag on each table) and refuses a deploy that would exceed 25 (D1400).
- **S3 is not free on new accounts.** Small apps pay cents (1 GB is $0.023 a month); the CLI says so instead of calling it free. An app that only needs small blobs can keep them in `items` (up to 256 KiB).
- **Step Functions** gives 4,000 standard state transitions always free ([pricing](https://aws.amazon.com/step-functions/pricing/), read 2026-10-21); it is an escape hatch, not a port (§6.6).

### 7.3 Google Cloud

| Port | Service | Free | Source |
|---|---|---|---|
| Compute | Cloud Run, request-based billing, `min-instances: 0` | 2M requests, 180,000 vCPU-s, 360,000 GiB-s, 1 GiB egress in North America, aggregated per billing account | [free features](https://docs.cloud.google.com/free/docs/free-cloud-features), 2026-10-13; [Cloud Run pricing](https://cloud.google.com/run/pricing), read 2026-10-21 |
| `items` | Firestore, **the `(default)` database only** | 1 GiB; per day 50,000 reads, 20,000 writes, 20,000 deletes; 10 GiB egress. Named databases get no free quota; TTL deletes are not free | [Firestore pricing](https://cloud.google.com/firestore/pricing), read 2026-10-21 |
| `blobs` | GCS | 5 GB-month **only in us-east1, us-west1 and us-central1**; 5,000 Class A, 50,000 Class B | Free features, 2026-10-13 |
| `queue` | Cloud Tasks (HTTP push to the Cloud Run service) | 1M billable operations per billing account | [Cloud Tasks pricing](https://cloud.google.com/tasks/pricing), read 2026-10-21 |
| `schedule` | Cloud Scheduler | 3 jobs per **billing account** | [Cloud Scheduler pricing](https://cloud.google.com/scheduler/pricing), read 2026-10-21 |

- Cloud Run functions are now billed as Cloud Run; the target deploys a container with a small Node server that wraps the `fetch` handler (the shim).
- **One project per app** is the default, because Firestore's free database is one per project; apps can share a project's `(default)` database with a collection prefix per app when the customer prefers (Q1100).
- Pub/Sub (10 GiB free) is the alternative for fan-out to several consumers; Cloud Tasks is closer to the port's one-consumer queue with per-task retry.

### 7.4 Azure

| Port | Service | Free | Source |
|---|---|---|---|
| Compute | Azure Functions, **Flex Consumption** | 250,000 executions and 100,000 GB-s per subscription (on-demand meters only) | [Functions pricing](https://azure.microsoft.com/en-us/pricing/details/functions/), read 2026-10-21 |
| `items` | Cosmos DB for NoSQL, **free-tier account** | 1,000 RU/s and 25 GB for the account's lifetime; one per subscription; opt in at creation; provisioned or autoscale throughput, **not serverless** | [Cosmos DB free tier](https://learn.microsoft.com/en-us/azure/cosmos-db/free-tier), 2026-04-27 |
| `blobs` | Blob Storage | 5 GB LRS hot for 12 months on a free account (unverified: seen only on a staging copy of the free-services page) | — |
| `queue` | Queue Storage (Functions queue trigger) | No free allowance found (unverified); costs cents at these volumes | — |
| `schedule` | Timer trigger | Counted as executions | — |

- **Why Flex Consumption, not Consumption:** the Consumption plan's free grant is larger (1M executions, 400,000 GB-s), but Linux Consumption retires on 2028-09-30 and gets no new language versions until then ([Consumption plan](https://learn.microsoft.com/en-us/azure/azure-functions/consumption-plan), 2026-04-09). Windows Consumption is unaffected and is an option (Q1101).
- **One Cosmos DB free-tier account per subscription:** apps share it, one container each, with throughput shared at the database level (a 1,000 RU/s database); the CLI refuses an app that would push the account past 1,000 RU/s or 25 GB.
- Functions need a storage account (for the host), which costs cents a month.

### 7.5 Vercel

| Port | Service | Hobby | Source |
|---|---|---|---|
| Compute | Vercel Functions (Node) | 1M invocations, 4 hours of Active CPU, 360 GB-hours of memory, 1M CDN requests, 100 GB transfer; 300 s maximum duration; pauses for up to 30 days when exceeded | [Hobby](https://vercel.com/docs/plans/hobby), 2026-09-14 |
| `items` | Upstash Redis (Marketplace, free: 256 MB, 500,000 commands a month) or Neon (free: per project 1 GB and 100 CU-hours; suspends after 5 minutes idle) | Vendor pages (undated; Upstash's own page disagrees with itself on the database count) | [Marketplace storage](https://vercel.com/docs/marketplace-storage), 2026-09-17; [Upstash](https://upstash.com/pricing/redis); [Neon](https://neon.com/pricing) |
| `blobs` | Vercel Blob | (verify the Hobby allowance) | — |
| `queue` | Vercel Queues | First 1M operations (4 KiB chunks; idempotent sends count double) | [Queues pricing](https://vercel.com/docs/queues/pricing), 2026-08-12 |
| `schedule` | Vercel Cron | (verify the Hobby count and frequency) | — |

- **Hobby is "non-commercial, personal use only"** ([fair use](https://vercel.com/docs/limits/fair-use-guidelines#commercial-usage), read 2026-10-21). A business deploys to Pro ($20 a month with $20 of usage included, [Pro](https://vercel.com/docs/plans/pro-plan), 2026-09-15), so on Vercel "free tier" means "inside Pro's included usage" for customers and Hobby only for personal projects. The CLI asks which.
- **`items` on Upstash** maps keys to `pk:sk` with a sorted set per partition for `list`; conditional writes use a Lua script (`WATCH`-free). Neon is the alternative when the app also uses SQL natively.
- Vercel Workflows (50,000 events on Hobby, [pricing](https://vercel.com/docs/workflows/pricing), 2026-09-16) is an escape hatch (§6.6).

### 7.6 The free-tier traps, in one list

- AWS: API Gateway and S3 are credits-only on new accounts; DynamoDB is free only in provisioned mode; the 25 units are per account and Region.
- Google Cloud: one free Firestore database per project; GCS free only in three US regions; 3 Cloud Scheduler jobs per billing account.
- Azure: one Cosmos DB free-tier account per subscription, chosen at creation, never serverless; Linux Consumption retiring; the free account's $200 credit lasts 30 days.
- Cloudflare: 10 ms CPU per request on Free; KV's 1,000 writes a day; 5 Cron Triggers per account.
- Vercel: Hobby is non-commercial.

### 7.7 The `loams` target (D1396)

On a Loams data plane, the app runs as Loams Functions (§24, T0 workerd), which already runs the Workers module shape. The ports map onto Loams:

| Port | On Loams |
|---|---|
| `items` | A Live table per store, through §24 §4.2's KV binding (`loams-kv`) |
| `blobs` | The namespace's prefix in the bucket (§24's `r2Bucket` binding) |
| `queue` | A Loams Jobs queue (§26) |
| `schedule` | A Durable schedule (D210) |
| `loams` | In-process access, no edge |

This target depends on §24's F1 and §26's JB1, neither built; AB1 adds it when they land. It is how an app grows out of a free tier without a rewrite.

### 7.8 How a deploy works (D1406)

```
loams app deploy --target aws [--region eu-west-1] [--allow-paid] [--plan-only]
loams app status | usage | logs | destroy
```

1. **Build:** run the app's build (`npm run build` by default), then bundle `main` with the target's entry shim (`@loams/app/shims/<target>`) using esbuild.
2. **Check:** read `[app.limits]`, compare with the free-tier table (D1401) and the account's existing ledger (other apps' shares of DynamoDB capacity, the Cosmos DB account, Cron Triggers); refuse with the numbers if it does not fit, unless `--allow-paid` (D1400).
3. **Render infrastructure:** for AWS, Google Cloud and Azure, a root module in `.loams/app/<target>/` from `@loams/app`'s templates (official providers, state in a bucket in the customer's account that the first deploy creates); for Cloudflare, a `wrangler.jsonc`; for Vercel, `vercel.json` and the Build Output API.
4. **Apply** with the customer's own credentials (D1404), printing what will be created and what it costs beyond the free tier (normally $0).
5. **Caps and budgets** (D1402) are part of the same apply.

## 8. When an app needs Loams (D1397–D1399, D1409)

### 8.1 The binding and the edge

`[app.loams]` makes the deploy configure `@loams/client` for the target's edge path (§69 §8):

| Target | Reach (§69) | Credential now | From ER2 |
|---|---|---|---|
| Cloudflare | Workers VPC binding, passed as `transport.fetch`; or the Access hostname | `lk_` key in Worker secrets | Unchanged (no workload identity on Workers) |
| AWS | Function in the data plane's VPC to the internal NLB, or the Access hostname | `lk_` key from Secrets Manager | `GetWebIdentityToken` → Loams token |
| Google Cloud | Direct VPC egress to the internal load balancer, or the Access hostname | `lk_` key from Secret Manager | Metadata ID token → Loams token |
| Azure | VNet integration to the internal load balancer, or the Access hostname | `lk_` key from Key Vault | Managed identity token (Q1090 of §70) |
| Vercel | Access hostname; Vercel PrivateLink when the data plane is on AWS | `lk_` key in sensitive env vars | Vercel OIDC → Loams token |

Placing an app function in a VPC costs nothing extra on Lambda, but reaching the internet from that VPC then needs NAT, which is not free (§69 §5.3); the Access hostname keeps a free-tier app free.

**The app's own endpoints are public if the app is** (D1409): a Function URL with `AuthType: NONE` or a Worker route is the app's surface, authenticated by the app. The never-public rule (D1311) is about Loams' data plane, which the app reaches only through the edge listener.

### 8.2 Which data plane (D1398)

- **One per app** for isolation, or **one shared** by the customer's apps: each app gets its own namespace and environment and an environment-bound key or trust policy, so a compromised app reaches only its own data (§69's namespace reach).
- The cheapest data plane is §70's Cloudflare phase 0 (about $46 a month) or a VM (about $60–$82).
- **Ostrium does not run a shared data plane** for customers' apps (D1378). A partner or the customer's own platform team may run one; that is their data plane.

### 8.3 Live apps (D1399)

A Live app's queries and mutations are deterministic transactions over Live's store and run in the engine (§20 §6). App BYOC hosts the rest:
- HTTP routes and server actions that call `env.loams.live.mutation(...)` and `query(...)`;
- queue consumers and schedules;
- the frontend's static assets, whose browser clients subscribe to Live through the data plane's edge (a browser path is out of §69's scope today, so subscriptions stay with the console or a server relay; Q1104).

## 9. Cost guards (D1400–D1403)

### 9.1 Free by default (D1400)

- `loams app deploy` fails when the declared `[app.limits]`, multiplied out to a month, exceed any free allowance on the target, and prints each line: "Firestore writes: 5,000/day declared, 20,000/day free: OK"; "Cloud Scheduler jobs: 4 declared, 3 free per billing account: **over**".
- `--allow-paid` deploys anyway and prints the estimated monthly cost beyond the free tier.
- Undeclared limits default to conservative values (1,000 requests a day, estimate), so an app without a `[app.limits]` table still gets checked.

### 9.2 The free-tier table (D1401)

- `docs/guides/app-byoc/free-tiers.md` is the human source: one row per allowance, with its link and the date read.
- `scripts/app-byoc/gen-free-tiers` turns it into `free-tiers.json`, which ships in `@loams/app`, so the CLI works offline.
- A nightly job fetches each source page and fails when a number in the table no longer appears there, so the table cannot silently rot.

### 9.3 Caps at deploy time (D1402)

| Target | Hard cap | Budget and action |
|---|---|---|
| Cloudflare Free | The plan stops at its limits | None needed |
| AWS | DynamoDB provisioned capacity with autoscaling off; Lambda reserved concurrency (default 5); SQS has no cap | AWS Budgets cost budget (default $1) with an alert; an optional budget action that attaches a deny policy to the app's role (Q1102) |
| Google Cloud | Cloud Run `max-instances` (default 2); Firestore has no cap | A Cloud Billing budget with alerts; an optional Pub/Sub-triggered breaker that sets `max-instances` to 0 |
| Azure | Flex Consumption `maximumInstanceCount` (default 2); Cosmos DB's provisioned RU/s | A Cost Management budget with an action group; an optional breaker that stops the function app |
| Vercel Hobby | The plan pauses | — |
| Vercel Pro | Spend management's on-demand amount (default $200, set lower by the CLI if asked) | Vercel's own notifications |

### 9.4 Usage warnings and the Meter (D1403)

- **The usage poller** reads each target's own figures once a day: CloudWatch metrics (`Invocations`, `ConsumedReadCapacityUnits`, SQS `NumberOfMessagesSent`), Cloud Monitoring, Azure Monitor metrics, Cloudflare's GraphQL Analytics API, Vercel's usage API. It runs as `loams app usage` from the developer's machine, or as a scheduled function inside the app's own account (one more cron, counted against the allowance).
- **Warnings** at 80% and 100% of each allowance, by email or a webhook the customer names.
- **With a Loams data plane**, the poller also sends one usage event per metric and day (`io.loams.dev.usage.app.v1`, with `target`, `metric`, `quantity`, `period`) to the customer's own Meter (§63). The allowances are loaded as entitlements (`soft_limit` at the allowance), so the Meter's thresholds (D1129), alerts and console pages apply without new code. **These are the customer's own cloud bills**: nothing reaches Ostrium, and the events never bill anyone; they inform the customer.
- **Gate at deploy** (with a data plane): the deploy calls `Gate::check` for the app's projected usage, so an org that sets a spend policy in its Meter refuses deploys that would exceed it (D1125's call pattern, used for planning only).

## 10. Deploy credentials (D1404)

| Who deploys | AWS | Google Cloud | Azure | Cloudflare | Vercel |
|---|---|---|---|---|---|
| A developer (`loams app deploy`) | Their AWS CLI session (SSO) | `gcloud` application default credentials | `az login` | `wrangler login` | `vercel login` |
| The Factory's deploy stage, CI | A role trusting the Loams workload issuer (D1158) or GitHub OIDC, `AssumeRoleWithWebIdentity` | A workload identity pool provider for the same issuer | A federated credential on an app registration | A scoped API token in Loams Secrets | A team token in Loams Secrets |

No credential is written into the app, the manifest or the repository. The roles the deploy uses are scoped to the resources tagged with the app's name.

## 11. The Factory, Studio and the marketplace (AB2)

- **Software Factory:** stage 6 of `factory.run` (D-SF-13) gains an `app-byoc` mechanism beside the Argo CD rollout: when the repository has `loams.toml` `[app]`, the stage runs `loams app deploy --target <the environment's target>` with federated credentials (D1404) and records the deploy's outputs (URL, version) on the run.
- **Studio:** a "Deploy" action on a project with `[app]` that shows the target picker, the free-tier check (§9.1) and the plan, then deploys from the session's worktree.
- **Marketplace:** a package's manifest declares `[app.targets]`; the install screen says "runs free on Cloudflare, AWS, Google Cloud, Azure" or "needs a Loams data plane", from §4's table.
- **Factory templates** start from `web/examples/app-byoc-notes` and a Mastra agent template with `[app.loams]`.

## 12. Conformance (D1407)

- **The ports suite** (`sdks/typescript/packages/app/conformance/`): the same cases for every adapter (conditional writes, prefix pagination, TTL hiding, blob ranges and signed URLs, queue retries and dead-lettering, idempotent redelivery, cron firing).
- **On PRs, locally:** Miniflare/workerd for Cloudflare; DynamoDB Local and ElasticMQ for AWS (or floci, which the M2 DynamoDB metastore jobs use); the Firestore emulator and a Cloud Tasks emulator; Azurite and the Cosmos DB emulator; Upstash's local Redis for Vercel.
- **Nightly, on real free tiers:** a Cloudflare Free account and the owner's AWS account (§70 §12.2) in AB0; Google Cloud, Azure and a Vercel team in AB1 once their accounts exist (owner actions). Each run deploys the example app, drives the suite through it, reads the usage poller's figures and destroys everything. These runs stay inside the free allowances (estimate: a few thousand requests a night), so they cost nothing beyond S3's cents.

## 13. Phases (D1405)

### 13.1 AB0: Cloudflare and AWS

Cloudflare and AWS go first because:
- Cloudflare's Free plan stops at its limits, so a mistake cannot bill anyone, and the contract is its native module format;
- AWS has the owner's test account, and Lambda, DynamoDB (provisioned) and SQS are always free;
- together they cover the owner's example (Lambda plus DynamoDB) and the two clouds whose startup programmes BY0 targets first (§70 §13.5).

Content: the manifest's `[app]` table; `@loams/app` (ports, types, the Cloudflare and AWS adapters and shims, the conformance suite); `loams app deploy|status|usage|logs|destroy` for `cloudflare` and `aws`; the free-tier table and generator; the refusal; caps and AWS Budgets; the example app; the nightly. **5–7 engineer-weeks** (estimate). Gate: none beyond the CLI (CL1); it does not need a data plane.

### 13.2 AB1: Google Cloud, Azure, Vercel, the Meter, the `loams` target

**7–9 engineer-weeks** (estimate). Gates: test accounts for Google Cloud, Azure and Vercel (owner actions); §63's ME1 for the Meter link; §24 F1 and §26 JB1 for the `loams` target; §69 ER0 for `[app.loams]` over the edge (ER2 for workload identity).

### 13.3 AB2: the Factory, Studio and the marketplace

**4–6 engineer-weeks** (estimate). Gates: SF4's deploy stage; Studio's project model; the marketplace manifest schema.

## 14. What not to build (D1408)

- **A framework.** Hono, Nitro and OpenNext exist.
- **A hosting service.** Every app runs in the customer's account.
- **Free-tier imitations of Loams features** (vector search in D1, graph memory in DynamoDB). Apps that need them use a data plane (D1397).
- **Multi-item transactions, secondary indexes or queries in `items`.** The escape hatch exists (§6.6).
- **A portable workflow port in AB0.** Each cloud's workflow service differs in model; Loams Durable is the portable one (Q1099).
- **A Dapr sidecar.** Serverless free tiers cannot run one.
- **Deploying into Ostrium's accounts.**

## 15. Relation to earlier decisions

| Decision | Effect |
|---|---|
| **D170** ("not a Vercel clone") | **Kept.** App BYOC deploys to the customer's own clouds; Loams hosts nothing and adds no frontend features |
| **D171, D181** (workerd and the Workers presets; the `fetch` contract) | **Extended:** the app contract is `fetch` plus `queue` and `scheduled` (D1391); the `loams` target runs it on workerd |
| **D177** (`loams.toml` `[functions]`) | **Extended** with `[app]` in the same file |
| **D375** (the `Runner` trait; external runners for BYOC accounts) | **Consistent:** app BYOC is §24 §16's "BYOC accounts that want their own cloud bill", done as deploys the customer owns rather than runners Loams drives |
| **Q97** (Jobs: no Postgres or DynamoDB backend) | **Not affected:** the `queue` port is not Loams Jobs; on the `loams` target it uses Jobs as built |
| **D692** (production Live means TiKV) | **Kept:** Live stays in the engine (D1399) |
| **D1125, D1128, D1129** (Gate, policy, thresholds) | **Used** for the customer's own cloud usage (D1403); no Ostrium billing |
| **D1308, D1311, D1315** (edge reach) | **Used** by `[app.loams]` (§8) |
| **D1158** (the Loams workload issuer) | **Used** for the Factory's deploy credentials (D1404) |
| **D270** (CloudEvents on streams) | **Used** for queue messages (D1410) |

## 16. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A cloud changes or removes a free allowance (AWS did in July 2025) | High over a year | Medium | The dated table and its nightly check (D1401); the refusal uses the current table |
| The ports' common floor is too small for real apps | Medium | Medium | Escape hatches (§6.6); the `loams` target for growth |
| A customer gets a bill despite the guards (S3, Queue Storage, NAT, logs) | Medium | Low | The deploy names every non-free line; budgets on by default; caps where they exist |
| Vercel Hobby's non-commercial term is ignored | Medium | Medium | The CLI asks; Pro is the default for teams |
| Shims diverge in edge cases (`waitUntil`, streaming, headers) | Medium | Medium | The conformance suite runs the same app on every target |
| One account's shared free capacity (DynamoDB 25 units, one Cosmos DB account, 3 Scheduler jobs) runs out across apps | Medium | Low | The per-account ledger and the refusal (§7.2, §7.4) |
| The Meter link is mistaken for billing | Low | Medium | The events say what they are, go only to the customer's Meter, and bill nobody |

## 17. Open questions (Q1095–Q1109), answered

The owner answered every question with its default on 2026-10-21.

| # | Question | Default (answered 2026-10-21 by the owner) |
|---|---|---|
| Q1095 | Is the owner's AWS test account on the Free plan or the Paid plan? | **Move it to the Paid plan** before relying on it for the nightly (the Free plan closes after six months and offers only some services); the credits still apply. Owner check |
| Q1096 | Python apps in AB0? | No: TypeScript and JavaScript in AB0; Python handlers (Lambda, Cloud Run, Azure Functions, Vercel; Workers' Python is beta) in AB1 |
| Q1097 | `items` on Cloudflare: D1 or a Durable Object per partition? | **D1** (larger free write allowance, SQL for conditional writes); DOs as an escape hatch |
| Q1098 | `items` on Vercel: Upstash Redis or Neon? | **Upstash Redis** for `items`; Neon for apps that use SQL natively |
| Q1099 | Add a `steps` (workflow) port later? | Not before AB2; revisit when three Factory templates need one, and then consider Loams Durable as its `loams` adapter |
| Q1100 | Google Cloud: one project per app, or apps sharing a project's free Firestore database? | **One project per app** by default; sharing with a collection prefix on request |
| Q1101 | Azure: Flex Consumption or Windows Consumption? | **Flex Consumption** (Linux, current); Windows Consumption on request for its larger grant |
| Q1102 | Turn on the budget breakers (deny policy, `max-instances` 0) by default? | **Off by default, offered at deploy**: a breaker stops a production app; the alert is on by default |
| Q1103 | Where does the usage poller run by default? | `loams app usage` on demand; a scheduled function in the app's account when the user opts in |
| Q1104 | Browser subscriptions to Live from an app-BYOC frontend? | Out of AB scope; needs a browser edge path that §69 excludes today |
| Q1105 | Default region per target? | The cheapest free-tier region near the user: `us-east-1` (AWS), `us-central1` (Google Cloud, a free GCS region), `eastus` (Azure); Cloudflare and Vercel are global |
| Q1106 | Default Lambda architecture? | **arm64** (cheaper per GB-s beyond the free tier; same free allowance) |
| Q1107 | Where does `@loams/app` live? | `sdks/typescript/packages/app/` in this repository, published as `@loams/app` with the SDK's release workflow |
| Q1108 | Generated OpenTofu for apps, or the clouds' own CLIs? | **OpenTofu** for AWS, Google Cloud and Azure (one engine, reviewable plans, the same as §70); `wrangler` and the Vercel CLI for those two |
| Q1109 | Does the Factory default to app BYOC or to the customer's data plane for a new app? | **App BYOC on the environment's target** unless the app's manifest has `[app.loams]` and the data plane is the only target it allows |

## 18. Sources

Repository (read 2026-10-21): §20 §6, §24 (§4, §9, §16), §26, §39 §10, §45, §53 §5.8, §63 §10–§15, §68 §1.2, §69, `docs/marketplace/publishing.md`, `docs/ecosystem/deploy-buttons.md`.

Vendor pages: linked inline above, read 2026-10-21.
