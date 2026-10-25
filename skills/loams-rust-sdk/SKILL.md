---
name: loams-rust-sdk
description: "Integrate the loams Rust crate into a service: connect, namespaces, collections, documents, hybrid search, SQL, graph, streams, consistency tokens, idempotency, errors, and retries. Use when writing or reviewing the Rust data path."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Loams Rust SDK

The `loams` crate is the public client. Do not call the engine's internal APIs or
hand-write Connect/HTTP requests.

## When to use

- A Rust service needs to read or write Loams.
- A review finds a custom HTTP client, a retry loop, or a hand-built filter.
- An upgrade changes the SDK surface.

## Inputs

- `LOAMS_ENDPOINT`, `LOAMS_API_KEY` or `LOAMS_TOKEN`, `LOAMS_NAMESPACE`.
- [Rust SDK README](../../reference/sdk/rust.md),
  [runtime contract](../../reference/sdk/runtime-contract.md),
  [design 44](../../reference/design/44-unified-api-and-sdks.md).

## Dependency

```toml
# Local `loams dev` serves http://, so TLS is not needed.
loams = { version = "0.1", default-features = false }

# Production/agents over https, with machine-key assertions:
# loams = { version = "0.1", features = ["tls", "machine-key"] }
```

The crate is 0.x. Pin the minor and read the release notes before upgrading.

## Connect

```rust
let loams = loams::Loams::from_env()?.session_consistency(true);
let opts = loams::CallOptionsOverrides::new;
```

`from_env` reads `LOAMS_ENDPOINT`, `LOAMS_API_KEY`/`LOAMS_TOKEN`, and
`LOAMS_NAMESPACE`. `session_consistency(true)` makes a read see this process's
own writes; a cross-process read needs the consistency token.

## Create idempotently

```rust
use loams::collection_v1::{CreateCollectionRequest, CreateNamespaceRequest};
use loams::builders::Schema;

let ns = loams.namespace().unwrap_or("company").to_owned();
ignore_already_exists(loams.namespaces().create(
    CreateNamespaceRequest { namespace: ns, ..Default::default() }, opts()).await)?;

let schema = Schema::new().text("title").text("body").keyword("tags")
    .vector("embedding", 768);
ignore_already_exists(loams.collections().create(
    CreateCollectionRequest { name: "notes".into(), schema: schema.to_struct()?.into(), ..Default::default() },
    opts()).await)?;
```

Treat "already exists" as success. Do not delete and recreate in a request path.

## Read and write

```rust
let notes = loams.collection("notes", None);
notes.upsert(&[loams::Document::new(1u64)
    .source(serde_json::json!({"title": "Refund policy", "tags": "policy"}))], opts()).await?;

let hits = notes.search(
    notes.query()
        .text(loams::builders::TextRetriever::fields("refund", ["body"]))
        .vector("embedding", &embedding, loams::builders::VectorOptions::k(10))
        .fuse(loams::builders::Fusion::rrf(Some(60)))
        .filter(loams::builders::Filter::eq("tags", "policy"))
        .limit(10)
).await?;
```

Use builders (`Schema`, `Filter`, `Search`, `Sort`, `TextRetriever`,
`VectorOptions`, `Fusion`) instead of constructing proto JSON by hand.

## Errors and retries

- Match `LoamsError`/`ErrorKind`; map `NotFound`, `AlreadyExists`,
  `InvalidArgument`, `PermissionDenied`, `Unauthenticated`, `Unavailable`,
  `DeadlineExceeded`, `ResourceExhausted` to your API.
- Reads and idempotent RPCs may retry; mutations retry only with an idempotency
  key. The SDK mints a UUIDv7 when the request has a key field.
- Use the SDK's retry policy. Do not wrap calls in your own unbounded loop.
- Pass deadlines per call via `CallOptionsOverrides`; do not rely on a global
  timeout alone.

## Consistency

- A write returns a consistency token.
- Pass the token to a read that must see the write across processes.
- Parse, merge, and print tokens with the SDK's `ConsistencyToken`; never build
  the string by hand.

## Guardrails

- Never put credentials in a URL or query string.
- Never log documents, tokens, or embeddings at info level.
- Do not use SQL string interpolation; use parameters.
- Do not assume a collection exists; create it idempotently at startup.

## References

- [Rust SDK](../../reference/sdk/rust.md)
- [Runtime contract](../../reference/sdk/runtime-contract.md)
- [Design 44](../../reference/design/44-unified-api-and-sdks.md)
