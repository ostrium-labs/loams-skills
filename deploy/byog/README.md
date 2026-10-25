# BYOG deployment kit

Bring your own ground: the stack runs in the company's cloud account, on a
block disk the company owns, with the company's bucket and keys. Ostrium runs no
data plane for this path. Read [`docs/byog-rules.md`](../../docs/byog-rules.md)
before changing anything here.

## What is supported

The supported tier is **one VM running the light compose stack**. The container
tier (ECS, Cloud Run, Container Apps), the Kubernetes tier, `loams byoc deploy`,
and the bucket capability probe are not built upstream. Do not plan around them.

The upstream installer is
[`deploy/release/byoc/install.sh`](https://github.com/ostrium-labs/loams/blob/v0.1.1/deploy/release/byoc/install.sh).
It installs Docker over SSH, copies the compose bundle, writes `.env`, starts
the stack, and waits for health. The VM needs **no inbound port**.

## Install

Pin `LOAMS_VERSION` and keep the `v` tag and the image tag in step. The
installer reads cloud identity from the VM; do not put cloud access keys in
`.env`.

```sh
# AWS: instance role; bucket is in the same account and region.
deploy/release/byoc/install.sh \
  --cloud aws --bucket s3://company-loams --region us-east-1 \
  --version 0.1.1 --yes ubuntu@10.0.0.10

# GCP: service account on the VM; project holds the secrets.
deploy/release/byoc/install.sh \
  --cloud gcp --bucket gs://company-loams --project company-prod \
  --version 0.1.1 --yes ubuntu@10.0.0.10

# Azure: managed identity; Key Vault holds the tailnet key and setup token.
deploy/release/byoc/install.sh \
  --cloud azure --bucket az://loams/company --storage-account companyprod \
  --vault company-prod-kv --version 0.1.1 --yes azureuser@10.0.0.10
```

Run it from a machine on the company tailnet. SSH to the VM is for the
installer only; operators use Session Manager, IAP, or `az vm run-command`.
`--render-env-only` prints the `.env` and `secrets/engine.env` without
connecting, which is the review step before a production install.

## Files

| File | Use |
|---|---|
| `.env.example` | The settings that matter for a company install. Copy the full upstream `.env.example` for the rest. |
| `cloud-init.example.yaml` | AWS example: clone the pinned tag, read secrets from Secrets Manager, start the light stack. Replace the placeholders. |
| `compose.override.yaml` | Optional log rotation and stop grace. Drop next to the release `compose.yaml`. |
| `runbook.md` | Install, verify, back up, restore, upgrade, destroy. |

## After install

1. Put the tailnet name in `company/company.yaml` and the deployment record.
2. Open `https://loams.<tailnet>.ts.net/ui/` from a tailnet device and complete
   setup with `LOAMS_SETUP_TOKEN`.
3. Create API keys for services; give each service its own principal and grant.
4. Point the Rust service at the endpoint and key; see `starter/`.
5. Schedule disk snapshots and bucket versioning; run a restore drill.

## Never do

- Open an inbound security-group, firewall, or NSG rule.
- Put `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, or a service-account JSON in
  `.env`.
- Use NFS, EFS, Azure Files, Filestore, or a FUSE bucket mount for the embedded
  stores.
- Set `LOAMS_VERSION=latest` in a company deployment.
- Restore over a live install; restore into a new install and cut over.
