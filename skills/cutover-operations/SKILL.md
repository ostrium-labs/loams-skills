---
name: cutover-operations
description: "Prepare and run production cutover, rollback, backup and restore drills, upgrades, and day-2 operations for a Loams install. Use before the first production traffic and for every upgrade or incident."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Cutover and operations

A migration is not done at cutover; it is done when the team can restore, upgrade,
and diagnose it without the migration team.

## When to use

- Stage 5 and 6.
- A production change, upgrade, or restore drill.
- An incident needs a runbook that does not exist yet.

## Inputs

- `company/architecture/target.md`.
- The service cutover plan and the data migration report.
- `deploy/byog/` and the upstream release/upgrade guides.
- [Design 70](../../reference/design/70-byoc-clouds.md),
  [Design 63](../../reference/design/63-loams-meter.md).

## Cutover runbook

1. **Preconditions.** Backups current; restore drill passed; dashboards and
   alerts live; rollback owner and window named; change approved.
2. **Freeze.** Stop schema changes and non-essential deploys in the window.
3. **Canary.** Route a small percentage; watch errors, p95, saturation, and
   consistency.
4. **Ramp.** 1% → 10% → 50% → 100%, with a pause at each step.
5. **Verify.** Contract checks, sampled data parity, and a read-your-write check.
6. **Rollback.** If a gate fails: flip the flag, restore traffic, keep writes
   dual until the divergence is understood.
7. **Close.** Record start/end, incidents, decisions, and follow-ups. Remove the
   old path only after the window.

## Day-2 runbooks

- **Backup:** disk snapshot + bucket versioning; record the retention and the
  owner.
- **Restore:** new disk from snapshot, point the stack at it, verify, then
  reopen traffic. Time it.
- **Upgrade:** read notes, back up, pin the new version, upgrade, verify, keep
  the snapshot until stable.
- **Incident:** identify the layer (client, engine, store, network, identity),
  check the dashboard, capture logs, and escalate.
- **Capacity:** watch disk, RAM, bucket growth, and queue lag; scale before the
  limit.
- **Revocation:** disable the principal, delete tuples, rotate keys.
- **Support bundle:** collect logs, versions, and config with secrets redacted.

## Outputs

- `company/runbooks/cutover-<service>.md`.
- `company/runbooks/operate.md`.
- `company/runbooks/restore-drill.md`.
- `company/runbooks/upgrade.md`.
- `company/runbooks/incident.md`.

## Guardrails

- No cutover without a tested rollback.
- No upgrade without a backup.
- No restore drill on production data without approval.
- No runbook with a secret in it; reference the secret store.
- The light stack is single-node; plan for downtime, not failover.

## References

- [BYOG kit](../../deploy/byog/README.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
- [Migration playbook](../../docs/migration-playbook.md)
