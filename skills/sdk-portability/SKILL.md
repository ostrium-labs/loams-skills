---
name: sdk-portability
description: "Decide which services and clients stay in their language and use one of the 13 Loams SDKs, and keep behaviour identical across them with the conformance corpus. Use before rewriting an edge service or adding a language."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# SDK portability

"Rewrite the stack in Rust" means rewrite the data-owning core. The edges can
stay where they are and use their SDK.

## When to use

- A team proposes rewriting a web, mobile, data, or infrastructure service in
  Rust only to use Loams.
- A language needs to be added or removed from the company's matrix.
- A cross-language behaviour difference appears.

## Inputs

- `company/stack-inventory/inventory.md`.
- `company/architecture/target.md`.
- [13-SDK matrix](../../reference/sdk/README.md),
  [runtime contract](../../reference/sdk/runtime-contract.md),
  [fixtures](../../reference/sdk/fixtures.md).

## Procedure

1. **Classify the edge.** Does it own data and hot-path logic, or does it call
   an API? Owns data → Rust candidate. Calls an API → keep the language.
2. **Check SDK maturity.** All 13 are 0.x. Not every language has every app
   module yet; the matrix in `reference/sdk/README.md` is the source of truth.
3. **Choose the transport.** Connect unary is the default; gRPC where the
   language has a native stack; the Connect fallback for hosts without one.
4. **Pin the package** and record the version in `company/sdk-matrix.md`.
5. **Run the conformance suite** for each language. Do not accept "it works in
   TypeScript" as evidence for another language.
6. **Keep one behaviour spec.** Tokens, idempotency, retries, consistency,
   pagination, and error reasons are shared; per-language wrappers must not
   reinterpret them.

## Outputs

- `company/sdk-matrix.md`:

  | Service/client | Language | Package | Version | Transport | Conformance | Keep/Rewrite |
  |---|---|---|---|---|---|---|

- ADRs for "keep this language" decisions.
- CI entries that run the language's conformance suite where possible.

## Guardrails

- Do not hand-write REST or Connect calls; use the SDK.
- Do not generate a per-language behaviour spec; use the shared contract.
- Do not assume a package is published; check the registry and the release
  status before committing to it.
- A language without a required app module is a reason to wait or bridge, not to
  invent one.

## References

- [13 SDKs](../../reference/sdk/README.md)
- [Runtime contract](../../reference/sdk/runtime-contract.md)
- [Fixtures](../../reference/sdk/fixtures.md)
- [Design 44](../../reference/design/44-unified-api-and-sdks.md)
