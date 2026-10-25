#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPO="${UPSTREAM_REPO:-https://github.com/ostrium-labs/loams.git}"
REF="dev"
WITH_AGENTS=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref) REF="${2:?}"; shift 2 ;;
    --repo) REPO="${2:?}"; shift 2 ;;
    --with-agents) WITH_AGENTS=1; shift ;;
    -h|--help) echo "usage: $0 [--ref <commit|tag|branch>] [--repo <url>] [--with-agents]"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

TMP="$ROOT/.sync-tmp.$$"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/loams"
git -C "$TMP/loams" init -q
git -C "$TMP/loams" remote add origin "$REPO"
git -C "$TMP/loams" fetch -q --depth 1 origin "$REF"
git -C "$TMP/loams" checkout -q FETCH_HEAD
SRC="$TMP/loams"
COMMIT=$(git -C "$SRC" rev-parse HEAD)
DATE=$(git -C "$SRC" show -s --format=%cs HEAD)
LATEST=$(git ls-remote --tags --refs "$REPO" 'v*' 2>/dev/null | awk -F/ '{print $NF}' | sort -V | tail -1 || true)
[[ -n "$LATEST" ]] || LATEST="unknown"

mkdir -p reference/design reference/plans reference/sdk reference/upstream
rm -f reference/design/*.md reference/plans/*.md reference/sdk/*.md

cp "$SRC/docs/design/README.md" reference/design/
for f in 00-pitch.md 01-architecture.md 13-decision-log.md 44-unified-api-and-sdks.md \
         63-loams-meter.md 70-byoc-clouds.md 71-app-byoc-free-tiers.md \
         76-identity-and-authorization.md 78-loams-cloud-on-tidb-x.md; do
  cp "$SRC/docs/design/$f" reference/design/
done

cp "$SRC/docs/plans/README.md" reference/plans/
for f in 2026-10-10-api2-unified-api-sdks.md 2026-10-19-sdk-parity.md \
         2026-10-21-by1-byoc-clouds.md 2026-10-21-ia1-identity-authz.md \
         2026-10-14-up1-upgrades.md 2026-10-23-lc1-loams-cloud.md; do
  cp "$SRC/docs/plans/$f" reference/plans/
done

cp "$SRC/sdks/README.md" reference/sdk/README.md
cp "$SRC/sdks/rust/README.md" reference/sdk/rust.md
cp "$SRC/docs/sdk/runtime-contract.md" reference/sdk/runtime-contract.md
cp "$SRC/docs/sdk/fixtures.md" reference/sdk/fixtures.md
cp "$SRC/docs/sdk/toolchain-2026-10.md" reference/sdk/toolchain-2026-10.md

if [[ "$WITH_AGENTS" -eq 1 ]]; then
  for f in 50-loams-desktop-daemon.md 52-loams-memory.md 53-studio-ade.md \
           56-loams-sandbox.md 72-software-factory-loop.md 73-agent-marketplace.md; do
    cp "$SRC/docs/design/$f" reference/design/
  done
  for f in 2026-10-10-cl1-loams-cli.md 2026-10-21-ft1-factory-loop.md 2026-10-21-mk1-agent-marketplace.md; do
    cp "$SRC/docs/plans/$f" reference/plans/
  done
fi

cat > reference/upstream/SOURCE.md <<EOF
# Upstream snapshot

- Upstream: $REPO
- Ref: \`$REF\`
- Commit: \`$COMMIT\`
- Commit date: $DATE
- Latest release tag: \`$LATEST\`
- Snapshot date: $(date +%F)
- License: Apache-2.0 (see \`NOTICE\` at the repository root)
- Curated by: Ostrium Labs

This directory is a **read-only snapshot** of selected Loams design, plan, and SDK
documents. Do not edit these files by hand. Run \`scripts/sync-reference.sh\` to
refresh the snapshot, then review the diff as a pull request.

The snapshot is curated: it contains the architecture, the full decision log, the
unified API/SDK material, the BYOG and identity designs, metering, and the
adoption-relevant plans. It is not the complete Loams documentation set.

\`SHA256SUMS\` pins every file below \`reference/\`. \`scripts/check-reference.sh\`
verifies it in CI. The \`NOTICE\` file records the upstream attribution.
EOF

cd reference
find design plans sdk -type f -print0 | sort -z | xargs -0 sha256sum > upstream/SHA256SUMS
echo "reference: synced $REF at $COMMIT ($(wc -l < upstream/SHA256SUMS) files)"
