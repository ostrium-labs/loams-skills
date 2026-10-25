# UP1 — Upgrades Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Work task by task, test first. Each task lists its files, the interfaces it must produce, the tests that must exist and pass before it is done (exact function names), its steps and its pull-request size. Where this plan gives exact names, paths, flags, exit codes or defaults, use them verbatim. Where it gives a contract and named tests, write the code to that contract, and record any deviation in "Rulings made during execution" at the end of this file. The code is not pre-written in this plan.
>
> **Status: In progress; D923–D945 and the T9–T23 rulings approved by the owner** (2026-10-16). Track UP, design [§54](../design/54-upgrades.md) (D923–D945, Q820–Q834). Owner direction of 2026-10-13: upgrades ship in **v0.1.0-preview.3**, so preview.2 → preview.3 is the first upgrade that keeps data; weekly previews follow; v0.1.0's exit criteria include a tested upgrade path. The six rules, the CLI, the client rules and the release gate are the owner's (§54 §1.1); this plan builds them and does not redesign them.

**Goal:** a self-hoster at v0.1.0-preview.2 runs `loams upgrade --to 0.1.0-preview.3` and keeps every product's data, with a snapshot taken first and `loams upgrade rollback` to undo it; every later release does the same one step at a time; and no release reaches `main` unless CI has upgraded the previous release's stack, with fixture data in every product, to the candidate and rolled it back.

**Architecture** (§54 §1.3, §6, §10):
- **`crates/loams-migrate`** (new, server side): the format registry, `StackRecord`, the ledger and the migration lease over `MetaStore`, the `Migration` trait and runner, the per-store contexts, the snapshot and restore engine, the `verify` probes, and the static list of migrations. A dependency of `loams` (always on; it is small and has no new third-party crates beyond what `loams` already links).
- **`loams migrate`** (new server command in `crates/loams/src/main.rs`, beside `durable migrate`): `run`, `plan`, `status`, `verify`, `snapshot`, `restore`. In compose it is the one-shot `migrate` service of the `loams` image.
- **`loams upgrade`** (client side, no storage crates): `check`, `--to`, `rollback`, `status`, `snapshots list|prune`. It drives `docker compose`/`podman compose` in the stack directory and runs every data-touching step as `loams migrate …` in a container of the target image. It lives in `crates/loams-cli` if CL1 Task 1 has merged, else in a new `crates/loams-upgrade` library with CL1's no-server-dependency guard, flattened into the `loams` binary, which CL1 later moves under `loams-cli` (Task 0 rules which).
- **Release data:** `release/upgrade-path.toml`, copied into the signed release manifest and the compose bundle.
- **The gate:** `conformance/upgrade/` (fixture writers and checkers) and the `Upgrade: previous release → candidate` job in `ci.yml`.

**Tech Stack:**
- Rust 1.97 (workspace `rust-version`), edition 2024, workspace lints (`unsafe_code = "forbid"`).
- Workspace crates: `loams-meta`, `loams-meta-tikv`, `loams-store`, `loams-collection`, `loams-live`, `loams-durable`, `loams-graph`, `loams-pg-control`, `loams-git`, `loams-sim` (failpoints), `loams-proto`.
- `rusqlite` (the workspace's pin) for the SQLite backup API; `fail` (the workspace's pin, feature `failpoints` only) for the crash matrix; `serde`, `toml`, `serde_json`, `ulid`, `sha2` from the workspace. No new third-party crate without the 14-day rule.
- External, pinned: BR `pingcap/br:v8.5.8` (if Task 0 rules BR in), by digest in `release/companions.toml`.
- Python 3.12 + `pytest` for `conformance/upgrade/` fixtures (the existing conformance style), with the published SDKs of the **previous** release and raw HTTP/SQL where an SDK lacks a call.
- CI: the compose-capable heavy runner for the gate; GitHub-hosted runners for everything else.

**Spec:**
- [§54](../design/54-upgrades.md) (all of it).
- [§10](../design/10-operations.md) §6–§7, [§03](../design/03-storage-formats.md) §3.4, §6–§7.
- [§30](../design/30-loams-cli.md) §5, §6.3, §8.3, §17 and [CL1](2026-10-10-cl1-loams-cli.md) (Global Constraints, Tasks 1, 7, 24, 25).
- [§44](../design/44-unified-api-and-sdks.md) (`GetInstance`, `buf breaking`), [API2](2026-10-10-api2-unified-api-sdks.md) (the SDK fixtures).
- [§45](../design/45-loams-live-production.md) §4.3, [§46](../design/46-loams-postgres-production.md) §12, [§47](../design/47-loams-sql-production.md) §5, [§48](../design/48-loams-graph-production.md) §14.4, [§49](../design/49-loams-house-production.md) §19, [§50](../design/50-loams-desktop-daemon.md) §5.3, [§51](../design/51-loams-postgres-fork.md) §9, [§41](../design/41-multitenant-byoc-control-plane.md) §8.
- [PG2](2026-10-08-pg2-postgres-production.md) (branches, `pg-control`), [NF1](2026-10-09-nf1-neon-fork.md) (the pin file), [DD1](2026-10-09-dd1-desktop-daemon.md) (the daemon), [CLI2](2026-10-01-cli2-release-and-install.md) (the signed manifest).
- `docs/release/{preview,publishing,desktop}.md`, `docs/ci/README.md`.

## Global Constraints

- **Worktree and branches.** Work in `~/Documents/Ostriumlabs/loams-wt/up1`. One branch per task, `up1-t<N>`, from fresh `origin/dev`. **Each task is its own pull request into `dev`**, reviewed by one CodeRabbit round (with the 20-minute rate-limit rule), then merged with `gh pr merge <n> -R ostrium-labs/loams --admin --merge` once local checks pass. `git commit -s` (DCO). Stage only your own paths; never `git stash`; never commit `.superpowers/`. Commit areas: `migrate`, `upgrade`, `loams`, `release`, `deploy`, `ci`, `console`, `desktop`, `sdk`, `conformance`, `docs`.
- **Rust builds** use the shared target directory (`~/Documents/.cargo/config.toml`). Never set `CARGO_TARGET_DIR`, never build in `/tmp`. One cargo build at a time. Build and test the touched crates (`cargo test -p loams-migrate`), and `cargo test -p loams --features <the release features>` before a commit that touches `loams`. Never `cargo build --workspace --all-features` locally.
- **Compose stacks** run locally only when no cargo build is running, with the project name prefixed `up1-` and random high ports. Never `docker/podman volume prune` or `system prune`; remove only containers, volumes and images you created, by id.
- **Old data is never changed in place before the switch.** Every migration writes beside the old data and switches with one atomic step (§54 §6.3). A reviewer rejects a migration that rewrites an existing object, key or file before its switch.
- **Every persisted Loams format is in the registry.** A crate that defines or bumps a format constant updates `loams_migrate::formats` in the same pull request; `format_constants_match_registry` (Task 1) fails otherwise.
- **The client side links no storage crate.** `cargo tree -p <the upgrade crate> -e normal --prefix none` contains none of `loams-migrate`, `loams-meta*`, `loams-store`, `loams-collection`, `loams-live*`, `loams-graph`, `grafeo*`, `loams-durable`, `loams-pg-control`, `loams-tikv`, `rusqlite` (Task 12 test `upgrade_client_has_no_storage_dependencies`).
- **No destructive action without `--yes`** or a typed TTY confirmation (§30 §6.4): `upgrade --to`, `rollback`, `snapshots prune`. Without a TTY and without `--yes`: exit 3, `confirmation_required`.
- **Secrets.** Snapshot directories are 0700; files in them 0600. No `.env` value, API key, `house.key` or TiKV byte is ever printed, logged or put in a JSON output; snapshot listings show paths, sizes and digests only (`snapshot_listing_has_no_secret_values`, Task 8).
- **Pins.** Exact versions at least 14 days old; images by digest. Record each in the commit that adds it.
- **The release images and default features do not change** except for what a task names (the `migrate` service, the new volumes).
- **Hermetic tests.** Unit and crate tests use `loams-store`'s in-memory or `file://` store and the metastore's in-process backends. Compose tests are `#[ignore]` and run in the gate job or by hand.

## Rulings (from §54 and the owner's direction; binding for execution)

| # | Ruling | Source |
|---|---|---|
| R1 | Data-keeping upgrades start at v0.1.0-preview.3; preview.1 → preview.2 stays a fresh stack | D923 |
| R2 | Every persisted format carries a version; readers support N and N−1; the engine exits 78 (`format_unreadable`, `migration_incomplete`, `stack_unrecognised`) rather than start on data it cannot read | D924 |
| R3 | One step at a time; each preview is a step; from v0.1.0 a minor line is one step; `migrate` refuses a non-adjacent source (`upgrade_step_not_adjacent`) | D925 |
| R4 | preview.2 is adopted by fingerprint as migration `0000-adopt-preview-2` (Q820's default) | D926 |
| R5 | `migrate` is a one-shot service of the `loams` image; the engine, House and `pg-control` depend on its successful completion (Q828's default) | D927 |
| R6 | The ledger, the stack record, the lease and the snapshot roots live under `_system/upgrade/` in the metastore | D928 |
| R7 | Migrations are idempotent, resumable, expand-then-switch, classified `Reversible` or `PointOfNoReturn`, compiled in, never downloaded | D929 |
| R8 | Snapshot first, writers stopped, run by the target image; immutable bucket objects are pinned as GC roots, not copied | D930 |
| R9 | One snapshot kept until the next upgrade verifies; 1.5× free space required (Q822's default) | D931 |
| R10 | Rollback restores the snapshot; `--keep-writes` only when every applied migration is `Reversible` and no pin changed (Q824's default) | D932 |
| R11 | Third-party pins change only with a green gate; PD → TiKV → TiDB order; no downgrade across majors; no Postgres major through `loams upgrade` | D933 |
| R12 | `loams upgrade` is a client group; data-touching steps run as `loams migrate …` in target-image containers | D934 |
| R13 | The sequence of §54 §10.3, with the door closed from quiesce to verify and automatic rollback on failure (Q823's default); `--to` beyond one step walks the chain (Q825's default) | D935 |
| R14 | `release/upgrade-path.toml` is the source; the CLI trusts only the signed manifest's copy | D936 |
| R15 | `stack restart --upgrade` and the desktop run the same `migrate` with a snapshot | D937, D940 |
| R16 | The hosted console supports engine N and N−1; older engines get the upgrade page (Q830's default) | D938 |
| R17 | `buf breaking` also against the previous release tag; SDKs additive-only | D939 |
| R18 | The gate is `Upgrade: previous release → candidate`, full tier, in `CI required`, nightly on `dev`, and by label `ci:upgrade` | D941 |
| R19 | Release notes and the CHANGELOG carry the generated "Upgrading" section | D942 |
| R20 | Images by digest; snapshots on the host, 0700; no new credentials for `migrate` | D944 |
| R21 | The safekeeper is drained before any recreate; preview.3 gives it a volume | D945 |

## Review Focus

1. **Old data changed before the switch.** A migration that overwrites a key, object or file it read, before its final atomic step. Expected: never. Tests: Task 3 `failed_migration_leaves_old_data_readable`; Task 6 `every_store_context_writes_beside_then_switches`.
2. **A crash in the middle leaves a stack that neither version opens.** Expected: the new binary resumes, or the snapshot restores. Tests: Task 3 `crash_at_every_failpoint_converges`; Task 8 `restore_after_crash_mid_snapshot_refused`; Task 13 `kill_cli_mid_upgrade_then_rerun_resumes`.
3. **GC deletes what a snapshot needs.** Expected: never while the snapshot exists. Test: Task 8 `gc_between_snapshot_and_restore_keeps_pinned_objects`.
4. **A migration falsely claims `Reversible`.** Expected: the gate fails and names it. Test: Task 20 `reversibility_step_fails_on_false_claim` (a planted migration).
5. **The engine starts on data it cannot read.** Expected: exit 78. Tests: Task 4 `newer_format_refused_at_startup`, `failed_migration_blocks_startup`.
6. **Postgres WAL lost on a container recreate.** Expected: never. Test: Task 9 `safekeeper_recreate_waits_for_drain`.
7. **A client writes while the stack is half upgraded.** Expected: the door is closed. Test: Task 13 `door_closed_during_upgrade`.
8. **A secret leaks** into output, logs or a snapshot listing. Tests: Task 8 `snapshot_listing_has_no_secret_values`; Task 13 `upgrade_output_has_no_env_values`.
9. **A downgrade slips through.** Tests: Task 13 `component_downgrade_refused`; Task 18 `older_engine_refused_on_point_of_no_return_data`.

---

## File structure

```
crates/loams-migrate/
  Cargo.toml
  src/lib.rs
  src/formats.rs            (re-exports loams_common::formats; Task 1)
  src/stack.rs              (StackRecord, StoredVersions; Task 1)
  src/ledger.rs             (LedgerEntry, Ledger, MigrationLease; Task 2)
  src/migration.rs          (Migration, Class, Store, Step, Cursor, Estimate; Task 3)
  src/runner.rs             (plan, run, resume, abandon; Task 3)
  src/startup.rs            (check_startup -> StartupVerdict; Task 4)
  src/adopt.rs              (0000-adopt-preview-2 and its fingerprint; Task 5)
  src/cx/{mod.rs,bucket.rs,meta.rs,live.rs,durable.rs,house.rs,pg_control.rs,host.rs}   (Task 6)
  src/snapshot/{mod.rs,manifest.rs,bucket.rs,raft.rs,sqlite.rs,dir.rs,tikv.rs,postgres.rs,gc_root.rs}  (Tasks 8, 9)
  src/restore.rs            (Tasks 8, 9)
  src/verify.rs             (the read probes; Task 7)
  src/migrations/{mod.rs,m0000_adopt_preview_2.rs,m0001_safekeeper_volume.rs}   (Tasks 5, 10)
  tests/{formats.rs,ledger.rs,runner.rs,startup.rs,adopt.rs,cx.rs,snapshot.rs,restore.rs,verify.rs}
  tests/fixtures/preview2/  (a preview.2 stack's metastore dump, bucket pointers, SQLite file; Task 5)
crates/loams/src/main.rs                     (`migrate` command; Task 7)
crates/loams/src/migrate_cli.rs              (Task 7)
crates/loams/src/server.rs                   (startup gate; Task 4)
fabric/…/house startup                        (startup gate; Task 4)
crates/loams-pg-control/src/…                 (startup gate; Task 4)
crates/loams-upgrade/ or crates/loams-cli/src/upgrade/   (Tasks 12–14; Task 0 rules which)
  src/{lib.rs,stackdir.rs,compose.rs,manifest.rs,check.rs,run.rs,rollback.rs,snapshots.rs,door.rs,errors.rs,output.rs}
  tests/{check.rs,run.rs,rollback.rs,snapshots.rs,errors.rs,schemas/}
deploy/release/compose.yaml                  (migrate service, depends_on, upgrade-snapshots volume; Tasks 7, 10)
deploy/loams-postgres-dev/compose.yaml        (safekeeper volume; Task 10)
deploy/release/ci/compose.upgrade.yaml        (the CI override; Task 20)
deploy/release/byoc/upgrade.sh                (Task 22)
release/upgrade-path.toml                     (Task 11)
release/companions.toml                       (BR digest; Task 9)
scripts/release/{check-upgrade-path.sh,upgrade-notes.sh}   (Task 11)
.github/workflows/{release-github.yml,release-compose.yml,ci.yml,nightly.yml}   (Tasks 7, 11, 17, 20)
conformance/upgrade/{conftest.py,fixtures/<product>.py,run.py,README.md}   (Task 19)
web/packages/console-host/src/{capabilities.ts,boot.ts}, web/plugins/shell/…   (Task 16)
apps/desktop-electron/src/main/engine/{migrate.ts,supervisor.ts}             (Task 18)
buf.yaml, sdks/fixtures/…                     (Task 17)
docs/release/{preview.md,upgrades.md}, deploy/release/README.md, CHANGELOG.md   (Task 23)
```

## Shared contracts (all tasks use these names)

### Rust (`loams-common::formats`, re-exported by `loams-migrate`, then `loams-migrate`'s own types)

```rust
pub struct FormatId(pub &'static str);                 // "collection.manifest", "live.keys", "durable.sqlite", …
pub struct FormatSpec { pub id: FormatId, pub store: Store, pub current: u16, pub min_read: u16 }
pub static REGISTRY: &[FormatSpec];

pub enum Store { Bucket, Meta, Live, Durable, House, PgControl, Host }
pub enum Class { Reversible, PointOfNoReturn }

pub struct StackRecord { pub release: String, pub formats: BTreeMap<String, u16>,
                         pub migrated_at: SystemTime, pub image_digest: Option<String> }

pub enum LedgerState { Pending, Running, Done, Failed }
pub struct LedgerEntry { pub seq: u32, pub id: String, pub release_from: String, pub release_to: String,
                         pub store: Store, pub class: Class, pub state: LedgerState, pub cursor: Option<Vec<u8>>,
                         pub attempts: u32, pub started_at: Option<SystemTime>, pub finished_at: Option<SystemTime>,
                         pub error: Option<LedgerError>, pub image_digest: Option<String>, pub code_digest: [u8; 32] }

#[async_trait] pub trait Migration: Send + Sync { /* §54 §6.3, plus: */
    async fn abandon(&self, cx: &mut Cx) -> Result<()>;      // remove partial writes before a rollback
}
pub static MIGRATIONS: &[&dyn Migration];             // ordered; ids "NNNN-kebab-name"

pub enum StartupVerdict { Ready, NewStack, Refuse { code: &'static str, detail: String } }
pub async fn check_startup(meta: &dyn MetaStore, data_dir: &Path, release: &str) -> Result<StartupVerdict>;
```

Metastore keys: `_system/upgrade/stack`, `_system/upgrade/lease`, `_system/upgrade/ledger/<seq:08>-<id>`, `_system/upgrade/snapshots/<ulid>` (§54 §6.2).

### `loams migrate` (server command)

```
loams migrate run      --meta URL [--live-store URL] --bucket URL --data-dir D [--house-dir D] [--pg-control-dir D]
                       [--target-release V] [--abandon ID] [--no-snapshot-i-accept-data-loss]
loams migrate plan     (same stores) --output json         # what run would do; read-only
loams migrate status   (same stores) --output json         # stack record + ledger
loams migrate verify   (same stores) [--engine URL] --output json
loams migrate snapshot (same stores) --to DIR [--tikv br|cold] [--pageserver URL] [--pg-control URL] --output json
loams migrate restore  (same stores) --from DIR --output json
```

Exit codes: 0 done; 1 internal; 2 usage; 65 (`EX_DATAERR`) a migration failed (ledger says which); 75 (`EX_TEMPFAIL`) the lease is held; 78 refused (R2's codes, `upgrade_step_not_adjacent`, `snapshot_required`).

### `loams upgrade` (client)

The tree and exit codes of §54 §10.1. JSON outputs (`check`, `status`, `snapshots list`, the final report of `--to` and `rollback`) derive `schemars::JsonSchema` and have schema snapshots in `tests/schemas/`.

### `release/upgrade-path.toml`

```toml
[[release]]
version     = "0.1.0-preview.3"
from        = "0.1.0-preview.2"
free_factor = 1.5
[[release.migration]]
id = "0000-adopt-preview-2";  store = "meta"; class = "reversible"; summary = "Record preview.2's versions"
[[release.migration]]
id = "0001-safekeeper-volume"; store = "host"; class = "reversible"; summary = "Give the safekeeper a volume (after a drain)"
[release.components]
pd = "v8.5.8"; tikv = "v8.5.8"; tidb = "v8.5.8-loams.1"; loams_postgres = "<fork_release>"; chdb = "<pin>"; grafeo = "v0.5.43-loams.1"
[[release.env]]
add = "LOAMS_UPGRADE_SNAPSHOT_DIR"; default = ""

[[release]]
version = "0.1.0-preview.2"
from    = "fresh"
```

## Execution order

1. Task 0.
2. **Core** (Tasks 1–7) in order; Task 5 can start beside Task 4. Task 3 creates `release/upgrade-path.toml`, so nothing in the core waits for Task 11.
3. **Snapshot and restore** (Tasks 8–10); Task 9 after Task 0's BR ruling.
4. **Release data** (Task 11) beside Tasks 8–10.
5. **CLI** (Tasks 12–14) after Tasks 7, 8 and 11.
6. **Clients** (Tasks 16, 17) any time after Task 1; Task 18 after Task 8 (and DD1's supervisor if it has landed).
7. **Gate** (Tasks 19, 20) after Task 14; Task 19's fixture writers can start after Task 0.
8. Tasks 15, 21, 22, 23, then Task 24 on the preview.3 release pull request.

**The preview.3 critical path:** Tasks 0–14, 16, 17, 19, 20, 23, 24. Tasks 15, 18, 21 and 22 may follow in a later preview, before v0.1.0.

---

## UP1a — Core: formats, ledger, runner, startup gate (Tasks 0–7)

### Task 0: Reconcile with the code as built, and verify the third-party facts

**Files:** this plan's "Rulings made during execution"; §54 §17 if a fact there is wrong; a scratch compose stack under `deploy/release` with project `up1-t0`, never committed.

Steps:
1. Record, with file paths, as rulings:
   - every Loams format constant and its reader's check (the §54 §17 table), and which formats have **no** version today (each becomes a Task 1 registry entry with a version-0 rule: "absent means 1");
   - the `MetaStore` calls for a `_system` key space: is there a reserved namespace or prefix, and does the TiKV backend give a transaction over several keys (the ledger write plus the stack record)?
   - whether CL1 Task 1 (`crates/loams-cli`) has merged; if not, Tasks 12–14 create `crates/loams-upgrade` (R12);
   - the GC's root sources in `loams-collection`/`loams-log` (where Task 8 adds snapshot roots);
   - the Durable store's schema level (Resonate's migration table or `user_version`) and how `loams durable migrate` reports it.
   - a marker that only preview.2 data has (graph streams named `_graph.<gr>`, Live deployment records, or another), so Task 5 can refuse preview.1 stacks; record "none found" if there is none.
2. **Verify** against primary sources, recording versions and commands:
   - BR v8.5.8: backup and restore of a TiKV cluster with API V2 keyspaces (`loams_meta`, `loams_live_dev` and TiDB's), with `local://` storage when the path is mounted into `tikv`. Run it on the scratch stack. **Rules Q821** (BR or cold copy).
   - TiDB on a version bump: does it upgrade each keyspace's system tables on start, or only in `CreateDatabase` (§47 §5)?
   - The fork's pageserver API: branch creation at the last record LSN, `remote_consistent_lsn` per timeline, and the safekeeper's `flush_lsn` (paths and fields).
   - Compose: `service_completed_successfully` with the bundle's minimum Compose version (v2.20.3) and with `podman compose`.
3. Build preview.2's stack from its bundle (or from `release/v0.1.0-preview.2` if not yet tagged), write a little data in each product, and copy its metastore dump, bucket pointers and SQLite file into `crates/loams-migrate/tests/fixtures/preview2/` (no secrets; regenerate API keys as test values). Record the commands in the fixture's `README.md`.
4. Confirm D923–D945 and Q820–Q834 are still free in the decision log; renumber in §54 and here if not.
5. Commit `docs(up1): task 0 rulings`.

**PR size:** S (docs and fixtures).

### Task 1: The format registry and the stack record

**Files:** create `crates/loams-common/src/formats.rs` (the only definition of `FormatId`, `FormatSpec`, `REGISTRY` and `check`) and `crates/loams-migrate/{Cargo.toml,src/lib.rs,src/formats.rs (a re-export only),src/stack.rs,tests/formats.rs}`; workspace `Cargo.toml`; one `#[test]` in each crate that owns a format (`loams-log`, `loams-collection`, `loams-meta`, `loams-live`, `loams-graph`, `loams-git`, `loams-durable`, `loams-pg-control`, fabric's House crate), as Task 0 listed.

**Interfaces:** `FormatId`, `FormatSpec`, `REGISTRY`, `StackRecord` (postcard-encoded behind a one-byte value version, `0x01`), `StoredVersions::readable_by(&REGISTRY) -> Result<(), Unreadable>`, and `formats::check(id, found) -> Result<(), Unreadable>` accepting `min_read..=current`.

**Readers move to the range.** §54 §17 lists the readers that check the exact version today (WAL, segment, `OPCM`, `OPPD`, `OPDL`, the record codec, the PK value, the split trailer, `OPDB`, hot artifacts and owner records, link counters, the TiKV metastore's `FORMAT`, Live's `format`, graph manifest, change-set and portable snapshot, Git's `DOC_VERSION`, `pg-control`'s `FORMAT`). Each calls `formats::check` instead. Owning crates depend only on `loams-common` for this (never on `loams-migrate`), which keeps the graph acyclic. Formats with no version today (the Raft log and `meta.redb` records, pointer values, the graph catalog document, Live's `JournalEntry`, Git's `.lpk`, the House directory) are registered with `current = 1` and the rule "absent means 1"; none is rewritten.

Tests:
- `registry_ids_are_unique`
- `registry_min_read_is_current_or_one_below`
- `stack_record_round_trips`
- `stack_record_value_version_is_pinned` (a byte literal golden)
- `readable_range_accepts_n_and_n_minus_one`
- `readable_range_refuses_n_plus_one_and_n_minus_two`
- `format_constants_match_registry` (one per owning crate; it compares the crate's constant with `REGISTRY`)
- `reader_accepts_previous_version` (one per converted reader: a planted object at `current − 1` opens when `min_read` allows it; with every format at 1 the test uses a registry override)
- `reader_refuses_next_version` (one per converted reader)

Steps: write the tests first; add the registry with Task 0's list; convert the readers one crate at a time (one commit per crate); add the per-crate assertions; `cargo test -p loams-migrate` and each touched crate.

Commit `feat(migrate): format registry, stack record and range checks`. **PR size:** L (~1200 lines across about twelve crates; split into "registry" and "readers" pull requests if review asks).

### Task 2: The ledger and the migration lease

**Files:** `src/ledger.rs`, `tests/ledger.rs`.

**Interfaces:** the reserved namespace `_system`: `CreateNamespace` and every admin API refuse it (`namespace_reserved`), and list calls omit it; `Ledger::open(meta)`, `Ledger::entries()`, `Ledger::append(entry)`, `Ledger::update(seq, |e| …)` (fenced by the lease epoch), `Ledger::stack()`, `Ledger::set_stack(record)` in the same transaction as the last entry's `Done`; `MigrationLease::acquire(meta, holder, ttl=30s)` renewed every 10 s; `release()`.

Tests:
- `ledger_append_and_read_back` (TiKV and Raft backends, through the metastore conformance harness)
- `ledger_update_rejected_with_stale_epoch`
- `lease_excludes_second_holder`
- `lease_expires_and_is_taken_over`
- `stack_record_and_last_entry_commit_together`
- `ledger_keys_live_under_system_upgrade_prefix`
- `system_namespace_is_reserved` (create refused, list omits it)

Commit `feat(migrate): ledger and migration lease in _system`. **PR size:** M (~500 lines).

### Task 3: The `Migration` trait and the runner

**Files:** `src/migration.rs`, `src/runner.rs`, `tests/runner.rs`, test-only migrations in `tests/support/`; **creates `release/upgrade-path.toml`** with its first two entries (preview.2 `from = "fresh"`; preview.3 `from = "0.1.0-preview.2"` with an empty migration list that Tasks 5 and 10 fill). The runner's tests use their own path files in `tests/fixtures/paths/` (pre-release chains and a 0.1 patch line). Task 11 adds the checks, the manifest and the notes on top of this file.

**Interfaces:** the trait of §54 §6.3 with `abandon`; `Runner::plan(target) -> Plan { from, to, steps: Vec<PlannedStep { id, class, estimate }> }`; `Runner::run(target, opts) -> Report`: takes the lease, checks adjacency against `release/upgrade-path.toml`'s compiled copy (`include_str!`): exact-predecessor for pre-releases (`from = "0.1.0-preview.2"`), line-based from v0.1.0 (`from = "0.1"` accepts a stored 0.1.x with x < y, refuses x ≥ y, another line or a pre-release), and exact again for a new line's first minor; runs pending migrations in order, checkpoints each cursor in the ledger, calls `verify` after `Done`, writes the stack record last. Failpoints (`fail` crate, `failpoints` feature): `migrate::before_apply`, `migrate::after_apply_before_checkpoint`, `migrate::after_checkpoint`, `migrate::before_switch`, `migrate::after_switch_before_done`.

Tests:
- `plan_lists_pending_migrations_in_order`
- `run_applies_and_records_done`
- `rerun_done_migration_is_noop`
- `resume_from_cursor_after_failure`
- `crash_at_every_failpoint_converges` (each failpoint × each test migration; rerun; the result equals a clean run)
- `failed_migration_leaves_old_data_readable`
- `non_adjacent_source_refused` (`upgrade_step_not_adjacent`)
- `patch_line_source_accepted` (0.1.1 → 0.1.4 runs only the migrations not `Done`)
- `patch_line_newer_or_other_line_refused` (0.1.5 → 0.1.4, 0.0.9 → 0.1.4, 0.1.0-preview.9 → 0.1.4)
- `point_of_no_return_needs_snapshot` (`snapshot_required` unless `--no-snapshot-i-accept-data-loss`)
- `code_digest_mismatch_refuses_resume` (`migration_mismatch`)
- `abandon_removes_partial_writes`

Commit `feat(migrate): migration runner with resume and failpoints`. **PR size:** M (~800 lines).

### Task 4: The startup gate

**Files:** `src/startup.rs`, `tests/startup.rs`; `crates/loams/src/server.rs` (call before any listener), the House front's startup in `fabric/`, `crates/loams-pg-control/src/…` startup; the desktop's engine uses the same `loams` startup.

**Interfaces:** `check_startup` (§ shared contracts). `Ready` serves; `NewStack` writes the stack record at the binary's release and serves; `Refuse` logs one line with the code and exits 78. Codes: `format_unreadable`, `migration_incomplete`, `stack_unrecognised`.

Tests:
- `empty_store_is_new_stack_and_stamped`
- `current_stack_is_ready`
- `newer_format_refused_at_startup`
- `older_than_n_minus_one_refused_at_startup`
- `failed_migration_blocks_startup`
- `running_migration_blocks_startup`
- `unstamped_nonempty_store_refused_unless_preview2_fingerprint` (uses Task 5's fingerprint once merged; until then `stack_unrecognised`)
- `loams_exits_78_on_refusal` (`crates/loams/tests`, spawns the binary)

Commit `feat(loams): refuse to start on unreadable data`. **PR size:** M (~400 lines).

### Task 5: Adopting preview.2 (`0000-adopt-preview-2`)

**Files:** `src/adopt.rs`, `src/migrations/m0000_adopt_preview_2.rs`, `tests/adopt.rs`, `tests/fixtures/preview2/` (Task 0).

**Interfaces:** `Fingerprint::check(cx, declared_release: Option<&str>) -> Result<Preview2, Mismatch { store, detail }>` per §54 §5.3. It succeeds only with a positive marker: Task 0's data marker, or `declared_release == Some("0.1.0-preview.2")` (`loams migrate run --declared-release`, which the CLI fills from the stack directory's `LOAMS_VERSION` and the desktop from `.loams-stack-version`); decoding alone never suffices. (metastore record kinds decode with preview.2's encodings; the bucket's pointer envelopes are at preview.2's versions; Live's keyspace decodes; the Durable schema level; the House layout if present). The migration writes the ledger entry and the stack record at `0.1.0-preview.2`; class `Reversible` (it only adds `_system/upgrade/*`, which preview.2 never reads).

Tests:
- `preview2_fixture_is_recognised`
- `preview1_fixture_is_refused` (a hand-made preview.1 shape; `stack_unrecognised`)
- `decodable_store_without_marker_is_refused` (preview.2-shaped data, no data marker, no declared release)
- `declared_preview2_with_decodable_data_is_adopted`
- `corrupt_pointer_envelope_is_refused`
- `adoption_is_idempotent`
- `adoption_writes_only_system_upgrade_keys` (diff of the metastore before and after)
- `preview2_reads_after_adoption` (the preview.2 record decoders read everything back)

Commit `feat(migrate): adopt preview.2 stacks by fingerprint`. **PR size:** M (~500 lines plus fixtures).

### Task 6: Store contexts

**Files:** `src/cx/{mod.rs,bucket.rs,meta.rs,live.rs,durable.rs,house.rs,pg_control.rs,host.rs}`, `tests/cx.rs`; `crates/loams/src/main.rs` (`durable migrate` delegates to the Durable context).

**Interfaces:** `Cx` gives a migration typed access to exactly its store, each with an expand-then-switch helper: `BucketCx::{read_pointer, write_object_new, cas_pointer}`, `MetaCx::{scan_prefix, txn}`, `LiveCx::{scan, txn}` (TiKV keyspace), `DurableCx::{conn, user_version, swap_file}` (writes a new SQLite file, then `rename`), `HouseCx::{dir, swap_dir}` (sibling directory, then `rename`), `PgControlCx::{records, swap_file}`, `HostCx` (records `.env` and compose changes for the CLI to apply; it writes nothing on the host itself). Resonate's own migrations become the step `durable-resonate` that `run` calls, and the engine opens SQLite with Resonate's `migrate = false` plus a schema check (as `mysql::check_schema` does), so serving never migrates implicitly. The House context adds a `FORMAT` file to the House data directory and `PRAGMA user_version = 1` to `catalog.sqlite` (absent reads as 1).

Tests:
- `every_store_context_writes_beside_then_switches` (per context: interrupt before the switch, the old data reads unchanged)
- `durable_migrate_command_still_works` (the existing CLI test stays green)
- `house_swap_is_atomic_rename`
- `meta_txn_is_all_or_nothing`
- `host_cx_writes_nothing_on_the_host`
- `engine_does_not_migrate_sqlite_on_open` (a database one Resonate migration behind is refused with `migration_incomplete`, not migrated)
- `house_dir_format_stamp_absent_reads_as_one`

Commit `feat(migrate): store contexts with expand-then-switch helpers`. **PR size:** L (~900 lines).

### Task 7: `loams migrate` and the compose `migrate` service

**Files:** `crates/loams/src/{main.rs,migrate_cli.rs}`, `crates/loams-migrate/src/verify.rs`, `crates/loams-migrate/tests/verify.rs`, `deploy/release/compose.yaml`, `.github/workflows/release-compose.yml`.

**Interfaces:** the `loams migrate` command of the shared contracts (`run`, `plan`, `status`, `verify`; `snapshot` and `restore` arrive in Task 8). The compose service of §54 §6.1; `loams`, `house` and `pg-control` gain `depends_on: migrate: condition: service_completed_successfully`. `verify` runs §54 §10.4's probes and reports per product `ok|empty|failed`.

Tests:
- `migrate_subcommands_parse` (each subcommand and its flags)
- `migrate_plan_json_matches_schema`
- `migrate_run_on_current_stack_exits_quickly` (under 2 s on a fixture stack; target ~1 s)
- `verify_reports_empty_for_unused_products`
- `verify_fails_on_unreadable_collection`
- release-compose additions: `migrate` has no ports, no tailscale, no socket; `loams`, `house` and `pg-control` depend on it with `service_completed_successfully` (jq assertions in `release-compose.yml`)

Steps: tests first; wire the command; add the service and dependencies; run `docker compose --env-file .env.example config -q` with every profile; bring up a scratch stack (`up1-t7`) and check that the engine starts after `migrate` exits 0 and does not start when it exits 65.

Commit `feat(loams): migrate command and the compose migrate service`. **PR size:** M (~600 lines).

## UP1b — Snapshot, restore and preview.3's own migration (Tasks 8–10)

### Task 8: Snapshots and restore of Loams-owned stores

**Files:** `src/snapshot/{mod.rs,manifest.rs,bucket.rs,raft.rs,sqlite.rs,dir.rs,gc_root.rs}`, `src/restore.rs`, `tests/{snapshot.rs,restore.rs}`; the GC root hook in the crate Task 0 named; `deploy/release/compose.yaml` (the `upgrade-snapshots` volume).

**Interfaces:** `loams migrate snapshot --to DIR` and `restore --from DIR` for S1–S3 (pointer copies, the GC root, Raft snapshot and redb copy), S9 (pg-control file), S10 (House directory), S11 (SQLite backup API), S15 (agentd); `snapshot.json` (§54 §7.3) written last and its digest recorded under `_system/upgrade/snapshots/<ulid>`; `snapshots list|prune` primitives (prune removes the GC root, then the directory); free-space estimate `Estimate::snapshot_bytes`.

Tests:
- `snapshot_manifest_written_last`
- `restore_after_crash_mid_snapshot_refused` (`snapshot_incomplete`)
- `snapshot_then_restore_round_trips_every_loams_store`
- `gc_between_snapshot_and_restore_keeps_pinned_objects`
- `prune_removes_gc_root_then_files`
- `sqlite_snapshot_uses_backup_api_and_is_consistent` (writes during backup on a test db)
- `snapshot_dir_is_0700_and_files_0600`
- `snapshot_listing_has_no_secret_values`
- `estimate_within_20_percent_of_actual` (fixture stack)

Commit `feat(migrate): snapshot and restore of Loams-owned stores`. **PR size:** L (~1000 lines).

### Task 9: Snapshots and restore of TiKV and Postgres

**Files:** `src/snapshot/{tikv.rs,postgres.rs}`, `src/restore.rs`, `tests/snapshot.rs` (`#[ignore]` compose tests), `release/companions.toml` (BR by digest, if Q821 rules BR), `deploy/release/compose.yaml` (`upgrade-snapshots` mounted into `tikv` for BR).

**Interfaces:**
- Helpers (§54 §10.1's boundary): only the target `loams` image and, if Q821 rules it in, `pingcap/br@sha256:…`; each with `cap_drop: ALL`, `no-new-privileges`, a read-only root filesystem, no ports, no socket, the compose default network only; volumes limited to the step's data volumes and `upgrade-snapshots`; credentials limited to read-only mounts of `secrets/engine.env` (an `s3://` bucket) and the `pg-control` API key. The pageserver and safekeeper APIs are called from the `loams migrate` helper only.
- TiKV (Q821 per Task 0): `--tikv br` runs BR as a helper container (`pingcap/br@sha256:…`) against `pd:2379` into `/snapshots/<id>/tikv`; restore goes into a fresh cluster at the snapshot's TiKV version (the CLI recreates `pd-data` and `tikv-data` first). `--tikv cold` copies both volumes with PD and TiKV stopped (done by the CLI's helper container; `migrate` records it in the manifest).
- Postgres: `branch_every_timeline(pageserver | pg-control, name = "upgrade-<id>")`; `drain_safekeepers(timeout = 5 min)` waits for `remote_consistent_lsn ≥ flush_lsn` per timeline; restore through `RestoreBranch` or the pageserver's timeline swap; a cold copy of `pageserver-data` and `rustfs-data` when the Loams Postgres pins change (Q829).

Tests:
- `tikv_backup_restore_round_trip` (`#[ignore]`, compose: meta, Live and a TiDB table survive)
- `tikv_restore_targets_snapshot_version` (refuses a newer TiKV)
- `postgres_branch_per_timeline` (`#[ignore]`, compose)
- `postgres_restore_branch_swaps_back` (`#[ignore]`)
- `safekeeper_recreate_waits_for_drain` (fake pageserver and safekeeper status servers; a lagging timeline blocks until it catches up, then times out with `postgres_not_drained`)
- `cold_copy_requires_services_stopped`
- `helper_containers_follow_the_boundary` (the rendered helper specs: caps, read-only root, networks, volumes and mounts as listed)

Commit `feat(migrate): TiKV and Postgres snapshots`. **PR size:** L (~900 lines).

### Task 10: preview.3's compose changes and `0001-safekeeper-volume`

**Files:** `deploy/loams-postgres-dev/compose.yaml` (a `safekeeper-data` volume for `safekeeper1`), `deploy/release/compose.yaml`, `src/migrations/m0001_safekeeper_volume.rs`, `release/upgrade-path.toml` (entry), `tests/adopt.rs` or a new `tests/m0001.rs`.

**Interfaces:** the migration is a `Host`-store step: it records in the ledger that the drain (Task 9) completed and that the CLI may recreate `safekeeper1` with its volume; class `Reversible` (preview.2's compose ignores the volume; a drained safekeeper starts empty).

Tests:
- `m0001_requires_drain_record`
- `m0001_is_reversible_and_idempotent`
- `compose_safekeeper_has_volume` (release-compose jq assertion)
- `postgres_rows_survive_safekeeper_recreate` (`#[ignore]`, compose: write rows, drain, recreate, read)

Commit `feat(deploy): safekeeper volume with a drained migration`. **PR size:** S (~250 lines).

## UP1c — Release data (Task 11)

### Task 11: `release/upgrade-path.toml`, the signed manifest, the bundle and the notes

**Files:** `release/upgrade-path.toml`, `scripts/release/{check-upgrade-path.sh,upgrade-notes.sh}`, `.github/workflows/release-github.yml` (bundle and notes), the CLI2 manifest writer (the `upgrade` object), the repository-checks job in `ci.yml` (the CHANGELOG check), `crates/loams-migrate/build.rs` or `include_str!` of the path file.

**Interfaces:** §54 §11. `check-upgrade-path.sh` verifies: every compiled migration id listed once under the release that introduced it; `from` chains to `fresh`; component pins equal `.env.example`, `release/sqldb-images.toml` and `deploy/pins/loams-postgres-images.toml`. `upgrade-notes.sh <version>` renders the "Upgrading" section. The repository check fails a pull request into `main` whose CHANGELOG entry for the release in `Cargo.toml` lacks the section.

Tests (bats or shell self-tests, as `scripts/docs/check-decision-ids.sh --self-test` does):
- `check_upgrade_path_self_test` (planted: an unlisted migration, a broken chain, a pin mismatch)
- `upgrade_notes_render_golden` (golden Markdown for preview.3)
- `changelog_without_upgrading_section_fails`
- `bundle_contains_upgrade_path` (the bundle step's output listing)
- `manifest_upgrade_object_matches_path_file`

Commit `release(upgrade): upgrade path, manifest, bundle and notes`. **PR size:** M (~500 lines).

## UP1d — The CLI (Tasks 12–15)

### Task 12: `loams upgrade check`, `status` and `snapshots`

**Files:** the upgrade crate or module (Task 0's ruling): `src/{lib.rs,stackdir.rs,compose.rs,manifest.rs,check.rs,snapshots.rs,errors.rs,output.rs}`, `tests/{check.rs,snapshots.rs,errors.rs,schemas/}`; `crates/loams/src/main.rs` (flatten the group).

**Interfaces:** stack directory discovery (`--dir`, `LOAMS_STACK_DIR`, the current directory with a `compose.yaml` whose `name` is `loams`); a `Compose` driver over `docker compose` or `podman compose` (detected; `platform_unsupported` when neither); manifest fetch and minisign verification through CLI2's verifier (Q282's key; until it exists, SHA-256 against `SHA256SUMS` and a `--allow-unsigned` that only previews accept, recorded as a ruling); `check` (§54 §10.2) using `GetInstance` or a `loams migrate plan` container; `status`; `snapshots list|prune`. Errors and exit codes of §54 §10.1.

Tests:
- `stack_dir_discovery_order`
- `non_loams_compose_dir_is_not_a_stack`
- `check_json_matches_schema`
- `check_reports_blockers` (non-adjacent, downgrade, low space, failed migration)
- `check_reads_plan_when_engine_down`
- `prune_requires_yes`
- `error_code_and_exit_code_table`
- `upgrade_client_has_no_storage_dependencies`

Commit `feat(upgrade): check, status and snapshots`. **PR size:** M (~800 lines).

### Task 13: `loams upgrade --to`

**Files:** `src/{run.rs,door.rs}`, `tests/run.rs` (unit, with a fake `Compose`), `crates/loams/tests/upgrade_e2e.rs` (`#[ignore]`).

**Interfaces:** the twelve steps of §54 §10.3; the door (stop and start `tailscale`, `house-tailscale`); quiesce order; `.env` changes from the manifest (old values kept, a `.env.bak-<from>` written 0600); the chain walk for `--to` beyond one step (Q825); automatic rollback on failure in steps 8–11 (exit 10) unless `--no-auto-rollback`; the host history file `.loams-upgrade/history.jsonl`; git-checkout stacks accepted only at the target's clean tag (Q827); `--bundle DIR` for a local, unverified bundle whose images are already present (CI and git checkouts).

Tests:
- `steps_run_in_documented_order` (fake compose records calls)
- `pull_failure_changes_nothing`
- `door_closed_during_upgrade` (no sidecar running between steps 4 and 12)
- `migrate_failure_rolls_back_automatically`
- `verify_failure_rolls_back_automatically`
- `no_auto_rollback_leaves_failed_state_and_door_closed`
- `env_changes_keep_user_values`
- `component_downgrade_refused`
- `chain_walk_stops_at_first_failure`
- `kill_cli_mid_upgrade_then_rerun_resumes` (`upgrade_in_progress` detected from the history file and the ledger; rerun continues)
- `upgrade_output_has_no_env_values`
- `local_bundle_is_reported_unverified`
- `preview2_to_candidate_e2e` (`#[ignore]`; the gate runs it)

Commit `feat(upgrade): upgrade --to with snapshot, ordered restart and verification`. **PR size:** L (~1100 lines).

### Task 14: `loams upgrade rollback`

**Files:** `src/rollback.rs`, `tests/rollback.rs`, `crates/loams/tests/upgrade_e2e.rs`.

**Interfaces:** §54 §8: restore every component, bring back the old bundle and `.env`, start the previous release, verify, open the door; print the lost-writes window before confirming; `--snapshot ID`; `--keep-writes` gated by the ledger (all applied `Reversible`, no pin change), else exit 6 `rollback_needs_snapshot`; exit 11 `rollback_failed` with the manual steps.

Tests:
- `rollback_restores_every_component` (fake compose plus `migrate restore` calls)
- `rollback_prints_lost_writes_window`
- `keep_writes_refused_with_point_of_no_return`
- `keep_writes_refused_after_pin_change`
- `rollback_failure_exits_11_with_steps`
- `rollback_e2e_restores_fixtures` (`#[ignore]`)

Commit `feat(upgrade): rollback from the snapshot`. **PR size:** M (~600 lines).

### Task 15: Local stacks: `stack restart --upgrade` and `loams dev`

**Files:** if CL1 Task 7 has merged, its supervisor (`stack restart --upgrade`); otherwise `crates/loams/src/main.rs` only (a documented `loams migrate snapshot` + `run` sequence for `loams dev --data-dir`), and an amendment note in CL1's rulings.

**Interfaces:** D937. `stack describe` gains `migration_pending`. `loams dev` refuses with exit 78 as Task 4 says; its message names `loams migrate run --data-dir <dir>` (or `loams stack restart --upgrade`).

Tests:
- `restart_upgrade_snapshots_then_migrates` (if CL1 Task 7 exists)
- `dev_refusal_message_names_migrate_command`
- `embedded_raft_snapshot_restore_round_trip`

Commit `feat(cli): local stacks migrate on restart --upgrade`. **PR size:** S (~300 lines).

## UP1e — Clients (Tasks 16–18)

### Task 16: The console supports engine N−1

**Files:** `web/packages/console-host/src/{capabilities.ts,boot.ts}`, the shell's routing and a `EngineTooOld` page in `web/plugins/shell/`, plugin manifests (a `requires: { service, minVersion }` field), `crates/loams/src/api/connect.rs` (`features` from the compiled features and served roles, empty today; `min_app_versions` from the manifest), console tests.

**Interfaces:** D938. `capabilities.ts` turns `GetInstance` into `has(service, minVersion)`; the shell hides routes and nav items whose plugin's `requires` fails; the blocking page for an engine older than the console's `minEngine` (N−1) or without `GetInstance`.

Tests:
- `console_supports_previous_engine` (Vitest with two `GetInstance` fixtures, N and N−1: no route renders a missing service)
- `missing_service_hides_nav_item`
- `engine_older_than_n_minus_one_shows_upgrade_page`
- `min_app_versions_filled_from_manifest` (Rust, `crates/loams`)
- `get_instance_reports_features` (Rust, `crates/loams`: one entry per compiled product feature and role)

Commit `feat(console): engine N−1 support by capability`. **PR size:** M (~500 lines).

### Task 17: Additive-only protos and SDKs

**Files:** `.github/workflows/ci.yml` (`app-protos` gains a second `buf breaking` against the previous release tag read from `release/upgrade-path.toml`), `sdks/fixtures/` (the removed-symbol check), `buf.yaml` (comment the rule for leaving `unstable`).

Tests:
- `buf_breaking_against_previous_release` (the CI step; a planted removal in a scratch branch fails it, recorded in the PR)
- `removed_public_symbol_fails_fixture_check` (per supported SDK, a planted removal)

Commit `ci(protos): breaking check against the previous release`. **PR size:** S (~250 lines).

### Task 18: The desktop runs `migrate`

**Files:** `apps/desktop-electron/src/main/engine/{migrate.ts,supervisor.ts}`, the renderer's error view, DD1's daemon supervisor if it has landed (`crates/loams-agentd*` engine start).

**Interfaces:** D940. Before `engine.start()`: `resources/bin/loams migrate status --data-dir <userData/engine>`; if pending, `migrate snapshot --to userData/upgrade-snapshots/<id>` then `migrate run`; on failure keep the engine stopped and offer **Restore the previous version's data** and **Report**. Downgrade rule: an older bundled engine starts only if every newer ledger entry is `Reversible`.

Tests:
- `migrate_runs_before_engine_start`
- `migrate_failure_keeps_engine_stopped_and_offers_restore`
- `older_engine_refused_on_point_of_no_return_data`
- `older_engine_allowed_on_reversible_data`
- `desktop_keeps_one_snapshot` (Q831)

Commit `feat(desktop): migrate the engine's data after an update`. **PR size:** M (~500 lines).

## UP1f — The release gate (Tasks 19–20)

### Task 19: Fixture writers and checkers

**Files:** `conformance/upgrade/{conftest.py,run.py,README.md}`, `conformance/upgrade/fixtures/{collections.py,streams_links.py,qdrant.py,es.py,live.py,graph.py,sql.py,postgres.py,house.py,durable.py,keys.py}`.

**Interfaces:** `run.py write --stack URL --out state.json` writes each product's fixture with the APIs the **previous** release serves (capability-gated through `GetInstance`; a product the previous release lacks is skipped and recorded) and records the expected reads; `run.py check --state state.json [--expect-new-writes present|absent]` reads every fixture through every API it was written with and compares; `run.py write-new` writes one new item per product after the upgrade.

Tests:
- `write_then_check_on_same_release_passes` (against a scratch stack)
- `check_detects_a_changed_document`
- `skips_products_the_previous_release_lacks`
- `house_fixture_uses_ddl_until_insert_ships`

Commit `test(conformance): upgrade fixtures for every product`. **PR size:** L (~900 lines).

### Task 20: The `Upgrade: previous release → candidate` job

**Files:** `.github/workflows/ci.yml` (job `upgrade-gate`, its path filter, full tier, `required`), `.github/workflows/nightly.yml`, `deploy/release/ci/compose.upgrade.yaml`, `docs/ci/README.md`.

**Interfaces:** §54 §14 steps 1–6; the job runs on the heavy compose runner, builds the candidate images with `build.sh --platforms linux/amd64`, starts the previous release from its signed bundle with the CI override (no Tailscale, `apikey` auth, loopback port), runs Task 19's `write`, `loams upgrade --to`, `check`, `write-new`, the reversibility step (the previous release reads the fixtures **and** the candidate's new writes on the migrated data), `rollback`, `check --expect-new-writes absent`, and (when all reversible) a second run with `rollback --keep-writes` that checks the previous release reads both the fixtures and the post-upgrade writes; uploads the ledger, `snapshot.json`, `check`'s JSON, timings and logs. Runs on pull requests into `main`, in the nightly on `dev`, and on pull requests into `dev` labelled `ci:upgrade`.

Tests:
- `reversibility_step_fails_on_false_claim` (a planted test-only migration claiming `Reversible`, behind a CI-only feature, in a scratch run recorded in the PR)
- `gate_in_ci_required` (the `required` job lists it; a repository check)
- the job itself, green from preview.2's bundle to `dev`'s head (recorded in the PR with timings; Q833's target checked)

Commit `ci(upgrade): previous-release upgrade gate`. **PR size:** M (~400 lines).

## UP1g — Policy, BYOC, docs and the first real run (Tasks 21–24)

### Task 21: The third-party bump policy in CI

**Files:** `scripts/release/check-upgrade-path.sh` (component pins, downgrade detection), `.github/workflows/ci.yml` (a pin change adds the `ci:upgrade` requirement), `docs/release/publishing.md`.

Tests:
- `pin_downgrade_fails_check`
- `major_bump_requires_point_of_no_return_entry`
- `pin_change_without_gate_label_fails` (repository check on pull requests that touch a pin file)

Commit `ci(release): third-party pin policy`. **PR size:** S (~200 lines).

### Task 22: BYOC upgrades

**Files:** `deploy/release/byoc/upgrade.sh`, `deploy/release/README.md`, `release-compose.yml` (shellcheck).

**Interfaces:** §54 §10.5: copy the CLI for the VM's architecture from the target bundle, run `loams upgrade --to V --yes --dir <bundle dir>` over SSH, stream output, return its exit code.

Tests:
- shellcheck clean
- `upgrade_sh_dry_run_prints_remote_commands` (`--dry-run`)

Commit `feat(deploy): byoc upgrade over ssh`. **PR size:** S (~150 lines).

### Task 23: Docs

**Files:** `docs/release/upgrades.md` (new: how upgrades work, the commands, snapshots, rollback, what is reversible, troubleshooting each exit code), `docs/release/preview.md` (replace "No upgrade path between previews" and "Back up the compose volumes yourself" with the preview.3 text, keeping the preview.1 → preview.2 note), `deploy/release/README.md`, `CHANGELOG.md` (preview.3's "Upgrading" section from Task 11), §54 §17 (as built), `docs/plans/README.md` (status).

Tests: `scripts/docs/check-decision-ids.sh`; the docs link check the repository runs.

Commit `docs(upgrade): upgrade guide and preview.3 notes`. **PR size:** M (docs).

### Task 24: The preview.3 rehearsal and exit

**Files:** this plan's rulings; §54's status line; the decision log rows' status.

Steps:
1. On the `dev` → `main` release pull request for v0.1.0-preview.3, confirm `Upgrade: previous release → candidate` ran preview.2 → preview.3 green, and record its timings (snapshot, migrate, downtime) against Q833's target.
2. By hand, on a machine with a real preview.2 stack built from the published bundle: `loams upgrade check`, `loams upgrade --to 0.1.0-preview.3`, the console against the upgraded engine, `loams upgrade rollback`, and the upgrade again. Record the output.
3. Mark D923–D945 *Approved* where the owner confirmed them, and §54's status *Implemented (preview.3)*.

Commit `docs(up1): preview.3 rehearsal`. **PR size:** S.

---

## Exit criteria (§54, with the owning tasks)

- [ ] Every Loams format is in the registry, with N/N−1 readers: Tasks 1, 4.
- [ ] The engine, House and `pg-control` refuse to start on unreadable data or an incomplete migration: Task 4.
- [ ] `migrate` runs idempotent, resumable migrations and records them in the `_system` ledger; a failure leaves the old data intact: Tasks 2, 3, 6, 7.
- [ ] preview.2 stacks are adopted; preview.1 stacks are refused: Task 5.
- [ ] Snapshots and restores of every store in §54 §3: Tasks 8, 9, 10.
- [ ] `loams upgrade check|--to|rollback|status|snapshots`: Tasks 12–14; local stacks and the desktop: Tasks 15, 18; BYOC: Task 22.
- [ ] The upgrade path, the signed manifest, the bundle and the release notes: Task 11; the pin policy: Task 21.
- [ ] The console with engine N−1; additive-only protos and SDKs: Tasks 16, 17.
- [ ] The gate in `CI required` on pull requests into `main`, green preview.2 → preview.3: Tasks 19, 20, 24.
- [ ] Docs: Task 23.

## Self-review

- **Spec coverage.**

  | §54 section | Task(s) |
  |---|---|
  | §3 What holds state | 0, 6, 8, 9, 10 |
  | §4 Rule 1 | 1, 4 |
  | §5 Rule 2, preview.2 adoption | 3, 5, 11, 13 |
  | §6 Rule 3, `migrate`, ledger, contract | 2, 3, 6, 7 |
  | §7 Rule 4, snapshot | 8, 9, 10 |
  | §8 Rule 5, rollback | 8, 9, 14 |
  | §9 Rule 6, third-party | 0, 9, 10, 11, 21 |
  | §10 The CLI | 12, 13, 14, 22 |
  | §11 Releases | 11 |
  | §12 Local stacks, clusters | 15 (clusters: not in UP1, D943) |
  | §13 Clients | 16, 17, 18 |
  | §14 The gate | 19, 20, 24 |
  | §16 Security | 8, 12, 13 |

- **Names.** `loams-migrate`, `loams migrate`, `loams upgrade`, `_system/upgrade/*`, `release/upgrade-path.toml`, the exit codes and error codes are defined once, in the shared contracts and §54 §10.1.
- **Review Focus.** Items 1–9 each name owning tests (Tasks 3, 4, 6, 8, 9, 13, 18, 20).

## Open questions for the owner

These are §54 §19's questions. Every one has a default the tasks follow until the owner answers.

| # | Question | Default | Needed by |
|---|---|---|---|
| Q820 | Stamp preview.2 (#518) before its tag, or adopt by fingerprint? | Adopt by fingerprint | Task 5 |
| Q821 | TiKV snapshot by BR or cold volume copy? | BR if Task 0 verifies it; else cold copy | Task 9 |
| Q822 | Snapshot retention | The latest, until the next upgrade verifies | Task 8 |
| Q823 | Automatic rollback on failed verification? | Yes, door closed | Task 13 |
| Q824 | `rollback --keep-writes` for fully reversible upgrades? | Yes | Task 14 |
| Q825 | `--to` beyond one step: walk the chain? | Yes, one verified step at a time | Task 13 |
| Q826 | Release pull requests into `main` merge only with `CI required` green (no admin bypass)? | Yes | Task 20 |
| Q827 | Upgrades from a git checkout? | Only at the target's clean tag | Task 13 |
| Q828 | `migrate` in the `loams` image? | Yes | Task 7 |
| Q829 | Postgres: branches, plus cold copies on a pin change? | Yes | Task 9 |
| Q830 | Console support window | N−1 | Task 16 |
| Q831 | Desktop snapshots before each engine upgrade? | Yes, one kept | Task 18 |
| Q832 | BYOC upgrades by copying the CLI to the VM? | Yes | Task 22 |
| Q833 | Downtime target | 5 min excluding pulls, under 10 GiB outside the bucket | Tasks 20, 24 |
| Q834 | 0.2.0 previews keep data from 0.1.x? | Yes | Before 0.2.0 |

## Rulings made during execution

### Task 0 (2026-10-15, on `dev` at `3ac1e866`)

The owner answered Q820–Q834 with their defaults on 2026-10-14 (decision log). D923–D945 and Q820–Q834 are each
declared exactly once in the decision log (`scripts/docs/check-decision-ids.sh` passes); nothing is renumbered.

**T0-1. Formats and their readers** (Task 1's registry list). Every reader below checks the **exact** version today
unless marked; Task 1 moves each to `formats::check`.

| Registry id (proposed) | Constant, file | Reader today |
|---|---|---|
| `log.wal`, `log.segment` | `VERSION = 1`, `crates/loams-log/src/{wal,segment}.rs` | exact |
| `collection.manifest` (`OPCM`) | `MANIFEST_FORMAT_VERSION = 1`, `crates/loams-collection/src/manifest.rs` | exact |
| `collection.pk_delta` (`OPPD`), `collection.dead_letters` (`OPDL`) | `PK_DELTA_VERSION = 1` (`pkindex.rs`), `DEAD_LETTERS_VERSION = 1` (`deadletter.rs`); both read through `pkindex::open_envelope(magic, version, ..)` | exact, in the one helper |
| `collection.record` | `CODEC_VERSION = 0x01`, `codec.rs` | exact |
| `collection.pk_value` | `VALUE_VERSION = 0x01`, `pkindex.rs` | exact |
| `text.split_trailer` (Quickwit's `QWFT` footer trailer) | `SPLIT_FOOTER_TRAILER_VERSION = 1`, written by `crates/loams-quickwit/src/storage/bundle_storage.rs`; read there and by `crates/loams-text/src/split.rs` and `crates/loams-hot/src/splits.rs` (`TRAILER_VERSION = 1` each) | exact, three readers of one format (**not in §54 §17** as one format) |
| `text.delete_bitmap` (`OPDB`) | `DELETE_BITMAP_VERSION = 1`, `crates/loams-text/src/bitmap.rs` | exact |
| `text.checksums` | `VERSION = 1`, `crates/loams-query/src/text/checksums.rs` | exact (**not in §54 §17**) |
| `hot.artifact` (`OPHD`/`OPHC`) | `ARTIFACT_FORMAT_VERSION = 1`, `crates/loams-hot/src/artifact.rs` | exact |
| `hot.owner` | `"v1"`, `crates/loams-hot/src/registry.rs` | exact string |
| `hnsw.flat` | `FLAT_VERSION = 1`, `crates/loams-hnsw/src/flat.rs` | exact (**not in §17**) |
| `link.counter` | `FORMAT_VERSION = 1`, `crates/loams-link/src/counter.rs` | exact |
| `git.segment`, `git.checkpoint` | `SEGMENT_VERSION`/`SEGMENT_MIN_VERSION`, `CHECKPOINT_VERSION`/`CHECKPOINT_MIN_VERSION`, `crates/loams-git/src/format/` | **range** already |
| `git.catalog_doc` | `DOC_VERSION = 1`, `crates/loams-git/src/catalog.rs` | exact |
| `git.pack` (`.lpk`), `git.head_hint` | none (a reserved zero word; unversioned JSON) | none: "absent means 1" |
| `meta.raft_snapshot` (`OPNMETA`) | `SNAPSHOT_FORMAT_VERSION = 8`, `crates/loams-meta/src/codec.rs` | **range 5–8** already |
| `meta.raft_log`, `meta.redb` | bare postcard (`Command` variant order) | none: "absent means 1" |
| `meta.tikv_record` | `FORMAT: u8 = 1`, `crates/loams-meta-tikv/src/keys.rs` | exact |
| `meta.pointer` | none (object path + CAS version) | none: "absent means 1" |
| `graph.manifest`, `graph.changeset`, `graph.portable` | `MANIFEST_FORMAT = 1` (`snapshot/manifest.rs`), `RECORD_VERSION = 1` (`changeset/codec.rs`), `PORTABLE_VERSION = 1` (`snapshot/portable.rs`) | exact |
| `graph.recovery_stamp` | `STAMP_FORMAT = 1`, `crates/loams-graph/src/recovery.rs` | exact (**not in §17**) |
| `graph.catalog_doc` | none (`struct Document { graphs }`, `catalog.rs`) | none: "absent means 1" |
| `live.record` | protobuf `format = 1` checked as `!= 1` in `crates/loams-live/src/{docs,catalog,deploy,backfill,txn}.rs` | exact (literal `1`, no constant: Task 1 adds one) |
| `live.journal` | `JournalEntry`, no format field | none: "absent means 1" |
| `durable.sqlite` | Resonate's `_sqlx_migrations` (`0001_initial`) | see T0-6 |
| `house.dir`, `house.catalog` | no stamp; `catalog.sqlite` tables `namespaces`, `tables` (`fabric/crates/loams-house/src/catalog/local.rs`) | none: "absent means 1" |
| `auth.directory` | none: the auth principal directory, JSON in `system/directory/`, named by pointer `principal-directory` in namespace `_system` (`crates/loams/src/auth/directory.rs`). It holds API key and session hashes | none: "absent means 1" (added by Task 1) |
| `pg_control.record` | `FORMAT: u8 = 1`, `crates/loams-pg-control/src/model.rs` | exact |
| `pg_control.secrets_file`, `pg_control.ca` | `FILE_FORMAT = 1` (`secrets/file.rs`), `CA_FORMAT = 1` (`runtime/certs.rs`) | exact (**not in §17**) |
| `pg_control.compose_state` | `STATE_FORMAT = 1` (`runtime/compose.rs`, pg-runner's state files) | written, **never checked** (**not in §17**) |

Out of the registry, with the reason: wire-only versions that are never persisted (`loams-live` `CURSOR_VERSION`,
`loams-query` `TICKET_VERSION`, `loams-git` `WIRE_VERSION`, `loams-house-ipc` `PROTOCOL_VERSION`,
`loams-safekeeper` `PROTO_VERSION`); stores that keep their own ledger (agentd's `schema_migrations`,
`WORKSPACE_SCHEMA_VERSION = 2`; the desktop sidebar browser's ledger and profile versions); and crates no compose
stack runs (`loams-safekeeper`'s TiKV/journal formats, keyspace `loams_pgwal`; `loams-sqlrouter`'s `RECORD_FORMAT`).
Each joins the registry when a stack starts persisting it.

**T0-2. The `_system` key space.** The `MetaStore` trait (`crates/loams-common/src/meta/store.rs`) has **no generic
key-value call and no multi-key transaction**. What it has: namespaces; per-namespace **pointers** (`cas_pointer`:
string key → string value with a CAS version, optionally fenced by a lease epoch; key and value each at most
`MAX_KEY_LEN` = 1024 bytes); and global **leases** (`acquire_lease`/`renew_lease`/`release_lease`, fenced epochs).
Each call applies atomically on both backends (one TiKV transaction through `loams-tikv`'s runner; one Raft
`Command`). **Correction (Task 1):** the namespace `_system` **already exists in preview.2**. The auth principal directory
(`crates/loams/src/auth/directory.rs`, `SYSTEM_NAMESPACE`) creates it on its first write and keeps the pointer
`principal-directory` there, so §54 §17's "no `_system` namespace" is wrong. Nothing reserves the name, though:
`validate_name` accepts it (only streams, links and collections refuse a leading `_`), and the API layer checks
nothing, so a user could also have created it. Ruling for Task 2:
- The ledger uses the existing primitives, so neither backend's record or snapshot format changes, and preview.2's
  binaries still read the metastore after adoption (this is what makes `0000-adopt-preview-2` `Reversible` on the
  Raft backend too; a new `Command` variant or snapshot v9 would not be):
  - the namespace `_system` (created by `migrate`; the metastore keeps accepting it);
  - pointers `upgrade/stack`, `upgrade/ledger/<seq:08>-<id>` and `upgrade/snapshots/<ulid>` in it;
  - the lease `_system/upgrade/lease` (every ledger CAS carries its fence).
- There is no call that lists pointers. Task 2 adds one read-only trait method, `pointers_with_prefix(consistency,
  namespace, prefix)`, the pointer twin of `leases_with_prefix`. On Raft it is a state read and on TiKV a prefix
  scan, so neither backend's stored format changes. The other implementations (`loams-safekeeper`'s and the
  `loams-common` test double) get it too.
- Values are a one-byte value version plus postcard, base64url-encoded, kept under 1024 bytes:
  - error messages are truncated to 256 bytes;
  - cursors are capped at 256 bytes;
  - the stack record's format map is keyed by registry id (Task 1 asserts the encoded record fits).
- **Deviation:** without a multi-key transaction, "stack record and last entry commit together" becomes "the stack
  record is the commit point". The runner marks the last entry `Done`, then CASes `upgrade/stack`. A crash between
  the two leaves every entry `Done` and the stack at the old release. Startup refuses that state with
  `migration_incomplete`, and a rerun of `migrate run` only writes the stack record. Task 2's test
  `stack_record_and_last_entry_commit_together` checks this ordering.
- Reserving `_system`: `CreateNamespace` in the API layer (`crates/loams/src/api`) refuses it with
  `namespace_reserved`, and list calls omit it. Task 5's fingerprint accepts a `_system` namespace that holds only the
  `principal-directory` pointer and `upgrade/*` pointers (no streams, collections, links or graphs), and
  refuses anything else (`stack_unrecognised`, naming the namespace). The `principal-directory` pointer is
  another conditional preview.2 marker (T0-6): preview.1 had no auth directory.

**T0-3. The upgrade client crate.** CL1 Task 1 has not merged: there is no `crates/loams-cli`. Tasks 12–14 create
`crates/loams-upgrade` (a library with CL1's no-server-dependency guard), flattened into the `loams` binary as
`loams upgrade`.

**T0-4. GC root sources** (where Task 8's snapshot roots must be honoured). Bucket objects are deleted in:
- `crates/loams-log/src/gc.rs`: retired WAL objects and segments (`retired_expired`), and orphans;
- `crates/loams-collection/src/gc.rs`: reachability from `collection_roots`, and orphans;
- `crates/loams-link/src/gc.rs`;
- `crates/loams-git/src/gc.rs`;
- `crates/loams-graph/src/catalog.rs` (a deleted graph's purge);
- `crates/loams-graph/src/snapshot/manifest.rs` (old graph snapshots).

Two more stores in the bucket keep **mutable manifests of their own**, outside the metastore:
- each collection's Lance dataset (`ns/<ns>/collections/<c>/lance/_versions/*.manifest`). An `OPCM` names a Lance
  version, so a restored metastore needs that version's files to still exist;
- each PK index, which is a SlateDB database (`ns/<ns>/pk/<collection>/db/manifest/*`, `compacted/*.sst`; crate
  `loams-pk`). Its `PkWatermark` is checked against the manifest, so an index that is ahead of a restored manifest
  is rebuilt (Task 8 verifies this).

So Task 8 must also hold back Lance's version cleanup and SlateDB's own GC while a root exists, or treat them as
caches. A TiKV/BR snapshot is opaque, so a root cannot list the objects it reaches by reading the snapshot.

Recommended for Task 8: the root records the snapshot's creation time T. While the root exists, every deletion
site keeps every object created at or before T. The bucket is immutable, so the snapshot's state reaches only
objects created by T. The cost is the space that compaction frees during one release period. This is simpler and
safer than a per-object root list. Task 8 confirms or replaces it.

**T0-5. Durable.** `standalone` defaults to SQLite at `<data-dir>/durable/default.db`, in the compose stack at
`/var/lib/loams/durable/default.db` (`crates/loams/src/main.rs`, `apply_durable`). It opens with
`servers.server_sqlite.migrate = true` (`crates/loams-durable/src/config.rs`), so serving migrates implicitly. The
schema level is the rows of Resonate's `_sqlx_migrations` (`success = 1`). `loams durable migrate` prints only
`loams durable: <store> is migrated`. `mysql::check_schema` only checks that at least one migration is applied.
Task 6 adds the per-id comparison against Resonate's compiled migrator.

**T0-6. A preview.2-only data marker: none unconditional.** preview.1 (`v0.1.0-preview.1`) and preview.2 use the
same stack layout (TiKV `loams_meta` and `loams_live_dev`, the same release features). Every format is at the same
version in both. What preview.2 adds over preview.1 appears only when a product is used:
- Graph's implicit streams `_graph.<gr>` (GR1 Task 12; preview.1's were `graph.<gr>`);
- Live catalog kind `0x07`, table jobs (LV1 Task 8).

Ruling for Task 5:
- Adoption takes the declared release (`--declared-release 0.1.0-preview.2`, from the stack's `LOAMS_VERSION` or
  the desktop's `.loams-stack-version`) as the required positive marker.
- The conditional markers strengthen it.
- A legacy `graph.<gr>` stream, which only preview.1 wrote, is a negative marker: it refuses with
  `stack_unrecognised` even when the declared release says preview.2.

**T0-7. BR on TiKV v8.5.8 with API V2 keyspaces (rules Q821: BR).** Checked with `pingcap/br:v8.5.8`
(`sha256:cddec1ce88dfe1f81afdd23f0024233fecde87ee2a6997d705c8b77eb9aa76eb`, built 2026-08-27) on a scratch
PD+TiKV v8.5.8 with the release `tikv.toml`/`pd.toml` (API V2; keyspaces `loams_meta`, `loams_live_dev`, `sql_dev`,
`loams_pgwal`):
- `br backup full` covers only TiDB's tables. `br backup txn` (experimental) without `--start`/`--end` backs up the
  whole transactional key range `[, )`, every keyspace's `x`-prefixed keys included, at one `BackupTS`. Observed:
  12 ranges, 74 regions, 1703 keys, all succeeded.
- `local://` storage works when the same host directory is mounted into `tikv` and BR at the same path (TiKV writes
  the SSTs, BR the `backupmeta`).
- Restore is `br restore txn` into a fresh PD+TiKV. Keyspace ids come from PD's `pre-alloc` order, so the same
  `pd.toml` gives the same ids. PD's TSO in the new cluster is ahead of `BackupTS` because the wall clock moved on.
  Restore round trip on the T0-11 fixture stack:
  - `br restore txn` into a fresh PD+TiKV restored all 1890 keys.
  - `loams_meta` and `loams_live_dev`, dumped again, were **byte-identical** to the dumps taken before the backup.
  - TiDB started on the restored `sql_dev` without bootstrapping again, and read its table.
  - The preview.2 engine on the restored cluster read the collection (count 2), the stream (both records), the
    graph (both nodes) and the Durable promise.
- No Loams store uses raw KV in the compose stack. Only `loams-safekeeper` (`tikv_raw.rs`) does, and it does not
  run there. So `txn` covers everything.

Because `txn` is marked experimental, the cold copy (`--tikv cold`) stays as the fallback, and Task 9 implements
both.

**T0-8. TiDB on a version bump.** TiDB bootstraps its keyspace when the process starts:
- the log shows `bootstrap.go … "bootstrap successful" [keyspaceName=sql_dev]`;
- `mysql.tidb.tidb_server_version` is `232` on `v8.5.8-loams.1`;
- `loams-sqldb` renders one `tidb-server` pool per branch keyspace (`crates/loams-sqldb/src/render/tidb.rs`).

So a TiDB pin bump runs TiDB's upgrade DDL for each keyspace when its `tidb-server` first starts on the new
version, not in `CreateDatabase`. That follows upstream's `BootstrapSession` → `upgrade` when the stored version is
older; it is checked by the gate on the first TiDB pin bump (D933). §54 §15's (verify) for §47 §5 resolves to
"on start, per keyspace".

**T0-9. The pageserver and safekeeper APIs.** The compose stack runs **upstream** Neon images pinned by digest
(`deploy/pins/loams-postgres-images.toml`, `ghcr.io/neondatabase/neon@sha256:7a4f1249…`), not a fork build yet.
Checked on a scratch `deploy/loams-postgres-dev` stack (project `up1-t0-pg`):
- **Branch:** `POST /v1/tenant/{t}/timeline` with `{new_timeline_id, ancestor_timeline_id, ancestor_start_lsn}` at
  the parent's `last_record_lsn` creates the branch (`ancestor_lsn` equals it). `loams-postgres`'s
  `NeonClient::create_timeline` with `TimelineCreate::branch` already sends this.
- **Pageserver LSNs:** `GET /v1/tenant/{t}/timeline/{tl}` gives `last_record_lsn`, `disk_consistent_lsn` and
  `remote_consistent_lsn` (`loams_postgres::pageserver::TimelineInfo`).
- **Safekeeper LSNs:** `GET http://safekeeper1:7676/v1/tenant/{t}/timeline/{tl}` gives `flush_lsn`, `commit_lsn`,
  `remote_consistent_lsn` and `backup_lsn` (`loams_postgres::wal::WalClient`).
- **The drain has to force an upload.** With the compute stopped, `remote_consistent_lsn` stays behind the
  safekeeper's `flush_lsn` until the pageserver's `checkpoint_timeout` (10 min by default). The checkpoint API
  (`PUT …/checkpoint`) answers 400 "compiled without testing APIs" on this image. What works:
  `PATCH /v1/tenant/config {"tenant_id", "checkpoint_timeout": "5s"}` brought `remote_consistent_lsn` up to
  `flush_lsn` within about 20 s. Then `{"checkpoint_timeout": null}` restores the default (the effective config
  shows `10m` again).
- Task 9's `drain_safekeepers`:
  1. lower `checkpoint_timeout` per tenant;
  2. poll until every timeline's pageserver `remote_consistent_lsn` ≥ the safekeeper's `flush_lsn` (timeout 5 min);
  3. restore the tenant's previous override.
- The safekeeper's own `backup_lsn` lags too. That is the WAL D945 protects: recreating `safekeeper1` before the
  drain would lose it.
- Without `pg-control` there is no timeline swap or rename. Restoring a branch means pointing the compute at the
  `upgrade-<id>` branch's timeline id. `pg-control`'s `RestoreBranch` (`proto/loams/postgres/v1/postgres.proto`)
  resets a branch in place. Task 9 keeps both paths.
- The `postgres-control` profile (`pg-control`, `pg-runner`, volumes `pg-control-data`, `pg-runner-data`,
  `pg-runner-token`) is **already in preview.2**. §54 §3 S9's "new in preview.3" is corrected, and `pg-runner-data`
  (pg-runner's compose records, `STATE_FORMAT`) is added beside S9.

**T0-10. Compose `depends_on: condition: service_completed_successfully`.** Checked with `podman compose` (Podman
6.1.1, which runs Docker Compose v5.2.0):
- a one-shot service that exits 0 starts its dependant;
- one that exits 65 stops `up` with `service "migrate" didn't complete successfully: exit 65`, and the dependant
  does not start.

The condition is in the Compose specification since Compose 1.29, so the bundle's minimum (v2.20.3) has it. No
`podman-compose` (Python) is involved, because `podman compose` delegates to Docker Compose here.

**T0-11. The preview.2 fixture.** `crates/loams-migrate/tests/fixtures/preview2/` (its `README.md` gives the
commands) holds:
- the `loams_meta` and `loams_live_dev` keyspaces as JSON lines;
- the Durable SQLite file;
- the bucket objects the pointers name.

It was written by the engine built from `dev` at `3ac1e866`, the state preview.2 is tagged from. The release
workflow had not published the preview.2 images yet, so it is not the published image. The fixture holds a
collection, a stream, a graph (with its `_graph.<gr>` stream, T0-6's conditional marker) and two Durable promises.

The new example `crates/loams-tikv/examples/dump_keyspace.rs` dumps a keyspace from one snapshot. It is read-only,
and it is how the gate (Task 20) and Task 5's TiKV-backed test re-capture a stack.

Hermetic tests cannot run TiKV (`loams-tikv` has no in-memory backend; its tests need `LOAMS_TEST_PD`). So Task 5
reads the metastore dump through `loams-meta-tikv`'s record decoders over an in-memory key-value view. Writing the
dump into a real keyspace is an `#[ignore]`d test.

**T0-12. Crates that own formats and their dependency on `loams-common`.** `loams-live`, `loams-durable`,
`loams-pg-control`, `loams-quickwit` and fabric's `loams-house` do not depend on `loams-common` today. Task 1 adds
the dependency; it is light (serde, thiserror, async-trait, tracing). The fabric workspace already depends on
engine crates by path (`loams-store`, `loams-cache`), so `loams-house` takes `loams-common` the same way.

**Release features for local checks:** the release image builds `loams` with
`live,live-tikv,durable,graph,tikv,pgwire,mysql-wire,stream-grpc` beside the defaults
(`deploy/release/Containerfile.loams`). Task 4's `cargo test -p loams` uses that set.

### Tasks 9–23 (2026-10-15/16)

The pull requests have the details; this is the list.
- **T9-1** (#638): the release `loams` image runs `migrate` and is built without `postgres`. So `loams-postgres` (the
  pageserver and safekeeper HTTP client) is in the default build through `loams-migrate`, and PG2's default-build
  guard now checks `loams-pg-control` only.
- **T10-1** (#641): the included Postgres compose file cannot depend on `migrate`. A hand-run `compose up` across
  preview.2 → preview.3 is unsupported, and `loams upgrade` recreates `safekeeper1` only after `migrate` succeeds.
- **T11-1** (#644): until CLI2's signed manifest exists, the manifest's `upgrade` object is its own checksummed asset
  (`loams-upgrade-<version>.json`).
- **T12-1, T12-2** (#647): a downloaded bundle is checked against the release's `SHA256SUMS`, and there is no
  `--allow-unsigned`. The crate's module split follows the work.
- **T13-1 to T13-3** (#650):
  - TiKV is copied cold (one `tar`) when the running stack's `tikv` does not mount the snapshot volume (preview.2),
    and with BR otherwise;
  - the snapshot's host files live in `<stack>/.loams-upgrade/snapshots/<id>/host/`;
  - verify reads the engine's `/ready` through `compose exec`, because the engine listens on loopback.
- **T14-1** (#651): without `pg-control`, a rollback lists the snapshot's Postgres branches but does not repoint
  computes.
- **T14-2** (#651): `migrate run` of an older release over newer data is a no-op when the formats are readable and
  every newer migration is `Reversible` and done. This matches the startup gate, and `--keep-writes` needs it.
- **T16-1 to T16-3** (#645):
  - plugins declare `requires.engine` beside the existing `requires.api`;
  - the House and Live plugins keep their own "not served" pages;
  - `GetInstance.features` keys are `feature.*` and `role.*`.
- **T17-1** (#646): the SDKs' public symbols are found with per-language declaration patterns.
- **T19-1, T19-2** (#652):
  - the fixtures run in a container that shares the `loams` container's network namespace;
  - Live is checked as served, with no deployed app yet.
- **T20-1, T20-2** (#656):
  - on `dev` the candidate is tagged as the newest release of the train;
  - the planted false-`Reversible` migration is not built, and the `--keep-writes` step is the reversibility check.

  The gate's first green run, preview.2 → candidate, took (in seconds): upgrade 67, rollback 34,
  `rollback --keep-writes` 28. The candidate build took 815.
- **T22-1** (#653): the bundle carries no binaries. `byoc/upgrade.sh` fetches the release's Linux package on the VM
  (amd64 only).

Gaps carried forward:
- agentd's directory is not in the CLI's snapshot;
- `pg-runner`'s own compute projects are not quiesced;
- the Postgres and House fixtures are skipped in the gate (no compute, no House profile);
- Tasks 15, 18 and 24 are open.
