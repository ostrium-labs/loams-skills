# The conformance corpus and its harness

Design [§44](../design/44-unified-api-and-sdks.md) §10.4, decision **D617**;
SDK1 Task 4. The runtime contract is [R1–R10](runtime-contract.md); this page is
what actually pins those clauses and — just as important — what does not.

Every SDK's suite replays the same corpus, so "the same behaviour in thirteen
languages" (D617) is measured rather than asserted. The rule is one sentence and
it is enforced in code by [`required.mjs`](../../sdks/conformance/required.mjs),
not by prose:

> A language passes conformance when it has run **every** fixture marked
> `required` in `manifest.json`. The only permitted skip is a fixture marked
> `transport: grpc-only`, on the Connect-unary fallback (D613, for Ruby and PHP
> hosts without a native gRPC extension).

## The layout

```
sdks/fixtures/
  manifest.json          the authority: every fixture, required or not, and the
                         clause each one pins. The runner reads this and nothing
                         else, so the skip rule has one definition
  index.json             the thirteen `loams dev` cases (SDK2 Task 0's shape)
  status.json            what each transport answered, per case
  error.json             one row per recorded reason, and the reasons no server
                         can raise yet, with why
  state.json             the ordered multi-step scenarios and the invariant each
                         one exists to pin
  faults.json            the injected faults (the only thing here not recorded)
  recorded/*.json        the thirteen `loams dev` cases, byte for byte
  recorded/apps-mock/*.json  the twenty app-mock scenarios, one file each
  recorded/app/*.json    the SDKP app scenario (`app_collections`, `app_search`,
                         `app_graph`; `app_sql` and `app_streams` with P0b/P0c),
                         recorded from `loams dev`, replayed through each SDK's
                         public facade
  app-scenario.md        the same scenario in prose
  builders/*.json        the builder corpus: what every SDK's `Schema`, `Filter`
                         and `Search` builders write, case by case
  tokens.json            the consistency-token corpus (parse, print, merge,
                         malformed), shared with `loams-collection`'s
                         `token_corpus_matches`
  facade-index.json      every facade call, generated (`gen.sh index`)
  results/<lang>.json    what a suite says it ran; the input to the 100% bar.
                         **Generated, not committed** (D640) — gitignored, written
                         by the suite on every run, and CI generate-then-checks
```

## Two servers, because neither does everything

`loams dev` is the **primary** target. It answers a successful unary call and both
structured-reason refusals, in every encoding.

It cannot answer a paged list, an idempotency-keyed mutation or a resumable
stream, because `loams.collection.v1` has not landed and the app packages are not
served by it at all (design §44 §8; plan API1 Tasks 2–4). Those come from
**`loams-apps-mock`** (AP0 Task 5), which implements the app services for real —
its `acceptance` module is the server's own decision rules — and which is stateful,
so an idempotency replay and a stream resume mean something.

Both are recorded by one script:

```sh
loams dev --listen 127.0.0.1:8080 --data-dir "$(mktemp -d)" --no-live &
LOAMS_TEST_ENDPOINT=http://127.0.0.1:8080 node sdks/conformance/record-fixtures.mjs
```

`--no-live` matters: the unavailable-service cases are `loams.live.v1` refusing
with `feature_not_in_variant`, which is what it answers while Live is not running.
With Live on (the default since LV1 Task 12) `loams dev` serves the package on its
main port, and `LiveService/Query` answers `not_found` instead.

`--source loams-dev` records only the primary half; `--source apps-mock` only the
second. The mock is started per scenario by
[`apps-mock.mjs`](../../sdks/conformance/apps-mock.mjs), so a scenario replays in
isolation with no ordering dependency on the others.

## Recorded, not written

Every case was captured from a real server, because a hand-written expectation
only proves an SDK agrees with whoever wrote it. The recorder checks each
expectation as it records it, so a fixture committed with the wrong expectation
fails at record time rather than becoming thirteen wrong tests.

Two things are recorded as **prefixes**, because the alternative is not possible:
a stream never ends, so `stream_drop`'s and the watch fixtures' steps record the
first N frames and close. A fixture's `volatile` list then names the fields a
re-recording is allowed to move:

- the server-generated `instance_id`, which `loams dev` derives from its data
  directory, so every fresh one reports a different value;
- the mock's seed timestamps, which are relative to `now`;
- the base64 frame payloads of a recorded stream, where a field name cannot reach
  — a coarser mask, stated in the manifest rather than assumed.

Nothing else may drift. `verify-corpus.mjs --drift` re-records into a temporary
directory and fails on any other difference; it never writes to the corpus.

## The three projections

The recorded bytes are the source; these three files are what a suite reads.

| File | The question it answers | Used by |
|---|---|---|
| `status.json` | what did each transport actually answer? | the transport tests, and R10 |
| `error.json` | which reasons exist, and which cannot be produced yet? | `error_reason_mapping` |
| `state.json` | what has to hold **across** a sequence of calls? | `retry_reuses_idempotency_key`, `stream_resume_with_cursor` |

`status.json` exists because **HTTP status is not the error**. Connect answers a
refusal with a status and a JSON body; gRPC-Web answers `200` and puts the code in
`grpc-status`. A client that reads only status codes passes the first and
silently succeeds on the second. `mock_error_encodings` records one refusal in all
four encodings so that difference is bytes rather than folklore.

`error.json`'s `unproducible` list is the honest half of `error_reason_mapping`. A
suite covers the whole reason registry; today **seven reasons are recorded and
nineteen are not**, and the list says which and why. Deleting a row from it without
the server landing is how a stub quietly becomes the specification.

`state.json` is where R3 and R7 live, and it is a different shape from the other
two: a single-request fixture is a lookup, but a scenario is only meaningful **in
order**, and two of its steps are often byte-identical (the idempotency replay
sends the same request twice). Each row states the cross-step invariant rather
than a per-step expectation.

## The fault injector

Everything above is a recording. The six faults in `faults.json` are the
deliberate exception, and the file says so in its first line: a retryable
`unavailable` needs a dependency to be down, a mid-stream disconnect needs
something to drop the connection, and `token_expired` needs a token endpoint that
does not exist yet. What is still real is the **wire shape**, and that is what the
SDK has to parse.

A request asks for one with a header, so no language has to learn a fake URL to
test a retry:

```sh
curl -H 'loams-test-fault: unavailable' http://127.0.0.1:PORT/loams.instance.v1.InstanceService/GetInstance
```

`unavailable`, `resource_exhausted`, `deadline_exceeded` (R2's three retryable
classes), `retry_after` (R2's server-sent `RetryInfo.retry_delay`, which no proto
carries yet), `token_expired` (R1's refresh trigger) and `stream_drop` (R7's
resume, with real frames and real cursors and the socket destroyed with no end
frame). An unknown name is a `400`, never a pass-through: a typo must not hand a
suite a healthy answer.

## Running it

```sh
sdks/conformance/run.sh                    # against real servers; fails on drift
sdks/conformance/run.sh --no-server        # replay the committed corpus
sdks/conformance/run.sh --drift            # re-record and fail on a difference
sdks/conformance/run-all.sh                # CI: corpus, harness, every language
sdks/conformance/run-all.sh --harness-only # the first two layers only
sdks/conformance/run-test.sh typescript stream_resume_with_cursor   # one test
node sdks/conformance/verify-corpus.mjs    # the gate, on its own
node sdks/conformance/check-languages.mjs --check <lang> --suite app   # SDKP's app suite
LOAMS_TEST_ENDPOINT=… node sdks/conformance/record-fixtures.mjs --source app  # re-record it
node --test sdks/conformance/              # the Task 4 tests
```

`run-all.sh` runs three layers cheapest first, because they fail for different
reasons: the **corpus** verifies, the **harness** replays every fixture byte for
byte and injects every fault, then the **languages** run their six named tests
each. `--strict` turns drift warnings into a failure.

A suite reports what it ran in `sdks/fixtures/results/<language>.json`:

```json
{
  "transport": "connect",
  "tests": ["typescript_error_reason_mapping"],
  "ran": ["live_query_json", "mock_state_idempotent_decide"],
  "skipped": [{ "fixture": "…", "reason": "…" }]
}
```

`ran` is the only thing that counts — a test that passes without touching a
required fixture has not run it — and the runner rejects a name that is not in the
corpus, so a typo cannot read as "that one is done".

**The file is generated, never committed** (D640). A committed report is a claim
about a corpus, made by a suite, and it stops being true the moment either moves
without anything touching the file: a corpus that gains a required fixture, or a
suite that stops driving one. `check-languages.mjs` would then read the previous
run's answer and call it coverage, which is the one thing a gate must not do.
So a suite writes it on every run, clears any earlier one first so a crashed run
leaves none rather than a stale one, and CI generates then checks:

```yaml
- run: pnpm --filter @loams/client test
- run: node sdks/conformance/check-languages.mjs --check typescript
```

`run-test.sh` on a **single** test therefore finds no report and prints its
existing note instead of checking coverage — which is the honest outcome, since
one test does not cover 28 fixtures. Only a whole-suite run can answer the
question.

The TypeScript suite is the reference implementation, because it is the only
language marked `verified: true` and so the only one whose command a runner
actually executes. Three things in it are worth copying and two are worth
knowing about:

- **`ran` is derived, never listed.** The required set is read from
  `manifest.json`; a suite that carried its own copy of the bar would absorb a
  corpus that grows instead of turning red on it.
- **The SDK re-serialises the recorded request**, so `fixture-server.mjs`'s
  byte-for-byte comparison is a real check on the encoder. Replaying recorded
  bytes with `fetch` would pass no matter what the SDK wrote.
- **A recorded code is read from the recording's own bytes** — Connect error
  body, `grpc-status`, or the end-of-stream frame — because an HTTP status cannot
  tell these refusals apart: `400` covers both `invalid_argument` and
  `failed_precondition`, and a gRPC-Web refusal is a `200`.
- **A recorded stream is a prefix** and has no end frame, so a correct client
  reports the missing one. Accepted only when the recording says it was
  truncated, and only for that error.
- **D610's idempotency key means seven app-mock mutations cannot be replayed
  through the SDK's own call path**: they were recorded without a key, and the
  fixture server correctly refuses a request that is not the recorded one. Those
  steps go through the generated client and are still read back through the
  SDK's error mapping.
- **A live `loams dev` writes no report.** It does not serve the app packages,
  so a partial report must not sit in front of the gate.

## What this cannot pin today

Stated here so it is not discovered later:

- **R2 has no recording.** The three retryable classes exist only as injected
  faults, because no server produces them on demand. `RetryInfo` is on no proto at
  all.
- **R4 has nothing.** No RPC carries a `consistency_token`, and its encoding is
  not in the protos yet. Every SDK pins R4 against a stub, deliberately: a
  silently-merged token reads stale data, which is worse than a failure.
- **R6 has nothing end to end.** `ListApprovals` declares `page_size` and answers
  `next_page_token` and honours **neither** — it returns every match and no token.
  `mock_status_list_is_not_paged` is the recording that pins that fact, so when
  API1 Task 2 lands a real `ListCollections` the fixture starts returning a token
  and changes. R6's end-to-end half arrives then.
- **`token_expired` is injected only.** `loams dev` has no authentication at all
  (`WhoAmI` answers `not_implemented`) and the mock knows fresh and stale sessions,
  where stale is `step_up_required`. R1's refresh is pinned against
  `mock_error_step_up_required` and the injected `token_expired`.
- **The mock's `GetInstance` has no `services[]`**, so D600's catalogue half of R5
  is only exercised against `loams dev`.
- **No CORS preflight.** R10's gRPC-Web half is recorded and its static half (no
  Node built-in at a default entry point) is a property of the SDK, but a preflight
  needs an origin and a server that answers one, and this repository has neither.
- **The TS SDK drives all 28 required fixtures but reaches 15 of them through the
  generated stubs, not the facade.** `ApprovalService` and `DeviceService` carry no
  `loams.options.v1.module` annotation yet (API1 Tasks 2–4), so there is no
  `loams.approvals` to call and the driver uses the `CallInvoker` those methods
  would delegate to anyway. When the annotations land, the same fixtures run
  through `loams.approvals` with no change to the corpus.
- **The other eleven languages still write no report.** TypeScript is the first
  because it is the only language marked `verified: true`; the rest are the
  reference implementation to copy, and until they do,
  `check-languages.mjs --check` has nothing to check for them.
- **`--drift` under-reports TypeScript, and it is the static check being static.**
  It counts the fixture names a suite's *sources* contain, so it sees the 15 the
  suite names literally and not the 13 the driver reads out of
  `manifest.json`. Both numbers are true and they answer different questions:
  `--drift` is what a suite pins in text, `--check` is what a run exercised.
  Only the second one is the bar.
