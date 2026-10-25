# Fictional example: Acme

This is a filled example for shape only. Do not copy its facts.

| Service | Owner | Language | Runtime | Criticality | Data stores | SLO | Disposition |
|---|---|---|---|---|---|---|---|
| storefront-api | commerce | TypeScript | ECS | tier-1 | postgres-orders, redis-cart | 99.95% | bridge |
| search-api | discovery | Python | EKS | tier-1 | elasticsearch-catalog, qdrant-recs | 99.9% | rewrite |
| order-worker | commerce | Go | EKS | tier-1 | kafka-orders, postgres-orders | 99.9% | rewrite |
| analytics-etl | data | Python | Airflow | tier-2 | clickhouse-events, s3-lake | 99.5% | bridge |
| admin-ui | internal | TypeScript | Vercel | tier-2 | postgres-orders | 99.5% | keep |
| legacy-crm | sales | PHP | VM | tier-2 | mysql-crm | 99.0% | keep |

## Data map

| Store | Engine | Size | Growth | Retention | PII | Consumers | Disposition |
|---|---|---|---|---|---|---|---|
| postgres-orders | PostgreSQL 15 | 400 GB | 20 GB/mo | 7 years | yes | storefront-api, order-worker | bridge |
| elasticsearch-catalog | Elasticsearch 8 | 120 GB | 5 GB/mo | 2 years | no | search-api | rewrite |
| qdrant-recs | Qdrant | 30 GB | 2 GB/mo | 90 days | no | search-api | rewrite |
| kafka-orders | Kafka | 2 TB | 100 GB/mo | 7 days | yes | order-worker | rewrite |
| clickhouse-events | ClickHouse | 1 TB | 80 GB/mo | 2 years | pseudonymous | analytics-etl | bridge |
| s3-lake | S3 | 5 TB | 200 GB/mo | 7 years | yes | analytics-etl | keep |

## Risks

| Risk | Impact | Owner | Mitigation |
|---|---|---|---|
| Elasticsearch cluster is end-of-life | Search outage | discovery | Rewrite to collections in wave 1 |
| No restore drill for orders | Data loss | commerce | Drill before cutover |
| Kafka retention is 7 days | Replay gap | commerce | Backfill from Postgres |
