#!/usr/bin/env python3
"""Check relative Markdown links outside the pinned reference snapshot."""
from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parent.parent
LINK_RE = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SKIP_DIRS = {".git", "reference", "target", ".sync-tmp"}
SKIP_FILES = {"LICENSE", "CODE_OF_CONDUCT.md"}


def main() -> int:
    errors: list[str] = []
    checked = 0
    for path in sorted(ROOT.rglob("*.md")):
        rel = path.relative_to(ROOT)
        if any(part in SKIP_DIRS for part in rel.parts) or rel.name in SKIP_FILES:
            continue
        in_fence = False
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line.strip().startswith("```"):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            for target in LINK_RE.findall(line):
                target = target.strip().split(" ")[0].strip("<>")
                if target.startswith(("http://", "https://", "mailto:", "#")):
                    continue
                target = unquote(target.split("#", 1)[0])
                if not target:
                    continue
                resolved = (path.parent / target).resolve()
                checked += 1
                if not resolved.exists():
                    errors.append(f"{rel}:{lineno}: broken link {target}")
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"links: {checked} relative links checked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
