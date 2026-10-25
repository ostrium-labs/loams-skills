# Security

## Scope

This repository is a template: agent skills, documentation snapshots, a Rust
starter, and deployment examples. It is not a Loams distribution and it does not
run a service for anyone.

## Reporting

Report a vulnerability in the template (for example, a secret committed by
mistake, a dependency confusion risk, or an unsafe deployment example) to
**security@ostriumlabs.org**. Do not open a public issue for a vulnerability.

Report a vulnerability in Loams itself through the Loams repository's security
policy: https://github.com/ostrium-labs/loams/security/policy.

## The rules this template repeats

- **Never expose Loams to the public internet.** A BYOG install has no inbound
  rule; reach it over your own tailnet.
- **Never commit secrets.** `.env` files are ignored. Use your cloud's secret
  store and workload identity.
- **Pin and verify artifacts.** Use an explicit Loams version and verify the
  release signature; never deploy `latest`.
- **Use a local block disk** for the embedded stores. Do not use NFS, EFS, Azure
  Files, Filestore, or a FUSE mount of a bucket.
- **Keep company data in the company's account.** Ostrium runs no data plane for
  a BYOG install.
