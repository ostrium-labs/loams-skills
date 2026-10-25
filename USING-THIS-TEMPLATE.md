# Using this template

## 1. Create the private repository

Preferred (GitHub CLI):

```sh
gh repo create <your-org>/<your-repo> --private \
  --template ostrium-labs/loams-skills --clone
cd <your-repo>
```

Or use **Use this template → Create a new repository** on GitHub, choose
**Private**, and clone it.

A template is better than a fork here: you get a fresh history, you can choose
private visibility, and you are not tied to the upstream repository's settings.

## 2. Bootstrap the company

```sh
make bootstrap
```

The script asks for (or reads from flags):

- company name and slug;
- cloud (AWS, GCP, Azure);
- identity provider and OIDC issuer;
- the Loams version to pin;
- the languages you will keep at the edges.

It writes `company/company.yaml`, creates `company/decisions/D0001-adopt-loams.md`,
installs the skills for your agent, and prints the first three actions.

Use `scripts/bootstrap-company.sh --dry-run` to see what it would change.

## 3. Install the skills

```sh
make skills                 # all supported agents
scripts/install-skills.sh --target opencode
scripts/install-skills.sh --target claude
scripts/install-skills.sh --target codex
scripts/install-skills.sh --target cursor
scripts/install-skills.sh --target all --copy   # Windows / no symlinks
```

The canonical skills live in `skills/`. The installer links them into
`.claude/skills`, `.opencode/skills`, and `.agents/skills`, and writes a Cursor
rule that points at `AGENTS.md`.

## 4. Start the adoption

Open the repo in your agent and ask it to run the `loams-adoption` skill. It
reads `company/company.yaml`, checks the stage gates, and proposes the next
artifact.

The first three stages are always:

1. `stack-inventory` — systems, data, owners, SLAs, costs.
2. `loams-fit` — replace / keep / bridge, with an ADR per component.
3. `target-architecture` — namespaces, identity, data flows, BYOG topology.

Do not write Rust before stage 2 has decisions. Do not migrate data before a
pilot passes.

## 5. Keep or detach the upstream snapshot

Keep it if you want the design rationale and the decision log next to your
decisions. Refresh it:

```sh
scripts/sync-reference.sh --ref <commit-or-tag>
git diff reference/
```

Detach it if you want to stop tracking upstream:

```sh
scripts/detach-upstream.sh
```

That removes the `sync-upstream` workflow and leaves the pinned snapshot in
place, so the skills' local design references keep working. Delete `reference/`
by hand only if you also rewrite the links into it.

## 6. Configure CI and deployment

- `make check` before every pull request.
- Set `LOAMS_VERSION` in `deploy/byog/.env` (never `latest`).
- Follow `deploy/byog/README.md` for the one-VM light stack.
- Keep `.env` out of git. Use your cloud secret store.

## 7. Make it yours

- Replace the fictional example in `company/stack-inventory/`.
- Add ADRs under `company/decisions/`; do not edit the upstream decision log.
- Adapt `starter/` to your first bounded context.
- Keep `company/AGENTS.md` current; it is the first file an agent reads.
