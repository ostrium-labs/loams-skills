# Rust conventions

The starter is the reference implementation. These rules keep services
consistent.

## Toolchain

- Edition 2024, Rust 1.97.1 (`rust-toolchain.toml`).
- `resolver = "3"`.
- One workspace per deployable group, not one per crate.

## Layout

```
crates/domain/     # types, invariants, ports; pure
crates/platform/   # Loams client, config, telemetry; all I/O
crates/api/        # HTTP/Connect/gRPC, handlers, main
tests/             # cross-crate integration
```

Dependencies point inward: `api` → `platform` → `domain`. `domain` never depends
on `loams`, `actix-web`, or `tokio`.

## Errors

- One error enum per crate with `thiserror`.
- `anyhow` only in `main` or a CLI edge.
- Map Loams errors to your API at the boundary; do not leak `LoamsError` to
  clients.
- Never `unwrap`/`expect` in a request path.
- Include a stable machine-readable reason in API errors.

## Async

- `tokio` multi-thread runtime.
- No blocking I/O on the runtime; use `spawn_blocking` for filesystem or CPU.
- Every external call has a deadline.
- Cancellation is normal: do not leave partial state without an idempotency key.

## Loams

- Use `loams::Loams::from_env()`.
- Create namespaces and collections idempotently at startup.
- Use builders, not hand-written proto JSON.
- Use the SDK's retry policy; mutations need an idempotency key.
- Pass consistency tokens where read-your-write crosses processes.
- Use parameters for SQL; never interpolate.

## Config

- One typed `Config` struct, parsed once.
- Fail fast with a message that names the variable, not its value.
- `LOAMS_ENDPOINT`, `LOAMS_API_KEY`/`LOAMS_TOKEN`, `LOAMS_NAMESPACE`,
  `BIND_ADDR`, `RUST_LOG`.
- Secrets come from the environment or the secret store, never a file in git.

## Logging and telemetry

- `tracing` with structured fields.
- Include request id, tenant, principal, operation, duration, outcome.
- Never log documents, tokens, embeddings, or PII.
- OTLP is optional; metrics and logs must still work without it.

## Lints

```toml
[lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"

[lints.clippy]
dbg_macro = "warn"
todo = "warn"
unwrap_used = "warn"
```

## Tests

- Unit tests in the crate; integration under `tests/`.
- `tower::ServiceExt::oneshot` for handlers.
- An ignored E2E test gated by `LOAMS_ENDPOINT`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`.
