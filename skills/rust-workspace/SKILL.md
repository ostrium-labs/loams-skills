---
name: rust-workspace
description: "Create or extend the company's Rust workspace for Loams services: crate boundaries, edition and toolchain, errors, config, tracing, lints, and tests. Use when starting the Rust rewrite or adding a bounded context."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Rust workspace

Use the `starter/` workspace as the shape; do not invent a new one per service.

## When to use

- Stage 3, when the pilot starts.
- A new bounded context needs a crate.
- A service is being extracted from a monolith.

## Inputs

- `company/architecture/target.md`.
- The first bounded context and its API.
- `starter/` and [Rust conventions](../../docs/rust-conventions.md).

## Shape

```
Cargo.toml            # workspace, resolver 3
rust-toolchain.toml   # 1.97.1
crates/domain/        # types, invariants, ports; no I/O
crates/platform/      # Loams client, config, telemetry
crates/api/           # transport + main
tests/                # cross-crate integration
```

Rules:

- `domain` depends on nothing but `serde`, `thiserror`, and small pure crates.
- `platform` owns the `loams` client and all I/O.
- `api` owns HTTP/Connect/gRPC and maps errors to transport codes.
- One binary per deployable; keep `main.rs` thin.

## Procedure

1. Copy `starter/` or add a crate to its workspace. Keep one workspace per
   deployable group, not one per file.
2. Set `edition = "2024"`, `rust-version = "1.97"`, `license = "Apache-2.0"`.
3. Add workspace lints: `unsafe_code = "forbid"`, `unwrap_used = "warn"`,
   `todo = "warn"`, `dbg_macro = "warn"`.
4. Define errors per crate with `thiserror`; use `anyhow` only at the binary edge.
5. Put config in one typed struct, parsed once from env; fail fast with a clear
   message. Never log secrets.
6. Initialize `tracing` with `RUST_LOG` and optional OTLP.
7. Add tests: pure domain tests, platform tests with a fake, API tests with
   `tower::ServiceExt::oneshot`, and an ignored E2E test gated by
   `LOAMS_ENDPOINT`.
8. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`.

## Outputs

- Workspace and crate skeletons.
- `docs/rust-conventions.md` updates if the company adds a rule.
- CI job that runs the three commands.

## Guardrails

- Do not put Loams types in `domain`; keep the domain independent.
- Do not add `unwrap`/`expect` in request paths; map the error.
- Do not add a dependency that duplicates one already in the workspace.
- Do not use `unsafe`.

## References

- [Rust conventions](../../docs/rust-conventions.md)
- [Starter](../../starter/README.md)
- [Loams Rust SDK](../../reference/sdk/rust.md)
