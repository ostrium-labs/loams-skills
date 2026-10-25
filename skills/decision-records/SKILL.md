---
name: decision-records
description: "Record company architecture decisions for a Loams migration as ADRs, including the upstream decision they depend on, alternatives, consequences, and deviations. Use whenever a replace/keep/bridge choice, tenancy model, or cutover is decided."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Decision records

The upstream decision log explains Loams. Your ADRs explain your company.

## When to use

- Any fit decision (`replace`, `keep`, `bridge`, `defer`).
- Tenancy, identity, deployment shape, consistency, or cutover.
- A deviation from an upstream default or a Loams design decision.

## Inputs

- [Upstream decision log](../../reference/design/13-decision-log.md).
- The relevant design section.
- The company's constraints and the alternatives considered.

## Procedure

1. **Pick the next id.** `company/decisions/D0001-<slug>.md`, monotonically
   increasing. Never renumber.
2. **Write the context.** The forces: traffic, data, compliance, team, cost,
   deadline.
3. **State the decision** in one sentence, present tense.
4. **List alternatives** and why each was rejected.
5. **Record consequences**: positive, negative, and the migration cost.
6. **Link upstream.** Cite the Loams design section or `D####` the decision
   depends on, or say "no upstream decision; company-specific".
7. **Status.** `proposed`, `accepted`, `superseded by D####`, or `rejected`.
8. **Review.** An accepted decision that changes a production boundary needs a
   named owner and a date.

## Template

```markdown
# D0001: Adopt Loams for the company data platform

- Status: proposed
- Date: YYYY-MM-DD
- Owner: <name/role>
- Upstream: design 01, design 70; no single D-number

## Context
...

## Decision
...

## Alternatives
- ...
- ...

## Consequences
- Positive: ...
- Negative: ...
- Migration: ...

## Follow-up
- ...
```

## Outputs

- `company/decisions/D####-<slug>.md`.
- An index row in `company/decisions/README.md`.

## Guardrails

- Do not edit the upstream decision log; it is a snapshot.
- Do not use an ADR to hide an unowned risk.
- A "defer" ADR names the missing fact and who will supply it.
- A deviation from upstream is allowed; an undocumented deviation is not.

## References

- [Upstream decision log](../../reference/design/13-decision-log.md)
- [Design 01](../../reference/design/01-architecture.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
