---
name: loams-fit
description: "Map each inventoried component to Loams primitives and decide replace, keep, bridge, or defer, with an ADR per component. Use after stack inventory and before target architecture."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Loams fit

The deliverable is a decision per component, not a wish list.

## When to use

- Stage 1, after `company/stack-inventory/` exists.
- A new system appears and needs a disposition.
- A "replace" decision needs a challenge.

## Inputs

- `company/stack-inventory/inventory.md` and `data-map.md`.
- Access patterns: read/write ratio, query shapes, consistency, latency, volume.
- [Design 01](../../reference/design/01-architecture.md),
  [Design 44](../../reference/design/44-unified-api-and-sdks.md),
  [Design 70](../../reference/design/70-byoc-clouds.md),
  [Design 76](../../reference/design/76-identity-and-authorization.md).

## Mapping

| Existing component | Loams primitive | Notes |
|---|---|---|
| Elasticsearch / OpenSearch | Collections + search | Text, vector, hybrid, filters, highlights |
| Qdrant / Pinecone / pgvector | Vector fields + search | Native vector, not a separate service |
| Neo4j / graph DB | Graph over tables/collections | Check traversal depth and query shape |
| Kafka / NATS / SQS | Streams | Check ordering, replay, consumer groups |
| ClickHouse / warehouse | House / analytics tables | Check SQL dialect and freshness |
| PostgreSQL / MySQL | SQL + wire compatibility | Compatibility first, native later |
| S3 / GCS / Blob | Bucket | Company-owned bucket and KMS key |
| Temporal / durable workflows | Durable runs | Check retry, timers, signals, history |
| Auth0 / Keycloak / Okta | OIDC issuer + Zitadel/OpenFGA | Keep the company IdP where possible |
| Cron / batch | Durable + streams | Check idempotency and catch-up |
| Custom search service | Collections + SDK | Rewrite only the hot path in Rust |

## Procedure

1. For each component, state the **access pattern** in numbers: QPS, p95, data
   size, growth, consistency, retention, query shapes.
2. Name the Loams primitive that covers it. If none does, say so.
3. Choose:
   - **Replace**: Loams owns the data and the API.
   - **Keep**: not a fit now; connect through an SDK or leave it alone.
   - **Bridge**: Loams in front, old store behind, for a bounded period.
   - **Defer**: no decision yet; name the missing fact and owner.
4. Write an ADR in `company/decisions/` using the `decision-records` skill.
5. Mark the risk: data loss, consistency, latency, cost, skills, vendor.
6. Produce `company/architecture/fit.md` with a summary table and links to ADRs.

## Guardrails

- Do not choose "replace" without a migration and rollback path.
- Do not use Loams as a cache for a system that needs strong multi-region writes
  unless the design supports it.
- Do not bridge forever; every bridge gets an expiry and an owner.

## References

- [Design 01](../../reference/design/01-architecture.md)
- [Design 44](../../reference/design/44-unified-api-and-sdks.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
- [Design 76](../../reference/design/76-identity-and-authorization.md)
