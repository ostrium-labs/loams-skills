#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
missing=0
for f in README.md LICENSE NOTICE SECURITY.md CONTRIBUTING.md AGENTS.md \
         docs/migration-playbook.md reference/upstream/SOURCE.md \
         skills/loams-adoption/SKILL.md starter/Cargo.toml deploy/byog/README.md; do
  if [[ ! -e "$f" ]]; then echo "missing $f" >&2; missing=1; fi
done
[[ $missing -eq 0 ]] || exit 1
if [[ -f .env ]]; then echo "committed .env" >&2; exit 1; fi
if git ls-files --error-unmatch .env >/dev/null 2>&1; then echo ".env is tracked" >&2; exit 1; fi
count=$(find skills -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l)
[[ "$count" -ge 16 ]] || { echo "expected 16 skills, found $count" >&2; exit 1; }
if grep -RInE 'AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{30,}|-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----' \
  --exclude-dir=reference --exclude-dir=.git --exclude=LICENSE . >/dev/null 2>&1; then
  echo "possible secret committed" >&2
  grep -RInE 'AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{30,}|-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----' \
    --exclude-dir=reference --exclude-dir=.git --exclude=LICENSE . >&2 || true
  exit 1
fi
echo "template: hygiene ok ($count skills)"
