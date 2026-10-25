# Identity and authorization

- Owner: CHANGE_ME
- Status: draft
- Upstream: design 76, IA1; `skills/identity-authz`
- Company IdP: see `company/company.yaml`

## Principals

| Principal | Kind | Authenticates with | Used by | Grants |
|---|---|---|---|---|
| `user:<sub>` | human | company OIDC | console, CLI | per team |
| `apikey:<id>` | service | API key | Rust services, jobs | namespace read/write |
| `machine:<id>` | agent | machine key | agents, CI | explicit, short-lived |
| `tailscale:<login>` | human | tailnet identity | console over tailnet | per tailnet ACL |

No shared keys between services. Each service gets its own principal and its own
key or machine identity. Keys live in the company secret store, never in git or
an image.

## Authentication

- Console and human CLI: OIDC authorization code + PKCE against the company
  issuer. Record issuer, audience, and allowed emails/groups.
- Services: API keys hashed at rest by Loams. Rotate on a schedule and on
  suspected exposure.
- Agents: machine keys with a short lifetime and an explicit scope. Do not give
  an agent a human's token.
- CI: a dedicated principal per repository/environment, with the smallest grant
  that passes the conformance suite.

## Authorization

| Namespace | Read | Write | Admin | Notes |
|---|---|---|---|---|
| `CHANGE_ME-dev` | service principals | service principals | platform | non-production |
| `CHANGE_ME-prod` | service principals | service principals | platform + security | production |

- Prefer namespace-scoped grants over global grants.
- Agents get read plus the narrow write paths they need; no wildcard grants.
- Human admin is break-glass and audited.
- Record every grant change in an ADR or the access log.

## Tailnet

- The engine is reachable only through the tailnet sidecar.
- Tailnet ACLs admit the company identity group, not `*`.
- No inbound cloud firewall rule.
- Operator shells use Session Manager, IAP, or `az vm run-command`.

## Secrets

| Secret | Store | Rotation | Consumer |
|---|---|---|---|
| Tailnet auth key | company secret store | per policy | compose sidecar |
| Setup token | company secret store | one-time, then clear | first-run setup |
| Service API keys | company secret store | 90 days | services |
| Machine keys | company secret store | 30 days | agents |
| Cloud credentials | instance role / managed identity | automatic | engine, bucket |

No static cloud keys in `.env` or the image. The bucket uses the VM identity.

## Audit

- Loams request logs carry the principal, namespace, operation, and outcome.
- Auth failures and permission denials are alerted.
- Key creation, rotation, and revocation are recorded with the actor and ticket.
- Review grants quarterly and after every offboarding.
