---
name: agent-platform
description: "Design an agent product on Loams: agentd, sandboxed runs, memory, durable execution, Studio, software factory, marketplace, and Mastra. Use when the company's product includes agents rather than only data services."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Agent platform (optional)

Use this skill only if the company is building agents or agent-powered software.
The core migration skills do not need it.

## When to use

- The product runs agents, tools, or generated software.
- A team wants Studio, the software factory, or a marketplace.
- An agent needs durable runs, sandboxing, or memory.

## Inputs

- `company/architecture/target.md`.
- `company/company.yaml`.
- [Design 63: Meter](../../reference/design/63-loams-meter.md),
  [Design 71: app BYOG](../../reference/design/71-app-byoc-free-tiers.md).
- Upstream design docs 50, 52, 53, 56, 72, and 73 (optional snapshot group).

## Procedure

1. **Separate the concerns.** Model calls, tool execution, durable runs, memory,
   sandbox, UI, and billing are different failure domains.
2. **Durable runs.** Use Loams durable execution for retries, timers, signals,
   and history. Every tool call must be idempotent or carry an idempotency key.
3. **Sandbox.** Run untrusted code in the sandbox, not in the API process. Give
   the sandbox a short-lived token; never an IdP key.
4. **Memory.** Store conversation and long-term memory in collections with
   retention and tenancy. Do not put one tenant's memory in another's namespace.
5. **Agents as principals.** Each agent is a machine user with explicit grants;
   it never inherits its owner's permissions.
6. **Metering.** Bill per run, tool call, token, or storage; define the unit and
   make the meter idempotent.
7. **Studio / factory / marketplace.** Use them when the product needs an IDE,
   generated software, or distribution. Do not build all three on day one.
8. **Mastra.** If the agent layer is TypeScript, `loams-mastra` is the bridge;
   keep Loams as the data and durable layer.

## Outputs

- `company/architecture/agents.md` — runs, tools, sandbox, memory, identity.
- `company/architecture/agent-billing.md` — metering units and limits.
- ADRs for sandbox, memory retention, and agent permissions.

## Guardrails

- No long-lived credentials in a sandbox.
- No shared memory namespace across tenants.
- No unbounded agent loop without a budget and a stop switch.
- No marketplace or factory launch before the metering and revocation paths work.

## References

- [Design 63](../../reference/design/63-loams-meter.md)
- [Design 71](../../reference/design/71-app-byoc-free-tiers.md)
- [Design 76](../../reference/design/76-identity-and-authorization.md)
- [Design index](../../reference/design/README.md)
