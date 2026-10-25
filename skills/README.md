# Skills

Sixteen [Agent Skills](https://agentskills.io) drive the adoption. Each is a
directory with a `SKILL.md` and YAML frontmatter. The canonical copy is here;
`scripts/install-skills.sh` links it into your agent's directory.

## Install

```sh
make skills
# or
scripts/install-skills.sh --target opencode
scripts/install-skills.sh --target claude
scripts/install-skills.sh --target codex
scripts/install-skills.sh --target cursor
scripts/install-skills.sh --target all --copy
```

## The suite

| Skill | Stage | Purpose |
|---|---|---|
| [loams-adoption](loams-adoption/SKILL.md) | all | Entry point, stage detection, gates |
| [stack-inventory](stack-inventory/SKILL.md) | 0 | Services, data, owners, costs |
| [loams-fit](loams-fit/SKILL.md) | 1 | Replace / keep / bridge / defer |
| [target-architecture](target-architecture/SKILL.md) | 2 | Tenancy, schemas, flows, topology |
| [rust-workspace](rust-workspace/SKILL.md) | 3 | Crates, errors, config, lints |
| [loams-rust-sdk](loams-rust-sdk/SKILL.md) | 3 | The Rust data path |
| [data-migration](data-migration/SKILL.md) | 4 | Backfill, dual-write, verify |
| [app-migration](app-migration/SKILL.md) | 5 | Strangler-fig cutover |
| [sdk-portability](sdk-portability/SKILL.md) | 1–5 | The 13 SDKs at the edges |
| [byog-deploy](byog-deploy/SKILL.md) | 2, 6 | Own-ground deployment |
| [identity-authz](identity-authz/SKILL.md) | 2 | Zitadel, OpenFGA, agents |
| [observability-metering](observability-metering/SKILL.md) | 5–7 | SLOs, dashboards, cost |
| [conformance-testing](conformance-testing/SKILL.md) | 3–5 | Tests and gates |
| [decision-records](decision-records/SKILL.md) | all | Company ADRs |
| [cutover-operations](cutover-operations/SKILL.md) | 5–6 | Cutover, rollback, restore |
| [agent-platform](agent-platform/SKILL.md) | optional | Agents, sandbox, factory |

## Authoring

- One directory per skill; the directory name equals `name`.
- The description says **when to use** it, not just what it is.
- Link upstream material with relative paths into `../reference/`.
- Run `python3 scripts/validate-skills.py`; it writes `manifest.json`.
