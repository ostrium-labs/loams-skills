---
name: app-migration
description: "Migrate a service or bounded context to Rust on Loams with a strangler-fig plan: seams, API compatibility, dual-run, feature flags, canary, and rollback. Use when moving traffic from an existing service to the new implementation."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Application migration

Move one bounded context at a time. The old service stays the fallback until the
new one has earned the traffic.

## When to use

- Stage 3–5: a pilot or production service is being replaced.
- A monolith is being split.
- A rewrite needs a traffic-shift and rollback plan.

## Inputs

- The service's API contract and callers.
- `company/architecture/target.md` and the relevant ADRs.
- The migrated data plan (or a plan to dual-read during the transition).
- SLOs, error budget, and on-call owner.

## Procedure

1. **Find the seam.** Choose an interface the old service exposes: HTTP route,
   queue consumer, RPC method, or cron. Prefer a seam with a small contract and
   measurable traffic.
2. **Freeze the contract.** Capture requests/responses, status codes, errors,
   pagination, idempotency, and auth. Add contract tests before the rewrite.
3. **Build the new service** in Rust on Loams using `rust-workspace` and
   `loams-rust-sdk`.
4. **Dual-run.** Send a copy of traffic to the new service in shadow mode; do
   not serve its responses. Compare outputs, latency, and errors.
5. **Canary.** Route a small percentage, then ramp. Keep the old path warm.
6. **Cut over.** Flip the flag; monitor the SLOs and the error budget.
7. **Retire.** Remove the old path only after the rollback window and a clean
   postmortem-free period.

## Outputs

- `company/runbooks/cutover-<service>.md` — flag, ramp, abort, rollback.
- Contract tests and a shadow-comparison report.
- ADR for the rewrite and the cutover.

## Guardrails

- No big-bang cutover for a stateful service.
- No rewrite without a rollback switch that a human can flip.
- Do not change the public contract during the migration unless the ADR says so.
- If the old service is the only writer, dual-write or bridge; do not fork the
  source of truth.

## References

- [Migration playbook](../../docs/migration-playbook.md)
- [Design 01](../../reference/design/01-architecture.md)
- [Cutover and operations](../cutover-operations/SKILL.md)
