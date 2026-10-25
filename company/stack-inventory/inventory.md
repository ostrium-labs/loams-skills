# Stack inventory

Replace the example row with your services. Keep credentials and customer names
out of this file.

| Service | Owner | Language | Runtime | Criticality | Data stores | SLO | Disposition |
|---|---|---|---|---|---|---|---|
| example-api | platform | TypeScript | Kubernetes | tier-1 | postgres-main | 99.9% | unknown |

## Notes

- Criticality: `tier-1` (revenue), `tier-2` (internal), `tier-3` (best effort).
- Disposition: `rewrite`, `keep`, `bridge`, `retire`, `unknown`.
- Every `unknown` is a blocker at the fit gate.
