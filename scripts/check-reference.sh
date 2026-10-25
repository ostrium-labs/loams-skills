#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/reference"

test -f upstream/SOURCE.md || { echo "missing reference/upstream/SOURCE.md" >&2; exit 1; }
test -f upstream/SHA256SUMS || { echo "missing reference/upstream/SHA256SUMS" >&2; exit 1; }
for field in "Upstream:" "Ref:" "Commit:" "License:"; do
  grep -q "$field" upstream/SOURCE.md || { echo "SOURCE.md missing $field" >&2; exit 1; }
done
sha256sum -c upstream/SHA256SUMS >/dev/null
echo "reference: $(wc -l < upstream/SHA256SUMS) files verified"
