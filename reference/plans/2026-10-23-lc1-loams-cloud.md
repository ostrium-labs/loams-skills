# LC1 — Loams Cloud on TiDB X: Dev on Your Own Devices, Prod in Your Own Cloud — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Work task by task. Where this plan gives exact names, paths, flags or tests, use them verbatim, and record any deviation in "Rulings made during execution" at the end.
>
> **Status: Approved** (owner, 2026-10-23, all defaults; stub). Track **LC**, design [§78](../design/78-loams-cloud-on-tidb-x.md) (D1700–D1735, Q1260–Q1277). **LC0 starts once the owner has created the no-card TiDB Cloud org (Q1277).** The public opening of LC2 waits on counsel (Q1276); D1701, which amends D1323, is approved.
>
> **Coverage:** this plan specifies **LC0**, the spikes, task by task, and **LC2a**, invite-only access on WorkOS AuthKit (the owner's amendments of 2026-10-23; the first phase users see), task by task. LC0 writes no product code. LC1, the rest of LC2 and LC3–LC5 are outlined at milestone level; each gets full task specifications after the LC0 report.
>
> **Inputs:**
> - the research report [`docs/research/tidb-x-and-hcp.md`](../research/tidb-x-and-hcp.md) (2026-10-23);
> - §70 and the [BY1 plan](2026-10-21-by1-byoc-clouds.md), which §70 and §71 share;
> - the [ER1 plan](2026-10-20-er1-edge-reach.md);
> - the [IA1 plan](2026-10-21-ia1-identity-authz.md) and the IA0 spike report;
> - the [SD1 plan](2026-10-13-sd1-studio-ade.md).

**Goal (LC0):** before any product code, find out whether TiDB X can carry Loams' dev state the way §78 assumes, and at what cost:
- tenant cells on Starter;
- workspace handoff on Filesystem;
- the control plane on a Worker through the HTTP driver;
- device enrolment on IA0's Zitadel stack;
- whether TiDB Cloud Zero can hold try-out SQL;
- whether the defaults hold on a managed work laptop.

**Exit (LC0):** `docs/plans/lc0-report.md`, with:
- a go or no-go for each spike;
- the measured numbers (per-tenant storage visibility, RU attribution, drain latency, writes per agent session, Worker request volume per device);
- answers to §78's (verify) items;
- recommended values for Q1263–Q1265.

## Global Constraints

- **Owner actions first.** **The owner already has a TiDB Cloud Starter account** (2026-10-23). It must have **no card** on it (Q1277); if it has one, the owner creates a separate no-card organization for LC0. The owner hands over the credentials listed in "Owner steps and credentials" below. No LC0 task starts until they are in place. Agents never sign up for or create accounts on TiDB Cloud, Cloudflare or HCP. The Cloudflare account is Ostrium's existing one, and the owner grants a scoped API token.
- **Cost:** every LC0 spike stays inside free allowances.
  - Starter: the 5 free instances.
  - Filesystem: no card, so the no-card caps apply (1 file system per region, 2,000 files, 2 GB).
  - Workers: the Free plan, unless Task 3 needs Paid, which the owner approves.
  - Teardown at the end of each task is part of the task.
- **No product code in LC0.** Spike code lives under `spikes/lc0-<task>/`, is marked `publish = false` or `"private": true`, is never wired into a crate, the CLI or the release, and is deleted or kept as a fixture by LC1's first task.
- **Secrets:** TiDB API keys, Filesystem tokens and SQL passwords are never committed, logged or written into the report. Spikes read them from the environment.
- **Worktrees and branches:** `~/Documents/Ostriumlabs/loams-wt/lc0-<task>`, with one branch and one PR per task into `dev`. Commit with `git commit -s`. Never stash.
- **Light builds:** no Rust build in LC0 except Task 5's, which runs `cargo test -p` on one spike crate. Never `--workspace`, and never in `/tmp`.
- **Review:** routine spike PRs get no AI review. Task 5 (identity) and the LC0 report get the `review:greptile` label.
- **GitHub-hosted runners only.** No LC0 spike runs in CI, because they need the owner's accounts.

## Review Focus

1. **A tenant reads or writes another tenant's database in a cell.** Expected: never. Test: `cell_grants_isolate_tenants` (Task 1).
2. **A scoped Filesystem token reaches outside its prefix.** Expected: never. Test: `scoped_token_cannot_escape_prefix` (Task 2).
3. **A handoff loses drained work.** Expected: never; undrained work is reported, not lost silently. Test: `handoff_after_drain_is_complete` (Task 2).
4. **A device token outlives deactivation by more than a few seconds.** Expected: under 5 s. Test: `deactivated_device_is_denied` (Task 5).
5. **A spike leaves resources behind.** Expected: never. Each task ends with `teardown_leaves_nothing`.

## LC0 — Spikes (Tasks 0–7)

### Task 0 — Rulings and pins

Check each item and record the answer in "Rulings made during execution":

- **Owner rulings.** Whether the owner has answered Q1260–Q1277, and which defaults changed.
- **Pins.** The `ti` CLI release used (install script URL and version); the `@tidbcloud/serverless` version (it must be at least 14 days old); Zitadel and OpenFGA at IA0's pins.
- **Dependencies.** Whether IA2 and IA4 (machine users) have landed; whether SD1's remote agentd (D909) is on `dev`; whether ME1 has started.
- **Re-reads.** Re-read the report's Starter, Filesystem and PG-preview pages. Note any limit that changed since 2026-10-23.

Files: this plan. Commit: `plans(lc1): LC0 Task 0 rulings and pins`.

### Task 1 — Starter tenant cells (spike)

**What to build:** `spikes/lc0-cells/`, a script in TypeScript or Python using a MySQL driver over TLS. Against one free Starter instance it:
1. creates *N* tenant databases (N = 50, 200, 500), each with its own SQL user and grants only on its database;
2. runs a small mixed workload per tenant;
3. measures the following.

| Measure | How |
|---|---|
| Isolation | Each tenant user tries `SELECT`, `INSERT`, `SHOW DATABASES` and `information_schema` reads on another tenant's database |
| Per-tenant storage visibility | `information_schema.TABLES` `DATA_LENGTH` and `INDEX_LENGTH` per schema, against the console's figure |
| Per-tenant activity | Whether `PROCESSLIST`, `CLUSTER_PROCESSLIST`, `SLOW_QUERY` or `STATEMENTS_SUMMARY` give per-user activity on Starter (the report says `STATEMENTS_SUMMARY` is Premium-only) |
| RU attribution | Whether any per-user or per-database RU figure exists |
| Connection caps | Whether `MAX_USER_CONNECTIONS` (per user) is enforced |
| Credential rotation | Whether `ALTER USER … PASSWORD EXPIRE` and password rotation work |
| The spending limit | With the instance at its free quota (or a throttled test), confirm new connections are denied and existing ones throttled |
| DDL cost | `CREATE DATABASE` and `CREATE USER` latency at each N |

**Tests:** `cell_grants_isolate_tenants`, `cell_storage_is_visible_per_tenant`, `teardown_leaves_nothing`.

**Report section:** the cell size to use (§78 D1717's default of 200), whether per-tenant fairness is observable, and a go or no-go for cells.

Commit: `spikes(lc0): Starter tenant cells (LC0 Task 1)`.

### Task 2 — Filesystem workspaces and handoff (spike)

**What to build:** `spikes/lc0-fs/`, scripts driving `ti fs` on two machines:
- a Linux VM or container with `fuse3` and `/dev/fuse`;
- a macOS machine (WebDAV, and FUSE if macFUSE is installed).

The runs:
1. Create a file system; clone a real repository into it with `ti fs-git clone-git-workspace`; mount it with the `coding-agent` profile.
2. Run a scripted "agent session" against it: 30 edits, a test run, a commit. Count the **write operations** it caused (from the console's usage, or the billing API, read before and after) and the files it created.
3. **Handoff:** create a layer for the "run"; drain; checkpoint; fork the layer on the second machine; hydrate Git; check that the tree matches. Then repeat **without** draining (kill the mount on machine A), and record what machine B sees.
4. **Scoped tokens:** a token for `/t/a` with `read,list,write`, TTL 1 h. It must fail on `/t/b`, fail to mount `/`, and fail after `disable-file-system-token`. Time the propagation.
5. **Limits:** fill to the no-card limits (2,000 files, 2 GB) and confirm writes stop and reads work.
6. **Usage visibility:** check whether per-file-system usage is readable by API or CLI, or only in the console.

**Tests:** `scoped_token_cannot_escape_prefix`, `handoff_after_drain_is_complete`, `undrained_handoff_is_detected`, `teardown_leaves_nothing`.

**Report section:**
- writes per session and the cost per session at $0.50 per 1,000;
- drain latency;
- whether the 5-minute drain interval (D1713) is right;
- whether per-file-system limits or usage are API-reachable;
- a go or no-go for the hosted cohort (Q1263).

Commit: `spikes(lc0): Filesystem workspaces, handoff and token scoping (LC0 Task 2)`.

### Task 3 — The control plane on a Worker (spike)

**What to build:** `spikes/lc0-worker/`, a Worker with `@tidbcloud/serverless`. It:
- creates and reads device and lease records in a free Starter instance;
- runs one Durable Object per user, holding hibernating WebSockets from 20 simulated devices that ping every 30 s.

**Measure:**
- the latency of single-statement writes;
- the idempotency-key pattern under retries;
- the requests and duration billed per device per day, with and without hibernation, read from the Workers dashboard or analytics;
- whether the Free plan's limits hold for the 20 devices.

**Tests:** `lease_is_exclusive_under_races`, `device_reconnects_after_sleep`, `teardown_leaves_nothing`.

**Report section:** requests per device per month, whether Workers Free or Paid is needed (§78 D1727), and the driver's behaviour on transient errors.

Commit: `spikes(lc0): the control plane on a Worker with the HTTP driver (LC0 Task 3)`.

### Task 4 — TiDB Cloud Zero for try-out SQL (research, plus one owner-run test)

Read TiDB Cloud Zero's documentation and terms:
- Can a third party (Loams Cloud) create Zero databases on users' behalf, programmatically?
- Is there an API, or only the SKILL command?
- What are the rate limits?
- How does the claim flow move a database into the user's own account?

The owner runs one manual creation and claim, to confirm the flow. No agent creates an account.

**Report section:** whether Q1264's Zero option is allowed and practical.

Commit: `docs(lc0): TiDB Cloud Zero for try-out SQL (LC0 Task 4)`.

### Task 5 — Device enrolment on IA0's stack (spike)

**What to build:** `spikes/lc0-device/`, a small Rust binary (one crate, `publish = false`) against the `conformance/identity/` compose (Zitadel and OpenFGA at IA0's pins). It:
1. runs the **OIDC device authorization grant** for a person;
2. creates a machine user of kind `device` under the person's org;
3. registers a key pair, with the private key in the OS keychain through the `keyring` crate where available, and a file with mode 0600 otherwise;
4. gets 15-minute access tokens by private-key JWT;
5. deactivates the device and measures how long until a token request and a protected call are denied.

Also check whether OpenFGA's MySQL datastore runs against a free Starter instance (§78 §17).

**Tests:** `device_flow_issues_tokens`, `deactivated_device_is_denied` (under 5 s), `device_cannot_act_for_another_org`.

**Provenance:** this spike may read IA0's report and `conformance/identity/`. It must not be read by loams-identity's clean-room agents (memory `loams-identity-zitadel`).

Commit: `spikes(lc0): device enrolment on Zitadel (LC0 Task 5)`.

### Task 6 — The company-device check (research and a manual run)

On a managed-like laptop (a macOS or Windows user account without admin rights, behind an HTTPS proxy with a re-signing CA, `mitmproxy` or similar), install Loams Desktop's per-user daemon (D784) and confirm:
- no admin prompt;
- the outbound WebSocket and HTTPS through the proxy;
- the corporate CA from the OS store;
- MySQL over TLS on 4000 through the proxy, or its failure, with the HTTP fallback;
- no inbound listener except loopback;
- clean removal.

Then confirm that `tsnet` inside a test daemon joins a second tailnet while the system Tailscale client stays on the first. The report §4.3 says a device is on one tailnet at a time.

**Report section:** §78 §10's table, row by row, as pass or fail.

Commit: `docs(lc0): the company-device check (LC0 Task 6)`.

### Task 7 — The LC0 report

Write `docs/plans/lc0-report.md` with:
- each task's go or no-go and numbers;
- the answers to §78's (verify) items;
- recommended values for Q1263, Q1264 and Q1265, and the cell size;
- a costed estimate of LC2 at 100, 1,000 and 10,000 users against D1730's ceiling;
- any change §78 needs, as a list for the owner.

Label the PR `review:greptile`.

Commit: `docs(lc0): the LC0 report`.

## LC2a — Invite-only access on WorkOS AuthKit (Tasks 0–12)

> **The first LC phase that users see.** It covers the owner's rulings of 2026-10-23:
> - invite-only (D1736);
> - WorkOS AuthKit for the hosted try-out (D1750);
> - the sign-in policy: Google, GitHub, Microsoft, or a work-email code (D1751; Microsoft added the same day). Sign in with Apple is rejected for now (D1758, owner, 2026-10-23).
>
> **Status: Approved** (owner, 2026-10-23, all defaults): D1736–D1751 and Q1278–Q1296. The design is [§78 §25](../design/78-loams-cloud-on-tidb-x.md#25-invite-only-access-workos-authkit-and-the-sign-in-policy-d1736d1757-owner-amendments-of-2026-10-23). D1752–D1757 were approved by the owner later the same day. **No LC2a code starts until the owner hands over the credentials listed under "Owner steps and credentials" below.**
>
> **Prerequisites:**
> - §25's API plan: approved (owner, 2026-10-23);
> - LC0 Task 3 (the Worker spike) reports a go;
> - **the owner already has a WorkOS free-tier account** (2026-10-23). Staging credentials go to an agent, as listed below; production keys are never handed to agents;
> - **the Resend account is still to create**, by the owner. Agents never sign up for anything.
>
> **Before the waitlist page goes live:** counsel's privacy policy (Q1276) must be published, and `mail.loams.dev` must be verified in Resend.
>
> **This plan writes no code.** It specifies the tasks for the agents who will.

**Goal:**
- A person can join the waitlist and confirm the address.
- WorkOS emails them an invitation.
- They redeem it with Google, GitHub, Microsoft or a work-email code, and land in a Loams Cloud account.
- No other sign-in method, and no uninvited person, can create a try-out account, even when a method is enabled in the dashboard by mistake.
- Only an allow-listed Ostrium admin with a fresh TOTP step-up can invite.
- Every step is audited.
- The running cost is $0.

**Exit:**
- `invite_only_happy_path` passes against Miniflare and the WorkOS **staging** environment;
- every Review Focus test passes;
- the sign-in policy suite passes;
- the LC2a section of the threat model is written;
- the go-live checklist is ticked by the owner.

**Architecture:**
- `deploy/loams-cloud/`: the Worker of D1727 on **Workers Free**, with:
  - Durable Objects (SQLite-backed), Cron Triggers and the Turnstile secret;
  - the WorkOS API key, client id, actions secret and webhook secret;
  - the Resend API key;
  - the HMAC and AES keys.
- `deploy/loams-cloud/schema/0001_waitlist.sql`: the tables.
- `deploy/loams-cloud/policy/`: the pinned domain lists and `email-domains.toml`.
- `crates/loams`: the `loams cloud admin …` commands.
- loams-site: the pages, in a separate PR there.

**Global constraints, in addition to LC0's:**
- **No real sends in tests.** Use WorkOS **staging**, which sends from `workos.dev` (V), and Resend's test mode, or a recorded fake. Real sends go only to addresses the owner controls, until the owner opens the waitlist.
- **No plaintext address** in logs, audit rows, metrics, errors or snapshots. A test greps for that.
- **$0:** no task may add a paid plan. A task that finds it needs one stops and reports.
- **Greptile:** every LC2a PR carries `review:greptile`.

**Review focus:**
1. **Enumeration.** Waitlist responses are identical across address states. Test: `waitlist_responses_do_not_enumerate`.
2. **The gate.** No WorkOS user becomes a Loams Cloud account without an invitation accepted by that user. Tests: `uninvited_signup_is_refused`, `account_requires_accepted_invitation`.
3. **The sign-in policy.** The policy suite below.
4. **Admin only.** Every `/admin/*` call fails without the org role, the allow-list or a fresh TOTP step-up. Tests: `admin_requires_role_allowlist_and_step_up`, `agents_cannot_be_cloud_admin`.
5. **Deletion.** After "remove me" or `DELETE /me`, no readable address remains with us, and the WorkOS user is gone. Test: `deletion_leaves_only_hmac_and_deletes_workos_user`.
6. **Signatures.** Actions, webhooks and our action responses are verified and signed, with a timestamp tolerance. Tests: `action_rejects_bad_signature`, `webhook_rejects_bad_signature_and_replay`.

**The sign-in policy suite** (D1751, D1752, D1753), in `deploy/loams-cloud/test/signin_policy.test.ts`:

| Test | Asserts |
|---|---|
| `google_any_account_allowed` | A Google sign-in with a `gmail.com` or Workspace address is allowed |
| `github_allowed_when_30_days_old` | A GitHub sign-in is allowed when the account is at least 30 days old |
| `microsoft_personal_account_allowed` | A Microsoft sign-in with an `outlook.com`, `hotmail.com` or `live.com` account is allowed, even though those domains are blocked for email codes |
| `microsoft_work_account_uses_email_code` | A work or school (Entra) account isn't offered through Microsoft OAuth (WorkOS requires "Personal Microsoft accounts only"); the same person signs in with a work-email code |
| `apple_refused_everywhere` | `AppleOAuth` is off in AuthKit (rejected for now, D1758), and an `AppleOAuth` success is denied by the action, refused at the callback, and suspended by reconciliation |
| `apple_private_relay_never_valid_for_email_code` | A Magic Auth sign-in at `privaterelay.appleid.com` is refused |
| `github_too_new_refused_without_spending_invite` | Under 30 days: `IDP_ACCOUNT_TOO_NEW`; the waitlist invite stays open, and a fresh WorkOS invitation is sent |
| `magic_auth_work_domain_allowed` | A work-email code is allowed for a domain on no list, with an MX record |
| `magic_auth_free_mail_refused` | One case per named family (`gmail.com`, `googlemail.com`, `outlook.com`, `hotmail.com`, `live.com`, `yahoo.co.uk`, `icloud.com`, `me.com`, `proton.me`, `protonmail.com`, `aol.com`, `gmx.de`, `mail.ru`, `yandex.ru`, `qq.com`, `163.com`) is refused at the registration action, the authentication action and the callback |
| `magic_auth_disposable_refused` | A domain on the disposable list is refused |
| `override_allow_beats_list` | A domain in `email-domains.toml`'s `allow` is accepted even if a list includes it |
| `override_block_beats_absence` | A domain in `block` is refused even if no list includes it |
| `password_refused_everywhere` | A simulated `authentication_method: "Password"` is denied by the action, refused at the callback, and suspended by reconciliation |
| `passkey_refused_everywhere` | The same for `Passkey` |
| `sso_refused_everywhere` | The same for `SSO` |
| `other_social_refused` | GitLab, LinkedIn, Bitbucket or Xero OAuth is refused |
| `provider_age_rule_is_github_only` | Google and Microsoft sign-ins are not age-checked; GitHub's is |
| `reconciliation_catches_dashboard_mistake` | With the actions endpoint down (fail-open forced in a test double), a `authentication.password_succeeded` event suspends the account and revokes its sessions within one reader cycle |
| `waitlist_accepts_free_mail_but_not_disposable` | The waitlist rule (D1753) |
| `waitlist_copy_explains_rule` | A snapshot test of the form copy in §25.4 (in loams-site) |

### Task 0 — Rulings, pins and the (verify) items

Check each item and record the answer in "Rulings made during execution". Where a finding changes §25, stop and report it to the owner.

- **WorkOS**, in the staging environment the owner creates:
  1. which authentication methods can be switched off on the free tier (password, passkeys, other social providers), and that Microsoft OAuth with our own Entra app ("Personal Microsoft accounts only") has no per-provider charge;
  1. whether Google or Microsoft expose any account-age signal through WorkOS (expected: none);
  2. the setting that disables sign-up while invitations still work;
  3. whether the action context names the authentication method or provider;
  4. the `authentication_method` values for Google and GitHub OAuth;
  5. whether GitHub identities' `idp_id` is GitHub's numeric user id;
  6. whether resending an invitation changes its token or expiry;
  7. whether the MFA API is included free;
  8. whether AuthKit branding is free;
  9. the current data location;
  10. the terms on commercial use of the free tier (for counsel, Q1276).
- **Resend:** commercial use on Free; its region and subprocessors; domain verification for `mail.loams.dev`; how it reports bounces and complaints to the Worker.
- **Cloudflare (Workers Free):**
  1. Durable Objects' free limits, and whether the WebSocket 20:1 ratio applies to the free daily limit;
  2. the Cron Trigger count on Free;
  3. the rate-limiting binding on Free;
  4. Turnstile Siteverify's single-use rule.
- **Domain lists:** pin `Kikobeats/free-email-domains` (MIT) and `disposable-email-domains` (CC0-1.0) by revision, and check the licences of `free-email-domains`' sources.

Files: this plan. Commit: `plans(lc1): LC2a Task 0 rulings and pins`.

### Task 1 — The control-DB schema

`deploy/loams-cloud/schema/0001_waitlist.sql` holds §25.3's tables:
- `waitlist`, `waitlist_tokens`, `waitlist_suppressed`, `invites` (with `workos_invitation_id`), `accounts` (with unique `workos_user_id`), `workos_events_cursor` and `audit`;
- inside TiDB X's limited features: no `time_to_live` attribute and no partitioning.

**Tests:**
- `schema_applies_cleanly`;
- `email_hmac_is_unique`;
- `workos_user_id_is_unique`;
- `no_plaintext_email_column_outside_enc`.

Commit: `loams-cloud: the waitlist, invite and account schema (LC2a Task 1)`.

### Task 2 — Email: Resend for the Worker, and as WorkOS's provider

- **The Worker's templates:** `confirm`, `already_listed` and `removal`, each with a text and an HTML part, a footer removal link, a `List-Unsubscribe` header, and no tracking.
- **Sending:** through Resend's API over `fetch`. A pacer in a Durable Object enforces the shared daily budget (D1738: 100 a day across both senders, at most 60 of them invitations).
- **WorkOS's emails:** the owner configures Resend as WorkOS's custom email provider. This task writes the step-by-step guide in `deploy/loams-cloud/README.md` and a check, `email_provider_is_resend`, that reads the WorkOS environment's provider through the API if one exists, or else is a manual checklist item.

**Tests:**
- `templates_render_text_and_html`;
- `every_email_has_removal_link`;
- `pacer_shares_daily_budget_with_invitations`;
- `resend_bounce_suppresses`.

Commit: `loams-cloud: waitlist email through Resend (LC2a Task 2)`.

### Task 3 — The public waitlist endpoints

These are `POST /waitlist`, `POST /waitlist/confirm` and `DELETE /waitlist` as in §25.10, with:
- Turnstile;
- rate limits;
- the disposable and no-MX check (free-mail allowed, D1753);
- enumeration-safe responses;
- HMAC and AES storage;
- log redaction.

**Tests:**
- `waitlist_responses_do_not_enumerate`;
- `confirm_is_single_use`;
- `turnstile_token_reused_is_refused`;
- `waitlist_accepts_free_mail_but_not_disposable`;
- `rate_limits_apply_per_prefix_and_address`;
- `tokens_are_redacted_from_logs`.

Commit: `loams-cloud: the waitlist API with double opt-in (LC2a Task 3)`.

### Task 4 — Admin authorisation

- **The checks:**
  - an AuthKit access token verified against the JWKS;
  - `org_id` equal to the Ostrium WorkOS org, and `role` equal to `cloud_admin`;
  - the user in `deploy/loams-cloud/config/admins.toml`, the allow-list, which changes only by a reviewed PR;
  - a TOTP step-up: `POST /admin/mfa/enroll` and `POST /admin/mfa/verify` on WorkOS's MFA API, with `step_up_until` stored against the session's `sid`, for 15 minutes.
- **The authorizer:** `loams-authz` checks `platform:loams-cloud#cloud_admin`, with a model addition that excludes agents and service accounts, and `BuiltinAuthorizer` parity.
- **The CLI:** `loams cloud admin` logs in with WorkOS CLI Auth (the device flow) and runs `mfa enroll|verify`.
- **The endpoints:** `GET /admin/waitlist`, `DELETE /admin/waitlist/{id}` and `GET /admin/audit`.

**Tests:**
- `admin_requires_role_allowlist_and_step_up`;
- `step_up_expires_after_15_minutes`;
- `agents_cannot_be_cloud_admin`;
- `admin_list_pages_and_filters`.

Commit: `loams-cloud: cloud_admin with allow-list and TOTP step-up (LC2a Task 4)`.

### Task 5 — Invitations through WorkOS

- **The endpoints:** `POST /admin/invites` queues up to 50, with an `Idempotency-Key`. The queue calls WorkOS `POST /user_management/invitations` with `email` and `expires_in_days: 14` and no organisation, and stores `workos_invitation_id`. It respects the daily budget.
- `GET /admin/invites`.
- `POST /admin/invites/{id}/revoke` maps to WorkOS revoke.
- `POST /admin/invites/{id}/resend` maps to WorkOS resend, at most 3 times; `extend` revokes and sends a new invitation.

**Tests**, against WorkOS staging and a recorded fake:
- `batch_skips_unconfirmed_and_suppressed`;
- `invite_calls_workos_once_per_entry`;
- `resend_limit_enforced`;
- `revoke_mirrors_workos_state`;
- `daily_budget_blocks_batch`.

Commit: `loams-cloud: WorkOS invitations, resend and revoke (LC2a Task 5)`.

### Task 6 — WorkOS configuration and the actions

- **The checklist**, `deploy/loams-cloud/workos/CHECKLIST.md`, for the owner to apply in the dashboard:
  - production: Google OAuth, GitHub OAuth, **Microsoft OAuth** (the owner's own Entra app registration, "Personal Microsoft accounts only"; its client secret's expiry, at most 24 months, goes in the owner's calendar) and Magic Auth **on**; **Apple off** (rejected for now, D1758); everything else **off**; sign-up **disabled**;
  - AuthKit's MFA **off**;
  - the redirect URI `https://api.loams.dev/cloud/v1/auth/callback`;
  - the actions endpoints, with **fail closed** for both;
  - the webhook endpoint;
  - the custom email provider set to Resend;
  - branding with the display name "Loams Cloud";
  - the "Ostrium" organisation with the `cloud_admin` role.
- **A read-only checker,** `loams cloud admin check-workos`, compares what the API exposes against the checklist and fails on any difference.
- **The actions:** `POST /internal/workos/actions/registration` and `/authentication`. They verify `WorkOS-Signature` with a tolerance, apply §25.4's rules and return a signed verdict.

**Tests:**
- `action_rejects_bad_signature`;
- `registration_action_requires_pending_invite`;
- `authentication_action_refuses_suspended_account`;
- the method and domain cases of the policy suite, at the action layer.

Commit: `loams-cloud: WorkOS actions and the configuration checklist (LC2a Task 6)`.

### Task 7 — Sign-in, callback and redemption

- **The endpoints:** `GET /auth/login` (AuthKit with `provider=authkit`, PKCE and a state cookie), `GET /auth/callback`, `POST /auth/logout` and `GET /me`.
- **What the callback does** (§25.5):
  - it exchanges the code;
  - it re-checks the policy;
  - it requires the invitation `accepted` with `accepted_user_id` equal to the user;
  - it checks the GitHub age through the identities API and a GitHub App token;
  - it creates the account and sets `external_id`;
  - it creates the Durable Object;
  - on refusal, it deletes the new WorkOS user.

**Tests:**
- `invite_only_happy_path`;
- `uninvited_signup_is_refused`;
- `account_requires_accepted_invitation`;
- `different_email_redeems_and_records_both`;
- `existing_account_does_not_create_second`;
- `github_too_new_refused_without_spending_invite`;
- `refusal_deletes_new_workos_user`.

Commit: `loams-cloud: AuthKit sign-in and invitation redemption (LC2a Task 7)`.

### Task 8 — The WorkOS events reader and reconciliation

- **The reader:** a Cron Trigger every minute reads `GET /events` from `workos_events_cursor`, with the event types of §25.5.
- **The webhook:** `POST /internal/workos/webhook` verifies the signature, checks the timestamp tolerance and the source IPs, and only wakes the reader.
- **Reconciliation** (§25.4, layer 4): it suspends accounts and revokes sessions on any disallowed success, and on any user we did not create from an invite.

**Tests:**
- `reconciliation_catches_dashboard_mistake`;
- `reader_resumes_from_cursor`;
- `webhook_rejects_bad_signature_and_replay`;
- `wake_is_only_a_hint`;
- `user_deleted_in_workos_suspends_account`.

Commit: `loams-cloud: WorkOS events reader and reconciliation (LC2a Task 8)`.

### Task 9 — Lifecycle, deletion and retention

- **`DELETE /me`:** export, then delete our records and the WorkOS user (`DELETE /user_management/users/{id}`), within 7 days (D1731).
- **Retention jobs** for §25.6's table.
- **The hourly invite sweep.**

**Tests:**
- `deletion_leaves_only_hmac_and_deletes_workos_user`;
- `retention_jobs_delete_on_schedule`;
- `expired_invitations_are_mirrored`.

Commit: `loams-cloud: account lifecycle, deletion and retention (LC2a Task 9)`.

### Task 10 — The pages (in loams-site)

A separate PR in `ostrium-labs/loams-site`, reviewed by CodeRabbit.
- **The pages:** the waitlist form (Turnstile, the two optional fields, §25.6's notice naming the processors, and §25.4's copy on which addresses can sign in how), confirm, leave, error and welcome.
- **The sign-in button** starts at `GET /auth/login`.
- **No analytics** on `/cloud/*`.
- **Accessibility:** WCAG 2.2 AA.

Tests: the site's checks, `waitlist_copy_explains_rule`, and a Playwright flow against a mocked API.

Commit, in loams-site: `cloud: the waitlist and sign-in pages (LC2a Task 10)`.

### Task 11 — End to end, the abuse drill, and the free-tier budget

- **End to end:** Miniflare plus WorkOS staging, through the happy path and every refusal.
- **The abuse drill:** 1,000 scripted sign-ups from 10 prefixes.
- **The free-tier budget:** a measured week of synthetic use at 50 users and 100 devices, recording:
  - Worker and Durable Object requests per day;
  - CPU per route;
  - emails per day;
  - Starter RUs.

  Each is set against its free limit, to set Q1292's trigger.

**Tests:**
- `invite_only_happy_path` (end to end);
- `abuse_drill_stays_within_caps`;
- `free_tier_budget_report` (it writes `docs/plans/lc2a-free-tier-report.md`).

Commit: `loams-cloud: end-to-end, abuse and free-tier tests (LC2a Task 11)`.

### Task 12 — Threat model and the go-live checklist

- **The threat model:** `docs/security/loams-cloud-threat-model.md`, LC2a section.
  - The threats: enumeration, waitlist flooding, invitation forwarding, a sign-in method switched on by mistake, an actions outage (fail-closed), webhook forgery and replay, admin takeover (role, allow-list and step-up), WorkOS or Resend account compromise, data-subject requests, and free-tier exhaustion used as denial of service.
- **The go-live checklist,** which the owner ticks:
  - counsel's policy is published;
  - `mail.loams.dev` is verified, and DMARC is set;
  - the WorkOS checklist passes `check-workos`;
  - the first admin is allow-listed and enrolled in TOTP;
  - the daily caps are set.

Label the PR `review:greptile`.

Commit: `docs(security): Loams Cloud invite-only threat model and go-live checklist (LC2a Task 12)`.

## Owner steps and credentials (recorded 2026-10-23)

**What the owner has already, 2026-10-23:**
- **a TiDB Cloud Starter account;**
- **a WorkOS free-tier account.**

**Still to create:** the Resend account, needed for LC2a Task 2.

**No LC0 or LC2a code starts until the credentials below exist.**

**Where credentials go:**
- **files in the owner's home directory,** like `~/gh.txt`;
- **mode `0600`;**
- **never pasted into chat,** never committed, never logged.

Agents read them with `set -a; . ~/<file>; set +a` in the one command that needs them, and never print them.

| File | Contents | Scope, least privilege | Needed by |
|---|---|---|---|
| `~/tidbcloud-lc0.env` | `TIDB_CLOUD_PUBLIC_KEY`, `TIDB_CLOUD_PRIVATE_KEY`, `TI_REGION_CODE=aws-us-east-1` | An organization-level API key on the **no-card** organization used for LC0. The `ti` CLI's Filesystem commands require **Organization Owner** (Filesystem quick start), so use a dedicated organization or project that holds nothing else if possible. Revoke it after LC0 | LC0 Tasks 1–3, 5 |
| `~/workos-staging.env` | `WORKOS_API_KEY` (the **staging** environment's secret key), `WORKOS_CLIENT_ID` (staging) | **Staging only.** Production keys are never given to an agent; the owner applies production settings from LC2a Task 6's checklist | LC2a Tasks 4–8, 11 (and LC0 Task 5 if run on WorkOS) |
| `~/cloudflare-lc0.env` | `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_API_TOKEN` | A token limited to **Workers Scripts: Edit** on Ostrium's account (Durable Objects and Cron Triggers deploy with the script), for a `loams-cloud-lc0` Worker on a `workers.dev` route only | LC0 Task 3 |

**Later, not needed now:**
- `~/resend.env`, holding `RESEND_API_KEY`: a sending-access key restricted to `mail.loams.dev`, created by the owner after creating the Resend account. Needed for LC2a Task 2.
- `~/workos-staging.env`, adding `WORKOS_WEBHOOK_SECRET` and `WORKOS_ACTIONS_SECRET`. WorkOS shows them when the staging endpoints are registered in LC2a Tasks 6 and 8.
- `~/turnstile-staging.env`, holding the Turnstile site key and secret for a test widget. Needed for LC2a Task 3.
- A GitHub App token for the GitHub-age check, scoped to read public user data only. Needed for LC2a Task 7.

## LC1–LC5 outline

| Milestone | Scope | Depends on |
|---|---|---|
| **LC1: the `tidbx` target and multi-device dev (BYOG)** | `deploy/targets/<id>.toml` manifests for all six targets plus `devices` (D1707, D1735), generated into §78 §13's tables and joined to §71's `free-tiers.json`; `loams target list\|show`; `[environments.*]` in `loams.toml` and `loams env list`; the `tidbcloud` backend for `loams.sqldb.v1` (BYOG: instance per database, TiDB branches) with its conformance column; agentd workspaces on Filesystem (a layer per run, drain on approval and handoff, `coding-agent` profile) using the user's own `ti` session; `loams run move` between the user's own devices over their tailnet; optional `tsnet` in agentd; the `tidbx` conformance suite (D1734), nightly and non-blocking on the owner's no-card org | LC0 report; SD1 remote agentd |
| **LC2: the hosted try-out** | `deploy/loams-cloud/` (a Worker, per-user Durable Objects, the control DB schema); `loams cloud login\|device\|status\|export\|move`; device machine users (IA4); cells and the cell map; cohort Filesystem with budgets and the kill switch; the try-out plan in the Meter (ME1); abuse controls; `loams.dev/cloud` pages in loams-site (a separate PR in that repository, CodeRabbit-reviewed there); the threat model; `LOAMS_CORS_ORIGINS` default change and the networking map; Ostrium's OpenTofu root with R2 state (D1729). Invite-only throughout (D1736); its first slice is LC2a above | D1701 approved; IA2, IA4; ME1's enforcement path; counsel |
| **LC3: Cloudflare** | The `cloudflare` target wired to BY0's Cloudflare template (§70 §7, D1367) and AB0's Cloudflare adapter (§71); Tunnel-based device-to-device as the documented non-tailnet path (§69 ER0) | BY0 Cloudflare, AB0, ER0 |
| **LC4: Vercel** | The `vercel` target wired to AB1's Vercel adapter and §70 §8's integration; Vercel Sandbox as an `agents` option for short runs | AB1 |
| **LC5: production targets and `loams promote`** | `aws`, then `gcp`, then `azure` targets wired to BY0/BY1 (§70) and AB0/AB1 (§71); `loams promote --from --to` opening a PR in the company's infrastructure repository; company orgs and the identity mapping (D1725); the company-run control plane guide (D1726) | BY1, AB1, IA3 |

## Rulings made during execution

### LC0 Task 0: rulings and pins (2026-10-23)

- **R-LC0-1, owner rulings.** The owner answered Q1260–Q1277 on 2026-10-23 with every default. The same day's §25 amendments change three of those answers in effect:
  - Q1263 and Q1265: the hosted Filesystem cohort is off and D1730's ceiling is $0 until the owner sets a budget (Q1291, D1756).
  - Q1266: the invite codes are replaced by waitlist email invites (D1749). The providers and the GitHub-age rule stand.
  - D1714, for the try-out only: device identity is WorkOS CLI Auth plus device keys issued by the control plane (D1755).

  The owner created the no-card LC0 organization (Q1277) and handed over the credentials on 2026-10-23.
- **R-LC0-2, pins.**
  - **`ti`:** v0.2.10 (`e6fbd15`, 2026-10-15). It was installed from the release tarball and checked against `ti_checksums.txt`, not by piping `install.sh` to a shell. The script is at `https://github.com/tidbcloud/ti-cli/releases/download/v0.2.10/install.sh` (Apache-2.0).
  - **The companion `ti-drive9`:** the install script fetches it from `https://releases.drive9.ai/latest/`, which isn't versioned. LC0 ran drive9 build `572cac6`, checked against that directory's `SHA256SUMS`. agentd (LC1) must pin the companion by checksum, because "latest" moves.
  - **The 14-day rule:** `ti` 0.2.10 was 8 days old. The rule binds the npm driver, and `ti` is a CLI the user installs, so LC1 pins the newest `ti` that is at least 14 days old when it ships.
  - **`@tidbcloud/serverless`:** 0.3.0 (2026-03-17).
  - **IA0's pins:** Zitadel `v4.19.4@sha256:90a3045b…` and OpenFGA `v1.22.0@sha256:9cf9a20a…`, from `conformance/identity/compose.yaml`.
  - **Anonymous telemetry:** `ti` collects it in release builds. LC0 turned it off in `~/.ti/.preferences` (`[telemetry] enabled = false`). agentd sets `TI_TELEMETRY=off`.
- **R-LC0-3, dependencies.**
  - **IA2:** Task 1 has landed (the `loams-authz` crate, #993). The rest of IA2, and IA4 (agents as machine users), have not.
  - **SD1:** remote agentd (D909) is on `dev` (`serve --remote`, SD1 Task 7, #548).
  - **ME1:** not started (a stub).
- **R-LC0-4, re-reads (2026-10-23).** No limit changed since the report.
  - The release notes have no Starter, Filesystem, branching, `ti` or driver entry after 2026-10-15. The newest entry is 2026-10-20 (Essential, Premium and BYOC only).
  - Starter is still 5 free instances per organization, each with 5 GiB row, 5 GiB columnar and 50M RUs a month. Connections: 400, or 5,000 with a spending limit. Transactions under 30 minutes. The PG preview is still 10 free instances, each with 50 GiB and 500M RUs.
  - The limitations page no longer states the 100-instance cap or the branch quota; the release notes (2026-06-02) and the branch docs still do.
  - The Filesystem limits page is unchanged: limits and usage are in the console only.
  - Two more rows matter to Task 1: `tidb_enable_auto_analyze` is read-only on Starter, and `SLOW_QUERY` and `STATEMENTS_SUMMARY` are console-only.
- **R-LC0-5, scope (owner, 2026-10-23, through the LC0 lead).**
  - **Task 5's Zitadel device enrolment is not run in LC0** ("Zitadel isn't needed now"): the try-out uses D1755. Only Task 5's TiDB question ran, whether OpenFGA's MySQL datastore works on Starter, in `spikes/lc0-device/` (PR #1115). The Rust device binary and its three tests wait for the self-hosted path.
  - **Task 6 (company device)** needs a managed-like macOS or Windows laptop, which agents don't have. The LC0 report gives the runbook for the owner.
  - **Task 2's macOS leg** needs the owner's Mac.
  - **Task 4's** one manual Zero creation and claim is the owner's.
- **R-LC0-6, labels.** `ti db create-db-cluster` has no label flag. LC0 named every instance `lc0-<task>` and added an `lc0` label through the API (`PATCH /v1beta1/clusters/{id}`, `updateMask: labels`), which works with the org key.
- **R-LC0-7, PRs.** Task 1 and Task 5's OpenFGA check share one PR (#1115). Tasks 4 and 6 are sections of `docs/plans/lc0-report.md`, committed with their own commit messages in the report's PR.

### LC2a Task 0: rulings, pins and the (verify) items (2026-10-23)

Checked against WorkOS's docs, its Node SDK (`workos-node@b9ffc7e`) and read-only calls on the **staging** environment. Resend was checked against its docs, a read-only API call and public DNS; Cloudflare against its docs. Nothing was changed in either dashboard.

On WorkOS staging, LC0 created two things, and neither sent any email (Resend's log shows 0 emails):
- one test user (`lc0-test-1@example.net`), which was deleted;
- one invitation (`lc0-resend-probe@example.com`), which was revoked, since WorkOS has no delete for invitations.

Items marked **owner** need the owner before LC2a code starts.

- **R-LC2a-1, the sender (owner ruling, 2026-10-23).** Loams Cloud sends from **`cloud@loams.dev`**, the root domain. The owner verified it in Resend. **Amends Q1278 and D1738** (`cloud@mail.loams.dev`) and the plan's "`mail.loams.dev` must be verified".
  - `mail.loams.dev` is already Brevo's: a CNAME to `mail-loams-dev.brand.brevosend.com`, its own SPF and MX.
  - The root also carries Brevo's verification and DKIM, and Cloudflare Email Routing's MX, SPF and DKIM.
  - So §25.2's "one sending domain, one reputation" isn't true: Resend, Brevo and Cloudflare share loams.dev.
  - **The root DMARC** is `p=none` with `rua=mailto:rua@dmarc.brevo.com`. It stays at `p=none` until Resend, Brevo and Email Routing are each confirmed aligned and `rua` points at a mailbox Ostrium reads. Then `quarantine` for 30 days, then `reject`, as in Q1278. **Owner.**
- **R-LC2a-2, Resend keys (owner).** The key in `~/resend.env` has **full access**, not sending only: `GET /domains`, `/api-keys`, `/emails`, `/webhooks` and `/audiences` all return 200. The account has one key. The owner creates a sending-only key restricted to loams.dev for the Worker and for agents. The full-access key stays only in WorkOS's custom-provider settings, which require one (V, [custom email providers](https://workos.com/docs/authkit/custom-email-providers)).
- **R-LC2a-3, Resend settings (owner).**
  - **Tracking:** the loams.dev domain has **open and click tracking on**. Click tracking rewrites links through `links.loams.dev`, which would put confirm and removal tokens in Resend's click logs, against §25.6. Turn both off.
  - **Region:** the domain sends from **`ap-northeast-1`**, while account data stays in the US (V, [regions](https://resend.com/docs/dashboard/domains/regions)). Moving it to `us-east-1` means deleting and re-adding the domain. That is the owner's choice.
- **R-LC2a-4, bounces and complaints. Amends §25.9.**
  - WorkOS "does not receive delivery results from a custom email provider", and Resend isn't supported for WorkOS's suppression management (V, [email](https://workos.com/docs/email)).
  - So **Resend is the only source of delivery results**, for WorkOS's mail and the Worker's alike. They arrive through one webhook to `POST /internal/resend/webhook`, signed by Svix (`svix-id`, `svix-timestamp`, `svix-signature`).
  - `email.bounced`, `email.complained` and `suppression.added` write `waitlist_suppressed`.
  - Resend Free allows **one** webhook endpoint, 100 emails a day and 3,000 a month. Over the limit it returns `429` with `daily_quota_exceeded`, and the `x-resend-daily-quota` header reports usage (V, [pricing](https://resend.com/pricing), [rate limit](https://resend.com/docs/api-reference/rate-limit)). Commercial use on Free isn't restricted (ToS §6, updated 2026-08-27).
- **R-LC2a-5, WorkOS (verify) items:**
  1. **Methods:**
     - Every method is a per-environment toggle, and no tier gates any of them. Social login has no per-provider charge, so Microsoft through our own Entra app costs $0 (V, [pricing](https://workos.com/pricing)).
     - Whether the dashboard insists on at least one method isn't checked (dashboard only).
     - **Account age:** Google and Microsoft expose no age signal. An identity has only `idp_id`, `type` and `provider` (V, staging and the [identity reference](https://workos.com/docs/reference/authkit/identity)).
  2. **Sign-up off, invitations on:** the environment's Sign-up toggle. "When a valid invitation code is present … registration is opened up" (V, [invite-only sign-up](https://workos.com/docs/authkit/invite-only-signup)).
  3. **The action context names the method:**
     - Both contexts carry an optional `authentication_method`, plus `ip_address` and `user_agent`.
     - The registration context also carries the **`invitation`** object, so the registration action matches `context.invitation.id` against `invites.workos_invitation_id` and falls back to the email.
     - When `authentication_method` is absent, the action reads the user's identities (V, SDK `action.interface.ts` and its fixture).
  4. **The `authentication_method` values:**
     - The policy's four: `GoogleOAuth`, `GitHubOAuth`, `MicrosoftOAuth`, `MagicAuth`.
     - The union also has `Password`, `Passkey`, `SSO`, `AppleOAuth`, the other social providers, and **`CrossAppAuth`, `ExternalAuth`, `MigratedSession` and `Impersonation`**. So the policy is an **allow-list** of those four. Everything else is refused at every layer.
     - The identity `provider` can read `GithubOAuth`, so compare case-insensitively (V, SDK).
  5. **GitHub `idp_id`** is GitHub's numeric user id. The SDK fixture's `108872335` resolves with `GET /user/108872335`. LC2a Task 11 confirms it on a live sign-in.
  6. **Resend keeps the token, the accept URL and `expires_at` (measured on staging).**
     - The test: create (`expires_in_days: 1`), wait 65 s, `resend`. The token's hash, the URL's hash and `expires_at` were unchanged; only `updated_at` moved.
     - Events `invitation.created`, `invitation.resent` and `invitation.revoked` were all emitted.
     - So `--extend` stays "revoke and send a new one" (D1747), and a resend never extends.
     - **There is no `invitation.expired` event,** so expiry is mirrored only by the hourly sweep (LC2a Task 9), not the events reader.
  7. **The MFA API is free:** MFA is inside AuthKit's free tier, with no separate line. Use AuthKit's user-scoped factors API (`/user_management/users/{id}/auth_factors`, which returned 200 on staging) for the TOTP step-up (V, [pricing](https://workos.com/pricing), [AuthKit MFA](https://workos.com/docs/reference/authkit/mfa)).
  8. **Branding is free:** logo, colours, display name and CSS have no tier gate. Only a custom domain costs $99 (V, [branding](https://workos.com/docs/authkit/branding)).
  9. **Data location:** the USA. The DPA covers EEA, UK and CH transfers with SCCs, and there is no selectable region (V, [DPA](https://workos.com/legal/data-processing-addendum)). The current subprocessor list is on trust.workos.com, which renders only with JavaScript, so the last list read is still the one dated 2024-12-11.
  10. **Commercial use of the free tier:** not banned. The terms (2026-08-04) say the free tier is "free of charge" and may be changed or ended "at any time, without notice" (§3(d)). Use is for "internal business purposes" (§2(a)), and §5 bars "service bureau" use. **For counsel (Q1276).**
- **R-LC2a-6, what `check-workos` can read.**
  - **With the API key:** redirect URIs, CORS origins, webhook endpoints, roles, SSO connections and organizations.
  - **With a widget token for the Ostrium admin**, through the Widgets GraphQL API: `passwordAuthEnabled`, `passkeyAuthEnabled`, `mfaEnabled`, `mfaRequired` and the branding display name.
  - **Not readable at all:** the social providers, Magic Auth, the Sign-up toggle, the custom email provider, the Actions endpoints and their failure mode, and impersonation. So `check-workos` automates the first two groups, and `email_provider_is_resend` is a **manual owner tick** (LC2a Tasks 2 and 6).
  - Add **"impersonation off"** to Task 6's checklist. It is off by default; a dashboard admin can turn it on.
- **R-LC2a-7, WorkOS tests.**
  - Automated tests use addresses at `example.com`, `example.net`, `example.org` or `.test`. WorkOS drops those sends before the email provider (V, [email](https://workos.com/docs/email)). LC0's invitation probe sent nothing.
  - `@workos/emulate` (MIT, v0.16.0) is a candidate offline double for the Miniflare tests (Tasks 5–8).
  - Staging has **no redirect URIs** yet. Adding the Miniflare and staging callbacks is an owner step, or an agent step through the API with the owner's permission.
  - CLI Auth works on staging: `POST /user_management/authorize/device` returns a device code, a user code, `expires_in: 300` and `interval: 5` (D1755).
- **R-LC2a-8, privacy notice (§25.6).**
  - Resend: account data in the US; sends from the domain's region; 22 subprocessors, all in the USA (list dated 2026-08-27).
  - **WorkOS events keep a deleted user's email for up to 90 days.** `user.deleted` carries `email` and `external_id`, as seen on staging. The notice says so, and Review Focus 5's test accepts it.
- **R-LC2a-9, Cloudflare Workers Free (docs; measured in LC0 Task 3).**
  1. **Durable Objects on Free:**
     - 100,000 requests a day, "includ[ing] … WebSocket messages", 13,000 GB-s, 5M rows read and 100,000 written, and 5 GB stored. Operations fail past a limit until 00:00 UTC. SQLite-backed objects only.
     - The page calls the 20:1 WebSocket ratio "for compute requests billing", and doesn't say how the Free daily limit counts messages. LC0 Task 3 measures it.
  2. **Cron Triggers:** 5 per account on Free, shared with the site Worker's crons, with 10 ms of CPU per invocation.
  3. **The rate-limiting binding:** GA, with no plan restriction listed. Counts are per location over a 10 s or 60 s period, so a strict global limit needs a Durable Object.
  4. **Turnstile Siteverify:** single use (a replay returns `timeout-or-duplicate`); tokens are valid for 300 s; `idempotency_key` allows safe retries.

  Also: Tunnel is on all plans, with no price. Zero Trust Free covers 50 users, but onboarding asks for payment details, so **Access isn't used at $0**.
- **R-LC2a-10, domain lists (D1753).**
  - **The pins:**
    - `Kikobeats/free-email-domains` at `18d04cbaa077cca754e129a417fbdf780dea332a` (2026-10-23, release 1.15.12, MIT);
    - `disposable-email-domains/disposable-email-domains` at `8c05972cf08375d540d50b96b15b7db360fb0a29` (2026-10-22, CC0-1.0).
  - **The sources of `free-email-domains`:**
    - Its only free-mail source is **HubSpot's CSV, which has no licence** (`scripts/sources.mjs`). Its other sources are disposable lists: MIT, CC0, BSD-3-Clause (7c/fakefilter, notice kept) and CC-BY-4.0 (GeroldSetz, through groundcat; attribution needed).
    - **For counsel (Q1276):** accept the HubSpot entries, or build our own free-mail set from the `block` families plus an MIT list. The second option changes D1753's wording.
  - **Vendoring:** the lists are vendored with a `NOTICE` that lists every licence.
- **R-LC2a-11, the TiDB items read for §25 and §17.**
  - The `tidbcloud/tidbcloud` Terraform provider 0.4.11 is listed on the OpenTofu registry (Q1273).
  - The billing API works with the LC0 org key. It gives monthly totals by project and service (`GET /v1beta1/bills/{month}`, $0 so far), daily rows per cluster (`/billsDetails`) and a cost explorer. It is per cluster at best, so it can't meter tenants inside a cell.

### LC2a status (2026-10-24)

Built against local doubles only (an in-memory or throwaway TiDB v8.5.8, fake WorkOS, Turnstile, mail and DNS). Nothing deployed; no credential file read.

- **Merged:** Task 1, the control-DB schema (#1128); Task 3, the waitlist API (#1171).
- **Built and tested, pushed as branches, not yet in pull requests** (the owner paused new work on 2026-10-24). Each is one commit on top of the previous, in plan order, and carries its rulings (R-LC2a-12 to R-LC2a-19) in this file:
  - `lc2a-admin`: Task 4, admin authorisation (`cloud_admin`, the allow-list, the TOTP step-up; `platform:loams-cloud#cloud_admin` in `deploy/openfga/core.fga`).
  - `lc2a-invites`: Task 5, WorkOS invitations, resend and revoke, and the shared daily email budget.
  - `lc2a-admin-cli` (on `lc2a-invites`): `loams cloud admin` (crate `loams-cloud-admin`).
  - `lc2a-lifecycle` (on `lc2a-invites`): Task 9, `DELETE /me`, deletion and retention, the hourly invite sweep.
  - `lc2a-worker`: the Worker entry, Durable Objects, crons and `wrangler.toml` (not deployed; CI runs a `--dry-run`).
  - `lc2a-threat-model`: Task 12's draft threat model and go-live checklist.
- **Blocked on the owner:** Task 2 (the sending-only Resend key, R-LC2a-2, and tracking off, R-LC2a-3); Task 6 (the WorkOS actions secret and staging settings); Task 7 (staging redirect URIs, and a GitHub App token for the age check); Task 8 (the webhook secret); Task 10 (loams-site pages, after Task 7); Task 11 (all of the above, plus the Turnstile test key); counsel on the free-mail list (R-LC2a-10, Q1276).

### LC2a Tasks 1–5: execution rulings (2026-10-23)

Tasks 1, 3, 4 and 5 were built before the owner's pending steps (the sending-only Resend key, WorkOS staging settings, the webhook and actions secrets, the Turnstile key), against local doubles only. Nothing was deployed, and no credential file was read.

- **R-LC2a-12, test doubles.** `deploy/loams-cloud` has no npm dependencies. The suites run on Node with type stripping. The control-DB suites run on a throwaway TiDB v8.5.8 on unistore (`scripts/test-tidb.sh`), reached through the mysql client, which binds `?` parameters the way `@tidbcloud/serverless` does. WorkOS, Turnstile, mail, DNS and rate limits are fakes in `test/harness/`. `@workos/emulate` (R-LC2a-7) isn't used, to stay dependency-free; the staging run is Task 11's.
- **R-LC2a-13, the write rule (LC0 Task 3).** Every control-DB write is one statement that is safe to repeat: an insert with a Worker-minted id or a natural key (`INSERT IGNORE`), or an update guarded by the state it leaves. API-level `Idempotency-Key`s live in `idempotency_keys`. In the Worker, control-DB writes go through a Durable Object created with a location hint near the Starter region; account-scoped writes go through the user's own DO (lc0-report §4).
- **R-LC2a-14, Task 3.** The address-dependent work of `POST /waitlist` and `DELETE /waitlist` runs after the response, so status, body, headers and timing don't depend on the address's state. A resolver failure during the MX check lets the address through (fail-open); the email-code check (Task 6) fails closed. The free-mail list isn't vendored (R-LC2a-10, counsel); only the disposable list is, at its pin, with its CC0 dedication and a `NOTICE` entry.
- **R-LC2a-15, Task 4.** `platform:loams-cloud#cloud_admin` is in `deploy/openfga/core.fga` (users only) with model tests. `BuiltinAuthorizer` doesn't exist yet (IA2 Task 3), so its parity lands with it. Tokens without a person's session (no `sid`, a non-`user_` subject, an `act` or `may_act` claim) are refused before the role is read. The access-token issuer is configuration; its exact value on staging is a (verify) for Task 11.
- **R-LC2a-16, the CLI.** `loams cloud admin` covers Task 4's and Task 5's endpoints, so it lands in one PR after Task 5 instead of half in each.
- **R-LC2a-17, Task 5.**
  - **Exactly one WorkOS invitation per entry.** WorkOS's create takes no idempotency key we could rely on, and a create whose response is lost may still have happened. So the queue claims an entry with a guarded update on `attempts`, and every retry first looks for the address's pending invitation (`GET /user_management/invitations?email=`, a (verify) on staging for Task 11) and adopts it. `invites.open_key` allows at most one open invite per entry across batches.
  - **The daily cap.** `POST /admin/invites` refuses a batch larger than today's remaining invitation budget less what is already queued (`409 DAILY_SEND_CAP`). A resend takes budget too. Sends are counted in the control DB's `email_budget` (Task 1), which is also the pacer Task 2's Worker mail will use. A failed create attempt still takes budget, because a lost response can't be told from a refusal; that only ever sends less. WorkOS's magic codes go through Resend uncounted, so the cap of 100 is a ceiling on what we cause.
  - **Revoke** mirrors WorkOS: an invitation already accepted or expired there is refused with `409 INVITE_NOT_REVOCABLE`, and our row takes WorkOS's state. Removing a waitlist entry revokes its open invitation at WorkOS.
  - **Resend** reads WorkOS first (there is no `invitation.expired` event, R-LC2a-5.6), counts the resend before sending, and keeps the expiry. `extend` revokes and creates a new invitation for the invite's full term from now, on the same invite row.
- **R-LC2a-18, Task 9.** `DELETE /me`, the deletion job, the retention jobs and the hourly invite sweep are built and tested on seeded accounts, because accounts come from Task 7's callback, which waits on the owner's WorkOS staging steps. There is no tenant data in LC2a, so the export step and the per-user Durable Object join the deletion job in LC2. Deletion keeps the contact and sign-in addresses' HMACs as suppressions, so a deleted account's address can't rejoin the waitlist for 24 months. Retention purges (unconfirmed after 7 days, no answer 30 days after the 12-month reminder, 30 days after an expired invite) delete without suppressing, so the person may join again. Each statement handles at most 200 rows per run.
- **R-LC2a-19, the Worker.** `deploy/loams-cloud/src/worker.ts` and `wrangler.toml` wire Tasks 3–5 and 9 for the live run: a ControlDb Durable Object runs every control-DB statement with `@tidbcloud/serverless` 0.3.0 (a 10 s bound on each request, connection strings scrubbed from errors) and is reached with `locationHint: "enam"`; RateLimiter objects hold §25.7's global windows; two Cron Triggers (hourly: drain, expiry sweep, deletions; daily: retention) of the account's five. `workers_dev = false` and no route, so nothing is reachable until the owner deploys. Until Task 2, the Worker's mailer refuses to send and the waitlist logs `waitlist.send_failed`. CI typechecks `worker.ts` against `@cloudflare/workers-types` and runs `wrangler deploy --dry-run`.

### Owner approvals (2026-10-24)

The owner approved the pending decisions below on 2026-10-24 ("approve all with your recommendations").

- **R-LC0-8, T3-1: device pings use Durable Object WebSocket auto-response.** Presence and lease renewal come from the auto-response timestamp (`getWebSocketAutoResponseTimestamp`), never from "socket open". A ping writes no rows. **Approved (owner, 2026-10-24).** (Design 78 §9.2, §17; LC0 report §4.)
- **R-LC0-9, T3-2: the per-user Durable Object does the TiDB writes.** It is created with a `locationHint` near the control DB's region (19–25 ms from the DO against 270–290 ms from a distant edge colo). The edge Worker doesn't write TiDB on hot paths. **Approved (owner, 2026-10-24).**
- **R-LC0-10, D1756 stays on Workers Free.** "On Workers Paid" is dropped from D1727. Workers Paid remains Q1292's trigger-based owner decision. **Approved (owner, 2026-10-24).**
- **R-LC0-11, D1713 amended.** Drain at handoff and at approval, plus every 5 minutes only when the mount is dirty, and never on a timer for Git-workspace mounts (LC0 Task 2: a timed drain on a Git workspace costs 14 writes for nothing). **Approved (owner, 2026-10-24).**
- **R-LC0-12, D1718 amended.** Per-filesystem limits need a card, and there is no per-filesystem write counter, so the write budget can't be enforced by the provider. The hosted cohort stays off (§25.13, Q1291). **Approved (owner, 2026-10-24).**

The decision log rows D1713, D1718 and D1727 and design 78 §9.2, §9.4, §14.5, §15.2 and §17 carry the same text.
