#!/usr/bin/env python3
"""Copy guessit's test suites into tests/fixtures, keeping hunch's own cases.

    python3 .github/scripts/refresh-guessit-fixtures.py <guessit checkout> <tag>

Run from the hunch repo root. For each fixture file hunch carries
(`movies.yml`, `episodes.yml`, `various.yml`, `rules/*.yml`), the
upstream file replaces the local one, and everything from the local
"hunch-specific cases" marker to the end of the file is appended again.
`community.yml` is hunch's own and is not touched. ATTRIBUTION.md gets
the new tag.

Used by .github/workflows/guessit-drift.yml and by hand when refreshing.
"""

import pathlib
import re
import subprocess
import sys

MARKER = "# --- hunch-specific cases (not in guessit) ---"


def main() -> None:
    upstream = pathlib.Path(sys.argv[1]) / "guessit" / "test"
    tag = sys.argv[2]
    fixtures = pathlib.Path("tests/fixtures")

    local_files = [fixtures / n for n in ("movies.yml", "episodes.yml", "various.yml")]
    local_files += sorted((fixtures / "rules").glob("*.yml"))

    for local in local_files:
        source = upstream / local.relative_to(fixtures)
        if not source.exists():
            print(f"skip {local}: not in guessit {tag}")
            continue
        text = local.read_text(encoding="utf-8")
        trailer = text[text.index(MARKER):] if MARKER in text else ""
        new = source.read_text(encoding="utf-8")
        if trailer:
            new = new.rstrip("\n") + "\n\n" + trailer
        local.write_text(new, encoding="utf-8")

    attribution = fixtures / "ATTRIBUTION.md"
    commit = subprocess.run(
        ["git", "-C", sys.argv[1], "rev-parse", "--short", "HEAD"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    text = attribution.read_text(encoding="utf-8")
    date = subprocess.run(
        ["git", "-C", sys.argv[1], "log", "-1", "--format=%cs"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    text, count = re.subn(
        r"\*\*guessit v[\d.]+\*\* \(tag `v[\d.]+`, commit `[0-9a-f]+`, [\d-]+\)",
        f"**guessit {tag}** (tag `{tag}`, commit `{commit}`, {date})",
        text,
    )
    if count != 1:
        sys.exit("ATTRIBUTION.md: source line not found; update it by hand")
    attribution.write_text(text, encoding="utf-8")
    print(f"fixtures refreshed from guessit {tag} ({commit})")


if __name__ == "__main__":
    main()
