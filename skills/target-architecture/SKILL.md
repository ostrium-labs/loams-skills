---
name: target-architecture
description: "Design the target Loams topology: namespaces and tenancy, collection schemas, data flows, identity, deployment, capacity, and failure domains. Use after fit decisions and before building the pilot."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Target architecture

Turn fit decisions into a topology a team can build and an operator can run.

## When to use

- Stage 2, after `company/architecture/fit.md` and the ADRs exist.
- A new environment, region, or tenant is added.
- A security or tenancy review asks for the data-flow diagram.

## Inputs

- `company/architecture/fit.md`.
- `company/company.yaml` (cloud, IdP, Loams version).
- Traffic, data volume, retention, and RPO/RTO targets.
- [Design 01](../../reference/design/01-architecture.md),
  [Design 70](../../reference/design/70-byoc-clouds.md),
  [Design 76](../../reference/design/76-identity-and-authorization.md).

## Procedure

1. **Choose the deployment shape.** Local dev (`loams dev`) → one VM light stack
   → profiles (`tikv`, `rustfs`) → Kubernetes only when upstream supports it.
   Record why. The current guides cover the one-VM tier; container and
   Kubernetes tiers are not built yet.
2. **Define namespaces.** One per environment and trust boundary, for example
   `prod`, `staging`, `dev`, and a namespace per external tenant if required.
   Namespaces are the unit of isolation; do not put two trust boundaries in one.
3. **Design collections and schemas.** Fields, types, vector dimensions,
   analyzers, filters, sort keys, retention. Avoid one giant collection.
4. **Design data flows.** Write path, read path, backfill, dual-write, events,
   analytics. Mark synchronous vs asynchronous and the consistency expectation.
5. **Design identity.** Human SSO, machine users, agent identities, service
   tokens, authorization tuples. See the `identity-authz` skill.
6. **Design the network.** Tailnet only, no inbound rule, egress for images and
   the tailnet. Name the break-glass path.
7. **Design capacity and failure.** Disk, RAM, IOPS, bucket growth, backup
   window, restore time, single-node risk, upgrade window.
8. **Write** `company/architecture/target.md` with Mermaid diagrams and a
   decision table. Link every choice to an ADR.

## Outputs

- `company/architecture/target.md`
- `company/architecture/data-flows.md` (optional)
- `company/architecture/capacity.md`
- ADRs for deployment shape, tenancy, identity, and consistency.

## Guardrails

- The engine is never public. A design that needs a public endpoint is rejected
  or fronted by a company-owned gateway with its own review.
- Do not assume replication on the light single-node stack.
- Do not mix tenants in a namespace without a written risk acceptance.

## References

- [Design 01](../../reference/design/01-architecture.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
- [Design 76](../../reference/design/76-identity-and-authorization.md)
- [BYOG rules](../../docs/byog-rules.md)
