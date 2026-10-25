# BYOG rules

BYOG is **bring your own ground** (the upstream docs also say BYOC, bring your
own cloud). Loams runs in the company's account, on the company's disk, with the
company's bucket and keys. Ostrium runs no data plane for this path.

## The rules

### 1. Never public

- No inbound security-group, firewall, or NSG rule.
- The VM may have a public address for **outbound** traffic only.
- Reach the engine over the company tailnet at
  `https://loams.<tailnet>.ts.net`.
- A shell is through Session Manager, IAP, or `az vm run-command`, not SSH from
  the internet.

### 2. Own account, own data

- Dedicated cloud project/subscription.
- Bucket owned by the company, with versioning.
- KMS key owned by the company.
- Secrets in the company's secret store.
- No hosted control plane.

### 3. No static cloud keys

Use the instance role (IMDSv2, hop limit 2), the service account metadata token,
or the managed identity. Do not put access keys in `.env`.

### 4. Pin and verify

- Set `LOAMS_VERSION` to an exact release. `latest` is refused.
- Verify the release signature (`SHA256SUMS.asc`) and the bundle
  (`SHA256SUMS`).
- Record the version in the deployment record.

### 5. Local block disk

The embedded stores (metastore, Live, Durable) need a local block disk: EBS,
Persistent Disk, or Azure managed disk. Never NFS, EFS, Azure Files, Filestore,
or a FUSE mount of a bucket.

### 6. Single node is not replicated

The light default stack is one node on one disk. Back up the disk and keep the
bucket versioned. Plan downtime, not failover.

### 7. Back up and drill

- Disk snapshot on a schedule.
- Bucket versioning on.
- A restore drill before production traffic.
- Keep the old snapshot through the upgrade window.

### 8. Upgrade deliberately

Read the notes, back up, pin, upgrade, verify, and keep the rollback path.

## What is not built yet

The upstream guide states the container tier (ECS, Cloud Run, Container Apps),
the Kubernetes tier, `loams byoc deploy`, the bucket capability probe, the
network-filesystem guard, and the support bundle are not built. Do not promise
them. Use the one-VM light stack.

## Deployment record

Keep this next to the runbook:

| Field | Value |
|---|---|
| Cloud / region | |
| VM / disk | |
| Loams version | |
| Tailnet / node | |
| Bucket / KMS key | |
| Secret store | |
| Owner / on-call | |
| Backup schedule | |
| Last restore drill | |
