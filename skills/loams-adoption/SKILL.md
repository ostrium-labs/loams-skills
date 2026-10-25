---
name: loams-adoption
description: "Plan, resume, or audit a company-wide migration to Loams on the company's own ground, with Rust for core services and the 13 SDKs at the edges. Use when starting adoption, deciding the next stage, or checking a migration gate."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Loams adoption

This is the entry-point skill. It does not replace the specialist skills; it
sequences them and keeps the program honest.

## When to use

- A company has copied this template and does not know where to start.
- A migration is mid-flight and someone asks "what is next?".
- A stage claims to be done and needs a gate review.

## Inputs

- `company/company.yaml` (company, cloud, IdP, Loams version, edge languages).
- `company/AGENTS.md` if present.
- Existing artifacts under `company/stack-inventory/`, `company/architecture/`,
  `company/decisions/`, `company/runbooks/`.
- [Migration playbook](../../docs/migration-playbook.md).

## Procedure

1. **Read the config.** If `company/company.yaml` is still the example, run the
   `stack-inventory` skill first or ask the user to run `make bootstrap`.
2. **Determine the stage** from artifacts, not from claims:

   | Stage | Evidence | Gate |
   |---|---|---|
   | 0 Baseline | `stack-inventory/inventory.md` + `data-map.md` | Owners, criticality, SLAs, costs recorded |
   | 1 Fit | `architecture/fit.md` + one ADR per component | Every component is replace/keep/bridge/defer |
   | 2 Target | `architecture/target.md` | Tenancy, identity, data flow, BYOG topology reviewed |
   | 3 Pilot | A Rust service in `starter/` or a new crate | E2E tests pass; latency and cost baselines recorded |
   | 4 Data | `runbooks/migration-<system>.md` | Backfill verified; dual-write and rollback tested |
   | 5 Cutover | `runbooks/cutover-<service>.md` | Canary healthy; rollback ready; owner on call |
   | 6 Operate | `runbooks/operate.md` + `runbooks/restore-drill.md` | Restore drill passed; upgrade tested |
   | 7 Optimize | `company/metering/` | Cost per run and SLOs within target |

3. **Name the next artifact.** One stage at a time. Do not start Rust before
   stage 1 decisions exist. Do not migrate data before the pilot passes.
4. **Write or update** `company/adoption-status.md` with: stage, evidence links,
   blockers, owner, next three actions.
5. **Hand off** to the specialist skill for the next stage.

## Outputs

- `company/adoption-status.md`
- A short next-action list with owners and dates.

## Guardrails

- A "replace" decision without an ADR is not a decision.
- A production cutover without a tested rollback is not approved.
- Never expose Loams publicly; never commit secrets.
- Do not rewrite a service in Rust just because the template is Rust. Rewrite
  where Loams changes the data path, the hot path, or the ownership boundary;
  keep the rest on its SDK.

## References

- [Migration playbook](../../docs/migration-playbook.md)
- [Design 01: architecture](../../reference/design/01-architecture.md)
- [Design 70: BYOG](../../reference/design/70-byoc-clouds.md)
- [Decision log](../../reference/design/13-decision-log.md)
