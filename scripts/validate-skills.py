#!/usr/bin/env python3
"""Validate Agent Skills and write skills/manifest.json."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"
NAME_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
LINK_RE = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
REQUIRED = {"name", "description", "license"}


def parse_frontmatter(path: Path) -> tuple[dict[str, str], dict[str, str], str]:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise ValueError("missing opening ---")
    end = text.find("\n---\n", 4)
    if end == -1:
        raise ValueError("missing closing ---")
    raw = text[4:end]
    body = text[end + 5 :]
    top: dict[str, str] = {}
    meta: dict[str, str] = {}
    in_meta = False
    for line in raw.splitlines():
        if not line.strip():
            continue
        if line.startswith("metadata:"):
            in_meta = True
            continue
        if in_meta and line.startswith("  "):
            key, _, value = line.strip().partition(":")
            meta[key.strip()] = value.strip().strip('"\'')
            continue
        if not line.startswith(" "):
            in_meta = False
            key, _, value = line.partition(":")
            top[key.strip()] = value.strip().strip('"\'')
    return top, meta, body


def check_links(path: Path, body: str) -> list[str]:
    errors: list[str] = []
    in_fence = False
    for line in body.splitlines():
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for target in LINK_RE.findall(line):
            target = target.strip().split(" ")[0].strip("<>")
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target = target.split("#", 1)[0]
            if not target:
                continue
            resolved = (path.parent / target).resolve()
            if not resolved.exists():
                errors.append(f"{path.relative_to(ROOT)}: broken link {target}")
    return errors


def main() -> int:
    if not SKILLS.is_dir():
        print("skills/ not found", file=sys.stderr)
        return 1
    errors: list[str] = []
    entries: list[dict[str, str]] = []
    seen: set[str] = set()
    for path in sorted(SKILLS.glob("*/SKILL.md")):
        try:
            top, meta, body = parse_frontmatter(path)
        except ValueError as exc:
            errors.append(f"{path.relative_to(ROOT)}: {exc}")
            continue
        for key in REQUIRED:
            if not top.get(key):
                errors.append(f"{path.relative_to(ROOT)}: missing {key}")
        name = top.get("name", "")
        if not NAME_RE.match(name):
            errors.append(f"{path.relative_to(ROOT)}: invalid name {name!r}")
        if name != path.parent.name:
            errors.append(f"{path.relative_to(ROOT)}: name {name!r} != directory")
        if name in seen:
            errors.append(f"{path.relative_to(ROOT)}: duplicate name {name!r}")
        seen.add(name)
        desc = top.get("description", "")
        if not 1 <= len(desc) <= 1024:
            errors.append(f"{path.relative_to(ROOT)}: description length {len(desc)}")
        if not meta.get("reviewed"):
            errors.append(f"{path.relative_to(ROOT)}: missing metadata.reviewed")
        if not meta.get("upstream"):
            errors.append(f"{path.relative_to(ROOT)}: missing metadata.upstream")
        errors.extend(check_links(path, body))
        entries.append(
            {
                "name": name,
                "description": desc,
                "license": top.get("license", ""),
                "path": str(path.relative_to(ROOT)),
                "version": meta.get("version", ""),
                "reviewed": meta.get("reviewed", ""),
                "upstream": meta.get("upstream", ""),
            }
        )
    if not entries:
        errors.append("no skills found")
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    manifest = {"schema": 1, "count": len(entries), "skills": entries}
    (SKILLS / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print(f"skills: {len(entries)} valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
