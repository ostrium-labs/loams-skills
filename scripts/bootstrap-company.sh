#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

NAME="Example Company"
SLUG="example"
CLOUD="aws"
IDP="https://auth.example.com"
VERSION="0.1.1"
DRY=0
DETACH=0
NONINTERACTIVE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --name) NAME="${2:?}"; shift 2 ;;
    --slug) SLUG="${2:?}"; shift 2 ;;
    --cloud) CLOUD="${2:?}"; shift 2 ;;
    --idp-issuer) IDP="${2:?}"; shift 2 ;;
    --loams-version) VERSION="${2:?}"; shift 2 ;;
    --dry-run) DRY=1; shift ;;
    --detach) DETACH=1; shift ;;
    --non-interactive) NONINTERACTIVE=1; shift ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

if [[ "$NONINTERACTIVE" -eq 0 && -t 0 ]]; then
  read -r -p "Company name [$NAME]: " v; NAME="${v:-$NAME}"
  read -r -p "Slug [$SLUG]: " v; SLUG="${v:-$SLUG}"
  read -r -p "Cloud (aws/gcp/azure) [$CLOUD]: " v; CLOUD="${v:-$CLOUD}"
  read -r -p "OIDC issuer [$IDP]: " v; IDP="${v:-$IDP}"
  read -r -p "Loams version [$VERSION]: " v; VERSION="${v:-$VERSION}"
fi

if [[ "$DRY" -eq 1 ]]; then
  echo "dry run: would write company/company.yaml, D0001, and install skills"
  echo "  name=$NAME slug=$SLUG cloud=$CLOUD issuer=$IDP loams=$VERSION"
  exit 0
fi

mkdir -p company/decisions company/stack-inventory company/architecture company/runbooks
cat > company/company.yaml <<EOF
# Company configuration. Read by the skills. Do not commit secrets.
company:
  name: "$NAME"
  slug: "$SLUG"
  cloud: "$CLOUD"
  region: "CHANGE_ME"
  oidc_issuer: "$IDP"
  oidc_audience: "loams"
loams:
  version: "$VERSION"
  namespace: "$SLUG"
  endpoint: "https://loams.<tailnet>.ts.net"
  auth_mode: "oidc"
deployment:
  shape: "one-vm"
  bucket: "s3://CHANGE_ME"
  kms_key: "CHANGE_ME"
  secret_store: "CHANGE_ME"
edges:
  rust: true
  languages: []
EOF

if [[ ! -f company/decisions/D0001-adopt-loams.md ]]; then
  cat > company/decisions/D0001-adopt-loams.md <<EOF
# D0001: Adopt Loams for the company data platform

- Status: proposed
- Date: $(date +%F)
- Owner: CHANGE_ME
- Upstream: design 01, design 70; no single D-number

## Context

$NAME is evaluating Loams as the data platform for its stack, running on its own
ground (BYOG) in $CLOUD. The company will rewrite data-owning services in Rust
and keep edge services on the Loams SDK for their language.

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
EOF
fi

cat > company/decisions/README.md <<'EOF'
# Company decisions

One ADR per file: `D####-<slug>.md`. Never renumber. Link the upstream design or
`D####` this decision depends on, or say it is company-specific.

| ID | Title | Status | Date |
|---|---|---|---|
| D0001 | Adopt Loams for the company data platform | proposed | - |
EOF

if [[ "$DETACH" -eq 1 ]]; then
  scripts/detach-upstream.sh --yes
fi

scripts/install-skills.sh --target all || true
echo
echo "Bootstrapped $NAME ($SLUG) on $CLOUD, Loams $VERSION."
echo "Next:"
echo "  1. Fill company/stack-inventory/ (use the stack-inventory skill)."
echo "  2. Record fit decisions in company/decisions/."
echo "  3. Design company/architecture/target.md."
