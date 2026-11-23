#!/usr/bin/env python3
"""Prepare a release commit: bump Cargo.toml and date the changelog.

Used by .github/workflows/release-pr.yml; also runs locally from the repo
root. It edits Cargo.toml and CHANGELOG.md only; the workflow refreshes
Cargo.lock with `cargo update --workspace` afterwards.

    python3 .github/scripts/prepare-release.py patch "Picks up the regex fix"

Dependency bumps since the last tag are always listed under
"Dependencies". Other commit subjects fill the section only when
[Unreleased] is empty (the usual case in maintenance mode).
"""

import datetime
import os
import re
import subprocess
import sys

REPO_URL = "https://github.com/lijunzh/hunch"


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout.strip()


def bump_version(current, part):
    major, minor, patch = (int(x) for x in current.split("."))
    if part == "minor":
        return f"{major}.{minor + 1}.0"
    if part == "patch":
        return f"{major}.{minor}.{patch + 1}"
    sys.exit(f"unknown bump '{part}' (use patch or minor; majors are released by hand)")


def commit_notes(last_tag):
    """Changelog sections from commit subjects since the last tag.

    Returns (dependencies, other). Dependency bumps are never written by
    hand, so they are always listed; other commits only fill an empty
    [Unreleased]. CI-only bumps (`ci(deps)`) don't reach users and are
    left out.
    """
    subjects = git("log", "--format=%s", f"{last_tag}..HEAD").splitlines()
    deps, other = [], []
    for s in subjects:
        if s.startswith("ci(deps)") or s.startswith("chore: release"):
            continue
        (deps if re.match(r"^(chore|fix|build)\(deps\)", s) else other).append(f"- {s}")
    dep_section = "### Dependencies\n\n" + "\n".join(deps) if deps else ""
    other_section = "### Changed\n\n" + "\n".join(other) if other else ""
    return dep_section, other_section


def main():
    part = sys.argv[1] if len(sys.argv) > 1 else "patch"
    summary = (sys.argv[2] if len(sys.argv) > 2 else "").strip()

    last_tag = git("describe", "--tags", "--abbrev=0", "--match", "v*")
    if not git("log", "--format=%h", f"{last_tag}..HEAD"):
        sys.exit(f"Nothing to release: main has no commits since {last_tag}.")

    cargo = open("Cargo.toml", encoding="utf-8").read()
    m = re.search(r'^version = "(\d+\.\d+\.\d+)"', cargo, re.M)
    if not m:
        sys.exit("No version line in Cargo.toml")
    current = m.group(1)
    if f"v{current}" != last_tag:
        sys.exit(f"Cargo.toml says {current} but the last tag is {last_tag}; release by hand.")
    new = bump_version(current, part)
    cargo = cargo[: m.start(1)] + new + cargo[m.end(1):]

    log = open("CHANGELOG.md", encoding="utf-8").read()
    head = "## [Unreleased]\n"
    start = log.index(head) + len(head)
    end = log.index("\n## [", start)
    body = log[start:end].strip()
    dep_section, other_section = commit_notes(last_tag)
    if not body:
        body = other_section
    if dep_section and "### Dependencies" not in body:
        body = f"{body}\n\n{dep_section}".strip()
    if summary:
        body = f"{summary}\n\n{body}".strip()
    if not body:
        body = "Maintenance release."
    today = datetime.date.today().isoformat()
    log = f"{log[:start]}\n## [{new}] - {today}\n\n{body}\n{log[end:]}"

    link = re.search(r"^\[\d+\.\d+\.\d+\]: ", log, re.M)
    new_link = f"[{new}]: {REPO_URL}/releases/tag/v{new}\n"
    log = log[: link.start()] + new_link + log[link.start():] if link else log + new_link

    open("Cargo.toml", "w", encoding="utf-8").write(cargo)
    open("CHANGELOG.md", "w", encoding="utf-8").write(log)

    out = os.environ.get("GITHUB_OUTPUT")
    if out:
        with open(out, "a", encoding="utf-8") as f:
            f.write(f"version={new}\nprevious={current}\n")
    print(f"{current} -> {new}")


if __name__ == "__main__":
    main()
