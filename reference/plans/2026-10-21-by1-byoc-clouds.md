# BY1 — BYOC on AWS, Google Cloud, Azure, Cloudflare and Vercel

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Work task by task, test first. Where this plan gives exact names, paths, flags or tests, use them verbatim. Record any deviation in "Rulings made during execution" at the end of this file.
>
> **Status: Approved** (2026-10-21; a stub: BY0 is specified, later phases at milestone level). Tracks BY and AB, designs [§70](../design/70-byoc-clouds.md) (D1350–D1384, Q1080–Q1094) and [§71](../design/71-app-byoc-free-tiers.md) (D1390–D1410, Q1095–Q1109). The owner set the direction on 2026-10-21 ("fully embrace BYOC"; app BYOC on free tiers) and approved the eight Cloudflare defaults the same day (D1366–D1371, the Cloudflare part of D1354, Q1086, Q1087). The owner approved both designs with every default on 2026-10-21. **Amended 2026-10-21 (D1580–D1582):** no cloud marketplace listings (BY2 ships the chart and operator from Ostrium's own artefacts only); the support bundle stays a local file with no Ostrium support address; support access is a maintainer debug session, community and best effort. This plan specifies **BY0** task by task. BY1–BY3 and AB0–AB2 are listed at milestone level and get full specifications when their gates are met.

**Goal (BY0):** a customer installs Loams into their own AWS account, Google Cloud project or Azure subscription on one VM, following one guide, with the bucket, its customer-managed key and the install's secrets in that account and **no static cloud keys on the host**; a customer deploys the single-node Cloudflare profile into their own Cloudflare account with one button; a Vercel user knows exactly what runs on Vercel and what runs elsewhere. Nothing is reachable from the internet, and the AWS row is tested nightly in the owner's credit account within about $30 a month.

**Architecture** (§70 §5–§9):
- **`crates/loams-store`** gains `probe` (the capability probe) and **`crates/loams`** the `loams store probe` command.
- **`crates/loams`** gains the network-file-system guard (D1353) and the checkpointed single node (D1354) behind `--checkpoint-to-bucket`.
- **`crates/loams-support`** (new) gives `loams support bundle` (D1376).
- **`deploy/byoc/{aws,gcp,azure}/`**: a cloud-init file, a secrets fetcher, a least-privilege policy and the deploy-buttons files, per cloud.
- **`deploy/release/byoc/install.sh`** gains `--cloud` and `--bucket`.
- **`deploy/cloudflare/`**: the phase 0 template (Worker, `LoamsNode` Durable Object, container, `wrangler.jsonc`, the Deploy button).
- **`docs/guides/byoc/`**: `index.md`, `aws.md`, `gcp.md`, `azure.md`, `cloudflare.md`, `vercel.md`.
- **`tests/byoc/`**: the smoke and exposure suites; **`scripts/byoc/aws-matrix.sh`** and two workflows for the nightly AWS row and its reaper.
- **`web/examples/vercel-mastra-loams/`**: a Mastra agent on Vercel calling a BYOC install.

**Tech stack:**
- Rust 1.97, edition 2024, workspace lints; workspace dependencies only (`object_store`, `tokio`, `clap`, `serde`, `tar`, `zstd` if already in the workspace, else `flate2` for `tar.gz`).
- Bash with `set -euo pipefail`, checked by shellcheck as `install.sh` is.
- cloud-init (`#cloud-config`), the AWS CLI v2, `gcloud`, `az`.
- `wrangler` and `@cloudflare/containers` from npm, pinned exactly, at releases at least 14 days old.
- GitHub Actions on GitHub-hosted runners only; `aws-actions/configure-aws-credentials` pinned by commit SHA.

**Spec:** [§70](../design/70-byoc-clouds.md) (all of it), [§71](../design/71-app-byoc-free-tiers.md) §7.2 for the AWS app row of the nightly (AB0), [§69](../design/69-edge-reach.md) §6.5 and §8, [§54](../design/54-upgrades.md) §10.5, `docs/ecosystem/deploy-buttons.md`, `pitch/research/cloudflare-byoc.md` §5.1 (outside the repository).

## Global Constraints

- **Worktrees and branches.** `~/Documents/Ostriumlabs/loams-wt/by0-<task>`, one branch and one PR per task into `dev`. CodeRabbit reviews; merge on green. Commit with `git commit -s`.
- **Light Rust builds.** `cargo build -p <crate>` with the configured jobs; never `--workspace` locally. Build output goes to the shared target in `~/Documents/.cargo/config.toml`; never build in `/tmp`.
- **No credentials anywhere in the repository, logs or chat.** Tests make their own keys. Cloud access in CI is GitHub OIDC into roles the owner creates (the owner checklist below). Secrets that CI needs (a Tailscale OAuth client, a Cloudflare API token) live in GitHub environments, put there by the owner.
- **Never public.** No cloud-init, script or template opens an inbound port from the internet: security groups, firewalls and NSGs have no inbound rules; Cloudflare's hostname is behind Access; `workers.dev` is off. The exposure test (Task 9) fails a run that finds an open port.
- **No static cloud keys on a cloud host.** S3, GCS and Blob are reached through the VM's identity. R2 keys (Cloudflare) are the only static keys, held as Worker secrets.
- **Pinned.** Every template pins a released Loams version and its image digests (deploy-buttons rule 2). `latest` never appears.
- **PR #973 first.** Tasks that start the stack assume PR #973's light default (its decisions D1340–D1343). Cloudflare phase 0 (Tasks 10–15) starts right after the 0.1.1 tag (Q1087, owner 2026-10-21).

## Review Focus

1. **An install opens a port to the internet.** Expected: never. Tests: `exposure_scan_finds_nothing` (Task 9), `cloud_init_has_no_inbound_rules` (Tasks 3–5), `cloudflare_template_has_no_workers_dev_route` (Task 13).
2. **A static cloud key lands on a VM.** Expected: never. Tests: `aws_env_has_no_access_key`, `gcp_env_has_no_service_account_key`, `azure_env_has_no_account_key` (Task 6).
3. **The embedded stores open on a network file system.** Expected: the engine refuses and exits 78. Test: `refuses_nfs_data_dir` (Task 1).
4. **A checkpointed node keeps committing after another claimed a newer epoch.** Expected: it stops accepting writes within one interval. Test: `superseded_node_self_fences` (Task 11).
5. **A support bundle contains a secret.** Expected: never. Test: `bundle_redacts_planted_secrets` (Task 8).
6. **The nightly AWS run leaves something running.** Expected: never past its TTL. Tests: the reaper's `reaper_deletes_expired` dry-run fixture (Task 9) and the `always()` teardown.

## Owner checklist

These are owner actions. No step puts a credential in the repository or in chat; values go into the AWS console or CLI and into GitHub's environment settings, by the owner.

### AWS (the account with $200 of credits for six months)

1. **Check the account's plan** (Billing and Cost Management → Free Tier). If it is on the **Free plan**, upgrade to the **Paid plan**: the Free plan closes the account after six months or when the credits run out and offers only some services; the credits still apply on the Paid plan (Q1095).
2. **Pick the CI region:** `us-east-1` (the defaults of §70 and §71 assume it).
3. **Create the GitHub OIDC identity provider** in IAM: provider URL `https://token.actions.githubusercontent.com`, audience `sts.amazonaws.com`.
4. **Create the role `loams-ci-byoc`** with:
   - a trust policy for `sts:AssumeRoleWithWebIdentity` from that provider, with conditions `token.actions.githubusercontent.com:aud = sts.amazonaws.com` and `token.actions.githubusercontent.com:sub = repo:ostrium-labs/loams:environment:byoc-aws`;
   - a maximum session duration of 1 hour;
   - the permissions policy `deploy/byoc/aws/ci-role-policy.json` from Task 9 (EC2, ECS, Lambda, DynamoDB, S3, KMS, Secrets Manager, SSM and IAM `PassRole` for the instance role only, limited to `us-east-1` and to resources tagged `loams:ci-run`).
5. **Create the budget:** AWS Budgets → cost budget `loams-ci`, $30 a month; alerts to the owner's address at 50%, 80% and 100% of actual and 100% of forecast; a **budget action** at 100% that applies the IAM policy `deploy/byoc/aws/ci-deny-policy.json` (deny `ec2:RunInstances`, `ecs:RunTask`, `ecs:CreateService`, `lambda:CreateFunction`) to `loams-ci-byoc`, with automatic execution.
6. **Run the one-time bootstrap** from your own machine, with your own AWS CLI session: `scripts/byoc/aws-matrix.sh bootstrap`. It creates the persistent test pieces (a KMS key `alias/loams-ci`, a state bucket `loams-ci-<account-id>`, the instance profile `loams-ci-instance`), tags them `loams:persistent=true`, and prints their names.
7. **Create the GitHub environment `byoc-aws`** in `ostrium-labs/loams` (Settings → Environments), restricted to the `dev` branch, with:
   - variables `AWS_ROLE_ARN` (the role's ARN) and `AWS_REGION` (`us-east-1`);
   - the secret `TS_OAUTH_CLIENT_SECRET` and the variable `TS_OAUTH_CLIENT_ID`: a Tailscale OAuth client with the `auth_keys` scope that mints **ephemeral, preauthorised** keys tagged `tag:byoc-ci` only. (Add `tag:byoc-ci` to the tailnet policy's `tagOwners`, allowed to reach only itself.)
8. **Turn on IAM outbound identity federation** (`GetWebIdentityToken`) only when ER2 reaches its tests; not needed for BY0.

Time: about an hour.

### Cloudflare

1. A Cloudflare account on **Workers Paid** ($5 a month; Containers need it). §69's nightly (Q1069) may share it.
2. A zone (or a subdomain of one Ostrium controls) for the phase 0 test hostname, an **Access** application on it with a service token for CI.
3. The GitHub environment `byoc-cloudflare` with `CLOUDFLARE_ACCOUNT_ID` (variable), `CLOUDFLARE_API_TOKEN` (secret: Workers Scripts, Containers, R2, Access and DNS edit on that zone only), `CF_ACCESS_CLIENT_ID` and `CF_ACCESS_CLIENT_SECRET` (secrets).
4. Record the phase 0 demo (Task 15).

### Google Cloud and Azure (still to come)

1. **Google Cloud:** a project with a billing account (the $300, 90-day trial or Google for Startups credits), a workload identity pool for GitHub Actions with a provider restricted to `repo:ostrium-labs/loams:environment:byoc-gcp`, a service account for CI, a $30 budget with alerts, and the GitHub environment `byoc-gcp`.
2. **Azure:** a subscription (Microsoft for Startups credits; the free account's $200 lasts only 30 days), an app registration with a federated credential for `repo:ostrium-labs/loams:environment:byoc-azure`, a role assignment on a resource group `loams-ci`, a $30 budget with an action group, and the GitHub environment `byoc-azure`.

Until these exist, Tasks 4 and 5 are tested by hand in a trial and their matrix rows stay off.

### Startup programmes (§70 §13.5)

The applications (Cloudflare for Startups at the end of BY0's first week; AWS Activate Founders and AWS open-source credits, Google for Startups and Microsoft for Startups at BY0 exit; Vercel for Startups at BY0 exit) and their wording are the owner's.

## BY0 — Works on each cloud (Tasks 0–17)

### Task 0 — Rulings

Check these and record the answers under "Rulings made during execution":
1. The engine reads GCS credentials from the VM's metadata server and Azure credentials from IMDS (managed identity) through `object_store` as `crates/loams-store/src/backend.rs` wires it, with no key in the environment. If not, the smallest change that makes it so, or the documented fallback (§70 §16).
2. Which of the pinned images (`deploy/pins/*.toml`, the `loams-fabric`, `loams-agentd` and Tailscale images) have arm64 manifests (D1384).
3. The public IPv4 hourly price on AWS, Google Cloud and Azure (for the guides' cost tables).
4. Whether the openraft + redb metastore's existing bucket snapshots (`docs/release/preview.md`, "the metastore … keeps its snapshots in the bucket") plus a copy of the redb log under a write pause are a consistent checkpoint, and how Live's redb and Durable's SQLite are copied consistently (SQLite's online backup API; redb under a write transaction). This fixes Task 11's method.
5. Cloudflare Containers: the kernel features House's L3 sandbox and Live's `isolated` mode need (Landlock, seccomp, user namespaces, cgroup v2 delegation), outbound internet, `cloudflared` (research P0.0). Recorded, not acted on: Task 14 depends on it, and `sandbox: off` for single-tenant installs is approved (Q1086).
6. The OpenTofu registry's module address rules for `ostrium-labs/terraform-loams-byoc` (Q1089), for BY1.

Commit: `docs: BY0 Task 0 rulings`.

### Task 1 — Refuse network file systems for the embedded stores (D1353)

Files:
- `crates/loams/src/store_guard.rs` (new): `pub fn check_local_block_fs(path: &Path) -> Result<(), StoreGuardError>`. On Linux it calls `statfs(2)` (through `nix` or `rustix`, whichever the workspace already has) and refuses `NFS_SUPER_MAGIC` (0x6969), `SMB_SUPER_MAGIC` (0x517B), `SMB2_MAGIC_NUMBER` (0xFE534D42), `CIFS_MAGIC_NUMBER` (0xFF534D42), `FUSE_SUPER_MAGIC` (0x65735546) and `V9FS_MAGIC` (0x01021997). On other platforms it returns `Ok`.
- `crates/loams/src/main.rs`: `Dev`, `Standalone` and `Cluster` call it on `--data-dir` before opening the metastore, Live's redb or Durable's SQLite, and exit 78 with: "the embedded stores need a local block disk; <path> is on <fs> (design §70 §5.2, D1353)". `LOAMS_UNSAFE_ALLOW_NETWORK_FS=1` skips it and logs a warning on every start (for experiments; unsupported).
- `docs/release/preview.md`: one sentence in "Durability" pointing at the rule.

Tests (`crates/loams/tests/store_guard.rs`):
- `refuses_nfs_data_dir`, `refuses_smb_data_dir`, `refuses_fuse_data_dir`: the magic-number table through an injectable `fn(&Path) -> io::Result<i64>`.
- `accepts_ext4_xfs_btrfs_tmpfs_overlay`.
- `override_env_skips_and_warns`.

Commit: `loams: refuse embedded stores on a network file system (BY0 Task 1, D1353)`.

### Task 2 — `loams store probe` (D1356)

Files:
- `crates/loams-store/src/probe.rs` (new): `pub async fn probe(store: &dyn ObjectStore, prefix: &Path) -> ProbeReport`. Steps, each recorded as `pass`, `fail` or `unknown` with a message:
  1. create-only PUT of `<prefix>/_probe/<ulid>/a` succeeds;
  2. the same create-only PUT fails with a precondition error;
  3. a conditional overwrite with the first ETag succeeds, and with a stale one fails;
  4. a range read returns the right bytes;
  5. list after write sees the object;
  6. a multipart upload of 2 × 5 MiB parts completes;
  7. everything under `_probe/<ulid>/` is deleted.
- `crates/loams/src/main.rs`: `loams store probe --bucket <url> [--json]`, reading credentials the way the engine does; exit 0 when 1–6 pass, 1 otherwise. Versioning is not probed here (`object_store` has no API for it); the cloud guides check it with the cloud CLI.
- `config/stores.sh` (PR #973's entrypoint) runs `loams store probe` once before the first start when `LOAMS_BUCKET` is not `file://`, and refuses to start on failure, writing the report to the log.

Tests:
- `probe_passes_on_memory_store`, `probe_passes_on_local_fs` (in `crates/loams-store`).
- `probe_reports_missing_conditional_put`: an `ObjectStore` wrapper that ignores `PutMode::Create`.
- `probe_cleans_up`.
- In `.github/workflows/release-compose.yml`, the existing RustFS profile job runs `loams store probe` against RustFS.

Commit: `loams-store: the bucket capability probe and loams store probe (BY0 Task 2, D1356)`.

### Task 3 — AWS VM files (D1356–D1359)

Files, following `docs/ecosystem/deploy-buttons.md`:
- `deploy/byoc/aws/README.md`: what it creates (one EC2 instance, an instance profile, a security group with no inbound rules), the permissions, the costs table from §70 §6.1, limits (single node, as PR #973 states them), and links to the guide.
- `deploy/byoc/aws/cloud-init.yaml` (`#cloud-config`): installs Docker and the compose plugin; downloads `loams-<version>-compose-bundle.tar.gz` and `SHA256SUMS` from the release (verify the asset names against `release-github.yml`), checks the checksum and the signature with `docs/release/loams-release-key.asc`; installs `loams-byoc-secrets` and a systemd unit `loams-byoc.service` (`After=docker.service`; `ExecStartPre=/usr/local/bin/loams-byoc-secrets aws`; `ExecStart=docker compose up -d`); writes `.env` with `LOAMS_VERSION`, `LOAMS_BUCKET=s3://<bucket>/loams`, `AWS_REGION`, and **no** `AWS_ACCESS_KEY_ID`; enables the unit. Inputs are cloud-init template variables documented in `variables.md`: `version`, `bucket`, `region`, `secret_prefix` (default `loams/<install>`), `profiles`.
- `deploy/byoc/aws/loams-byoc-secrets` (bash): `aws secretsmanager get-secret-value` with the instance role for `<secret_prefix>/tailnet-authkey`, `setup-token`, `api-keys` (optional), writing `TS_AUTHKEY` and `LOAMS_SETUP_TOKEN` into `.env` and `secrets/api-keys` with `umask 077`; never echoes a value; exits non-zero with the missing name.
- `deploy/byoc/aws/instance-policy.json`: `s3:GetObject`, `PutObject`, `DeleteObject`, `ListBucket`, `AbortMultipartUpload`, `ListMultipartUploadParts` on the one bucket and prefix; `kms:GenerateDataKey`, `kms:Decrypt` on the one key; `secretsmanager:GetSecretValue` on `<secret_prefix>/*`; nothing else (SSM's managed policy is attached separately and optional).
- `deploy/byoc/aws/variables.md`, `deploy/byoc/aws/CHANGELOG.md`.
- `CODEOWNERS`: `deploy/byoc/` and `deploy/cloudflare/` owned by the maintainers.

Tests (`tests/byoc/static/`, run in `.github/workflows/byoc-static.yml` on PRs that touch `deploy/byoc/**`, `deploy/cloudflare/**` or `tests/byoc/**`):
- `cloud_init_validates`: `cloud-init schema --config-file` on each cloud-init file.
- `cloud_init_has_no_inbound_rules`: the README's and guide's security-group commands have no `authorize-security-group-ingress`.
- `aws_env_has_no_access_key`: the rendered `.env` (Task 6's `--render-env-only`) contains no `AWS_ACCESS_KEY_ID` or `AWS_SECRET_ACCESS_KEY`.
- `instance_policy_is_scoped`: every statement names a resource ARN with the bucket, key or secret prefix; no `*` resource except where IAM requires it (`kms:` none).
- shellcheck on `loams-byoc-secrets`.

Commit: `deploy/byoc/aws: cloud-init, secrets fetch and instance policy (BY0 Task 3)`.

### Task 4 — Google Cloud VM files

The same layout as Task 3 in `deploy/byoc/gcp/`: `cloud-init.yaml` (passed as `user-data` metadata; Ubuntu's cloud-init reads it), `loams-byoc-secrets` with `gcloud secrets versions access latest --secret=<prefix>-tailnet-authkey` (Secret Manager names cannot contain `/`, so `loams-<install>-tailnet-authkey`), `.env` with `LOAMS_BUCKET=gs://<bucket>/loams` and no `GOOGLE_SERVICE_ACCOUNT_KEY`; `roles.md` naming the roles for the VM's service account (`roles/storage.objectUser` on the bucket, `roles/secretmanager.secretAccessor` on the install's secrets) and for the GCS service agent on the key (`roles/cloudkms.cryptoKeyEncrypterDecrypter`).

Tests: the static tests of Task 3 for this directory, including `gcp_env_has_no_service_account_key`.

Commit: `deploy/byoc/gcp: cloud-init, secrets fetch and roles (BY0 Task 4)`.

### Task 5 — Azure VM files

The same layout in `deploy/byoc/azure/`: `cloud-init.yaml` (passed with `--custom-data`), `loams-byoc-secrets` with `az login --identity` then `az keyvault secret show --vault-name <vault> --name <install>-tailnet-authkey --query value -o tsv`, `.env` with `LOAMS_BUCKET=az://<container>/loams`, `AZURE_STORAGE_ACCOUNT_NAME` and **no** `AZURE_STORAGE_ACCOUNT_KEY` (managed identity per Task 0 ruling 1); `roles.md` (`Storage Blob Data Contributor` on the container, `Key Vault Secrets User` on the vault, and the storage account's identity with `Key Vault Crypto Service Encryption User` on the key).

Tests: Task 3's static tests for this directory, including `azure_env_has_no_account_key`.

Commit: `deploy/byoc/azure: cloud-init, secrets fetch and roles (BY0 Task 5)`.

### Task 6 — `install.sh --cloud` (D1361)

Files:
- `deploy/release/byoc/install.sh` gains:
  - `--cloud aws|gcp|azure`: copies `deploy/byoc/<cloud>/loams-byoc-secrets` to the host, installs the systemd unit as cloud-init does, and runs the fetch once before the first start (so the tailnet key comes from the secret store, not the prompt);
  - `--bucket URL` (required with `--cloud`): writes `LOAMS_BUCKET` and the region or account variable, never a key;
  - `--secret-prefix P` (default `loams/<dir>` on AWS, `loams-<dir>` elsewhere);
  - `--render-env-only`: prints the `.env` it would write, with secret values replaced by `<from secret store>`, and connects to nothing (for tests).
- `deploy/release/README.md`: a "On a cloud VM" subsection pointing at `docs/guides/byoc/`.

Tests:
- `install_cloud_aws_renders_bucket_and_region`, `install_cloud_gcp_renders_bucket`, `install_cloud_azure_renders_account`: `--render-env-only` output against golden files in `tests/byoc/static/golden/`.
- The `*_env_has_no_*_key` tests of Tasks 3–5 use this output.
- shellcheck as today.

Commit: `byoc/install.sh: --cloud, --bucket and --render-env-only (BY0 Task 6)`.

### Task 7 — The VM guides

Files: `docs/guides/byoc/index.md` (what BYOC is, the three tiers, the persistence rule, the never-public rule, which guide to read, costs), `aws.md`, `gcp.md`, `azure.md`. Each cloud guide has the same sections, with copy-paste CLI commands only (no console click paths):
1. Before you start (CLI, a tailnet, a Tailscale auth key, the region).
2. The bucket (versioning, public access blocked, lifecycle for noncurrent versions after 30 days).
3. The key (create it; make it the bucket's default encryption; grant only the install's identity).
4. The secrets (create the tailnet key and the setup token in the secret store).
5. The identity (instance profile, service account or managed identity, with the policy or roles from `deploy/byoc/<cloud>/`).
6. The VM (create it with the cloud-init file and no inbound rules; or create it yourself and run `install.sh --cloud`).
7. Check it (`loams store probe` result in the log, the tailnet name, the console).
8. Backups (versioning plus a daily disk snapshot: Data Lifecycle Manager, a snapshot schedule, a snapshot policy; restoring into a new VM).
9. Upgrades (`byoc/upgrade.sh`, or the cloud's run-command; PR #973's note that embedded stacks are upgraded by hand for now).
10. Private reach (a pointer to §69 and to BY1 for the internal load balancer; not set up in BY0).
11. Costs (§70's table, dated).
12. Remove it (delete the VM; keep or delete the bucket and key).

Tests: `docs-snippets` check (the existing docs lint, if any; otherwise `tests/byoc/static/guides_reference_existing_files.sh` checks every `deploy/byoc/...` path a guide names exists).

Commit: `docs/guides/byoc: AWS, Google Cloud and Azure VM guides (BY0 Task 7)`.

### Task 8 — `loams support bundle` (D1376, D1581)

Files:
- `crates/loams-support/` (new, `publish = false`): `pub fn bundle(stack: &StackDir, out: &Path, opts: &BundleOptions) -> Result<Manifest>` using `loams_upgrade`'s `StackDir` and compose helpers. Contents per §70 §9.7: `versions.json`, `info.json` (`/v1/info`), `health.json`, `compose-ps.json`, `logs/<service>.log` (last 2,000 lines, through the audit redactor), `env.redacted` (values of keys matching `*KEY*`, `*SECRET*`, `*TOKEN*`, `*PASSWORD*`, `TS_AUTHKEY`, `LOAMS_SETUP_TOKEN` replaced by `<redacted>`), `secrets.names` (file names only), `probe.json` if a bucket is configured, `manifest.json` (every file and its SHA-256).
- `crates/loams/src/main.rs`: `loams support bundle [--stack-dir D] [-o FILE] [--list]`; `--list` prints the manifest without writing.
- `docs/guides/byoc/index.md`: "Sharing a support bundle": the bundle is a local file and the command sends nothing; review it with `--list`, then, if you choose, attach it to a GitHub issue on `ostrium-labs/loams` or share it on the Loams Discord (https://discord.gg/eK6NanHQq). No Ostrium support address (D1581).

Tests (`crates/loams-support/tests/`):
- `bundle_redacts_planted_secrets`: a fixture stack directory with planted values in `.env`, `secrets/` and a log line; the bundle contains none of them (byte search over every file).
- `bundle_reads_no_data`: the bundle never opens files under the data volume path.
- `bundle_lists_without_writing`.

Commit: `loams-support: loams support bundle (BY0 Task 8, D1376)`.

### Task 9 — The BYOC smoke and exposure suites, and the nightly AWS row (D1380)

Files:
- `tests/byoc/smoke.sh <engine-url>`: §70 §12.1 step 3 with the CLI and `curl` only: `loams store probe` result from `/v1/info`; create a collection, ingest 100 documents, a hybrid search through the Qdrant and ES APIs; a Live subscription receives one change; a durable workflow completes; `git push` to the bucket through `git-remote-loams`; a House query if House runs; a Postgres write and read if Postgres runs; `loams support bundle --list`.
- `tests/byoc/restart.sh <host>`: write a marker, `sudo systemctl restart loams-byoc` over SSM or the tailnet, check the marker survives.
- `tests/byoc/exposure.sh <public-ip>`: from the runner (outside the tailnet), a TCP connect scan of the 1,000 most common ports plus 8080, 8443, 6333, 9200, 5432, 4000 and 7781; fails on any open port.
- `scripts/byoc/aws-matrix.sh bootstrap|up|smoke|down|reap [--dry-run]`:
  - `up` launches `t3.large` from `deploy/byoc/aws/cloud-init.yaml` in a default-VPC public subnet with a security group of no inbound rules, tags `loams:ci-run=$GITHUB_RUN_ID` and `loams:ttl=<now+3h>`, a per-run bucket prefix in the state bucket, and a per-run secret holding an ephemeral Tailscale key minted from the OAuth client;
  - `down` terminates by tag and deletes the per-run prefix and secret;
  - `reap` deletes everything with `loams:ttl` in the past and reports untagged resources older than a day.
- `deploy/byoc/aws/ci-role-policy.json`, `deploy/byoc/aws/ci-deny-policy.json` (the owner checklist's policies).
- `.github/workflows/byoc-aws-nightly.yml`: `schedule` (03:30 UTC) and `workflow_dispatch`; environment `byoc-aws`; `permissions: id-token: write, contents: read`; `aws-actions/configure-aws-credentials` with `role-to-assume: ${{ vars.AWS_ROLE_ARN }}`, `role-duration-seconds: 3600`; joins the tailnet as an ephemeral `tag:byoc-ci` node; `up`, `smoke`, `restart`, `exposure`, then `down` in an `always()` step; `timeout-minutes: 50`; `concurrency: byoc-aws` (no parallel runs).
- `.github/workflows/byoc-reaper.yml`: hourly `reap` in the same environment.

Tests:
- `reaper_deletes_expired`, `reaper_keeps_persistent`, `reaper_reports_untagged`: `aws-matrix.sh reap --dry-run` against recorded `describe-*` JSON fixtures in `tests/byoc/fixtures/aws/`.
- `nightly_tears_down_in_always`: a workflow lint (`actionlint` plus a grep that `down` runs under `if: always()`).
- shellcheck on every script.
- **The cost check:** the nightly's runtime per run is printed; the first week's runs must average under $0.20 (estimate: $0.08 of EC2 plus cents), keeping the month near $3 and far under the $30 budget.

Commit: `tests/byoc: smoke, restart and exposure suites; the nightly AWS row (BY0 Task 9, D1380)`.

### Task 10 — Cloudflare spikes (research P0.0)

Run the PR #973 light image in a `standard-2` and a `standard-4` Container behind a minimal DO. Record: boot time and RSS; whether the engine reaches R2 through its S3 endpoint; outbound internet; `cloudflared`; the kernel features of Task 0 ruling 5; the R2 `If-None-Match` race (two concurrent create-only PUTs, exactly one wins, 100 trials). Output: a ruling per item and `deploy/cloudflare/SPIKES.md`.

Commit: `deploy/cloudflare: phase 0 spike results (BY0 Task 10)`.

### Task 11 — The checkpointed single node (D1354; approved for Cloudflare phase 0)

Files:
- `crates/loams/src/checkpoint.rs` (new): `Checkpointer` with `claim_epoch`, `checkpoint`, `restore_latest` and a background loop; the method fixed by Task 0 ruling 4. Layout in the bucket:
  - `_meta/epoch/<n:010>`: create-only, the claiming node's id and start time;
  - `_meta/checkpoints/<epoch:010>/<seq:020>/<part>`: the store files (content-addressed parts, unchanged parts referenced, not re-uploaded);
  - `_meta/checkpoints/<epoch:010>/<seq:020>.done`: create-only, last.
- `crates/loams/src/main.rs`: `--checkpoint-to-bucket` (env `LOAMS_CHECKPOINT=bucket`) and `--checkpoint-interval` (default `5s`, 1–60 s, Q1083) on `Standalone`. With it, start-up is: claim the next epoch → restore the newest complete checkpoint → open the stores → open the listeners. `SIGTERM` takes a final checkpoint before exit. At each checkpoint the node lists `_meta/epoch/`; if a newer epoch exists it refuses writes (503 with `superseded`) and exits 75.
- `/v1/info` gains `durability`: `"local"` or `"checkpointed(5s)"`.

Tests (`crates/loams/tests/checkpoint.rs`, on a local `file://` bucket and `InMemory`):
- `restore_after_graceful_stop_loses_nothing`.
- `restore_after_kill_loses_at_most_one_interval`: a writer acknowledges a write every 100 ms; `kill -9`; after restore, every write older than one interval plus the upload time is present.
- `superseded_node_self_fences`: start node A, start node B on the same bucket; B claims epoch n+1; A stops accepting writes within one interval and exits 75.
- `two_claims_one_winner`: concurrent `claim_epoch` calls; exactly one gets each number.
- `unchanged_parts_not_reuploaded`.
- `info_reports_durability`.

Commit: `loams: the checkpointed single node (BY0 Task 11, D1354)`.

### Task 12 — R2 conformance and Git on R2 (research P0.1, P0.5)

Files:
- `crates/loams-store/tests/r2.rs` behind `LOAMS_TEST_R2_*` environment variables (skipped without them; run by Task 15's workflow): the `loams-store` conformance cases and `probe` against R2.
- `deploy/cloudflare/src/git-credentials.ts`: a route on the front Worker, behind Access, that issues R2 temporary credentials scoped to `ns/<ns>/repos/` for one hour, so `git-remote-loams` never holds the account's R2 key.
- `docs/guides/byoc/cloudflare.md` § "Git on R2": `git push loams::s3://<bucket>/...` with those credentials, clone from a second machine.

Tests: `r2_conformance` (gated), `git_credentials_scope_is_prefix` (a unit test of the policy the Worker builds).

Commit: `loams-store, deploy/cloudflare: R2 conformance and scoped Git credentials (BY0 Task 12)`.

### Task 13 — The `deploy/cloudflare/` template and button (D1367, D1372)

Files, following the deploy-buttons convention:
- `deploy/cloudflare/README.md`: the "Deploy to Cloudflare" button (`https://deploy.workers.cloudflare.com/?url=https://github.com/ostrium-labs/loams/tree/<tag>/deploy/cloudflare`; verify the URL scheme), one line on what it creates, the version badge, the honest claim of §70 §7.5, limits (single node, preview, checkpoint window, no Postgres, 4 vCPU and 12 GiB), costs.
- `deploy/cloudflare/wrangler.jsonc`: the front Worker `loams-front`; `assets` from the console build (`web/apps/console`, `--env byoc`); DO class `LoamsNode` with a `containers` entry (image `./container/Dockerfile`, instance type `standard-2`, `max_instances: 1`); optional `HouseNode` (`standard-4`) behind a variable; R2 bucket binding `BUCKET`; `workers_dev: false`; a route on the customer's zone; secrets `LOAMS_SETUP_TOKEN`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `ACCESS_AUD`, `ACCESS_TEAM_DOMAIN`.
- `deploy/cloudflare/src/index.ts`: verifies `Cf-Access-Jwt-Assertion` against the team's JWKS and `aud` on **every** request (phase 0 puts the whole hostname behind Access; humans through the IdP, machines through service tokens), then forwards to `LoamsNode`. When ER0's `api/edge-allowlist.txt` exists, `lk_`-keyed requests on allow-listed routes may skip Access (BY3; D1372).
- `deploy/cloudflare/src/node.ts`: `LoamsNode extends Container`, default port 8080, `sleepAfter` at its maximum, an alarm keep-alive, environment for the container (`LOAMS_BUCKET=s3://<bucket>/loams`, `AWS_ENDPOINT_URL=https://<account>.r2.cloudflarestorage.com`, `AWS_REGION=auto`, `LOAMS_CHECKPOINT=bucket`), `SIGTERM` handling with the container's grace period.
- `deploy/cloudflare/container/Dockerfile`: `FROM ghcr.io/ostrium-labs/loams:<version>@sha256:<digest>`, entrypoint `loams standalone --checkpoint-to-bucket …`; nothing else added.
- `deploy/cloudflare/variables.md`, `CHANGELOG.md`, `package.json` (pinned `wrangler`, `@cloudflare/containers`).

Tests:
- `cloudflare_template_has_no_workers_dev_route`, `cloudflare_template_pins_digest`, `cloudflare_template_has_max_instances_1` (static, in `byoc-static.yml`).
- `front_refuses_without_access_jwt`, `front_refuses_wrong_aud`, `front_forwards_with_valid_jwt` (vitest with `@cloudflare/vitest-pool-workers`, keys made by the test).
- `wrangler deploy --dry-run` in `byoc-static.yml`.

Commit: `deploy/cloudflare: the phase 0 template and Deploy button (BY0 Task 13, D1367, D1372)`.

### Task 14 — House on Containers (research P0.4)

If Task 10 shows the sandbox's kernel features: House in `HouseNode` with Iceberg on R2 and the local catalog, HTTP 8123 through the front Worker only. If not: the same with `sandbox: off`, allowed for single-tenant installs (Q1086, owner 2026-10-21), behind the explicit variable `HOUSE_SANDBOX=off` that the README explains. A multi-tenant install never sets it.

Tests: `house_query_through_front` in the Task 15 run; `house_sandbox_off_requires_flag` (static).

Commit: `deploy/cloudflare: House in a Container (BY0 Task 14)`.

### Task 15 — Cloudflare phase 0 demo and nightly (research P0.6)

Files:
- `.github/workflows/byoc-cloudflare-nightly.yml` (environment `byoc-cloudflare`): `wrangler deploy` with a per-run name, `tests/byoc/smoke.sh` through the Access hostname with the CI service token, a container restart (`restart_after_graceful_stop_loses_nothing` end to end), `wrangler delete` and the per-run R2 prefix removed in `always()`.
- `docs/guides/byoc/cloudflare.md`: deploy, what runs where, the checkpoint window, Git on R2, House, costs, limits, removing it.
- The demo script in `deploy/cloudflare/DEMO.md` (deploy, ingest, hybrid search, a Live subscription, a durable workflow surviving a container restart, `git push` to R2, a House query); the recording is the owner's.

Tests: the nightly itself; `exposure` is not applicable (no IPs), replaced by `front_refuses_without_access_jwt` against the deployed hostname.

Commit: `deploy/cloudflare: the phase 0 nightly, guide and demo (BY0 Task 15)`.

### Task 16 — Vercel (D1373)

Files:
- `docs/guides/byoc/vercel.md`: what cannot run on Vercel and why; the four parts of §70 §8; connecting a Vercel project to a BYOC install through the Access hostname (`CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`, `LOAMS_ENDPOINT`, `LOAMS_API_KEY`, `LOAMS_NAMESPACE` as sensitive variables); Vercel PrivateLink when the install is on AWS (a pointer to §69 and BY1); Vercel OIDC from ER2; Hobby versus Pro.
- `web/examples/vercel-mastra-loams/`: a Mastra agent (`@mastra/core`, `@loams/client` with `accessServiceToken()` once ER0 ships it, plain headers before) deployed with `vercel deploy`, using Loams for retrieval and memory; a README; no keys committed.

Tests: `pnpm -C web/examples/vercel-mastra-loams build` in CI; a manual run against the AWS nightly install recorded in the guide.

Commit: `docs, examples: BYOC on Vercel (BY0 Task 16, D1373)`.

### Task 17 — BY0 exit

- The AWS nightly has passed five nights in a row and averages under $0.20 a run.
- The Cloudflare nightly has passed three nights in a row.
- The Google Cloud and Azure guides have each been followed once by hand (or are marked "tested by hand: pending account" until the owner's accounts exist).
- `docs/release/preview.md` gains a "BYOC on a cloud" row pointing at `docs/guides/byoc/`; `CHANGELOG.md` notes it.
- The track tables in `docs/plans/README.md` mark BY0 done.

## BY1 — Modules and container services (milestone level)

Gate: BY0; PR #973 merged. **8–10 engineer-weeks** (estimate).
- `deploy/tofu/modules/{aws,gcp,azure}-{vm,container}/` and `shared/`, examples, `tofu plan` on PRs, `apply`/`destroy` in the nightly; the mirror repository and registry listing (Q1089).
- `loams byoc deploy|status|upgrade|backup|bundle|destroy` (D1361) over the modules and `wrangler`; the estimate and the budget (D1381).
- The checkpointed node on ECS Fargate, a Cloud Run worker pool and Container Apps (Q1080).
- Postgres's remote storage in the cloud bucket (Neon `remote_storage`).
- `loams backup create|list|restore` into the bucket (D1364).
- The internal load balancer and PrivateLink, PSC and Private Link services in the modules (`private_reach`).
- The `factory` compose profile (D1379).
- The Vercel connectable-account integration (D1373).
- The Google Cloud and Azure nightly rows, once their accounts exist.

## BY2 — Kubernetes (milestone level)

Gate: OP1's Core level. **8–12 engineer-weeks after OP1** (estimate). The chart's `values-{eks,gke,aks}.yaml`; `{aws-eks,gcp-gke,azure-aks}` modules; the matrix's Kubernetes row with LOC; the Helm chart and operator published with the release; managed metastores as D58's backends ship (D1355). No cloud marketplace listings (D1580).

## BY3 — Cloudflare-native (milestone level)

Gate: BY0's Cloudflare phase 0 shipped. The research's phase 1 (4–8 weeks): `loams-meta-do` and Durable on DO SQLite (D1368), Jobs on Workflows and Queues (D1370), Git smart HTTP with a sequencer DO and an Artifacts mirror (D1369), `cloudflared` and Hyperdrive, K2 and Basin, `lk_` routes on the edge allow-list (D1372). Phase 2 (a quarter): Workers for Platforms for the multi-tenant Factory and marketplace.

## AB0 — App BYOC on Cloudflare and AWS (milestone level)

Design [§71](../design/71-app-byoc-free-tiers.md). Gate: none beyond CL1. **5–7 engineer-weeks** (estimate). Tasks, to be specified when it starts:
1. Rulings: item and message limits per target (§71 §6), `waitUntil` per target, Function URL streaming.
2. `loams.toml` `[app]` schema and its parser (`crates/loams` CLI side).
3. `sdks/typescript/packages/app/` (`@loams/app`): the port types, the Cloudflare adapters (D1, R2, Queues, Cron Triggers) and the AWS adapters (DynamoDB provisioned, S3, SQS, EventBridge Scheduler), the entry shims.
4. The ports conformance suite on Miniflare and on DynamoDB Local with ElasticMQ.
5. `loams app deploy|status|usage|logs|destroy` for `cloudflare` (wrangler) and `aws` (a generated OpenTofu root module; state in a bucket in the customer's account).
6. The free-tier table (`docs/guides/app-byoc/free-tiers.md`, `scripts/app-byoc/gen-free-tiers`, the nightly freshness check).
7. Cost guards v0: the refusal, DynamoDB capacity without autoscaling and the per-account ledger, Lambda reserved concurrency, an AWS Budgets budget.
8. `web/examples/app-byoc-notes` (Hono; items, blobs, a queue, a schedule).
9. The nightly app row: the example on a Cloudflare Free account and in the owner's AWS account, inside the free allowances (§71 §12), torn down in `always()`.

## AB1 and AB2 (milestone level)

**AB1** (7–9 engineer-weeks): Google Cloud, Azure and Vercel adapters and shims; the usage poller and the Meter link (D1403; gate ME1); the `loams` target (gate §24 F1, JB1); `[app.loams]` over the edge (gate ER0; ER2 for workload identity); Python handlers. **AB2** (4–6 engineer-weeks): the Factory's deploy stage (SF4), Studio's Deploy action, marketplace packages with `[app.targets]`, a Live split-app template.

## Rulings made during execution

**BY0 Tasks 0 (part), 3, 4 and 5 (2026-10-23).**

1. **Task 0 ruling 1 (identity without keys): yes on all three.** `object_store` 0.14.2, as `crates/loams-store/src/backend.rs` wires it: S3 falls back to IMDS role credentials; GCS falls back to the metadata server's service-account token when `GOOGLE_*` and the application default credentials are absent; Azure resolves IMDS managed identity with `AZURE_CREDENTIAL_TYPE=managed_identity` (and `AZURE_STORAGE_CLIENT_ID` for a user-assigned identity). The Azure account-key fallback of §70 §16 is not needed. The unit tests of `backend.rs` build the builders only; a real-account read is still to be done.
2. **The secrets fetcher uses `curl` and `jq`, not the cloud CLIs.** Ubuntu 24.04 has no `awscli` package (snap only), and the Google and Azure CLIs are large installs. Secrets Manager (SigV4 through `curl --aws-sigv4`), Secret Manager and Key Vault are plain REST, so one `lib/loams-byoc-secrets` serves the three clouds and is tested against a local mock (`tests/byoc/static/mock_cloud.py`). This replaces the plan's per-cloud scripts and their CLI calls.
3. **One script source, generated cloud-init files.** `deploy/byoc/gen-cloud-init.sh` builds each `cloud-init.yaml` from `lib/` and `<cloud>/config.env`, and CI fails if a committed file is out of date, so the scripts a VM runs are the files that are linted and tested. The `@@name@@` placeholders are filled by `deploy/byoc/render.sh` (strict value patterns, because the values reach a root-run script) or, later, by the OpenTofu module.
4. **AWS user-data is gzipped.** The file is about 20 KB and EC2 limits user-data to 16 KB; cloud-init accepts gzip, `render.sh --gzip` gives about 7 KB. Google Cloud (256 KB) and Azure (64 KB) take the plain file.
5. **Google Cloud scope is `cloud-platform`, not `storage-rw`.** Secret Manager's API rejects a narrower token. The IAM roles still limit the account to the bucket and the install's secrets.
6. **Docker comes from Docker's apt repository** (key checked by fingerprint), not Ubuntu's `docker.io`: `compose.yaml` uses `include:` and `env_file` with `path:`, which need Compose 2.24 or newer.
7. **The release bundle is verified against the project key before use:** `SHA256SUMS.asc` must carry a VALIDSIG for fingerprint 333AC94599D5A1EB2A416D1E83DC134D8CAF895D (the key in `docs/release/loams-release-key.asc`, embedded in the cloud-init file), and the bundle's SHA-256 must be in `SHA256SUMS`.
8. **Task 6 (`install.sh --cloud`):** `--bucket` takes the URL as written (`s3://`, `gs://`, `az://`), and the per-cloud flags are `--region` (aws), `--project` (gcp, default the VM's) and `--storage-account`, `--vault`, `--client-id` (azure), because the fetcher needs them. The script copies `deploy/byoc/lib/loams-byoc-secrets` and the unit to the host, writes `/etc/loams-byoc/config.env` and runs the fetch as root before the first start; `LOAMS_BYOC_OWNER` hands `.env` back to the ssh user who runs compose.
9. **Not done yet:** Task 0 rulings 2 (arm64 manifests), 3 (public IPv4 prices) and 4 (checkpoints), Tasks 1, 2, 6 to 17. `CODEOWNERS` does not exist in the repository, so no entry was added; the static checks run in `.github/workflows/byoc-static.yml`, which is not part of `CI required`.

**BY1 first slice: the VM modules (2026-10-23).**

1. **`deploy/tofu/modules/{aws-vm,gcp-vm,azure-vm,shared}`** and `deploy/tofu/examples/<module>` are built ahead of the rest of BY1, because the cloud-init files they render exist (BY0 Tasks 3 to 5, merged). `shared` reads `deploy/byoc/<cloud>/cloud-init.yaml` and fills the same placeholders with the same value patterns as `render.sh`; it is the only module that knows the template.
2. **Inputs differ from §70 §9.1 in two ways.** `tailnet_authkey_secret` is replaced by `secret_prefix` (the install reads `<prefix>/tailnet-authkey` by the naming convention of `loams-byoc-secrets`; AWS and Google Cloud look it up at plan time and fail if it is missing, Azure creates the vault and the VM waits for the secret), and `budget_monthly_usd`, `private_reach`, `tier` and `kms_key` as a create-or-existing pair are not built: `kms_key_arn` / `kms_key_id` exist, the rest are BY1 milestones. `estimated_monthly_usd` is an output.
3. **The setup token is created by the module** from `random_id` (its value is in the customer's state; `create_setup_token = false` turns it off). The Tailscale auth key is never a module input.
4. **Validation without an account:** `tofu fmt`, `tofu validate` on each example, `tofu test` with mock providers (`modules/*/tests/plan.tftest.hcl`: no inbound rule, IMDSv2 hop limit 2, private versioned encrypted bucket, shared keys off on Azure, no key in the rendered user data, `latest` refused) and `tflint` with the AWS, Google and Azure rulesets, in `.github/workflows/byoc-static.yml`. No `tofu apply` has been run on any module.
5. **Providers:** `hashicorp/aws ~> 6.0`, `hashicorp/google ~> 8.0`, `hashicorp/azurerm ~> 5.0`, `hashicorp/random ~> 3.6`, as dependencies from the OpenTofu registry; no HashiCorp BSL source is used. The mirror repository `ostrium-labs/terraform-loams-byoc` and the registry listing (Q1089) are not done; the modules read `../../../byoc/`, so the mirror's release workflow must copy `deploy/byoc/` beside them.
6. **Task 0 ruling 2 (arm64 manifests), checked 2026-10-23 against the registries:** every image of the light default stack has a linux/arm64 manifest: `loams`, `loams-agentd`, `loams-fabric`, `loams-pg-control` and `loams-postgres-storage` at 0.1.1 and the pinned digests, the Postgres compute image `loams-postgres-compute-v17@sha256:210b7141...`, `tailscale/tailscale:v1.90.6` and `nginx-unprivileged:1.29-alpine`. In the `tikv` profile `pingcap/pd` and `pingcap/tikv` v8.5.8 and `rustfs/rustfs:1.0.0` are multi-arch, but `ghcr.io/ostrium-labs/tidb:v8.5.8-loams.1` is a single (amd64) manifest. So an arm64 VM size is possible for the light default once someone runs it; it was not run, so the modules and guides still offer x86-64 only (D1384).

**Status at 2026-10-24 (owner paused BY1 here).**

- **Merged:** BY0 Tasks 3, 4, 5 (cloud-init, secrets fetch, policies per cloud), 6 (`install.sh --cloud`), 7 (VM guides), the bootstrap signature tests, Task 0 rulings 1 and 2, the exposure scan, the AWS matrix script, reaper and nightly workflows (part of Task 9), and the BY1 VM OpenTofu modules (`aws-vm`, `gcp-vm`, `azure-vm`, `shared`).
- **Not started:** Task 1 (network-file-system guard, D1353), Task 2 (`loams store probe`, D1356), Task 8 (`loams support bundle`), the rest of Task 9 (`tests/byoc/smoke.sh`, `restart.sh`, so the nightly runs only the exposure scan), Task 0 rulings 3 (public IPv4 prices), 4 (checkpoint method) and 6 (OpenTofu registry module address rules), Tasks 10 to 15 (Cloudflare), the Vercel guide, Task 11 (checkpointed node), Tasks 16 and 17, and the container, Kubernetes and `private_reach`/`budget_monthly_usd` parts of BY1.
- **Waiting on the owner:** the AWS CI role, budget and `byoc-aws` environment (owner checklist) before the nightly does anything; a first real-account apply per cloud. Nothing here has run on a real cloud account.
- **`byoc-static.yml`** stays a separate workflow, not part of `CI required` (not evaluated against the 3-minute bar before the pause).

