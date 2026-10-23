#!/usr/bin/env python3
"""The release notes for one version, read from CHANGELOG.md.

    scripts/release_notes.py 0.1.1 [--changelog PATH] [--out PATH]

Prints the body of the changelog's ``## 0.1.1`` section (the heading left
out, everything up to the next ``## `` heading), or writes it to --out.
Exits 3 with a message when the changelog has no such section or the
section is empty. The release job in .github/workflows/build.yml writes the
file this way and hands it to the GitHub release as its body;
scripts/release_ready.py uses the same lookup to refuse the tag command
while the section is missing.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CHANGELOG = "CHANGELOG.md"
_RELEASE_HEADING_RE = re.compile(r"^##\s")


def section(changelog: Path, version: str) -> str:
    """The notes under ``## <version>`` (``[<version>]`` and ``v<version>``
    are accepted, and anything may follow the version on the heading line),
    stripped; "" when the changelog has no such section or it is empty."""
    heading = re.compile(r"^##\s+\[?v?" + re.escape(version) + r"\]?(?:\s|$)")
    body: list[str] = []
    inside = False
    for line in changelog.read_text(encoding="utf-8").splitlines():
        if inside and _RELEASE_HEADING_RE.match(line):
            break
        if inside:
            body.append(line)
        elif heading.match(line):
            inside = True
    return "\n".join(body).strip()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("version", help="the release version, e.g. 0.1.1 (a leading v is dropped)")
    parser.add_argument("--changelog", type=Path, default=None,
                        help=f"the changelog (default: {CHANGELOG} in the repo owning this script)")
    parser.add_argument("--out", type=Path, default=None,
                        help="write the notes to this file instead of standard output")
    args = parser.parse_args(argv)
    version = args.version.removeprefix("v")
    changelog = args.changelog or Path(__file__).resolve().parents[1] / CHANGELOG
    if not changelog.is_file():
        print(f"{changelog} does not exist", file=sys.stderr)
        return 3
    notes = section(changelog, version)
    if not notes:
        print(f"{changelog} has no section for {version}", file=sys.stderr)
        return 3
    if args.out:
        args.out.write_text(notes + "\n", encoding="utf-8")
    else:
        print(notes)
    return 0


if __name__ == "__main__":
    sys.exit(main())
