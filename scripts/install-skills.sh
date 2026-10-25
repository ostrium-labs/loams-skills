#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET="all"
COPY=0
FORCE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target) TARGET="${2:?}"; shift 2 ;;
    --copy) COPY=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) echo "usage: $0 [--target all|claude|opencode|codex|cursor] [--copy] [--force]"; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done
[[ -d skills ]] || { echo "skills/ not found" >&2; exit 1; }

install_one() {
  local target="$1"
  if [[ "$COPY" -eq 1 ]]; then
    rm -rf "$target"
    mkdir -p "$(dirname "$target")"
    cp -R skills "$target"
    echo "copied  $target"
    return
  fi
  mkdir -p "$(dirname "$target")"
  if [[ -L "$target" ]]; then
    ln -sfn ../skills "$target"
    echo "linked  $target"
  elif [[ -e "$target" ]]; then
    if [[ "$FORCE" -eq 1 ]]; then
      rm -rf "$target"
      ln -s ../skills "$target"
      echo "relinked $target"
    else
      echo "exists  $target (use --force or --copy)" >&2
    fi
  else
    ln -s ../skills "$target"
    echo "linked  $target"
  fi
}

case "$TARGET" in
  all) install_one .claude/skills; install_one .opencode/skills; install_one .agents/skills ;;
  claude) install_one .claude/skills ;;
  opencode) install_one .opencode/skills ;;
  codex) install_one .agents/skills ;;
  cursor) ;;
  *) echo "unknown target: $TARGET" >&2; exit 2 ;;
esac

if [[ "$TARGET" == "all" || "$TARGET" == "cursor" ]]; then
  mkdir -p .cursor/rules
  cat > .cursor/rules/loams-skills.mdc <<'EOF'
---
description: Loams adoption skills
alwaysApply: true
---
# Loams adoption

Read `AGENTS.md`, then `company/AGENTS.md` if it exists, then
`skills/loams-adoption/SKILL.md`. Use the matching skill under `skills/` for each
stage. Never expose Loams publicly and never commit secrets.
EOF
  echo "wrote   .cursor/rules/loams-skills.mdc"
fi
