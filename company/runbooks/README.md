# Runbooks

Write these before the stage that needs them:

| Runbook | When | Minimum content |
|---|---|---|
| `migration-<store>.md` | Stage 4 | Mapping, order, backfill, verify, rollback |
| `cutover-<service>.md` | Stage 5 | Flag, ramp, abort, rollback, owner |
| `operate.md` | Stage 6 | Health, dashboards, alerts, on-call, escalation |
| `restore-drill.md` | Stage 6 | Snapshot, restore, verify, measured RTO |
| `upgrade.md` | Stage 6 | Backup, pin, upgrade, verify, rollback |
| `incident.md` | Stage 6 | Severity, diagnosis, escalation, comms |

No runbook contains a secret. Reference the secret store.
