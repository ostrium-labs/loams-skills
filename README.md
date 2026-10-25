# Loams Skills Template

**Adopt Loams on your own ground, rewrite your stack in Rust, and keep the edges you already have.**

This is a public GitHub template for companies that want to run [Loams](https://loams.dev)
in their own cloud account and migrate their services onto it. It gives your team
and your coding agents the same starting point:

- a **migration playbook** with stage gates;
- **16 agent skills** that drive inventory, fit, architecture, Rust, data
  migration, deployment, identity, testing, and cutover;
- a **pinned knowledge base** of the Loams design docs, the full decision log,
  and the SDK contract;
- a **compilable Rust starter** with a sample Loams-backed service;
- a **BYOG deployment kit** that follows the never-public, own-account rules.

> **Never expose Loams to the public internet.** A BYOG install has no inbound
> rule and is reached over your own tailnet. Company data stays in the company's
> cloud account.

## Use this template

Create a **private** repository from this template, then bootstrap it:

```sh
gh repo create <your-org>/<your-repo> --private \
  --template ostrium-labs/loams-skills --clone
cd <your-repo>
make bootstrap          # fills company/company.yaml, installs skills, creates D0001
```

Or use the **Use this template** button on GitHub and clone the result. See
[USING-THIS-TEMPLATE.md](USING-THIS-TEMPLATE.md) for the full checklist.

Then open the repository in your agent (opencode, Claude Code, Codex, Cursor) and
start with the **`loams-adoption`** skill. It reads `company/company.yaml` and
tells you the next stage.

## What is inside

| Path | What it is |
|---|---|
| [`skills/`](skills/) | Canonical Agent Skills. `make skills` installs them for your agents. |
| [`docs/migration-playbook.md`](docs/migration-playbook.md) | The seven-stage migration method and its gates. |
| [`reference/`](reference/) | Curated, pinned Loams design/plan/SDK snapshot with hashes. |
| [`starter/`](starter/) | Rust workspace: `actix-web` API over the `loams` crate, tests, env config. |
| [`deploy/byog/`](deploy/byog/) | Bring-your-own-ground examples and runbook. |
| [`company/`](company/) | The parts you fill in: inventory, architecture, decisions, runbooks. |
| [`scripts/`](scripts/) | Bootstrap, snapshot sync, skill install, validation. |

## Quick start (local)

```sh
# 1. Install the skills for your agent(s)
make skills

# 2. Start a local Loams
loams dev

# 3. Run the starter against it
cd starter
cp .env.example .env
cargo run
curl localhost:3000/healthz
```

The starter creates its namespace and collection idempotently, then exposes
notes CRUD and hybrid search. It uses `Loams::from_env()` and session
consistency, so a read sees the process's own write.

## The 13 SDKs

You do **not** have to rewrite every service in Rust. Rewrite the bounded
contexts that own data and hot paths in Rust; point the rest at Loams with the
SDK for their language. All SDKs are generated from the same protos and share a
conformance corpus.

| Language | Install | Typical use |
|---|---|---|
| Rust | `cargo add loams` | The rewrite, new services, data paths |
| TypeScript | `npm install @loams/client` | Web/BFF, Studio plugins, Mastra agents |
| Python | `pip install loams` | Data/ML, notebooks, pipelines |
| Go | `go get loams.dev/go@v0.1.1` | Infrastructure, CLIs, operators |
| C# | `dotnet add package Loams` | .NET services |
| Java | `dev.loams:loams:0.1.1` | JVM services |
| Kotlin | `dev.loams:loams-kotlin:0.1.1` | Kotlin/JVM and Android |
| Swift | SwiftPM from the Loams repo, `0.1.1` | Apple clients |
| Objective-C | `pod 'Loams', '0.1.1'` | Legacy Apple clients |
| Dart | `dart pub add loams` | Flutter clients |
| Ruby | `gem install loams` | Rails services |
| PHP | `composer require loams/loams` | PHP services |
| C++ | release tarball, `find_package(loams CONFIG)` | Native services |

See [`reference/sdk/README.md`](reference/sdk/README.md) and
[`reference/sdk/runtime-contract.md`](reference/sdk/runtime-contract.md). The SDKs
are 0.x: an API may change between minor versions.

## The rules

1. **Own ground.** Loams runs in your AWS/GCP/Azure account; the bucket, keys,
   and secret store are yours. Ostrium runs no data plane for this path.
2. **Never public.** No inbound rule. Reach the engine over your tailnet.
3. **No static cloud keys.** Use the instance role, metadata token, or managed
   identity.
4. **Pin and verify.** Use an explicit version and verify the release signature.
5. **Local block disk.** The embedded stores need a real disk, not NFS/EFS/FUSE.
6. **Back up.** The light single-node stack is not replicated.

## Provenance

`reference/` is a snapshot of
[`ostrium-labs/loams`](https://github.com/ostrium-labs/loams) at the commit in
[`reference/upstream/SOURCE.md`](reference/upstream/SOURCE.md), licensed
Apache-2.0. Hashes are in
[`reference/upstream/SHA256SUMS`](reference/upstream/SHA256SUMS). Refresh with
`scripts/sync-reference.sh`; detach with `scripts/detach-upstream.sh`.

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
