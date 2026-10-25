# Loams fit

One row per component. Link the ADR. A `replace` without a migration and rollback
path is not approved.

| Component | Access pattern (QPS, p95, size) | Loams primitive | Decision | ADR | Risk |
|---|---|---|---|---|---|
| example-search | 200 QPS, 80 ms, 120 GB | Collections + search | replace | D0002 | recall |

## Decision key

- `replace` — Loams owns the data and the API.
- `keep` — not a fit now; leave it alone or use a thin SDK.
- `bridge` — Loams in front, old store behind, with an expiry.
- `defer` — missing fact; name it and the owner.

## Open facts

| Fact | Owner | Needed by |
|---|---|---|
| | | |
