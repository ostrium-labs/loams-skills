# Agent instructions

This repository is a **template for adopting Loams**. Read this file first, then
`company/AGENTS.md` if it exists.

## Ground rules

- **Never expose Loams to the public internet.** No inbound rule. Tailnet only.
- **Never commit secrets.** `.env` is ignored; use the cloud secret store.
- **Company data stays in the company account.** Do not add a hosted control
  plane or send data to Ostrium.
- **Do not edit `reference/`.** It is a pinned upstream snapshot. Use
  `scripts/sync-reference.sh`.
- **Do not invent Loams APIs.** Read `reference/` and the Rust SDK docs; the SDK
  is 0.x and its public surface is what compiles.
- **Record decisions.** Every replace/keep/bridge choice gets an ADR in
  `company/decisions/`.

## Workflow

1. Read `company/company.yaml` and the relevant `company/` artifacts.
2. Run the matching skill from `skills/`.
3. Produce files, not just prose. Put company artifacts under `company/`.
4. Run `make check` before you finish.
5. Report the gate status and the next action.

## Skills

Start with `skills/loams-adoption/SKILL.md`. The installer links skills into the
agent-specific directories; the canonical copy is always `skills/`.

## Code

- Rust: edition 2024, Rust 1.97+, `starter/` conventions.
- Do not add a dependency without checking the workspace already has an
  equivalent.
- Do not add comments that restate the code.
- Keep the starter compilable: `cd starter && cargo check`.

## Safety

- A destructive migration or cutover needs an explicit human approval and a
  tested rollback.
- A production deploy needs a pinned version, a verified artifact, a backup, and
  a restore drill.
