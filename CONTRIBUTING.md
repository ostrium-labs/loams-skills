# Contributing

This is a public template. Companies are expected to copy it into a private
repository and change it there. Upstream contributions should improve the
template for everyone.

## Before you open a pull request

```sh
make check
```

That runs the skill validator, the reference hash check, the relative-link
check, the template hygiene check, and the Rust starter's fmt, clippy, and tests.

## Reference snapshot

Do not edit files under `reference/` by hand. They are a pinned snapshot. Refresh
them with:

```sh
scripts/sync-reference.sh --ref <commit-or-tag>
```

Review the resulting diff and update `reference/upstream/SOURCE.md` through the
script, not manually.

## Skills

- One directory per skill: `skills/<name>/SKILL.md`.
- The `name` in the frontmatter must match the directory.
- The description must say when to use the skill, not just what it is.
- Reference upstream material with relative links into `reference/`.
- Run `python3 scripts/validate-skills.py` after editing.

## Company forks

Do not send company-specific data, customer names, secrets, or internal
architecture to this public repository. Keep those in your private copy under
`company/`.
