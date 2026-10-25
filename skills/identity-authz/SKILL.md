---
name: identity-authz
description: "Design authentication and authorization for a Loams install: company IdP and OIDC, Zitadel identities, OpenFGA tuples, machine users, agent tokens, least privilege, and audit. Use when adding users, services, agents, or tenants."
license: Apache-2.0
metadata:
  upstream: "ostrium-labs/loams@bc90297"
  reviewed: "2026-10-25"
  version: "0.1.0"
---

# Identity and authorization

Zitadel answers **who**. OpenFGA answers **what they can do**. Loams enforces and
audits.

## When to use

- A company install needs SSO, service accounts, or agents.
- A new tenant, environment, or resource type is added.
- A permission review or audit asks for the tuple model.

## Inputs

- Company IdP and OIDC issuer, audience, and group/role mapping.
- `company/architecture/target.md`.
- [Design 76](../../reference/design/76-identity-and-authorization.md),
  [Design 78](../../reference/design/78-loams-cloud-on-tidb-x.md).

## Procedure

1. **Pick the auth mode.**
   - Single-user/local: built-in single-org authorizer; no Zitadel or OpenFGA.
   - Company: company OIDC issuer for people; Zitadel for machine users when
     needed; OpenFGA for fine-grained authorization.
   - Never make the admin console public.
2. **Map principals.** Human `user:<id>`; machine `service_account:<id>` or
   `agent:<id>`; `org:<id>`; `team:<id>`. Never use an email as an id.
3. **Model the hierarchy.** org → project → environment → resource. Application
   code checks only `can_*` relations through the `Authorizer` trait.
4. **Keep roles coarse.** Zitadel roles may sign in / administer the console.
   Resource permissions live in OpenFGA, not in IdP roles.
5. **Agents.** Each agent is a machine user with explicit grants; never inherit
   from its owner. Sandboxed agents never hold an IdP key; agentd/supervisor
   vend 15-minute Loams tokens.
6. **Revocation.** Disable the machine user/human and delete tuples. A disabled
   principal is refused on every request; do not rely on token expiry.
7. **Cache.** At most one decision cache, TTL ≤ 5 s, allows only. Bypass it for
   higher-consistency checks.
8. **Audit.** Zitadel events, OpenFGA changes, and Loams allow/deny decisions go
   to the `_audit` stream; retain per policy.
9. **Test.** A multi-org test proves no tuple links two orgs. A differential test
   keeps built-in and OpenFGA answers equal.

## Outputs

- `company/architecture/identity.md` — modes, principals, hierarchy, flows.
- `company/architecture/authz-model.fga` or a link to the company model.
- A tuple bootstrap and a revocation runbook.
- ADRs for auth mode, tenant isolation, and cache policy.

## Guardrails

- No shared admin account; no long-lived agent keys.
- No resource permission in an IdP role.
- No public admin console or OpenFGA endpoint.
- No authorization decision in the UI only; enforce server-side.

## References

- [Design 76](../../reference/design/76-identity-and-authorization.md)
- [Design 78](../../reference/design/78-loams-cloud-on-tidb-x.md)
- [Design 70](../../reference/design/70-byoc-clouds.md)
