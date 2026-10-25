# Data map

| Store | Engine | Size | Growth | Retention | PII | Consumers | Disposition |
|---|---|---|---|---|---|---|---|
| example-postgres | PostgreSQL | | | | | | unknown |

## Flows

```mermaid
flowchart LR
  API[example-api] --> PG[(example-postgres)]
```

Mark synchronous vs asynchronous and the consistency expectation on each edge.
