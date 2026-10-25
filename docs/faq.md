# FAQ

## Is this a Loams distribution?

No. It is a template: skills, a pinned documentation snapshot, a Rust starter,
and deployment examples. Loams itself is at
[`ostrium-labs/loams`](https://github.com/ostrium-labs/loams).

## Do we have to rewrite everything in Rust?

No. Rewrite the bounded contexts that own data and hot paths. Keep web, mobile,
data, and infrastructure edges on the SDK for their language. The 13 SDKs share
one runtime contract and conformance corpus.

## Do we have to use all 13 SDKs?

No. Use the ones your stack already has. The matrix is a menu, not a checklist.

## Where does our data live?

In your cloud account: your bucket, your disk, your KMS key. Loams runs on your
VM and is reachable only over your tailnet. Ostrium runs no data plane for a BYOG
install.

## Can we expose the Loams API to our customers?

Not directly. The engine is never public. If a product needs a public API, put a
company-owned gateway in front with its own auth, rate limits, and review.

## Is the light stack production-ready?

It is a single node with no replication. It is suitable for a pilot and small
production with backups and a written risk acceptance. Plan downtime for
upgrades and restores.

## Can we use Kubernetes?

The upstream one-VM guides are the supported path today. The Kubernetes tier is
not built. Do not design around it yet.

## How do we keep the design docs current?

`reference/` is pinned. Run `scripts/sync-reference.sh --ref <commit-or-tag>` and
review the diff. If you do not want upstream churn, run
`scripts/detach-upstream.sh`; the snapshot stays.

## How do agents find the skills?

Canonical skills are in `skills/`. `make skills` links them into
`.claude/skills`, `.opencode/skills`, and `.agents/skills`, and writes a Cursor
rule. The entry point is `skills/loams-adoption/SKILL.md`.

## Is the SDK stable?

It is 0.x. Pin the minor version and read the release notes. The runtime contract
is the stable part; the package surface may change.

## What about licensing?

Apache-2.0. The docs snapshot keeps upstream attribution in `NOTICE` and
`reference/upstream/SOURCE.md`.
