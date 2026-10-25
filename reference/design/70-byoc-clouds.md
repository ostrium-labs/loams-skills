# 70 — BYOC on AWS, Google Cloud, Azure, Cloudflare and Vercel

> **Amendment (approved 2026-10-23) by [§78](78-loams-cloud-on-tidb-x.md)** (Loams Cloud on TiDB X; D1700–D1735). D1351 and D1378 stand for BYOC; D1701 clarifies their wording, since Loams Cloud is a free try-out control plane and dev-state store, not a data plane. `tidbx` joins the target list as a **state** target, and the deploy order becomes TiDB X → Cloudflare → Vercel → AWS → Google Cloud → Azure, with AWS, Google Cloud and Azure as the production targets (D1706). Target manifests reference this document's tiers instead of repeating them (D1707).

> **Amendment (approved 2026-10-23) by [§79](79-void-and-vite-plus.md)** (Void and Vite+; D1770–D1791 approved by the owner). The console in every BYOC tier is built with Vite+ (D1772). Data-plane BYOC is otherwise unchanged: Void deploys only to Cloudflare and never deploys the Loams data plane.

Status: **Approved** · 2026-10-21. The owner approved the design with every default on 2026-10-21, after approving the eight Cloudflare defaults of the Cloudflare research earlier the same day (D1366–D1371, the Cloudflare part of D1354, Q1086, Q1087) and making an AWS account with $200 of credits for six months available for BY0's tests and the nightly AWS matrix (§12.2).

**Amended 2026-10-21 (D1580–D1582).** The owner removed the services offering (D1323), and three rulings followed for this design: **no cloud marketplace listings for now** (D1580, amends D1374 and Q1091's answer; BYOC installs only from Ostrium's own artefacts); **the support bundle stays a local file** the user may attach to a GitHub issue or share on Discord (D1581, amends D1376); and **the support grant becomes a maintainer debug session**: opt-in, time-boxed, revocable, community and best effort, with no contract or SLA (D1582, amends D1378). The text below is updated to match; the amended rows keep their original wording with a note.

Source: on 2026-10-21 the owner decided to **"fully embrace BYOC"** and is working on BYOC for **Cloudflare, Vercel, AWS, GCP and Azure**. A second requirement the same day: **app BYOC must map onto each cloud's free tiers** (for example AWS Lambda and DynamoDB). This document designs **data-plane BYOC**: Loams itself (the engine and its stack) installed into a customer's own account on each cloud. **App BYOC**, the apps and agents that the Software Factory, Studio and the agents marketplace build and deploy into a customer's account on serverless free tiers, is designed in [§71](71-app-byoc-free-tiers.md). The two share §4's principles and one CLI (§9.3).

Decisions **D1350–D1384** (D1385–D1389 kept free) and open questions **Q1080–Q1094** are declared in the [decision log](13-decision-log.md). §71 uses D1390–D1410 (D1411–D1419 kept free) and Q1095–Q1109.

**Numbering.** §69 reserved D1300–D1339 and Q1060–Q1079. PR #973 (the light default compose) uses D1340–D1343. This document and §71 take **D1350–D1419** and **Q1080–Q1109**, leaving D1344–D1349 free as a buffer, and take the numbers 70 and 71.

Plan: [BY1](../plans/2026-10-21-by1-byoc-clouds.md) (a stub; BY0 is specified task by task).

This document builds on the designs below; §15 says where it amends each one.
- [§41](41-multitenant-byoc-control-plane.md) and plan [MT4](../plans/2026-10-02-mt4-byoc-control-plane.md): the BYOC control plane, the outbound-only BYOC agent (D543), the tailnet mode (D587), release channels and rings (D545).
- [§18](18-metastore-backends-and-router.md) §8: D64's two BYOC modes (managed-meta, local-meta) and the data boundary; D58's Postgres and DynamoDB metastore backends.
- [§25](25-clever-cloud-stack.md) and the [MT3](../plans/2026-10-02-mt3-gitops-clever.md) plan: the CKE and Clever Cloud stack, the first managed-Kubernetes target.
- [§54](54-upgrades.md): `loams upgrade` and its snapshots; `deploy/release/byoc/upgrade.sh`.
- [§64](64-loams-infra-stack.md): the OpenTofu provider `ostrium-labs/loams` on `loams-tfplugin` (D1151), Loams Secrets and `SecretBackend` (D1154), workload trust (D1158); and D96's `KeyProvider` for customer-managed keys.
- [§65](65-loams-operator.md): `loams-operator`, `LoamsCluster`, `ObjectStore` and its capability probe (D1190), backups (D1188), exposure (D1195), `Diagnose` (D1192), and LOC.
- [§67](67-gpu-scheduling-runai.md): GPU node pools on Kubernetes, `compose.gpu.yaml` on a VM.
- [§69](69-edge-reach.md): reach classes (D1301), one edge door (D1302), Cloudflare Tunnel (D1305), cloud private links (D1315), workload-identity exchange (D1308), and part A of the never-public rule (D1311).
- [§66](66-loams-mastra.md), [§39](39-software-factory-and-loams-bot.md), [§53](53-studio-ade.md): Mastra, the Software Factory and Studio, which run on a BYOC data plane (§11).
- PR #973 (`compose-light-default`, open): the light default stack (D1340–D1343).
- The platform designs imported under D1119: [platform 03](../platform/design/03-cloudflare-target-and-credits.md) (PD20, PD24) and [platform 04](../platform/design/04-hosted-loams-cloud-on-cloudflare.md) (PD29, PD33, PD34, PD36).
- The Cloudflare research of 2026-10-21 (`pitch/research/cloudflare-byoc.md`, outside this repository), whose recommendations §7.4 records as decisions (approved by the owner on 2026-10-21), and the partner research of 2026-10-20 (`pitch/research/{aws,google,microsoft,vercel}.md`).

Markers, as in §64–§69:
- **(verify)** means the claim has not been checked against a primary source. The task that depends on it checks it first.
- **(estimate)** means the number was judged, not measured.
- **(unverified)** means a search on 2026-10-21 did not settle it.

Vendor facts were read on 2026-10-21 from the page linked beside each one. A date after a link is the page's own last-updated or published date; "read 2026-10-21" means the page shows none. Prices are on-demand list prices in one US region and exclude tax, support plans and credits.

---

## 1. Summary

### 1.1 The problem

Loams is self-hosted today in one way: the compose stack on a machine the user chooses, reached over the user's tailnet (`deploy/release/byoc/install.sh` over SSH). That is BYOC in the literal sense, and it works on any cloud's VM. But a customer who says "deploy it into our AWS account" means more:
- the bucket, the keys and the secrets are the cloud's own services in their account (S3 and KMS, not a folder and a file);
- the machine has a cloud identity, so no static keys sit on disk;
- private reach from their other workloads is the cloud's own private networking;
- the install is a command or a module in their existing IaC;
- someone has tested it on that cloud.

None of that is written down per cloud. Five clouds differ in what they can host: AWS, Google Cloud and Azure have VMs, container services and Kubernetes; Cloudflare has Containers with no persistent disk behind Durable Objects; Vercel has no place for a stateful engine at all.

### 1.2 The answer

- **One artefact set, three install tiers per cloud** (D1352): the release images, the compose bundle, the operator and its chart, and OpenTofu modules. The tiers are a **VM** (fastest; the light compose stack), a **container service** (no machine to patch), and **Kubernetes** (the operator; scale and HA).
- **The light stack by default, TiKV for scale** (PR #973, D1340–D1343): one VM with a block disk runs the whole platform at about 1 GB of idle memory. The `tikv` profile, or the operator on Kubernetes, is the path to several nodes.
- **A persistence rule** (D1353): the embedded stores need a block volume with `fsync` and a single writer. Hosts without one (Cloudflare Containers, ECS Fargate, Cloud Run, Container Apps) run a **checkpointed single node** (D1354) that snapshots its metadata to the bucket and states its recovery window, until a managed metastore removes the window (D1355).
- **The cloud's own services, in the customer's account** (D1356–D1358): the bucket through the host's cloud identity, the bucket's default encryption under a customer-managed KMS key, the install's secrets in the cloud's secret store.
- **Never public, as amended** (D1359): no Loams port on the internet. Operators and the console use the tailnet; workloads in the same cloud use its private link to the gateway's edge listener; others use Cloudflare Tunnel (§69).
- **Ostrium runs no data plane** (D1351, D1378): an install needs no connection to Ostrium. The BYOC agent and control plane of §41 are opt-in.
- **Per cloud:**
  - **AWS:** EC2 now; ECS Fargate only as a checkpointed node; EKS with the operator; S3, KMS, Secrets Manager; PrivateLink.
  - **Google Cloud:** Compute Engine now; Cloud Run only as a checkpointed node; GKE; GCS, Cloud KMS, Secret Manager; Private Service Connect.
  - **Azure:** a VM now; Container Apps only as a checkpointed node; AKS; Blob Storage, Key Vault; Private Link.
  - **Cloudflare:** a "Deploy to Cloudflare" template now: a front Worker and the console, a Durable Object owning one Container with the light image, R2 as the bucket, metadata checkpointed to R2. Durable Objects take over the metastore and Durable in phase 1; Workers for Platforms carries the multi-tenant Factory and marketplace in phase 2. A Cloudflare-native single-node profile amends PD29 and PD34 for this target only (§7.4).
  - **Vercel:** **no data plane on Vercel.** BYOC on Vercel means the customer's Loams in one of the clouds above; the Factory's and Studio's apps and agents on the customer's Vercel team; a Vercel Marketplace connectable-account integration; and edge reach with Vercel OIDC (§8).
- **Distribution** (§10, D1580): only Ostrium's own artefacts (GHCR images, the compose bundle, the Helm chart and operator, the OpenTofu modules); no cloud marketplace listings.
- **Shared pieces** (§9): OpenTofu modules per cloud and tier, the operator on Kubernetes, a `loams byoc deploy --cloud <c>` flow, upgrades and backups into the customer's bucket, observability into the customer's monitoring, and a local support bundle.
- **Conformance** (§12): a BYOC matrix per cloud and tier, run nightly on Ostrium's cloud test accounts with credits, with hard cost guards.

### 1.3 Per-cloud summary

| | AWS | Google Cloud | Azure | Cloudflare | Vercel |
|---|---|---|---|---|---|
| **VM tier (BY0)** | EC2, light compose | Compute Engine, light compose | VM, light compose | — (no VMs) | — |
| **Container tier (BY1)** | ECS Fargate, checkpointed node | Cloud Run worker pool, checkpointed node | Container Apps, checkpointed node | Containers behind a Durable Object, checkpointed node (**BY0**) | — |
| **Kubernetes tier (BY2)** | EKS + operator | GKE + operator | AKS + operator | — (no Kubernetes) | — |
| **Bucket** | S3 | GCS | Blob Storage | R2 | Data plane's cloud |
| **Keys** | KMS | Cloud KMS | Key Vault | R2's own encryption at rest (no customer KMS) | Data plane's cloud |
| **Secrets** | Secrets Manager | Secret Manager | Key Vault secrets | Worker secrets, Secrets Store | Vercel sensitive env vars (for the app side) |
| **Private reach** | PrivateLink to an internal NLB | Private Service Connect to an internal LB | Private Link service to an internal LB | Workers VPC, service bindings; Tunnel | Vercel PrivateLink to AWS; Tunnel + Access |
| **Identity → Loams token (ER2)** | `GetWebIdentityToken` | Metadata ID token | Managed identity token | — (API keys in Worker secrets) | Vercel OIDC |
| **Distribution** (D1580) | Ostrium's artefacts: GHCR images, compose bundle, Helm chart and operator, Tofu modules; no Marketplace listing | Same | Same | Deploy to Cloudflare button | Connectable-account integration |
| **Install UX** | `install.sh` / cloud-init (BY0); Tofu module (BY1); Helm chart and operator (BY2) | Same | Same | Deploy button, `wrangler deploy` | `vercel integration add`, env vars |
| **Small install, per month (estimate)** | about $73 | about $60 | about $82 | about $46–$180 | Vercel plan + the data plane's cloud |

Costs are in §6 and §7 per cloud. "Small install" is one 2 vCPU, 8 GiB node with a 50 GB disk, 50 GB in the bucket and a handful of keys and secrets.

### 1.4 Decisions

| # | Decision | § | Status |
|---|---|---|---|
| D1350 | **BYOC is the deployment model** (owner, 2026-10-21: "fully embrace BYOC"), in two tiers: **data-plane BYOC** (Loams itself in the customer's account; this document) and **app BYOC** (apps and agents built with Loams, on serverless free tiers; §71) | 1 | Approved (owner, 2026-10-21) |
| D1351 | **Five principles:** the customer's account, data and keys; Ostrium runs no data plane; one artefact set; the light stack by default with TiKV for scale; the never-public rule as amended by D1311 | 4 | Approved (owner, 2026-10-21) |
| D1352 | **Three install tiers per cloud, one artefact set:** VM (light compose), container service, Kubernetes (operator and chart); the same images, digests and release sets everywhere; nothing cloud-specific is compiled into an image | 5 | Approved (owner, 2026-10-21) |
| D1353 | **The persistence rule:** the embedded stores (openraft + redb, Live on redb, Durable on SQLite, Graph's local store) run only on a block volume with `fsync` and one writer; never on a network file system (EFS, Azure Files, Filestore, Cloud Storage FUSE, R2 over FUSE) | 5.2 | Approved (owner, 2026-10-21) |
| D1354 | **The checkpointed single node** for hosts with no persistent block disk: the embedded stores on ephemeral disk, a snapshot to `_meta/checkpoints/` in the bucket every 5 s and on `SIGTERM`, restored at boot, fenced by a create-only epoch object, with a self-fence; one instance at most; the recovery window is stated and the profile is labelled preview | 5.3 | Approved (owner, 2026-10-21) |
| D1355 | **Managed metastores remove the window:** when `loams-meta-dynamodb` and `loams-meta-postgres` (D58) ship, the container tier uses DynamoDB on AWS, and Postgres (Cloud SQL, Azure Database for PostgreSQL) on Google Cloud and Azure; Cloudflare's equivalent is the Durable Object metastore (D1368) | 5.4 | Approved (owner, 2026-10-21) |
| D1356 | **The bucket is the customer's and is reached with the host's cloud identity** (instance profile or EKS Pod Identity, the VM's or GKE's service account, a managed identity); no static keys on a cloud host; static keys only for R2 and off-cloud hosts; the store's capability probe runs at install and refuses a store without conditional writes | 6 | Approved (owner, 2026-10-21) |
| D1357 | **Keys:** until `KeyProvider` (D96) is built, the bucket's default encryption uses a customer-managed key (SSE-KMS with an S3 Bucket Key, GCS default CMEK, a storage-account CMK in Key Vault); per-namespace envelope keys arrive with `KeyProvider`; no Ostrium principal is ever granted use of a customer key | 6 | Approved (owner, 2026-10-21) |
| D1358 | **Secrets:** the install's secrets (tailnet key, setup token, API key hashes, link keys, R2 keys) live in the cloud's secret store and are fetched at boot by the host's identity into `secrets/` with mode 0600 (`loams-byoc-secrets`); `SecretBackend` (D1154) replaces the fetch when it lands | 6 | Approved (owner, 2026-10-21) |
| D1359 | **Networking:** no Loams port is reachable from the internet on any tier. The host needs only outbound access. Operators reach it over the tailnet (default) or the cloud's private shell (SSM Session Manager, IAP, Azure Bastion); workloads reach the gateway's edge listener over the cloud's private link or Cloudflare Tunnel (D1311, D1315) | 6 | Approved (owner, 2026-10-21) |
| D1360 | **Identity:** a same-cloud workload proves itself with its cloud identity and exchanges it for a 15-minute Loams token (D1308, ER2); until ER2, environment-bound `lk_` keys from the cloud's secret store | 6 | Approved (owner, 2026-10-21) |
| D1361 | **Install UX:** `loams byoc deploy --cloud <aws\|gcp\|azure\|cloudflare> [--tier vm\|container\|k8s]`, beside D849's `loams byoc enrol\|list\|revoke`; it wraps the OpenTofu modules (BY1) or `wrangler` (Cloudflare); BY0 is `install.sh` plus cloud-init and per-cloud guides | 9.3 | Approved (owner, 2026-10-21) |
| D1362 | **OpenTofu modules** in `deploy/tofu/modules/<cloud>-<tier>/`, on the clouds' official providers, published from a mirror repository to the OpenTofu registry as `ostrium-labs/loams-byoc/<cloud>`; state stays the customer's; they compose with the `ostrium-labs/loams` provider (D1151) once it exists | 9.1 | Approved (owner, 2026-10-21) |
| D1363 | **The Kubernetes tier is OP1's operator and chart** on EKS, GKE and AKS: block-storage classes for quorum members, workload identity for `ObjectStore`, internal load balancers or the tailnet for exposure; a per-cloud table of annotations and storage classes, no per-cloud controller code | 9.2 | Approved (owner, 2026-10-21) |
| D1364 | **Backups go to the customer's bucket:** bucket versioning plus a metastore snapshot (§65 §8.9); `loams backup create\|list\|restore` for compose and checkpointed installs (BY1); BY0 documents the cloud's disk snapshots; a restore always goes into a new install | 9.4 | Approved (owner, 2026-10-21) |
| D1365 | **Upgrades:** `loams upgrade` on the host (§54), driven by `byoc/upgrade.sh` over SSH or by the cloud's run-command (SSM Run Command, `gcloud compute ssh --tunnel-through-iap`, `az vm run-command`); the operator's release sets on Kubernetes; a Cloudflare install redeploys its Worker and image | 9.5 | Approved (owner, 2026-10-21) |
| D1366 | **A Cloudflare-native single-node and agent profile** (amends PD29 and PD34 for the Cloudflare BYOC target only): the engine and House run in Containers behind Durable Objects with R2 as the bucket; the k3s "Data BYOC" path of PD36 stays the path to scale | 7.4 | **Approved** (owner, 2026-10-21) |
| D1367 | **Cloudflare phase 0** (BY0): `deploy/cloudflare/` with a "Deploy to Cloudflare" button and `wrangler deploy`: a front Worker with the console's static assets, a `LoamsNode` Durable Object owning one Container on the light image, a second for House if its spike passes, R2 bound as the bucket, metadata checkpointed to R2 (D1354); single node, preview | 7.4 | **Approved** (owner, 2026-10-21) |
| D1368 | **On the Cloudflare target, Durable Object SQLite holds truth for the metastore and Durable** in phase 1 (`loams-meta-do`, a Resonate store on DO SQLite). Amends D382 for those two only; Loams Git keeps R2 as its truth and D382 stands for it | 7.4 | **Approved** (owner, 2026-10-21) |
| D1369 | **Git on Cloudflare:** R2 is Loams Git's truth, with a sequencer Durable Object per repository (D391) and smart HTTP on a Worker in phase 1; Cloudflare Artifacts is a mirror and a per-agent workspace host (D562), never the truth | 7.4 | **Approved** (owner, 2026-10-21) |
| D1370 | **Jobs on Cloudflare:** Loams Jobs (§26) gets its Cloudflare backend first, on Workflows and Queues with payloads on R2; the TiKV backend follows for self-hosting | 7.4 | **Approved** (owner, 2026-10-21) |
| D1371 | **Loams Postgres stays out of Cloudflare BYOC:** Hyperdrive to an external Postgres, or a data plane in another cloud joined by Tunnel (PD36), until an experiment shows safekeepers durable on Containers | 7.4 | **Approved** (owner, 2026-10-21) |
| D1372 | **On Cloudflare, the front Worker is the edge door:** it authenticates every request before the Durable Object (environment-bound `lk_` keys on the edge allow-list of D1322; the console and admin routes only behind a Cloudflare Access application); no Container port is reachable otherwise. That is a *brokered* path under D1301 and D1311 | 7.4 | Approved (owner, 2026-10-21) |
| D1373 | **Vercel hosts no data plane.** BYOC on Vercel is: the customer's Loams on AWS, Google Cloud, Azure or Cloudflare; apps and agents on the customer's Vercel team (§71); a connectable-account Marketplace integration that writes `LOAMS_ENDPOINT` and credentials into a project; reach over Access or Vercel PrivateLink, and Vercel OIDC exchange from ER2 | 8 | Approved (owner, 2026-10-21) |
| D1374 | **Marketplace listings deploy the same artefacts** into the customer's account and the product is free in every one of them; where a marketplace requires a paid version (AWS), the paid plan is the same artefacts with Ostrium's support; services are otherwise sold as private offers; no licence-manager or metering hooks in any image | 10 | Approved (owner, 2026-10-21); **amended by D1580** (owner, 2026-10-21: no marketplace listings, no support plan, no private offers) |
| D1375 | **Observability stays in the customer's account:** OTLP and Prometheus to the stack's own collector or the cloud's monitoring (CloudWatch, Cloud Monitoring, Azure Monitor); nothing is sent to Ostrium by default | 9.6 | Approved (owner, 2026-10-21) |
| D1376 | **`loams support bundle`:** versions, health, conditions, redacted configuration and recent logs into a local archive the customer reviews and sends; no data, no secrets; the same content as the BYOC agent's `Report` (D543) and the operator's `Diagnose` (D1192) | 9.7 | Approved (owner, 2026-10-21); **amended by D1581** (owner, 2026-10-21: a local file the user may share; no Ostrium support address) |
| D1377 | **The BYOC agent and control plane (§41) are opt-in.** An install works with no connection to Ostrium; `console.loams.dev` reaches it over the customer's tailnet as today | 9.8 | Approved (owner, 2026-10-21) |
| D1378 | **Ostrium hosts no customer data plane** on any cloud, including a "managed BYOC" in Ostrium's own account; support access is a time-boxed grant the customer opens (D587) | 4 | Approved (owner, 2026-10-21); **amended by D1582** (owner, 2026-10-21: the grant is a maintainer debug session, community and best effort, no contract or SLA) |
| D1379 | **The platform products run on the BYOC data plane:** Studio (remote agentd), the Software Factory package (D-SF-20) and marketplace packages install onto a BYOC install (compose profiles on a VM, the chart on Kubernetes). The apps and agents they build are app BYOC (§71) | 11 | Approved (owner, 2026-10-21) |
| D1380 | **A BYOC conformance matrix** per cloud and tier, run nightly on Ostrium's test accounts, with hard cost guards: GitHub OIDC into a CI role (no long-lived keys), tags, a TTL reaper, budgets with alerts and a deny action, the smallest sizes, teardown in `always()`. AWS first, in the owner's credit account within about $30 a month; Google Cloud and Azure when their accounts exist (owner actions) | 12 | Approved (owner, 2026-10-21) |
| D1381 | **Cost guards for customers:** `loams byoc deploy` prints a monthly estimate before it applies and creates a budget alert in the customer's account (opt-out) | 12.3 | Approved (owner, 2026-10-21) |
| D1382 | **Phases BY0–BY3** (§13): BY0 in days (VMs on AWS, Google Cloud and Azure; Cloudflare phase 0; Vercel docs), BY1 (Tofu modules and container services), BY2 (Kubernetes and marketplaces), BY3 (Cloudflare-native mappings and multi-tenant) | 13 | Approved (owner, 2026-10-21); **amended by D1580** (owner, 2026-10-21: BY2 has no marketplace listings) |
| D1383 | **Not built:** an Ostrium-hosted data plane; cloud-specific engine forks; a Kubernetes distribution; CloudFormation, ARM or Deployment Manager twins of the Tofu modules, except a thin wrapper where a marketplace requires its own format; metering in marketplace images; Postgres on Cloudflare; a data plane on Vercel | 14 | Approved (owner, 2026-10-21); **amended by D1580** (owner, 2026-10-21: no marketplace listings, so no marketplace wrappers; no metering in any image) |
| D1384 | **The default VM is x86-64, 2 vCPU and 8 GiB with a 50 GB block disk**, with arm64 sizes offered once every pinned image is verified multi-arch | 6 | Approved (owner, 2026-10-21) |
| D1580 | **No cloud marketplace listings for now** (amends D1374, D1382, D1383 and Q1091's answer): no listing, free or paid, on AWS Marketplace, Google Cloud Marketplace, Microsoft Marketplace or any other cloud marketplace; no support plan and no private offers. BYOC installs only from Ostrium's own artefacts: the GHCR release images, the compose bundle and `install.sh`, the Helm chart and operator, and the OpenTofu modules (published to the OpenTofu registry, and the Terraform registry if wanted). The "Deploy to Cloudflare" button and the Vercel connectable-account integration stay: both deploy from this repository and sell nothing. No licence manager, usage metering or phone-home in any image (kept from D1374). Marketplaces are revisited only if a paid product ever exists | 10 | Approved (owner, 2026-10-21) |
| D1581 | **The support bundle stays a local file** (amends D1376): `loams support bundle` writes an archive on the user's machine and sends nothing. The user may attach it to a GitHub issue on `ostrium-labs/loams` or share it on the Loams Discord (https://discord.gg/eK6NanHQq) if they choose. There is no Ostrium support address | 9.7 | Approved (owner, 2026-10-21) |
| D1582 | **Maintainer debug session** (amends D1378's support grant): when a user wants a maintainer to look at an install, they open a maintainer debug session, using the time-boxed tailnet grant of D587 (default 4 hours, at most 24, Q588). It is opt-in, time-boxed and revocable by the user at any time; it is community help on a best-effort basis, with no contract and no SLA | 4, 9.8 | Approved (owner, 2026-10-21) |

### 1.5 Phases

| Phase | What | Effort (estimate) | Startup programme it backs |
|---|---|---|---|
| **BY0** (days) | VM installs on AWS, Google Cloud and Azure with `install.sh` or cloud-init, the bucket on S3, GCS or Blob through the VM's identity, a customer KMS key on the bucket, secrets from the cloud's store, and a guide per cloud; Cloudflare phase 0 (the `deploy/cloudflare/` template and Deploy button); the Vercel guide and Mastra-on-Vercel example; a manual BYOC matrix run per cloud | 4–5 engineer-weeks across 3 or 4 parallel agents: about 1 week for the VMs and Vercel, about 2 weeks for Cloudflare | **Cloudflare for Startups** (once the Deploy button works); **AWS Activate Founders** and **AWS open-source credits**, **Google for Startups (Start)**, **Microsoft for Startups**; **Vercel for Startups** (the Vercel guide) |
| **BY1** | OpenTofu modules for the VM and container tiers on AWS, Google Cloud and Azure; `loams byoc deploy`; the checkpointed node on Fargate, Cloud Run and Container Apps; `loams backup`; `loams support bundle`; the nightly matrix; the Vercel connectable-account integration | 8–10 engineer-weeks | Credits used for the nightly matrix; the Vercel Marketplace Provider programme |
| **BY2** | The Kubernetes tier on EKS, GKE and AKS with OP1's operator; managed metastores when D58's backends ship (no marketplace listings, D1580) | 8–12 engineer-weeks after OP1 | None: the marketplace-gated programmes (AWS ISV Accelerate, Microsoft ISV Success, Google Cloud Marketplace enrolment) are not pursued while there are no listings (D1580) |
| **BY3** | Cloudflare phases 1 and 2: the DO metastore, Durable on DOs, Jobs on Workflows and Queues, Git smart HTTP on Workers, Workers for Platforms for the multi-tenant Factory and marketplace | 4–8 weeks (phase 1) plus a quarter (phase 2) | A Cloudflare Tier 2 upgrade or partner conversation |

## 2. Scope

**In scope:** installing the Loams data plane (engine, House, Postgres, Git, agentd and the gateway, on the light stack or the `tikv` profile) into a customer's own AWS account, Google Cloud project, Azure subscription or Cloudflare account; and what "BYOC on Vercel" means. Per cloud: compute, bucket, keys, secrets, networking and private reach, identity, install UX and cost. Distribution is Ostrium's own artefacts only (D1580). The shared install, upgrade, backup, observability and support tooling. The test matrix.

**Not in scope:**
- **Apps and agents built with Loams** and deployed into a customer's account on serverless free tiers: [§71](71-app-byoc-free-tiers.md).
- **A hosted Loams.** Ostrium runs `loams.dev`, `auth.loams.dev` and `console.loams.dev` and nothing else (D1317, D1378).
- **New edge or networking mechanisms.** §69 owns edge reach; this document picks among its paths per cloud.
- **The multi-tenant control plane** (§41). A BYOC install here is one customer's; §41's hub can manage it only if the customer enrols it (D1377).
- **Clouds beyond the five.** Clever Cloud's CKE is §25's; Hetzner, OVHcloud and others take the VM tier unchanged.

## 3. Where things are today (checked on `dev` at `f33d5c7d6` and PR #973's branch, 2026-10-21)

| Piece | State |
|---|---|
| `deploy/release/byoc/install.sh` | Built. Installs Docker on a host over SSH, copies the compose bundle, writes `.env` (tailnet key, setup token), starts the stack and waits for `/health`. PR #973 adds `--tikv`, `--rustfs`, `--profiles`, `--no-postgres` and starts the light default |
| `byoc/upgrade.sh`, `uninstall.sh`, `upgrade-test.sh` | Built. `upgrade.sh` fetches the target release's CLI on the host, checks it against `SHA256SUMS` and runs `loams upgrade` there (UP1 Task 22) |
| The light default stack (PR #973, open) | The engine on openraft + redb, Live on redb, Durable on SQLite, a local-folder bucket, House, Loams Postgres on local folders, agentd and Tailscale; about 1.0 GB idle (measured on Podman 5). `loams upgrade` refuses embedded-store stacks until `loams migrate` can work beside a running engine (PR #973) |
| Object storage | Built and shipped in 0.1.0: `LOAMS_BUCKET` takes `s3://`, `gs://`, `az://` and R2 through the S3 endpoint, read the way `object_store`'s `from_env()` reads them. On EC2 and EKS the instance role or web identity is used when no keys are set (`docs/release/preview.md`, "The bucket"). The GCS and Azure paths' use of the VM's metadata identity is (verify) |
| Images | `loams`, `loams-fabric`, `loams-agentd` and `loams-pg-control` are amd64 and arm64 manifests (`release-images.yml`). The Loams Postgres images pinned in `deploy/pins/` are (verify) for arm64 |
| `KeyProvider` (D96), `SecretBackend` (D1154) | Designed, not built. No crate exists |
| `loams-operator` (§65), the `loams-stack` chart | Designed (OP1), not built. `deploy/helm/` holds only `loams-connectors` |
| `loams-tfplugin`, the OpenTofu provider (§64) | Designed (IS1), not built |
| `loams-byoc-agent`, `loams-control` (§41) | Designed (MT4, CP1), not built |
| The gateway's edge listener (§69) | Designed and approved (ER0), not built. GW1's `pg` module is merged |
| `loams-meta-dynamodb`, `loams-meta-postgres` (D58) | Approved for M2, not built |
| Console | Hosted at `console.loams.dev`; `web/apps/console/wrangler.jsonc` has an `env.byoc` for the customer's own Cloudflare account, "coming soon" |
| `docs/ecosystem/deploy-buttons.md` | A draft convention for `deploy/<provider>/` (README, template, `variables.md`, `CHANGELOG.md`; BYOC, pinned, least privilege, no phone-home, tested) |
| Backups, support bundle | No `loams backup` and no `loams support bundle`. `loams upgrade` takes snapshots into a local volume (§54) |

## 4. Principles (D1351, D1378)

1. **The customer's account, data and keys.** Every resource an install creates is in the customer's account and billed there. The bucket is the source of truth (D1) and is theirs; its encryption key is theirs; its secrets are in their secret store. Deleting the account deletes Loams.
2. **Ostrium runs no data plane.** No Ostrium process sits on a request path, holds a bucket credential or can decrypt a byte. Help comes from a maintainer debug session the customer opens, time-boxed and revocable (D587, D1582), or from a support bundle the customer chooses to share (D1376, D1581). The BYOC agent is opt-in (D1377). There is no "managed BYOC" in Ostrium's account (D1378).
3. **One artefact set.** The same images by digest, the same compose bundle, the same operator and chart, the same Tofu modules for every cloud. A cloud gets a table of settings, never a fork (D1352, D1363).
4. **The light stack by default, TiKV for scale.** The default install is one node on the embedded stores (PR #973's D1340–D1343), because that is what fits a free trial, a credit allowance and a proof of concept. The `tikv` profile, then the operator on Kubernetes, are the path to several nodes and HA. A customer chooses the store before the first start; an install does not switch stores in place (PR #973).
5. **Never public, as amended by D1311.** The engine's listeners are on loopback, the tailnet, or an internal network behind `loams-gateway`. Callers off the tailnet use the gateway's edge listener over a private link or a brokered tunnel. No tier opens an inbound port from the internet (D1359).

## 5. The tiers

### 5.1 Three tiers (D1352)

| Tier | What runs | Store | Who it is for | Phase |
|---|---|---|---|---|
| **VM** | The compose stack (`deploy/release/compose.yaml`), light by default, `tikv` optional | Embedded on the VM's block disk; the bucket in the cloud | Trials, small teams, single-tenant production with stated limits (PR #973) | BY0 |
| **Container service** | The engine image as one task or instance; House as a second; Postgres not offered | Checkpointed (D1354), later a managed metastore (D1355) | Customers who will not patch a VM; the Cloudflare target | BY0 on Cloudflare, BY1 elsewhere |
| **Kubernetes** | `loams-operator` and a `LoamsCluster` (§65) | PVCs on block storage; TiKV for several engines | Scale, HA, platform teams | BY2 |

The tiers share images, configuration names (`LOAMS_*`), the bucket layout and the upgrade gate. A customer can move up a tier by restoring a backup into a new install (D1364).

### 5.2 The persistence rule (D1353)

The embedded metastore is openraft on redb; Live is redb; Durable is SQLite. Each commits with `fsync` and assumes one writer with exclusive file locks (PR #973's `migrate` checks the engine's lock with `flock`). That needs a local block device:
- **Allowed:** an EBS volume, a Persistent Disk or Hyperdisk, an Azure managed disk, a Kubernetes PVC on any of these, a local NVMe disk.
- **Not allowed:** EFS, Azure Files (SMB or NFS), Filestore, Cloud Storage FUSE, any FUSE mount of a bucket, Cloudflare Container disks that vanish on restart.

Network file systems either do not provide the locking and `fsync` semantics redb and SQLite rely on, or provide them with latency that makes every commit a network round trip. SQLite's own documentation warns against network file systems ([SQLite: use over a network](https://www.sqlite.org/useovernet.html), read 2026-10-21). This rule decides what the container tier can be on each cloud (§6).

### 5.3 The checkpointed single node (D1354)

Where no persistent block disk exists, the engine runs its embedded stores on the host's ephemeral disk and makes the bucket the place they survive:
- **Checkpoint.** Every 5 s (estimate; `LOAMS_CHECKPOINT_INTERVAL`) and on `SIGTERM`, the engine writes a consistent snapshot of the metastore (an openraft snapshot), Live's redb and Durable's SQLite (its backup API) to `_meta/checkpoints/<epoch>/<seq:020>/` in the bucket, then a create-only `…/<seq>.done` marker. Unchanged files are not re-uploaded (content-addressed parts, estimate of a few hundred kB per checkpoint for a small install).
- **Restore.** At boot, the engine reads the newest complete checkpoint of the newest epoch, restores the stores, and only then opens its listeners.
- **Fence.** Before restoring, the node claims the next epoch with a create-only PUT of `_meta/epoch/<n+1>` (`If-None-Match: *`). The bucket's conditional writes (required by D178) make the claim exclusive. A node that finds a newer epoch at its next checkpoint **stops accepting writes and exits** (the self-fence), so two instances that overlap during a redeploy cannot both keep committing for longer than one interval.
- **Recovery window.** A crash loses at most the writes of the last interval (5 s by default); a graceful stop loses none. Acknowledged writes in that window are lost; the guide says so in the first paragraph, and `/v1/info` reports `durability: checkpointed(5s)`.
- **One instance.** The platform must run at most one instance: `desiredCount: 1` on ECS, `max-instances: 1` on Cloud Run, `maxReplicas: 1` on Container Apps, one Durable Object name on Cloudflare. The epoch fence covers the moments when a platform briefly runs two.
- **What does not run here:** Loams Postgres (its safekeepers need durable disks), the `tikv` profile, the `quorum` and `express` stream classes, and anything needing a sandbox the host's kernel lacks (Q1086).

The Cloudflare research proposed exactly this for Containers ("checkpoint to R2 with a stated recovery window", P0.3). This document generalises it to every host without a block disk, so it is built once.

### 5.4 Managed metastores (D1355)

D58 approved two more `MetaStore` backends for M2, neither built: `loams-meta-postgres` (Lakekeeper's patterns) and `loams-meta-dynamodb` (one table, conditional writes). Each removes the checkpoint window on the container tier:

| Cloud | Metastore | Live and Durable | State |
|---|---|---|---|
| AWS | DynamoDB (on-demand) | Live on redb stays checkpointed; Durable on Postgres (Resonate's own plugin, §21) if Aurora is present | D58, not built |
| Google Cloud | Postgres on Cloud SQL or AlloyDB | Same | D58, not built |
| Azure | Azure Database for PostgreSQL | Same | D58, not built |
| Cloudflare | Durable Object SQLite (`loams-meta-do`, D1368) | Durable on DO SQLite (D1368) | New |

BY2 adopts them as they ship; nothing in BY0 or BY1 waits for them.

## 6. AWS, Google Cloud and Azure

The three hyperscalers get the same shape: a VM tier in BY0, a container tier in BY1, Kubernetes in BY2. The tables give the service for each concern and the facts that decide it.

### 6.1 AWS

| Concern | Choice | Facts and links |
|---|---|---|
| **VM tier** | EC2 `t3.large` (2 vCPU, 8 GiB) by default, `t4g.large` once every image is arm64 (D1384); a 50 GB gp3 data volume for the Docker volumes; Ubuntu 24.04 or Amazon Linux 2023; IMDSv2 only | `t3.large` $0.0832/h, about $60.74 a month; `t4g.large` $0.0672/h, about $49.06; gp3 $0.08/GB-month with 3,000 IOPS included (AWS price feed for us-east-1, published 2026-10-15; [EBS pricing](https://aws.amazon.com/ebs/pricing/), read 2026-10-21) |
| **Container tier** | ECS on Fargate, one task (`desiredCount: 1`), the checkpointed node (D1354). EFS is not used for the stores (D1353), and Fargate's EBS volumes are treated as ephemeral | Fargate mounts EFS and attaches EBS on platform version 1.4.0 and later, but an attached EBS volume is always new (empty or from a snapshot), and volumes of service-managed tasks "are always deleted upon task termination" ([ECS EBS volumes](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ebs-volumes.html), read 2026-10-21). Ephemeral storage is 20 GiB by default, up to 200 GiB ([Fargate storage](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/fargate-task-storage.html), read 2026-10-21). AWS publishes no `fsync` or SQLite guidance for EFS (unverified) |
| **Kubernetes tier** | EKS with OP1's operator; gp3 `StorageClass` for quorum members; EKS Pod Identity for `ObjectStore`, IRSA as the fallback | Control plane $0.10/h on standard support ([EKS pricing](https://aws.amazon.com/eks/pricing/), read 2026-10-21). Pod Identity needs no OIDC provider and does not work on Fargate ([Pod Identity](https://docs.aws.amazon.com/eks/latest/userguide/pod-identities.html), read 2026-10-21) |
| **Bucket** | S3, versioning on, Block Public Access on, a bucket policy that allows only the install's role; the instance profile, task role or Pod Identity supplies credentials | S3 supports `If-None-Match` and `If-Match` on `PutObject` ([what's new](https://aws.amazon.com/about-aws/whats-new/2024/11/amazon-s3-functionality-conditional-writes), 2024-11-25). S3 Standard $0.023/GB-month (price feed, 2026-10-15) |
| **Keys** | A customer-managed KMS key as the bucket's default encryption (SSE-KMS) with an S3 Bucket Key to cut KMS requests; the install's role gets `kms:GenerateDataKey` and `kms:Decrypt` on it only; `KeyProvider`'s per-namespace keys later (D96) | $1 per key per month, $0.03 per 10,000 requests ([KMS pricing](https://aws.amazon.com/kms/pricing/), read 2026-10-21) |
| **Secrets** | Secrets Manager: `loams/<install>/tailnet-authkey`, `setup-token`, `api-keys`, `link-key`; read at boot by `loams-byoc-secrets` with the instance role | $0.40 per secret per month, $0.05 per 10,000 calls ([Secrets Manager pricing](https://aws.amazon.com/secrets-manager/pricing/), read 2026-10-21) |
| **Networking** | For `small`: the VM in a public subnet with a security group that has **no inbound rules**, IMDSv2, and an auto-assigned public IPv4 for egress only. For `medium` or a customer's private subnet: no public IP, egress through a NAT gateway. Shell through SSM Session Manager | A public IPv4 address costs $0.005/h, about $3.65 a month (verify on the VPC pricing page); a NAT gateway about $33 a month before data (estimate, Q1082) |
| **Private reach** | Same-account workloads: the gateway's edge listener behind an internal NLB. Other accounts and Vercel: a PrivateLink endpoint service on that NLB, allowed for named principals (§69 §5.3) | NLB $0.0225/h (about $16.43 a month) plus NLCU-hours; an interface endpoint $0.01/h per AZ plus $0.01/GB (price feeds, 2026-09-11 and 2026-10-16) |
| **Identity → Loams token** | `sts:GetWebIdentityToken` after the account enables outbound identity federation (D1308, ER2) | A JWT with a custom `aud`, RS256 or ES384, 60 s to 1 h, at no extra cost ([AWS blog](https://aws.amazon.com/blogs/aws/simplify-access-to-external-services-using-aws-iam-outbound-identity-federation/), 2025-11-19) |
| **Distribution** | **No AWS Marketplace listing** (D1580): the install uses the GHCR images, the Tofu module and, on EKS, the Helm chart and operator. A listing would need a paid version and there is none to offer (D1323) | Verified 2026-10-21: "Developer, community, and BYOL editions of commercial software are supported if the seller provides an equivalent paid version on AWS Marketplace within 90 days of providing the free edition"; images must be in the Marketplace's ECR ([container policies](https://docs.aws.amazon.com/marketplace/latest/userguide/container-product-policies.html), read 2026-10-21). Fees: 20% on public container listings, 3% on private offers under $1M, 0.5% on professional-services private offers ([listing fees](https://docs.aws.amazon.com/marketplace/latest/userguide/listing-fees.html), effective 2024-01-05; the professional-services cut is dated 2026-06-16 in the research). Quick Launch for Helm on EKS was discontinued on 2026-03-01 ([container products](https://docs.aws.amazon.com/marketplace/latest/userguide/container-based-products.html), read 2026-10-21) |
| **Install UX** | BY0: `install.sh` over SSH, or `aws ec2 run-instances --user-data file://deploy/byoc/aws/cloud-init.yaml`; BY1: `loams byoc deploy --cloud aws` (the Tofu module); BY2: the Helm chart and operator on EKS | — |
| **Cost, small install** | **About $73 a month** (estimate): `t3.large` $60.74, 50 GB gp3 $4.00, an 8 GB root volume $0.64, 50 GB in S3 $1.15, one KMS key $1.00, four secrets $1.60, a public IPv4 $3.65. About $61 on `t4g.large` | Prices above |

### 6.2 Google Cloud

| Concern | Choice | Facts and links |
|---|---|---|
| **VM tier** | Compute Engine `e2-standard-2` (2 vCPU, 8 GB) by default, `t2a-standard-2` once every image is arm64; a 50 GB `pd-balanced` data disk; a dedicated service account with only the roles the install needs; Shielded VM | `e2-standard-2` $0.06701/h, about $48.92 a month; `pd-balanced` $0.10/GiB-month ([general-purpose pricing](https://cloud.google.com/products/compute/pricing/general-purpose), [disk pricing](https://cloud.google.com/compute/disks-image-pricing), read 2026-10-21) |
| **Container tier** | A **Cloud Run worker pool** with one instance, the checkpointed node. A worker pool has no endpoint, which suits an engine that joins the tailnet and needs no inbound HTTP. The volumes Cloud Run offers are not allowed for the stores (D1353) | Worker pools became GA on 2026-04-14 and have no endpoint and no autoscaling ([release notes](https://docs.cloud.google.com/run/docs/release-notes), 2026-10-20). Cloud Storage FUSE "does not provide concurrency control … (file locking)" ([FUSE mounts](https://docs.cloud.google.com/run/docs/configuring/services/cloud-storage-volume-mounts), 2026-10-20); "Cloud Run does not support NFS locking" ([NFS mounts](https://docs.cloud.google.com/run/docs/configuring/services/nfs-volume-mounts), 2026-10-20); the ephemeral disk (preview since 2026-04-20) is deleted when the instance shuts down ([ephemeral disk](https://docs.cloud.google.com/run/docs/configuring/services/ephemeral-disk), 2026-10-20). Max instances "can be exceeded for a brief period" ([max instances](https://docs.cloud.google.com/run/docs/configuring/max-instances), 2026-10-20), which is why D1354 fences by epoch |
| **Kubernetes tier** | GKE (Standard or Autopilot) with OP1's operator; `pd-balanced` or Hyperdisk Balanced `StorageClass`; Workload Identity Federation for GKE for `ObjectStore` | $0.10/h per cluster, with a $74.40 monthly credit per billing account that covers one zonal or Autopilot cluster ([GKE pricing](https://cloud.google.com/kubernetes-engine/pricing), read 2026-10-21); Workload Identity Federation is always on in Autopilot ([how-to](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity), 2026-10-13) |
| **Bucket** | GCS with uniform bucket-level access, public access prevention and object versioning; credentials from the VM's or the pod's service account | GCS create-only writes use `ifGenerationMatch=0` ([request preconditions](https://cloud.google.com/storage/docs/request-preconditions), read 2026-10-21). Standard $0.020/GiB-month ([GCS pricing](https://cloud.google.com/storage/pricing), read 2026-10-21) |
| **Keys** | A Cloud KMS key as the bucket's default CMEK; the GCS service agent gets `cloudkms.cryptoKeyEncrypterDecrypter` on it | A software key version is about $0.06 a month, $0.03 per 10,000 operations ([KMS pricing](https://cloud.google.com/kms/pricing), read 2026-10-21) |
| **Secrets** | Secret Manager, the same names as on AWS, read with the VM's service account | $0.06 per active version per month with 6 free; 10,000 accesses free ([Secret Manager pricing](https://cloud.google.com/secret-manager/pricing), read 2026-10-21) |
| **Networking** | For `small`: an external IP with a firewall that allows no ingress; for `medium`: no external IP and Cloud NAT. Shell through IAP TCP forwarding | External IPv4 in use is charged hourly (verify the rate) |
| **Private reach** | An internal load balancer in front of the edge listener for same-VPC workloads and Cloud Run Direct VPC egress (§69 §5.3); a Private Service Connect service attachment for other projects | A PSC endpoint is $0.01/h plus $0.01/GiB processed ([VPC pricing](https://cloud.google.com/vpc/network-pricing), read 2026-10-21) |
| **Identity → Loams token** | The metadata server's ID token with an audience (D1308, ER2) | `.../identity?audience=…&format=full` returns an RS256 JWT with `instance_id`, `project_id` and `zone`, valid up to an hour ([verifying instance identity](https://docs.cloud.google.com/compute/docs/instances/verifying-instance-identity), read 2026-10-21) |
| **Distribution** | **No Google Cloud Marketplace listing** (D1580): the GHCR images, the Tofu module and, on GKE, the Helm chart and operator. For a later revisit: Google accepts free listings, and its VM packages can be the Tofu module | Kubernetes apps and VM products may be Free, BYOL or usage-based ([Kubernetes pricing](https://docs.cloud.google.com/marketplace/docs/partners/kubernetes/select-pricing), [VM pricing](https://docs.cloud.google.com/marketplace/docs/partners/vm/select-pricing), 2026-10-13); VM packages are Terraform (recommended) or Deployment Manager ([VM package](https://docs.cloud.google.com/marketplace/docs/partners/vm/create-deployment-package), 2026-10-13). Fees 3% standard (unverified on a primary page) |
| **Install UX** | BY0: `install.sh`, or `gcloud compute instances create --metadata-from-file user-data=deploy/byoc/gcp/cloud-init.yaml`; BY1: `loams byoc deploy --cloud gcp`; BY2: the Helm chart and operator on GKE | — |
| **Cost, small install** | **About $60 a month** (estimate): `e2-standard-2` $48.92, 50 GiB `pd-balanced` $5.00, a 10 GiB boot disk $1.00, 50 GiB in GCS $1.00, one key $0.06, secrets within the free versions, an external IPv4 about $3.65 (verify) | Prices above |

### 6.3 Azure

| Concern | Choice | Facts and links |
|---|---|---|
| **VM tier** | `Standard_D2s_v5` (2 vCPU, 8 GiB) by default, `B2ms` for idle trials, `D2ps_v5` once every image is arm64; a 64 GiB Standard SSD (E6) data disk; a user-assigned managed identity; Ubuntu 24.04 | D2s_v5 $0.096/h (about $70.08 a month), B2ms $0.0832/h (about $60.74), D2ps_v5 $0.077/h (about $56.21); Standard SSD E6 $4.80 a month (Azure Retail Prices API, East US, read 2026-10-21) |
| **Container tier** | Azure Container Apps, one replica (`minReplicas: 1`, `maxReplicas: 1`), the checkpointed node; Azure Files is not used for the stores (D1353) | Storage is ephemeral (1–8 GiB by vCPU) or Azure Files over SMB or NFS ([storage mounts](https://learn.microsoft.com/en-us/azure/container-apps/storage-mounts), 2026-10-13). Up to 4 vCPU and 8 GiB per app on Consumption, 2 vCPU and 4 GiB in a Consumption-only environment ([containers](https://learn.microsoft.com/en-us/azure/container-apps/containers), 2025-05-15); dedicated profiles up to 32 vCPU ([workload profiles](https://learn.microsoft.com/en-us/azure/container-apps/workload-profiles-overview), 2025-11-04). A Microsoft answer says SQLite "doesn't play well" with SMB ([Q&A](https://learn.microsoft.com/en-us/answers/questions/346794/how-to-update-or-insert-sqlite-database-in-web-app), read 2026-10-21; not product documentation) |
| **Kubernetes tier** | AKS with OP1's operator; Premium SSD v2 or `managed-csi-premium` `StorageClass`; Microsoft Entra Workload ID for `ObjectStore` | Free tier: no control-plane fee; Standard $0.10/h ([tiers](https://learn.microsoft.com/en-us/azure/aks/free-standard-pricing-tiers), 2026-07-06). The price API also lists a "FreeTierInfrastructureCost" meter at $0.05/h from 2026-10-01 that no page explains (unverified). Premium SSD v2 about $0.08/GiB-month with 3,000 IOPS free |
| **Bucket** | Blob Storage in a StorageV2 account, public network access off or limited to the VNet, versioning and soft delete on; the managed identity supplies credentials | Conditional `Put Blob` with `If-None-Match: *` ([conditional headers](https://learn.microsoft.com/en-us/rest/api/storageservices/specifying-conditional-headers-for-blob-service-operations), read 2026-10-21). Hot LRS $0.0208/GB-month (price API, 2026-10-21) |
| **Keys** | A Key Vault key as the storage account's customer-managed key; the account's identity gets `Key Vault Crypto Service Encryption User` on that key | $0.03 per 10,000 operations, no per-key charge for Standard software keys; a Premium HSM RSA-2048 key is $1 a month (price API, 2026-10-21) |
| **Secrets** | Key Vault secrets, the same names, read with the managed identity | Same operation price |
| **Networking** | For `small`: a public IP with an NSG that allows no inbound; for `medium`: a NAT gateway. Shell through Azure Bastion or `az vm run-command` | — |
| **Private reach** | An internal Standard Load Balancer in front of the edge listener; a Private Link service on it for other VNets and subscriptions | Private Endpoint $0.01/h, $0.01/GB in and out (price API, 2026-10-21) |
| **Identity → Loams token** | A managed-identity access token whose `aud` is an application ID URI the install registers in the **customer's** tenant (`api://loams-<install>`), exchanged under a trust policy (a new ER2 issuer, Q1090) | IMDS puts the requested resource in `aud` ([VM token](https://learn.microsoft.com/en-us/entra/identity/managed-identities-azure-resources/how-to-use-vm-token), 2025-11-11), and the app must exist in the tenant (AADSTS50001 otherwise). IMDS's attested document (PKCS7, `vmId`, `subscriptionId`, a nonce) is the fallback ([IMDS](https://learn.microsoft.com/en-us/azure/virtual-machines/instance-metadata-service), 2025-07-29) |
| **Distribution** | **No Microsoft Marketplace offer** (D1580): the GHCR images, the Tofu module and, on AKS, the Helm chart and operator. For a later revisit: a Container offer (the chart as a CNAB bundle in ACR) with a free plan, never a managed application (it gives the publisher's tenant access to the customer's resource group, which D1378 rules out) | Container offers are Kubernetes apps only, packaged as CNAB; plans can be Free or BYOL ([containers](https://learn.microsoft.com/en-us/partner-center/marketplace-offers/marketplace-containers), 2025-07-31); 3% on transacted offers, 0% on BYOL ([transact](https://learn.microsoft.com/en-us/partner-center/marketplace-offers/marketplace-commercial-transaction-capabilities-and-considerations), 2026-07-22); solution templates are not transactable and exclude AKS; managed applications give the publisher access to the managed resource group ([plan an Azure Application](https://learn.microsoft.com/en-us/partner-center/marketplace-offers/plan-azure-application-offer), 2025-07-31) |
| **Install UX** | BY0: `install.sh`, or `az vm create --custom-data deploy/byoc/azure/cloud-init.yaml --assign-identity`; BY1: `loams byoc deploy --cloud azure`; BY2: the Helm chart and operator on AKS | — |
| **Cost, small install** | **About $82 a month** (estimate): D2s_v5 $70.08, E6 data disk $4.80, an E4 OS disk $2.40, 50 GB of Blob $1.04, Key Vault operations under $0.10, a Standard public IPv4 about $3.65 (verify). About $72 on B2ms, about $68 on D2ps_v5 | Prices above |

**OpenTofu providers:** the registry serves `hashicorp/aws` up to 6.69.1, `hashicorp/google` up to 8.7.0 and `hashicorp/azurerm` up to 5.10.0 ([registry API](https://registry.opentofu.org/v1/providers/hashicorp/aws/versions), read 2026-10-21).

### 6.4 What the three have in common

- **One cloud-init file per cloud,** differing only in how it reads secrets (`aws secretsmanager`, `gcloud secrets`, `az keyvault`) and which bucket URL it writes. Each installs Docker, fetches the compose bundle for a pinned release from GitHub Releases and checks it against `SHA256SUMS` and the release key ([signing](../release/signing.md)), writes `.env` with `LOAMS_BUCKET` and no static keys, starts the stack and reports the tailnet name to the serial console.
- **The bucket probe runs first** (`loams store probe`, BY0 Task 2): create-only PUT, conditional overwrite, range read, list after write, multipart, versioning on. A failure stops the install before the engine has written anything.
- **Postgres's remote storage** stays on the VM's disk in BY0 (Neon `LocalFs`, PR #973), backed up by disk snapshots. BY1 points the pageserver and safekeepers at the cloud bucket (Neon's `remote_storage` speaks S3, GCS and Azure; (verify) for GCS and Azure in the fork's pinned release).

## 7. Cloudflare

### 7.1 What Cloudflare can and cannot host

From the research of 2026-10-21 (sources in its §9; the key facts are repeated here with their pages):
- **Containers have no persistent disk.** "All disk is ephemeral by default"; a sleeping instance restarts on a fresh disk from the image. They are reached only through their Durable Object; end users cannot open raw TCP to them. Instance types go up to 4 vCPU, 12 GiB and 20 GB of disk by default ([limits](https://developers.cloudflare.com/containers/platform-details/limits/), 2026-10-12; [architecture](https://developers.cloudflare.com/containers/concepts/architecture/), 2026-09-30).
- **R2 has the conditional writes Loams needs:** `PutObject` with `If-Match` and `If-None-Match` ([S3 extensions](https://developers.cloudflare.com/r2/api/s3/extensions/), 2026-10-14), and free egress ([pricing](https://developers.cloudflare.com/r2/pricing/), 2026-10-01).
- **Durable Objects (SQLite)** give 10 GB per object and about 1,000 requests a second per object ([limits](https://developers.cloudflare.com/durable-objects/platform/limits/), 2026-06-01).
- **Not feasible today:** TiKV, production Loams Postgres (safekeepers need durable disks), the `quorum` stream class, public raw TCP without Enterprise Spectrum, and the Firecracker sandbox adapter.

So the light stack fits a Container only as a checkpointed node (D1354), and R2 is a natural bucket.

### 7.2 Component map (from the research's §3.2)

| Component | Phase 0 (BY0) | Phase 1 (BY3) | Phase 2 (BY3) |
|---|---|---|---|
| Engine (retrieval, Live, Durable, Graph) | One Container (`standard-2`, 1 vCPU, 6 GiB, estimate) behind the `LoamsNode` DO; checkpointed to R2 | Stateless query and worker Containers over the DO metastore | Per-namespace Containers named by DO |
| Metastore | openraft + redb, checkpointed | `loams-meta-do` (D1368) | Sharded per namespace |
| Durable | SQLite, checkpointed | A Resonate store on DO SQLite; timers as DO alarms (D1368) | — |
| House | A second Container (`standard-4`) if the day-1 sandbox spike passes (Q1086); HTTP 8123 through the Worker only | Basin Catalog as the external Iceberg catalog (D571) | — |
| Git | `git-remote-loams` straight to R2 with prefix-scoped temporary credentials from a small Worker | Smart HTTP on a Worker, a sequencer DO per repository, packs on R2, heavy work in Containers, an Artifacts mirror (D1369) | Workers for Platforms for per-tenant Git hosts |
| Jobs | Not offered | Workflows and Queues (D1370) | — |
| Postgres | Not offered (D1371) | Hyperdrive to an external Postgres, or Data BYOC through Tunnel | — |
| Console | Workers static assets (`web/apps/console`, `--env byoc`) | — | — |
| agentd and Studio sessions | Not in phase 0 | One Sandbox or Container per session, Containers snapshots (D574), Artifacts forks per agent | — |
| Functions | Real Workers | — | Workers for Platforms user Workers per tenant (PD33) |

### 7.3 The edge on Cloudflare (D1372)

On Cloudflare there is no tailnet and no VPC: the front Worker is the only door, so it plays the gateway edge listener's role (D1302):
- **Data-plane calls** carry an environment-bound `lk_` key (D1307). The Worker checks the route against the edge allow-list (D1322, compiled into the Worker from `api/edge-allowlist.txt`) and forwards to the Durable Object, which forwards to `containerFetch` on the engine's edge upstream port (D1304). The engine applies the allow-list and the credential policy again.
- **The console and admin routes** are served only on a hostname behind a Cloudflare Access application with the customer's identity provider; the Worker checks `Cf-Access-Jwt-Assertion` (as D1306 does for `access` listeners) before it forwards anything that is not on the allow-list.
- **`workers.dev` is off** for the front Worker; it is bound to the customer's zone, so Access can protect it.
- **No Container port is reachable** except through the DO.

Every request is authenticated before it reaches the stack, so this is a *brokered* path under D1301, and D1311 holds without the deferred public profile.

### 7.4 Decisions adopted from the Cloudflare research

The research's eight recommended defaults were **approved by the owner on 2026-10-21** and are recorded here:

| Research item | Decision here | Ruling (owner, 2026-10-21) |
|---|---|---|
| §7.1 Reverse PD29/PD34 for a Cloudflare BYOC target? | D1366: a Cloudflare-native single-node and agent profile, PD36's k3s Data BYOC unchanged for scale | Approved: add the profile; k3s stays for scale |
| §7.3 Phase 0 durability | D1354, D1367: checkpoint to R2, labelled preview | Approved: preview, a few seconds of metadata loss on a crash |
| §7.2 DO SQLite as truth | D1368: for the metastore and Durable, Cloudflare target only, phase 1 | Approved (amends D382; Git keeps R2) |
| §7.6 Git strategy | D1369: R2 as truth, sequencer DO, Artifacts as mirror and workspaces | Approved |
| §7.7 Jobs | D1370: Cloudflare backend first | Approved |
| §7.5 Postgres scope | D1371: out of Cloudflare BYOC | Approved: out for now |
| §7.4 The sandbox on Containers | Q1086 | Approved: `sandbox: off` allowed for single-tenant installs if the Container kernel lacks what the sandbox needs, behind an explicit flag |
| §7.8 Priority against 0.1.x | Q1087 | Approved: phase 0 starts right after the 0.1.1 tag (it depends on PR #973) |

### 7.5 Phase 0 in practice

- **The template:** `deploy/cloudflare/` per the deploy-buttons convention: `README.md` with the button, `wrangler.jsonc` (the template), `variables.md`, `CHANGELOG.md`; the Worker and DO in `deploy/cloudflare/src/`; a container `Dockerfile` that is `FROM` the pinned `loams` image and adds only the checkpoint settings.
- **Bindings:** `LOAMS_NODE` (DO class `LoamsNode`, with the container), `HOUSE_NODE` (optional), `BUCKET` (R2), `ASSETS` (the console), secrets `LOAMS_SETUP_TOKEN`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY` (the engine speaks S3 to R2's endpoint; a binding cannot be passed into a Container).
- **Keep-alive:** the DO keeps its container running with `sleepAfter` at its maximum and an alarm, because a sleeping container loses its disk; the checkpoint makes a wake safe anyway.
- **The honest claim** (from the research): "Loams deploys into your Cloudflare account with one command: R2 is the source of truth; Workers serve the API and console; a Durable Object runs Loams in a Cloudflare Container. Single node, preview: metadata is checkpointed to R2 every few seconds. Git repositories live directly on R2." No claim of multi-node, HA or Postgres.
- **Cost:** about $46 a month for an always-on `standard-2` at 10% CPU, about $134 for a `standard-4` at 25% (research §2, estimates), plus Workers Paid ($5) and R2 storage.
- **Effort:** 9–14 agent-days, about 2 weeks with 2 or 3 agents in parallel, demoable after about 1 week (research §5.1, estimate). Owner time about one day (account, Workers Paid, R2 token, recording).

## 8. Vercel (D1373)

**Vercel cannot host the Loams data plane, and this design does not pretend otherwise.** Vercel Functions are request-scoped with an ephemeral file system; there is no persistent disk, no long-lived process and no raw TCP listener. The engine, its metastore and Postgres need all three.

"BYOC on Vercel" therefore means four things, each honest about where it runs:

1. **The customer's Loams runs in another cloud they own:** AWS (best, because Vercel PrivateLink reaches AWS), Google Cloud, Azure or Cloudflare, installed by §6 or §7.
2. **The apps and agents the Factory and Studio build run on the customer's Vercel team** (§71: Vercel Functions, with Marketplace storage free tiers for app state). A Mastra or AI SDK agent on Vercel calls the customer's Loams for retrieval, memory and Durable.
3. **A Vercel Marketplace connectable-account integration** (BY1): the customer installs "Loams" from the integration's page or a Deploy Button; the integration asks for their Loams endpoint and an environment-bound key (or, from ER2, sets up a trust policy for the project's Vercel OIDC issuer), and writes `LOAMS_ENDPOINT`, `LOAMS_NAMESPACE`, `LOAMS_API_KEY` (or `LOAMS_TOKEN_EXCHANGE=vercel`) and `CF_ACCESS_CLIENT_ID`/`_SECRET` into the chosen projects. A native integration (billing through Vercel, a hosted resource per install) is not possible without a hosted Loams, which D1378 rules out. The connectable-account path gets a "Community" badge and a public listing only after 500 active installs (research `vercel.md` §2, from [create an integration](https://vercel.com/docs/integrations/create-integration), 2026-09-16).
4. **Edge reach** (§69 §8): the Access hostname with a service token by default; Vercel PrivateLink (Pro and Enterprise) to an NLB in front of the gateway when Loams runs on AWS; Vercel OIDC exchanged for a Loams token from ER2. Vercel issues a per-team OIDC token (`https://oidc.vercel.com/<team>`, `sub` = `owner:<team>:project:<name>:environment:<env>`, a custom audience allowed, two-hour lifetime) and documents AWS, Google Cloud, Azure and custom APIs as relying parties ([OIDC](https://vercel.com/docs/oidc), 2026-08-04; [reference](https://vercel.com/docs/oidc/reference), 2026-09-17).

What a Vercel customer pays: their Vercel plan, plus the data plane's cloud (§6, §7). Hobby is for "non-commercial, personal use only" ([fair use](https://vercel.com/docs/limits/fair-use-guidelines#commercial-usage), read 2026-10-21), so a business uses Pro ($20 a month with $20 of usage included; [Pro](https://vercel.com/docs/plans/pro-plan), 2026-09-15).

BY0 for Vercel is documentation: `docs/guides/byoc/vercel.md` and a "Mastra on Vercel with Loams" example (`web/examples/vercel-mastra-loams/`).

## 9. Shared pieces

### 9.1 OpenTofu modules (D1362)

```
deploy/tofu/
  modules/
    aws-vm/          aws-container/     aws-eks/
    gcp-vm/          gcp-container/     gcp-gke/
    azure-vm/        azure-container/   azure-aks/
    cloudflare/      (BY3; wrangler stays the default for phase 0)
    shared/          (cloud-init rendering, naming, tags, outputs)
  examples/<module>/  (a minimal root module each, used by the matrix)
```

- **Providers:** the official `hashicorp/aws`, `hashicorp/google` and `hashicorp/azurerm` from the OpenTofu registry, pinned; `cloudflare/cloudflare` for BY3. OpenTofu only (D1150); Terraform works but is not tested.
- **Inputs:** `name`, `region`, `release` (a version; images by the release's digests), `tier`, `size` (`small`, `medium`), `profiles` (`tikv`, …), `bucket` (create, or an existing name), `kms_key` (create, or an existing ARN, key name or key ID), `tailnet_authkey_secret` (a secret ID, never a value), `private_reach` (`none`, `internal-lb`, `privatelink`/`psc`/`private-link`), `budget_monthly_usd` (D1381).
- **Outputs:** the instance or service ID, the bucket URL, the key ID, the tailnet name once known, the private endpoint if any, `estimated_monthly_usd`.
- **State is the customer's.** The modules never configure a backend; the Loams state backend (D1153) is one option among the three D1153 documents.
- **Publishing:** a mirror repository `ostrium-labs/terraform-loams-byoc` written only by a release workflow, as D1151 does for the provider; registered in the OpenTofu registry (verify the module address rules).
- **Tested** by the matrix (§12) with `tofu plan` on every PR that touches them and `tofu apply`/`destroy` nightly.

### 9.2 Kubernetes (D1363)

BY2 adds nothing to the operator except data: a table per cloud in `docs/guides/byoc/` and the chart's values files `values-eks.yaml`, `values-gke.yaml`, `values-aks.yaml`:
- **Storage class** for `LoamsInstance` PVCs (gp3, `pd-balanced`/Hyperdisk Balanced, `managed-csi-premium`/Premium SSD v2).
- **Workload identity** for `ObjectStore` and backups: EKS Pod Identity, Workload Identity Federation for GKE, Entra Workload ID.
- **Exposure:** `tailnet` (default) or `internal` with the cloud's internal load-balancer annotation from D1195's fixed list (`service.beta.kubernetes.io/aws-load-balancer-scheme: internal`, `networking.gke.io/load-balancer-type: Internal`, `service.beta.kubernetes.io/azure-load-balancer-internal: "true"`), or `tunnel` (D1316).
- **Node pools:** one general pool; a GPU pool per §67 on request.

The Tofu modules `aws-eks`, `gcp-gke` and `azure-aks` create the cluster only when asked; most customers bring one.

### 9.3 `loams byoc` (D1361)

D849 reserved the `loams byoc` namespace for `enrol|list|revoke` (MT4). This design adds:

```
loams byoc deploy  --cloud aws|gcp|azure|cloudflare [--tier vm|container|k8s] [--region R]
                   [--size small|medium] [--profiles tikv,...] [--release V] [--plan-only]
loams byoc status  [--name N]          # health, versions, durability mode, estimated cost
loams byoc upgrade --to V [--name N]   # §9.5
loams byoc backup  create|list|restore # §9.4 (through the host's `loams backup`)
loams byoc bundle  [--name N]          # §9.7
loams byoc destroy [--name N] [--keep-bucket]   # the bucket is kept by default
```

- **What it does:** renders a root module from the chosen `deploy/tofu/modules/<cloud>-<tier>`, runs `tofu init` and `tofu plan`, prints the plan summary and the monthly estimate (D1381), asks once, applies, and waits for the stack's health over the tailnet. For Cloudflare it runs `wrangler deploy` on `deploy/cloudflare/`.
- **Credentials:** the cloud CLI's own (`aws`, `gcloud`, `az`, `wrangler`) as the user already has them; the CLI never stores cloud credentials.
- **Records:** `~/.loams/byoc/<name>.toml` (cloud, region, module version, state location, tailnet name), no secrets.
- **BY0 has no `loams byoc deploy`:** `install.sh` and the cloud-init files are the BY0 path.

### 9.4 Backups (D1364)

- **What a backup is** (§65 §8.9): bucket versioning plus a consistent metastore, Live and Durable snapshot, plus the stack's configuration (never secret values).
- **BY0:** the guide turns on bucket versioning and a daily disk snapshot of the data volume (AWS Data Lifecycle Manager, a Compute Engine snapshot schedule, Azure Backup or a snapshot policy), and documents restoring a disk into a new VM.
- **BY1:** `loams backup create` writes the same snapshot `loams upgrade` takes (§54) to `_backups/<id>/` in the customer's bucket instead of a local volume; `loams backup restore --into <dir>` builds a new stack directory from it; a checkpointed node's checkpoints are themselves restorable backups.
- **BY2:** `LoamsBackup` and `LoamsRestore` (§65).
- **Restores never overwrite a live install** (D1189's rule, extended to compose).

### 9.5 Upgrades (D1365)

- **VM:** `byoc/upgrade.sh user@host --to V` (built) over SSH or the tailnet; or the cloud's run-command with the same script, for hosts with no SSH. PR #973's limitation (embedded stacks are upgraded by hand until `loams migrate` works beside a running engine) applies, and BY0's guide says so.
- **Container tier:** a new task definition, revision or Container image; the checkpoint makes the swap safe, and the `migrate` step runs as a one-off task first.
- **Kubernetes:** a new release set for the operator (§65 §8.8).
- **Cloudflare:** `wrangler deploy` of the new template version; the DO drains the container on `SIGTERM` (up to 15 minutes of grace, research §2).

### 9.6 Observability (D1375)

- The stack exposes Prometheus metrics and OTLP as today. The guide shows the cloud's own agent scraping or receiving them (CloudWatch agent, Ops Agent, Azure Monitor agent); Cloudflare installs use Workers Logs and the container's logs in the dashboard.
- Health and durability mode are in `/v1/info` and `loams byoc status`.
- Nothing is sent to Ostrium. If the customer enrols the BYOC agent (D1377), its heartbeat carries versions and conditions only (D543).

### 9.7 Support bundle (D1376, D1581)

`loams support bundle [-o file]` (on the host, or `loams byoc bundle` from the operator's machine) writes a `tar.zst` with:
- versions and image digests, `/v1/info`, `/health` and `/ready`;
- the compose service states or the operator's conditions;
- the last 2,000 log lines per service, through the redactor used by audit logs;
- `.env` with every value whose key matches the secret list replaced by `<redacted>`, and `secrets/` as names only;
- the bucket probe's result and the bucket URL, never credentials.

No documents, vectors, rows or keys are read. The bundle is a local file and the command sends nothing (D1581). The user reviews the archive (`--list` prints its manifest) and, if they choose, attaches it to a GitHub issue on `ostrium-labs/loams` or shares it on the Loams Discord (https://discord.gg/eK6NanHQq). There is no Ostrium support address.

### 9.8 Connection to Ostrium (D1377)

None by default. The hosted console at `console.loams.dev` reaches an install over the customer's tailnet, from the customer's browser, exactly as today. Enrolling the BYOC agent (`loams byoc enrol`, MT4) is a separate, opt-in step for customers who want Ostrium's rollout rings or a maintainer debug session.

**Maintainer debug session (D1582).** When a user asks a maintainer to look at an install (on a GitHub issue or on Discord), the user can open a session: the time-boxed tailnet grant of D587 (default 4 hours, at most 24, Q588) to a maintainer's tailnet identity. It is opt-in (only the user opens it), time-boxed, and revocable by the user at any time. It is community help on a best-effort basis, with no contract and no SLA.

## 10. Distribution: Ostrium's own artefacts, no marketplaces (D1580, amends D1374)

This section first planned free listings on AWS Marketplace, Google Cloud Marketplace and Microsoft Marketplace, with an Ostrium support plan as the paid version AWS asks for and services sold as private offers (D1374, Q1091). The owner removed the services offering on 2026-10-21 (D1323), so there is nothing to sell, and the listings are dropped for now (D1580).

BYOC installs only from artefacts Ostrium publishes itself:

| Artefact | Tier | Published as | Phase |
|---|---|---|---|
| Release images, by digest | All | `ghcr.io/ostrium-labs/*` from `release-images.yml` | BY0 |
| Compose bundle and `install.sh`, cloud-init files | VM | The GitHub release; `deploy/byoc/<cloud>/` | BY0 |
| OpenTofu modules (D1362) | VM, container, Kubernetes | The OpenTofu registry as `ostrium-labs/loams-byoc/<cloud>` (the Terraform registry too, if wanted) | BY1 |
| Helm chart and operator (D1363) | Kubernetes | The release, with the chart's per-cloud values | BY2 |
| "Deploy to Cloudflare" button | Cloudflare | `deploy/cloudflare/` in this repository; it deploys the template into the customer's account | BY0 |
| Vercel connectable-account integration (D1373) | Vercel | Writes environment variables into the customer's projects; it installs no data plane and sells nothing | BY1 |

**Why no marketplace listing.** AWS accepts a free container listing only beside a paid one: "Developer, community, and BYOL editions of commercial software are supported if the seller provides an equivalent paid version on AWS Marketplace within 90 days of providing the free edition" ([container-based product requirements](https://docs.aws.amazon.com/marketplace/latest/userguide/container-product-policies.html), read 2026-10-21). AMI listings carry the same rule without the 90 days ("provided an equivalent paid version is also available in AWS Marketplace", [AMI-based product requirements](https://docs.aws.amazon.com/marketplace/latest/userguide/product-and-ami-policies.html), read 2026-10-21). With no support plan there is no paid version to offer. Google Cloud and Azure accept free listings (§6.2, §6.3), but each listing needs its own copy of the images in the marketplace's registry, its own review and its own upkeep, for a channel whose main use is buying through cloud spend; with nothing to buy, that work is not worth doing yet.

Rules that stay:
- **One artefact set.** The images the guides, modules and chart use are the release images by digest.
- **No licence manager, no usage metering, no phone-home** in any image (kept from D1374; the deploy-buttons convention). Loams is Apache-2.0 and nothing in the product is gated.
- **Revisit** marketplaces only if a paid product ever exists (D1580). §6's notes on each marketplace's rules are kept for that day.

## 11. The platform products on BYOC (D1379)

The owner's focus is the Software Factory, Studio and the agents marketplace on the data platform. They have two different relations to BYOC:

| Product | Runs where | How it is installed | What it deploys |
|---|---|---|---|
| **Studio** | `loams-agentd` on the BYOC install (in the light stack by default since PR #973), and the console | With the stack | Agent sessions and worktrees on the install |
| **Software Factory** | The factory package (D-SF-20: chart, catalog patch, manifest) and its collaboration apps (Forgejo, Plane, Zulip, GlitchTip, …) on the BYOC install; their Postgres is Loams Postgres | A compose profile `factory` on the VM tier (BY1); the chart on Kubernetes (BY2) | The apps and agents it builds: **app BYOC, §71** |
| **Agents marketplace** | Packages (agents, connectors, apps) installed into the customer's Loams (`docs/marketplace/publishing.md`) | The marketplace install flow against the BYOC install | Agents that need Loams run on the install; agents that do not can deploy as app BYOC (§71) |

The VM tier's default size (2 vCPU, 8 GiB) runs the stack and Studio. The Factory's collaboration apps add several Postgres databases and JVM or Python services; the guide recommends `medium` (4 vCPU, 16 GiB, estimate) for the `factory` profile.

## 12. Conformance and cost guards (D1380, D1381)

### 12.1 The matrix

| | VM | Container | Kubernetes |
|---|---|---|---|
| AWS | EC2 `t3.large`, light | Fargate, checkpointed | EKS, operator |
| Google Cloud | `e2-standard-2`, light | Cloud Run, checkpointed | GKE Autopilot, operator |
| Azure | `D2s_v5`, light | Container Apps, checkpointed | AKS free tier, operator |
| Cloudflare | — | Containers, checkpointed | — |
| Vercel (app side) | — | A Vercel function calling the AWS install over PrivateLink or Access | — |

Each cell, per run:
1. `tofu apply` (or `wrangler deploy`) from the module's example;
2. wait for health over the tailnet (the runner joins as an ephemeral tagged node) or the Cloudflare Access hostname;
3. the **BYOC smoke suite** (`tests/byoc/smoke.sh`): bucket probe; create a collection, ingest, hybrid search through the Qdrant and ES APIs; a Live subscription; a durable workflow; `git push` to the bucket; a House query where House runs; a Postgres write where Postgres runs; `loams support bundle` and a check that it contains no planted secret;
4. **the restart test:** restart the VM, task or container and check that every acknowledged write before a graceful stop survives; on checkpointed hosts, also kill without grace and check that losses are within one interval;
5. **the exposure test:** from an outside runner, scan the install's public addresses and fail on any open port;
6. `tofu destroy` in an `always()` step.

**Cadence:** `tofu plan` and the static checks on PRs; the VM row nightly; the container and Kubernetes rows weekly and before each release candidate (estimate; Q1093).

### 12.2 The test accounts and their cost guards

**AWS (available now).** The owner has an AWS account with **$200 of credits for six months** (owner, 2026-10-21). BY0's AWS tests run there: the EC2 light-stack install of this document and the Lambda and DynamoDB app-BYOC target of §71. The nightly AWS matrix runs there too, under these rules:
- **No long-lived keys.** GitHub Actions assumes an IAM role through GitHub's OIDC provider (`token.actions.githubusercontent.com`), trusted only for `repo:ostrium-labs/loams:environment:byoc-aws`; the role's session lasts at most one hour. The owner creates the provider, the role and its policy (plan BY1, owner checklist); nothing is stored in the repository or asked for in chat.
- **A spend ceiling of about $30 a month** so the credits last the six months (estimate: $200 / 6 ≈ $33). The nightly VM run is one `t3.large` for at most an hour (about $0.08) plus seconds of S3 and KMS, about $3 a month; the weekly Fargate run about $1 a month; Lambda and DynamoDB stay inside the always-free allowances (§71). The few persistent pieces (one test KMS key at $1 a month, the Terraform state bucket) are created once.
- **Budgets:** an AWS Budgets cost budget of $30 a month with alerts at 50%, 80% and 100% (actual) and 100% (forecast) to the owner's address, and a **budget action** at 100% that attaches a deny policy for `ec2:RunInstances`, `ecs:RunTask`, `ecs:CreateService` and `lambda:CreateFunction` to the CI role.
- **Auto-teardown:** every resource carries `loams:ci-run=<run-id>` and `loams:ttl=<iso-time>` (3 hours by default); `tofu destroy` runs in an `always()` step; an hourly reaper workflow deletes anything tagged past its TTL, and anything in the account's CI region without a `loams:ci-run` tag older than a day is reported (never deleted automatically).
- **The plan type matters.** A new account on AWS's Free plan closes after six months or when the credits run out, and the Free plan offers only some services; the Paid plan keeps the credits for up to 12 months and allows every service ([free tier plans](https://docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/free-tier-plans.html), read 2026-10-21; the FAQ gives 6 and 12 months in different places, unverified). EKS and Marketplace tests (BY2) need the Paid plan. Which plan the account is on is the first item of the owner checklist in [BY1](../plans/2026-10-21-by1-byoc-clouds.md).

**Google Cloud and Azure (owner actions, still to come).** A Google Cloud project with a billing account (the $300, 90-day trial, or Google for Startups credits) and an Azure subscription (Microsoft for Startups credits; the free account's $200 lasts only 30 days) are owner actions. Until they exist, BY0's Google Cloud and Azure guides are tested by hand in a short-lived trial and their matrix rows are off. Each gets the same rules when it arrives: workload identity federation from GitHub Actions (Google's workload identity pools, Azure federated credentials on an app registration), a budget with alerts, the TTL tags and the reaper.

**For every test account:**
- **The smallest sizes** that pass, one region per cloud, and spot or preemptible VMs where the cloud allows them for the VM row.
- **Concurrency 1 per cloud**, so a stuck run cannot multiply.
- **Cloudflare:** the Workers Paid account the edge nightly already needs (Q1069 of §69), with phase 0's container at `standard-2`, deployed for the run and deleted after it.

### 12.3 Cost guards for customers (D1381)

- `loams byoc deploy` and the Tofu modules print an **estimate** from a small price table in the module (`estimated_monthly_usd`), labelled as an estimate with its date.
- They create a **budget alert** in the customer's account (AWS Budgets, a Cloud Billing budget, an Azure Cost Management budget) at the given `budget_monthly_usd`, opt-out with `--no-budget`. Cloudflare has no budgets API for Containers; the guide points to its usage notifications (verify).
- The install's resources carry `loams:install=<name>` tags so the customer's cost explorer can group them.

## 13. Phases (D1382)

### 13.1 BY0: works on each cloud, in days

**Goal:** a customer can install Loams into their own AWS, Google Cloud or Azure account on a VM, with the bucket, key and secrets in that account and no static keys on the host, by following one guide; a customer can deploy the single-node Cloudflare profile with one button; a Vercel user knows exactly what runs where.

| Workstream | Content | Effort (estimate) |
|---|---|---|
| VMs (AWS, Google Cloud, Azure) | `loams store probe`; `loams-byoc-secrets` (a small shell script per cloud); `deploy/byoc/<cloud>/cloud-init.yaml`; `install.sh --cloud`; `docs/guides/byoc/{index,aws,gcp,azure}.md` with the IAM, bucket, key and secret commands; the AWS VM row of the matrix running nightly in the owner's credit account (§12.2); a manual run on Google Cloud and Azure recorded in their guides once their accounts exist | 5–7 agent-days |
| Cloudflare phase 0 | The research's P0.0–P0.6: spikes, R2 bucket conformance, the `deploy/cloudflare/` template and button, the checkpoint (D1354) in the engine, House if its spike passes, Git on R2 with prefix credentials, the demo | 9–14 agent-days |
| Vercel | `docs/guides/byoc/vercel.md`; `web/examples/vercel-mastra-loams/` calling a BYOC install through Access | 2 agent-days |
| Shared | `loams support bundle` (minimal: versions, health, redacted `.env`, logs); `tests/byoc/smoke.sh` | 3 agent-days |

**Total:** about 19–26 agent-days, **2 weeks of calendar time with 3 or 4 agents in parallel**; the VM guides land in the first week. Owner time: about 2 days (the AWS OIDC role and budget from the plan's checklist, which take about an hour; the Google Cloud and Azure accounts; Cloudflare Workers Paid and the recording; reviewing the guides). The checkpoint code is shared by Cloudflare now and the container tier in BY1. Cloudflare phase 0 starts right after the 0.1.1 tag (Q1087).

### 13.2 BY1: modules and container services

OpenTofu modules for the VM and container tiers on the three hyperscalers; `loams byoc deploy|status|upgrade|destroy`; the checkpointed node on Fargate, Cloud Run and Container Apps; Postgres's remote storage in the cloud bucket; `loams backup`; the full support bundle; the nightly matrix; the Vercel connectable-account integration; the `factory` compose profile. **8–10 engineer-weeks** (estimate). Gate: BY0; PR #973 merged.

### 13.3 BY2: Kubernetes

The chart's per-cloud values and the EKS, GKE and AKS modules; the matrix's Kubernetes row; LOC run on each managed Kubernetes; managed metastores as D58's backends ship. **8–12 engineer-weeks after OP1** (estimate). Gate: OP1's Core level passing.

### 13.4 BY3: Cloudflare-native

The research's phases 1 and 2: `loams-meta-do`, Durable on DOs, Jobs on Workflows and Queues, Git smart HTTP with a sequencer DO and an Artifacts mirror, wire protocols through `cloudflared` and Hyperdrive, K2 and Basin; then Workers for Platforms for the multi-tenant Factory and marketplace. **4–8 weeks, then a quarter** (research estimates). Gate: phase 0 shipped and the owner's ruling on D1368.

### 13.5 Startup programmes

| Programme | What it needs to see | Phase that backs the application | Apply |
|---|---|---|---|
| **Cloudflare for Startups** (Tier 3, $10k) | A product on Cloudflare: the Deploy button, R2 as truth, the Git-on-R2 demo (research §6) | BY0 Cloudflare phase 0 | When the demo is recorded (end of BY0's first week, estimate); ask for Containers coverage and limits, WfP and Artifacts contacts |
| **AWS Activate Founders** and **AWS Cloud Credits for Open Source** | An OSS product that drives S3, EC2 and KMS use; the credits fund the matrix | BY0 AWS VM guide | At BY0 exit |
| **Google for Startups Cloud Program** (Start tier for an unfunded company) | A live product on Google Cloud | BY0 GCP VM guide | At BY0 exit |
| **Microsoft for Startups** | A software product, not a consultancy (research `microsoft.md` §1) | BY0 Azure VM guide | At BY0 exit |
| **Vercel for Startups** (up to $30k) and the Marketplace Provider programme | The integration path | BY0 Vercel guide (startups); BY1 connectable-account integration (provider programme) | BY0 exit; BY1 |

The application wording (for-profit product company, the entity, the funding status) is the owner's (research §6 and §7.9). Programmes that need a marketplace listing (AWS ISV Accelerate, Microsoft ISV Success and Frontier Accelerate, Google Cloud Marketplace enrolment) are not pursued while there are no listings (D1580).

## 14. What not to build (D1383)

- **An Ostrium-hosted data plane**, a "managed BYOC" in Ostrium's account, or a relay (D1317, D1378).
- **Cloud-specific forks or builds of the engine.** A cloud gets settings and modules.
- **A Kubernetes distribution** or a node image of our own; the clouds' own Kubernetes and OSes are used.
- **CloudFormation, ARM/Bicep or Deployment Manager twins** of the Tofu modules.
- **Cloud marketplace listings** (AWS, Google Cloud, Azure or others), a support plan or private offers, until a paid product exists (D1580).
- **Licence managers or usage metering in any image.**
- **Postgres on Cloudflare** (D1371), **a data plane on Vercel** (D1373).
- **Network file systems for the embedded stores** (D1353).
- **A cost dashboard.** The customer's cloud has one; Loams tags resources and sets a budget (D1381).

## 15. Relation to earlier decisions

| Decision | Effect |
|---|---|
| **D64, D543, D587** (BYOC modes, the outbound agent, the tailnet mode) | **Kept.** Every install here is BYOC-local-meta; the agent is opt-in (D1377) |
| **D849** (`loams byoc enrol\|list\|revoke`) | **Extended** with `deploy`, `status`, `upgrade`, `backup`, `bundle`, `destroy` (D1361) |
| **PR #973's D1340–D1343** (the light default; embedded stores supported for single node; the upgrade gate) | **Used** as the default of every tier; D1353 states the disk they need; D1354 adds a checkpointed variant for hosts without one |
| **D58** (Postgres and DynamoDB metastores in M2) | **Used** by D1355 on the container tier |
| **D96** (customer-managed keys through `KeyProvider`) | **Unchanged.** D1357 uses bucket default encryption until it exists |
| **D1154** (`SecretBackend`) | D1358's fetch script is replaced by it when built |
| **D1151, D1153** (the OpenTofu provider and state backend) | **Complemented** by the BYOC modules (D1362), which use the clouds' providers |
| **D1188, D1189, D1190, D1192, D1195** (operator backups, restores, store probe, `Diagnose`, exposure) | **Used** on the Kubernetes tier; D1364 and D1376 bring the same ideas to compose; D1356's probe is D1190's on a VM |
| **D1311** (never public, part A) | **Applied** to every tier (D1359); D1372 shows the Cloudflare front Worker is a brokered path |
| **PD29, PD34** (Cloudflare is edge and stateless compute; stateful services on k3s) | **Amended for the Cloudflare BYOC target only** by D1366 (owner decision); unchanged for Ostrium's own hosted plans and for scale |
| **PD36** (Edge BYOC and Data BYOC on Cloudflare) | **Kept** as the path to scale; D1366 adds a third, single-node option |
| **D382** (Durable Object storage is a cache and a lease, never the truth) | **Amended for the Cloudflare target, metastore and Durable only**, by D1368 (owner decision); unchanged for Git and everywhere else |
| **D391, D562** (a sequencer per repository; Artifacts as mirror) | **Used** by D1369 |
| **Q97** (no Postgres or DynamoDB Jobs backend) | **Not reopened.** D1370's Cloudflare backend is a third `JobStore`, specific to the Cloudflare target; if the owner reads Q97 as "TiKV only everywhere", D1370 needs that ruling (Q1088) |
| **D1208** (Mastra on `lk_` keys plus `LOAMS_NAMESPACE`) | **Kept**; the Vercel integration writes those variables (D1373) |
| **D-SF-20** (the factory package) | **Used** by D1379's `factory` profile and chart |

## 16. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The checkpoint window loses acknowledged writes on a crash | Certain on a crash | Medium | Stated in the guide, `/v1/info` and the claim; 5 s default; managed metastores (D1355) and the DO metastore (D1368) remove it; the VM tier has no window |
| Two checkpointed instances overlap during a redeploy and both commit | Medium | High | The create-only epoch and the self-fence (D1354); the restart test in the matrix |
| Customers put the stores on EFS or Azure Files anyway | Medium | High | The engine refuses to open an embedded store on a network file system (a `statfs` check at start, BY0 Task 1) with a message naming D1353 |
| Fargate, Cloud Run or Container Apps change their storage options | Low | Low | The tier is a table row; the matrix shows it |
| GCS or Azure metadata credentials do not work through `object_store` as wired in the engine | Medium | Medium | BY0 Task 0 checks first; the fallback is a short-lived key from the secret store, documented as such |
| A customer can only buy software through a cloud marketplace (to draw down committed spend) | Medium | Low | Loams is free, so there is nothing to draw down; the customer installs from the GHCR images, chart and Tofu modules. Marketplaces are revisited only if a paid product exists (D1580) |
| Matrix costs exceed credits | Medium | Medium | §12.2's guards; VM row nightly, the rest weekly |
| Cloudflare Containers limits (4 vCPU, 12 GiB) cap a single node | High | Medium | Stated; ask Cloudflare for a raise (research §6); Data BYOC through Tunnel for scale |
| PR #973's limit makes upgrades of embedded stacks manual on BY0 installs | Certain until fixed | Medium | The guide says so; PR #973 plans the fix before 0.1.2's gate |
| The hyperscaler guides drift from the clouds' consoles | Medium | Low | The commands are CLI commands, run by the matrix |

## 17. Open questions (Q1080–Q1094), answered

The owner answered every question with its default on 2026-10-21 (Q1086 and Q1087 earlier the same day, with the rulings shown).

| # | Question | Default (answered 2026-10-21 by the owner) |
|---|---|---|
| Q1080 | Offer the container tier on AWS, Google Cloud and Azure in BY1 as a checkpointed node, or wait for the managed metastores? | **Offer it in BY1, labelled preview with its window**, since the code is the Cloudflare phase 0 code; the VM tier stays the recommended path |
| Q1081 | Default VM sizes? | `t3.large`, `e2-standard-2`, `Standard_D2s_v5`; arm64 (`t4g.large`, `t2a-standard-2`, `D2ps_v5`) once every pinned image is verified multi-arch (D1384) |
| Q1082 | A public IP with no inbound rules, or a private subnet with NAT, by default? | **Public IP with no inbound rules** for `small` (saves a NAT gateway, about $33 a month on AWS, estimate); NAT for `medium` and when the customer gives a private subnet |
| Q1083 | Checkpoint interval default? | 5 s, configurable from 1 s to 60 s |
| Q1084 | Where do the BY0 cloud-init files live: `deploy/byoc/<cloud>/` or `deploy/release/byoc/<cloud>/`? | **`deploy/byoc/<cloud>/`** beside `deploy/cloudflare/`, following the deploy-buttons layout; `deploy/release/byoc/install.sh` stays and gains `--cloud` |
| Q1085 | Does BY0 include Loams Postgres on the VM tier? | Yes, on the VM's disk with daily disk snapshots, as PR #973 ships it; `--no-postgres` documented for smaller VMs |
| Q1086 | ~~Allow `sandbox: off` for House and Live on Containers if the kernel lacks Landlock, user namespaces or cgroup delegation?~~ | ~~Yes, for single-tenant installs only, behind an explicit flag; never for a multi-tenant install~~ **Answered 2026-10-21 by the owner: the default** |
| Q1087 | ~~When does Cloudflare phase 0 start relative to 0.1.x work?~~ | ~~After PR #973 merges~~ **Answered 2026-10-21 by the owner: right after the 0.1.1 tag** (it depends on PR #973) |
| Q1088 | Is a Cloudflare-only `JobStore` (D1370) compatible with Q97's "no Postgres or DynamoDB jobs backend"? | **Yes**: Q97 rejected general-purpose second backends; a backend for a target that cannot run TiKV is a different case. Owner to confirm |
| Q1089 | One OpenTofu module repository or one per cloud? | **One** (`ostrium-labs/terraform-loams-byoc`) with submodules per cloud and tier, if the registry's address rules allow it (verify) |
| Q1090 | Add Azure managed identity as an ER2 issuer? | **Yes**: the Entra issuer for managed-identity tokens, with `aud` = the install's application ID URI, matched on the object ID |
| Q1091 | Which marketplace pricing shape? | Free listings; on AWS, a contract-priced support plan beside the free plan (AWS's paid-version rule); services as private offers where allowed (AWS and Microsoft professional services); otherwise services sold directly. **Amended by D1580** (owner, 2026-10-21): no marketplace listings, no support plan and no private offers; installs come from Ostrium's own artefacts |
| Q1092 | Recommend Vercel Hobby in the guide? | No: Pro for any business use, since Hobby is non-commercial; Hobby only for personal trials |
| Q1093 | Matrix cadence? | VM row nightly; container and Kubernetes rows weekly and on each release candidate; Cloudflare nightly once phase 0 ships |
| Q1094 | Make `loams byoc deploy` require OpenTofu installed, or bundle it? | Require it (`tofu` 1.11 or later on `PATH`), with a clear error and install link; bundling a Go binary in the Rust CLI is not worth it |

## 18. Sources

Repository (read 2026-10-21): §18 §8, §24 §9 and §16, §26, §41, §54, §64, §65, §66, §67, §68, §69, the platform designs 03 and 04, `deploy/release/byoc/*.sh`, `docs/release/preview.md`, `docs/ecosystem/deploy-buttons.md`, `docs/marketplace/publishing.md`, `.github/workflows/release-images.yml`, PR #973's description and branch.

Research (outside the repository): `pitch/research/cloudflare-byoc.md` (2026-10-21; its sources in its §9), `pitch/research/aws.md`, `google.md`, `microsoft.md`, `vercel.md` (2026-10-20).

Vendor pages: linked inline above, read 2026-10-21.
