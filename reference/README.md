# Reference snapshot

This directory is a **curated, pinned snapshot** of Loams documentation. It is
here so the skills and your agents can read the design rationale offline.

- Upstream: <https://github.com/ostrium-labs/loams>
- Pin, date, and license: [upstream/SOURCE.md](upstream/SOURCE.md)
- Hashes: [upstream/SHA256SUMS](upstream/SHA256SUMS)

## What is included

| Directory | Contents |
|---|---|
| `design/` | Architecture, the **full decision log**, unified API/SDKs, metering, BYOG, identity, cloud |
| `plans/` | The adoption-relevant plans: API/SDK, SDK parity, BYOG, identity, upgrades, cloud |
| `sdk/` | The 13-language matrix, Rust SDK guide, runtime contract, fixtures, toolchain |

Optional agent material is not copied by default. To add it:

```sh
scripts/sync-reference.sh --ref <commit-or-tag> --with-agents
```

That adds design docs 50, 52, 53, 56, 72, 73 and plans `cl1`, `ft1`, `mk1`.

## How to read it

Start with:

1. [Design 01: architecture](design/01-architecture.md)
2. [Design 44: unified API and SDKs](design/44-unified-api-and-sdks.md)
3. [Design 70: BYOG](design/70-byoc-clouds.md)
4. [Design 76: identity and authorization](design/76-identity-and-authorization.md)
5. [Decision log](design/13-decision-log.md)

The upstream docs sometimes link to pages outside this curated set, so not every
relative link resolves here. Follow the upstream repository for the full set.

## Do not edit

These files are overwritten by `scripts/sync-reference.sh`. Record company
decisions in `company/decisions/`, not here. The snapshot is Apache-2.0; see
`NOTICE` at the repository root.
