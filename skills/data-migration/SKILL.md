---
name: data-migration
description: "Plan and execute migration of company data into Loams from Postgres, MySQL, Elasticsearch, Qdrant, Neo4j, Kafka, ClickHouse, or object storage, with dual-write, backfill, verification, and rollback. Use after the pilot and before cutover."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Data migration

Data migration is a product change, not a copy job. The old and new systems must
agree on what a record means.

## When to use

- Stage 4, after a pilot passes.
- A store is being replaced by collections, SQL, graph, streams, House, or the
  bucket.
- A dual-write or backfill needs a verification plan.

## Inputs

- `company/stack-inventory/data-map.md`.
- `company/architecture/fit.md` and `target.md`.
- Source schemas, sample records, volumes, growth, and retention.
- [Design 01](../../reference/design/01-architecture.md),
  [Design 44](../../reference/design/44-unified-api-and-sdks.md).

## Strategy

Pick one per store:

| Strategy | When | Cost |
|---|---|---|
| Compatibility | The source protocol is supported (Postgres/MySQL wire, ES API) | Lowest; keep the client |
| Native API | You want Loams' schema, search, or consistency | Rewrite the data path |
| Bridge | Cut over gradually; Loams in front, old store behind | Dual-write complexity |
| Rebuild | The old data is disposable or can be regenerated | Longest; verify provenance |

## Procedure

1. **Map the schema.** Source fields → Loams fields, types, IDs, vectors,
   analyzers, filters. Write the mapping in `company/runbooks/migration-<store>.md`.
2. **Choose IDs.** Keep stable IDs where external references exist. Document
   how new IDs are minted (UUIDv7 is a good default).
3. **Backfill first.** Read-only snapshot → transform → write in batches with
   idempotency keys. Record counts and a content hash per batch.
4. **Dual-write.** Write to both systems behind a flag. Compare on read or in a
   sampler. Never dual-write without a divergence metric.
5. **Verify.** Counts per partition, checksums on a sample, query-result parity
   for the top queries, vector recall on a labeled set, and latency.
6. **Switch reads.** Shadow reads, then canary, then full. Keep writes dual until
   the rollback window closes.
7. **Rollback.** Keep the old store writable and the backfill replayable. Test
   the rollback before cutover.
8. **Retire.** Stop dual-write, revoke old credentials, archive or delete per
   retention, record the decision.

## Outputs

- `company/runbooks/migration-<store>.md` — mapping, order, commands, owners.
- Backfill and verification scripts (checked in, no secrets).
- `company/runbooks/rollback-<store>.md`.
- A short migration report with counts, hashes, and open gaps.

## Guardrails

- No migration without a measured rollback.
- No PII in logs or test fixtures; use synthetic samples.
- Do not delete the source until the retention and legal owners sign off.
- Do not run a backfill against production without a rate limit and a stop switch.

## References

- [Design 01](../../reference/design/01-architecture.md)
- [Design 44](../../reference/design/44-unified-api-and-sdks.md)
- [Migration playbook](../../docs/migration-playbook.md)
