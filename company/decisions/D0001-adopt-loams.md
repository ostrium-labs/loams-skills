# D0001: Adopt Loams for the company data platform

- Status: proposed
- Date: YYYY-MM-DD
- Owner: CHANGE_ME
- Upstream: design 01, design 70; no single D-number

## Context

The company is evaluating Loams as the data platform for its stack, running on
its own ground (BYOG). Data-owning services will be rewritten in Rust; edges stay
on the SDK for their language.

## Decision

Adopt Loams for the first bounded context after the fit and pilot gates pass.

## Alternatives

- Keep the current stores and add Loams only for new services.
- Migrate to a managed vendor.
- Build a custom platform.

## Consequences

- Positive: one data platform, company-owned ground, one SDK contract.
- Negative: migration cost and a new operational surface.
- Migration: inventory, fit, target, pilot, data, cutover, operate.

## Follow-up

- Complete `company/stack-inventory/`.
- Record fit decisions in ADRs.
