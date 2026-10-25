# Rust starter

A small `actix-web` service over the `loams` crate. Use it as the shape for the first
bounded context; replace the notes domain with yours.

## Run

```sh
loams dev
cp .env.example .env
cargo run
```

Then:

```sh
curl localhost:3000/healthz
curl -X POST localhost:3000/v1/notes \
  -H 'content-type: application/json' \
  -d '{"title":"Refund policy","body":"thirty days","tags":["policy"]}'
curl -X POST localhost:3000/v1/notes/search \
  -H 'content-type: application/json' \
  -d '{"q":"refund","tag":"policy"}'
```

The starter creates its namespace and collection idempotently at startup. It
uses `Loams::from_env()` and session consistency, so a read sees this process's
own write.

## Layout

| Crate | Role |
|---|---|
| `crates/domain` | Types and validation; no I/O |
| `crates/platform` | Loams client, schema bootstrap, calls |
| `crates/api` | HTTP routes and error mapping |

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `LOAMS_ENDPOINT` | required | `http://127.0.0.1:8080` for `loams dev` |
| `LOAMS_API_KEY` / `LOAMS_TOKEN` | none | Bearer for a secured install |
| `LOAMS_NAMESPACE` | `company` | Default namespace |
| `LOAMS_COLLECTION` | `notes` | Collection name |
| `BIND_ADDR` | `127.0.0.1:3000` | HTTP listen address |
| `RUST_LOG` | `info` | Log filter |

## Tests

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The E2E test is ignored by default:

```sh
LOAMS_ENDPOINT=http://127.0.0.1:8080 cargo test --test e2e -- --ignored
```

## Production notes

- Build with `loams = { version = "0.1.1", features = ["tls"] }` when the
  endpoint is `https://`. Add `machine-key` for agent assertions.
- Put the bind address behind the company gateway; do not expose Loams itself.
- Add OTLP, auth, and rate limits before production.
