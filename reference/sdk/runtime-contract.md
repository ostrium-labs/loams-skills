# The SDK runtime contract

Design §44 §7.4 states the behaviour in a paragraph; this page states it as
clauses a conformance suite can check, which is what makes the thirteen SDKs
comparable. SDK1 Task 2 owns the whole document. This is the TypeScript slice:
the clauses SDK2 Task 0 implemented and tests, with the rest marked as what is
still owed.

The conformance corpus is `sdks/fixtures` (design §44 §10.4), and each SDK's
suite replays it through its own public surface. A clause below is only true of
an SDK once its suite pins it.

## R1 — Credentials

A client's bearer comes from a token source and travels in
`Authorization: Bearer`, never in the query string and never in a URL. A
`401` carrying `ErrorInfo.reason = token_expired` triggers **exactly one**
refresh and **one** retry; a second expiry is reported. A source that cannot
refresh — an API key, which does not expire — makes the refresh a no-op.

*Pinned by* `typescript_token_source_refresh`. The token exchange
(`oidcExchange`) is written to RFC 8693 and is **not** exercised: the instance
serves no OAuth endpoint yet (MT, API1 Task 7).

## R2 — Retry classes and backoff

A call's retry class comes from the generated bindings, not from a guess: reads
(`NO_SIDE_EFFECTS`) and idempotent RPCs retry on their own; a mutation does not,
unless it carries an idempotency key. A retryable code is `unavailable`,
`deadline_exceeded` or `resource_exhausted`. The numbers are M1.6 Ruling 5's and
are the same in every SDK: base 100 ms, doubling, capped at 2 s, 3 retries,
**full** jitter. A server-sent `RetryInfo.retry_delay` replaces the computed
backoff, up to 30 s.

*Not yet implemented anywhere:* reading `RetryInfo`. No proto carries it, so
the jittered backoff is all an SDK can do today.

*Pinned by* `typescript_retry_reuses_idempotency_key`.

## R3 — Idempotency keys

A mutating call that carries an `idempotency_key` field is given one **per
logical call**, before the first attempt, and **the same key goes out on every
retry**. A key regenerated per attempt turns one write into two, which is the
failure the key exists to prevent. An SDK mints a UUIDv7 unless the caller
supplies a key. A request with no such field is left alone.

*Pinned by* `typescript_retry_reuses_idempotency_key`.

## R4 — Consistency tokens

A write answers with a consistency token; a read accepts one, so a caller that
just wrote can read its own write.

**The encoding** (amended by SDKP, 2026-10-19) is the server's,
`crates/loams-collection/src/token.rs`:

```
v1:s<stream>/p<partition>@<offset>,s<stream>/p<partition>@<offset>,…
```

Each item is the next offset after a write, for one `(stream, partition)`.
Every number is canonical decimal (no sign, no leading zeros), so a token has
one spelling. Streams and offsets are `u64` and partitions `u32`, and an SDK
keeps them exact (a `bigint`, an `int`, a `uint64`, never a double). Every SDK
exposes a `ConsistencyToken` value with:

- **`parse`**: the text form, or `invalid_argument` with reason
  `invalid_consistency_token` raised client-side for a malformed token, before
  any network call;
- **`to_string`**: `v1:` plus the items in the token's order;
- **`merge`**: both tokens' items, the larger offset kept per
  `(stream, partition)`, normalised (sorted by stream, then partition, one item
  per pair) — exactly what `ConsistencyToken::merge` on the server produces;
- **equality** after normalising.

`sdks/fixtures/tokens.json` holds the server's own parse, print, merge and
malformed cases; `<lang>_consistency_token_merge` runs it.

**The session store** is off by default. With `session_consistency` on:

- every token a response carries merges into the session: the
  `consistency_token`, `token` or `read_token` field, and the
  `loams-consistency-token` response header. A string that is not a `v1:` token
  (a page token, a bearer) is ignored; a `v1:` string that does not parse is
  counted and ignored, because the call it came back from has already happened
  and turning it into an error invites a second write;
- every **collection or SQL read** (`loams.collection.v1`, `loams.sql.v1`)
  whose request has a `consistency` field, and leaves it unset, is sent
  `at_least` the session's token, in that field. An explicit `consistency`
  always wins, and a call can opt out of the session;
- **graph reads never take the session's token.** A graph's token names one
  graph's stream and a Graph read refuses a token naming any other, so a
  session that mixed them would fail the read; a graph caller passes the token
  of its own write explicitly. The session keeps one merged token per package,
  and `merge` takes the max per `(stream, partition)` within it (SDKP-17).

*Pinned by* `<lang>_consistency_token_merge` (the corpus) and
`<lang>_session_read_your_writes` (a write's token is the next read's
`at_least`).

## R5 — Unavailable services

Design §44 §4, D600. Two halves, and an SDK needs both.

1. **Without calling.** `GetInstance.services[]` says which packages this binary
   carries. One call, no auth, cheap. `available()`, `served()`,
   `unavailable()` and `guard()` wrap it; the catalogue is cached for the life
   of the process and concurrent readers share one in-flight fetch.
2. **When the caller calls anyway.** Every RPC of an absent package answers
   `unimplemented` with `reason = feature_not_in_variant` and the variant in
   `metadata.variant`. An SDK turns that into a dedicated error type, so the
   branch is `instanceof FeatureNotInVariantError` or
   `err.reason === 'feature_not_in_variant'` — never the package name, which is
   a proto detail, and never the message.

`guard()` raises the *same* error type, so one `catch` covers "the guard said
no" and "the server refused", and the guard costs no RPC once the catalogue is
cached.

*Pinned by* `typescript_conformance_all_required_fixtures`, in all three shapes:
the guard, the unary refusal, and the refusal on a stream.

## R6 — Pagination

A paged call is `page_size` in, `next_page_token` out, and the facade exposes
both the raw page call and an iterator that follows the tokens to the end and
yields **items**, not pages. One function serves every paged RPC, because the
generated binding names the two fields; a per-call alias (`listAll`) appears
when there is a generated signature to hang it on.

*Pinned by* `typescript_pagination_iterator` against a stub, because **no RPC
is paged yet** — `ListCollections` arrives with API1 Task 2. The end-to-end half
of that test is a deliberate skip, not an omission: a fixture for an RPC the
server does not serve would test the stub rather than the SDK.

## R7 — Streams

A server stream is an `AsyncIterable`. There is no client streaming and no bidi
(D420): a browser cannot do it over `fetch`, and half-duplex works through
every proxy.

A stream is the one call where "retry it" is not enough. The server hands out
cursors; a reconnect resumes from the last one the client applied, or the client
silently misses everything that changed in between — worse than an error, because
a sync UI that is quietly stale looks like one that works. So an SDK tracks the
cursor of every message, re-opens from it on a retryable failure, and does not
re-yield what it already yielded. A failure the retry class does not cover —
notably an `unimplemented` stream — is reported rather than spun on.

*Pinned by* `typescript_stream_resume_with_cursor` against a stub, since the only
server stream (`LiveService/Watch`) is never served in any variant. The
end-to-end half is covered once a watchable service is served.

## R8 — Errors

Design §44 §7.4, D611. A failed RPC carries a Connect code and one
`loams.errors.v1.ErrorInfo`. The **code** gives the class (a taxonomy that
does not change within a major version) and the **`reason`** is the stable
branch. `reason` is a type in the SDK, generated from the registry page, so a
caller switching on it is exhaustive and a reason the registry has lost stops
compiling. `metadata` is structured context; `hint` is a next step in the
caller's locale.

Three cases are distinct and must not be conflated:

- a reason from a **newer** server, which this SDK's registry does not have: it
  is surfaced as text and flagged, not dropped, because losing it would leave a
  caller unable to tell "not supported here" from "not supported at all";
- a failure from **below the API** — a socket, a CORS rejection, an abort —
  which carries no `reason` at all;
- a `LoamsError`, which is a mapped one and is returned unchanged if mapped
  twice.

*Pinned by* `typescript_error_reason_mapping`, over all twenty-five reasons in
the registry.

## R9 — Version reporting

An SDK declares the proto revision it was generated from (`PROTO_REV`), and
reports the server's `GetInstance.api_versions` beside it. A package the SDK
speaks that the server does not serve is a **warning, not an exception**: the
SDK still works for the modules that are there, and the caller decides what a
missing one means.

*Pinned by* `typescript_conformance_all_required_fixtures`.

## R10 — The browser is a first-class target

One port serves the Connect protocol, gRPC and gRPC-Web (D600), and a browser can
only do gRPC-Web. So the same client object runs in a browser with no proxy and
no sidecar. An SDK's default entry point must therefore reach for no Node
built-in, and an HTTP/2 transport must live behind a subpath a bundler will not
follow.

*Pinned by* `typescript_grpc_web_transport_in_a_browser*, which runs the same
assertions over the Connect and gRPC-Web transports and checks the entry points
statically. **What it cannot pin** is a CORS preflight or a browser-specific
header rule: that needs an origin and a server that answers preflights, and no
test in this repository has one.

## R11 — The default namespace

A client may be built with a default namespace (`namespace` on the client,
`LOAMS_NAMESPACE` in `from_env`). Every request **in an app package** —
`loams.collection.v1`, `loams.graph.v1`, `loams.sql.v1`, `loams.stream.v1` —
that has a `namespace` field and leaves it empty is sent with the default. An
explicit namespace always wins. A request with neither is refused with
`invalid_argument`, reason `namespace_required` (`metadata.field =
"namespace"`), client-side, before any network call. A `namespace` field outside the app packages is not a Loams
namespace (`loams.live.v1` `CreateAppRequest.namespace` names an app's
environment) and is never filled or refused (SDKP-11).

*Pinned by* `<lang>_default_namespace`.

## R12 — Timeouts

Every unary call has a deadline: the client's default (30 s unless the client
says otherwise) or the call's own; zero means none. Streams have no default
deadline, only one a call sets. The deadline is **one budget for the whole
call**, retries and backoff included, and each attempt is sent with what is left
of it (`connect-timeout-ms` on Connect, `grpc-timeout` on gRPC). When the budget
runs out the call fails with `deadline_exceeded` and the client-side reason
`client_deadline_exceeded`, and is **never retried**: the budget is spent. A
server's own `deadline_exceeded` inside the budget is retried as R2 says.

*Pinned by* `<lang>_timeout_deadline`.

## R13 — `documents.write_batch`

A bulk upsert as a loop of ordinary `WriteDocuments` calls, chunked by
operations (default 10 000) and by encoded bytes (default 4 MiB; one operation
over the limit goes alone):

- one chunk in flight at a time, so order holds and a failure stops at a known
  point;
- each chunk gets its own idempotency key, minted before its first attempt and
  reused on that chunk's retries (R3);
- the result is the merge of every chunk's consistency token (R4);
- the first failed chunk stops the load with a `BatchError` carrying the
  client-side reason `batch_partially_applied`, how many chunks and operations
  were applied before it, their merged token, and the failed chunk's own error.
  The applied chunks stay applied.

*Pinned by* `<lang>_write_batch_chunks`.

## R14 — `streams.subscribe`

A client loop over unary `Fetch` (SDKP-2), so it works on every transport and
through every proxy, exposed as the language's native async iteration:

- the next offset is the last record's offset + 1, and the iterator exposes it
  (`offset`), so a caller can stop and resume without losing or repeating a
  record;
- an empty fetch backs off 100 ms, doubling, up to 1 s, and the backoff resets
  when a record arrives; with `max_wait_ms` the server long-polls instead;
- cancelling the iterator (break, `return`, a cancelled context or token) stops
  the loop, and no further fetch is sent;
- a fetch that fails after R2's retries ends the iteration with that error, and
  `offset` still says where to resume.

*Pinned by* `<lang>_streams_subscribe_resumes`.
