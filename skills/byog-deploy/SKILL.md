---
name: byog-deploy
description: "Deploy Loams on the company's own ground (BYOG): AWS, GCP, or Azure one-VM light stack, tailnet access, no inbound ports, workload identity, pinned artifacts, backups, and upgrades. Use when installing, changing, or auditing a company Loams deployment."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# BYOG deployment

BYOG means **bring your own ground**: the company's cloud account, bucket, KMS
key, secret store, and tailnet. Ostrium runs no data plane for this install.

## When to use

- Installing Loams for a pilot or production.
- Reviewing a deployment against the never-public rules.
- Planning an upgrade, backup, or restore drill.

## Inputs

- `company/company.yaml` (cloud, region, Loams version).
- [BYOG rules](../../docs/byog-rules.md).
- Upstream: `docs/guides/byoc/{index,aws,gcp,azure}.md` and
  `deploy/release/` in the Loams repository.
- [Design 70](../../reference/design/70-byoc-clouds.md).

## Choose a shape

| Shape | Use | Status |
|---|---|---|
| `loams dev` | Laptop, tests, skills | Available |
| One VM, light stack | Pilot and small production | Guides exist; not yet run on real cloud accounts |
| One VM, `tikv`/`rustfs` profiles | More data, more RAM/disk | Same caveat |
| Kubernetes / ECS / Cloud Run | Larger production | Not built yet; do not promise |

Start with one VM. It runs the engine on embedded stores, House, Loams Postgres,
agentd, and the Tailscale sidecar. It is single-node and not replicated.

## Procedure

1. **Prepare the account.** Create a dedicated project/subscription, a bucket
   with versioning, a KMS key, a secret store, and a service identity.
2. **Network.** No inbound rule. Egress only for the tailnet, registry, and
   GitHub. Reach the engine over `https://loams.<tailnet>.ts.net`.
3. **Pin.** Set `LOAMS_VERSION` to an exact release. Never `latest`.
4. **Install.** Prefer the upstream guide:
   - CLI pieces + cloud-init: `deploy/byoc/<cloud>/`
   - OpenTofu: `deploy/tofu/modules/<cloud>-vm/`
   - Existing VM: `deploy/release/byoc/install.sh --cloud <cloud>`
5. **Verify.** Check the release signature and `SHA256SUMS`, the tailnet node,
   `/healthz`, the console, and that no host port is bound to `0.0.0.0`.
6. **Persist.** Put the embedded stores on a local block disk (EBS, Persistent
   Disk, Azure managed disk). Never NFS, EFS, Azure Files, Filestore, or FUSE.
7. **Back up.** Snapshot the disk and version the bucket. Write the restore
   drill before production traffic.
8. **Upgrade.** Read the upgrade notes, back up, pin the new version, upgrade,
   verify, and keep the old disk snapshot until the window closes.
9. **Record** `company/runbooks/operate.md` and
   `company/runbooks/restore-drill.md`.

## Outputs

- `deploy/byog/.env` (not committed) from the example.
- A deployment record: cloud, region, VM, disk, version, tailnet, bucket, KMS,
  secret store, owner.
- `company/runbooks/operate.md` and `restore-drill.md`.

## Guardrails

- Never open an inbound port.
- Never use static cloud keys; use the instance role, metadata token, or managed
  identity.
- Never run the single-node stack as if it were replicated.
- Never upgrade without a backup and a restore path.
- The cloud-init and compose files here are examples; upstream owns the
  authoritative deploy code.

## References

- [BYOG rules](../../docs/byog-rules.md)
- [BYOG kit](../../deploy/byog/README.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
- [Design 71](../../reference/design/71-app-byoc-free-tiers.md)
