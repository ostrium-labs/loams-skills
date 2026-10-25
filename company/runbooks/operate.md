# Operate runbook

- Owner: CHANGE_ME
- On-call: CHANGE_ME
- Loams version: see `company/company.yaml`.
- Tailnet: `loams.<tailnet>.ts.net`.

## Health

| Check | Command | Expected |
|---|---|---|
| Engine | `curl https://loams.<tailnet>.ts.net/healthz` | 200 |
| Console | open `/ui/` | loads |
| Disk | cloud metric | below 80% |
| Bucket | cloud metric | versioning on |
| Backups | snapshot list | last < 24h |

## Dashboards

- Service RED metrics.
- Loams client: calls, retries, idempotency replays, consistency waits.
- Store: disk, RAM, IOPS, queue lag.
- Cost per run.

## Alerts

| Alert | Condition | Action |
|---|---|---|
| Engine down | healthz fails 2 min | page on-call |
| Disk > 80% | 15 min | expand disk / clean |
| Error burn | SLO burn > 2% | page service owner |
| Backup missed | no snapshot 24h | page platform |

## Escalation

1. Service owner.
2. Platform/ops.
3. Security for identity or key issues.
4. Loams upstream issue with logs and versions, secrets redacted.
