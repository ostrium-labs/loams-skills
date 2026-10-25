# Company agent instructions

Read `AGENTS.md` at the repository root first. This file adds company-specific
rules.

## Context

- Company: see `company/company.yaml`.
- Cloud and region: see `company/company.yaml`.
- Stage: see `company/adoption-status.md`; if missing, run `loams-adoption`.
- Decisions: `company/decisions/`. Read the relevant ADRs before proposing a
  change.

## Rules

- Do not invent a stage or skip a gate. If evidence is missing, say so.
- Do not put customer names, data, or secrets in this repository.
- Do not change a production boundary without an ADR and a named owner.
- Do not expose Loams publicly; the only path is the company tailnet.
- Do not run a destructive migration or cutover without explicit human approval.
- Keep `company/adoption-status.md` current after each stage.

## Outputs

Write artifacts, not just chat. Use the paths the skill names. Link them from
`company/adoption-status.md`.
