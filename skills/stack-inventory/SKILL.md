---
name: stack-inventory
description: "Inventory a company's services, data stores, workflows, identity, deployment, and costs as the baseline for a Loams migration. Use at the start of adoption or when a system is missing from the inventory."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Stack inventory

You cannot decide what Loams replaces until you know what exists and who owns it.

## When to use

- Stage 0 of adoption.
- A new team, region, or acquisition is added to scope.
- A migration plan references a system that is not in the inventory.

## Inputs

- Repository access, IaC (OpenTofu/Terraform/CloudFormation), Kubernetes
  manifests, compose files, CI configs, runbooks.
- Architecture diagrams and on-call rotations.
- Cloud billing exports and database metrics, if available.

Do not assume production access. Prefer code and documents; ask for read-only
metrics only when needed.

## Procedure

1. **List services.** For each: name, owner, language, runtime, repo, deployment
   target, criticality, SLO, data stores, upstream/downstream calls.
2. **List data.** For each store: engine and version, size, growth, schema owner,
   retention, PII classification, backup/restore, consumers.
3. **List workflows.** Queues, schedulers, cron, sagas, durable executions,
   webhooks, batch jobs.
4. **List identity and access.** IdP, human roles, machine users, service
   accounts, secrets, key management.
5. **List observability and cost.** Logs, metrics, traces, dashboards, alerts,
   monthly spend per service and store.
6. **Classify.** `rewrite`, `keep`, `bridge`, `retire`, or `unknown`. Unknown is
   allowed now; it becomes a blocker at the fit gate.
7. **Record risks.** Vendor lock-in, end-of-life versions, single points of
   failure, missing backups, unowned services.

## Outputs

- `company/stack-inventory/inventory.md` — service table.
- `company/stack-inventory/data-map.md` — store table and data flows.
- `company/stack-inventory/risk-register.md` — risk, impact, owner, mitigation.
- Optional `company/stack-inventory/costs.md`.

Use this table shape:

| Service | Owner | Language | Runtime | Criticality | Data stores | SLO | Disposition |
|---|---|---|---|---|---|---|---|

## Guardrails

- Do not put credentials, customer names, or raw PII in the inventory.
- A service with no owner is a finding, not a reason to skip it.
- "Unknown" data classification is a blocker for migration of that store.

## References

- [Migration playbook](../../docs/migration-playbook.md)
- [Design 01: architecture](../../reference/design/01-architecture.md)
