---
name: conformance-testing
description: "Design the test strategy for a Loams migration: unit, contract, integration, conformance, failure injection, load, and upgrade tests, with the gates that let a stage pass. Use when building the pilot or preparing for cutover."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Conformance testing

The question is not "does it compile?" but "does it behave the same under
failure, replay, and load?".

## When to use

- Stage 3, before the pilot is called done.
- Stage 4, before data migration.
- Stage 5, before cutover.
- A cross-language or upgrade regression appears.

## Inputs

- The service contract and SLOs.
- `reference/sdk/runtime-contract.md` and `reference/sdk/fixtures.md`.
- The company's test infrastructure and CI.

## Layers

| Layer | What it proves |
|---|---|
| Unit | Domain invariants; no I/O |
| Contract | The API shape and error codes callers depend on |
| Integration | The service against a real `loams dev` |
| Conformance | The SDK follows R1–R14 across languages |
| Failure injection | Timeouts, retries, duplicate writes, partial failure |
| Load | p95/p99, saturation, queue lag, recall |
| Upgrade | A version change does not break data or clients |
| Restore | A backup can be restored inside the RTO |

## Procedure

1. **Write the acceptance gates** per stage in
   `company/testing/acceptance.md`.
2. **Pin fixtures.** Use the shared corpus for SDK behaviour; add company
   fixtures for the domain.
3. **Test idempotency.** Send the same mutation twice; assert one effect and the
   same consistency token semantics.
4. **Test consistency.** Write, read in another process with the token; assert
   read-your-write when required.
5. **Inject faults.** Drop connections, delay, return `Unavailable`,
   `ResourceExhausted`, and `DeadlineExceeded`; assert bounded retries.
6. **Load test** at 2× expected peak; record p95/p99 and the saturation point.
7. **Upgrade test** on a copy: old data, new binary, old client, new client.
8. **Restore drill** from the disk snapshot and the bucket.
9. **Wire CI**: unit/contract on every PR; integration nightly; load and upgrade
   before a release.

## Outputs

- `company/testing/acceptance.md`.
- `company/testing/load-plan.md`.
- CI jobs and fixtures.
- A gate report per stage.

## Guardrails

- A green unit suite is not a conformance result.
- Do not test only the happy path.
- Do not load-test production without a stop switch and an owner.
- Do not accept an upgrade without a rollback path.

## References

- [Runtime contract](../../reference/sdk/runtime-contract.md)
- [Fixtures](../../reference/sdk/fixtures.md)
- [Migration playbook](../../docs/migration-playbook.md)
