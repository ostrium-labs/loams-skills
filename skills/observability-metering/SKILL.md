---
name: observability-metering
description: "Instrument a Loams-backed stack: structured logs, metrics, traces, OTLP, dashboards, SLOs, alerts, usage metering, and cost per run. Use when a service goes to production or when cost and reliability need a baseline."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Observability and metering

You cannot operate or price what you cannot see.

## When to use

- A pilot or production service is being instrumented.
- A cost, latency, or error-budget question needs an answer.
- A tenant or agent needs usage accounting.

## Inputs

- `company/architecture/target.md`.
- The service's SLOs and the company's existing observability stack.
- [Design 63: Meter](../../reference/design/63-loams-meter.md),
  [Design 01](../../reference/design/01-architecture.md).

## Procedure

1. **Logs.** Structured JSON with request id, tenant, principal, operation,
   duration, outcome. Never log documents, tokens, embeddings, or PII.
2. **Metrics.** RED (rate, errors, duration) per endpoint; USE (utilization,
   saturation, errors) per store; Loams client metrics (calls, retries,
   idempotency replays, consistency-token waits).
3. **Traces.** OTLP spans across the service, the Loams client, and the engine
   where available. Propagate request id and tenant.
4. **Dashboards.** One per service and one per store. Show p50/p95/p99, error
   rate, saturation, queue lag, search recall proxies, and cost.
5. **SLOs and alerts.** Define the SLO, the error budget, and pages that are
   actionable. Alert on burn rate, not on every blip.
6. **Metering.** Record usage per tenant, agent, and operation in Loams Meter or
   the company's billing pipeline. Define the unit (request, GB, vector query,
   agent run) and the price basis.
7. **Cost per run.** Attribute compute, storage, egress, and model tokens. Set a
   budget and an alert per tenant or product.
8. **Retention.** Logs, traces, and audit have different retention; record them
   and delete on schedule.

## Outputs

- `company/observability/dashboards.md` and exported dashboard JSON if the
  company uses one.
- `company/observability/slos.md`.
- `company/observability/metering.md` with units and cost model.
- Alerts wired to the on-call rotation.

## Guardrails

- No secrets or PII in telemetry.
- No unbounded cardinality (no user id, request id, or raw query as a metric
  label).
- Metering must be idempotent; a retry must not bill twice.
- Do not page on a metric nobody can act on.

## References

- [Design 63](../../reference/design/63-loams-meter.md)
- [Design 01](../../reference/design/01-architecture.md)
- [Cutover and operations](../cutover-operations/SKILL.md)
