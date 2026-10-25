#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
YES=0
[[ "${1:-}" == "--yes" ]] && YES=1
if [[ "$YES" -eq 0 && -t 0 ]]; then
  read -r -p "Stop syncing the upstream snapshot? The pinned copy stays. [y/N] " v
  [[ "${v:-}" == "y" || "${v:-}" == "Y" ]] || exit 0
fi
rm -f .github/workflows/sync-upstream.yml
cat > reference/upstream/DETACHED.md <<'EOF'
# Detached from upstream sync

The automatic `sync-upstream` workflow has been removed. The pinned snapshot in
this directory remains readable, and its hashes still verify. To refresh it
manually, restore the workflow from the template or run:

```sh
scripts/sync-reference.sh --ref <commit-or-tag>
```

To remove the snapshot entirely, delete `reference/` and then remove or rewrite
the links in `skills/` that point into it.
EOF
echo "detached: sync workflow removed; snapshot kept"
