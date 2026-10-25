# The Loams SDK for Rust (`loams`)

**0.1.1.** `loams` 0.1.1 tracks Loams 0.1.1. In 0.x the API may change between
minor versions; `loams = "0.1"` stays on the 0.1 line.

One `Loams` client with namespaced modules over the unified Connect API,
generated from the protos in [`proto/`](https://github.com/ostrium-labs/loams/tree/dev/proto).

```toml
[dependencies]
loams = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,ignore
use std::sync::Arc;
use loams::{ApiKey, CallOptionsOverrides, GetInstanceRequest, Loams, TransportOptions};

let loams = Loams::connect(TransportOptions::new("http://127.0.0.1:8080"))?
    .with_auth(Arc::new(ApiKey::new(std::env::var("LOAMS_API_KEY").unwrap_or_default())?));

// The first call any client makes: what this instance is, and no auth.
let info = loams.instance().get_instance(GetInstanceRequest::default(), CallOptionsOverrides::new()).await?;
println!("{} serves {:?}", info.name, info.api_versions);
```

## Build an app

The app-building layer (SDKP): builders, a collection handle, and the runtime
an app needs.

```rust,ignore
use loams::builders::{Filter, Fusion, Schema, Search, Sort, TextRetriever, VectorOptions};
use loams::{CallOptionsOverrides, Loams};

let loams = Loams::from_env()?          // LOAMS_ENDPOINT, LOAMS_API_KEY, LOAMS_NAMESPACE
    .session_consistency(true);         // read-your-writes across calls

let schema = Schema::new().text("title").text("body").keyword("tags").i64("n").vector("embedding", 4);

let request = Search::on("notes")
    .text(TextRetriever::fields("refund", ["body"]))
    .vector("embedding", &[1.0, 0.0, 0.0, 0.0], VectorOptions::k(10))
    .fuse(Fusion::rrf(Some(60)))
    .filter(Filter::and([Filter::eq("tags", "policy"), Filter::range("n").gte(1).build()]))
    .sort([Sort::score(None), Sort::pk(None)])
    .limit(5)
    .build()?;                           // a loams::collection_v1::SearchRequest
```

- **Default namespace (R11).** `with_namespace("app")` fills every app request
  that leaves `namespace` empty; an explicit one wins, and neither is
  `invalid_argument` before any network call (`loams::prepare_request`).
- **Deadlines (R12).** Unary calls default to 30 s (`with_timeout`, or
  `CallOptionsOverrides::timeout`; zero disables). The budget covers retries;
  running out is `deadline_exceeded` with `client_reason() ==
  Some("client_deadline_exceeded")`, never retried. Streams have no default.
- **Consistency tokens (R4).** `ConsistencyToken` parses, prints and merges the
  server's `v1:` tokens; the session merges every token a response carries.
- **Bulk writes (R13).** `CollectionHandle::write_batch` sends one chunk at a
  time with its own idempotency key and stops at the first failed chunk with a
  `BatchError` (`chunks_applied`, `rows_applied`, `token`).
- **Subscriptions (R14).** `loams::subscribe(from_offset, fetch)` is a
  `futures::Stream` over `Fetch` whose `offset()` says where to resume.

The typed modules are `loams.namespaces()`, `loams.collections()`,
`loams.documents()`, `loams.search()`, `loams.vector()` and `loams.graph()`, one
method per facade call; `loams.collection("notes", None)` returns a
`CollectionHandle` with `upsert`, `get`, `delete`, `patch`, `write_batch`,
`count`, `scroll_all`, `search` and `vector_search`, and
`loams::handle::query_all` pages a search. `loams.sql()` is `loams.sql`:
`query` answers `columns` and `rows`, `records(statement, vec![$1, …])`
answers one JSON object per row, and `loams::sql_records(&answer)` reads an
answer in hand; `query_arrow` answers Arrow IPC bytes (`arrow_ipc`), which the
`arrow-ipc` crate's `StreamReader` reads. `loams.streams()` is
`loams.streams`: `create`, `get`, `list` (and `list_all`), `produce` (set
`producer_id` and `producer_seq` so a retry cannot append twice), `fetch`,
`produce_events` and `fetch_events` (CloudEvents as a JSON batch); and
`loams.subscribe(request, options)` (R14) is a `futures::Stream` over `fetch`
whose `offset()` is where to resume. It is not `Send`: poll it on the task
that made it.
`examples/notes/main.rs` is the whole app scenario as a program. The session's
token is used only for collection and SQL reads: a graph's token names one
graph (SDKP-17). Durable workflows use the Resonate SDK pointed at
`LOAMS_ENDPOINT/durable/` (SDKP-1).

## Agents: a machine key (feature `machine-key`)

An agent registered at the identity provider (a Zitadel machine user, design 76 §10) signs in
with its private-key JWT key. `MachineKey` reads the key file, trades a 5-minute RFC 7523
assertion for an access token whose audience is the Loams project, caches it, and trades again
a minute before it expires:

```rust,ignore
// LOAMS_AGENT_KEY_FILE, LOAMS_AGENT_ISSUER, LOAMS_AGENT_PROJECT; `poster` is your FormPoster.
let source = loams::machine_key::MachineKey::from_env(poster)?;
```

The feature is off by default because it adds a pure-Rust RSA signer. A sandboxed agent holds no
key: agentd vends it a 15-minute token file instead.

## Building

The generated message and client code comes from `loams-proto` and
`loams-live-proto`, which generate it at build time with `protoc`. Install it
first (`apt install protobuf-compiler`, `brew install protobuf`, or a release
from <https://github.com/protocolbuffers/protobuf/releases>).

The default `tls` feature brings rustls with the webpki roots, for `https://`
endpoints. A loopback `http://` endpoint, which is what `loams dev` serves,
needs none of it: `default-features = false`.

## Licence

Apache-2.0. Source, issues and the other SDKs:
<https://github.com/ostrium-labs/loams/tree/dev/sdks>.
