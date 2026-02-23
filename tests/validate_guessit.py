#!/usr/bin/env python3
"""Integration test runner: validate hunch against guessit's YAML test vectors.

Parses guessit's YAML test files, runs hunch CLI against each test case,
and reports pass/fail rates per property and per file.

Usage:
    python3 tests/validate_guessit.py
"""

import json
import subprocess
import sys
from pathlib import Path

import yaml

HUNCH_BIN = Path(__file__).parent.parent / "target" / "release" / "hunch"
GUESSIT_TEST_DIR = Path(__file__).parent.parent.parent / "guessit" / "guessit" / "test"

# Properties hunch knows about (maps guessit names -> hunch JSON keys).
PROPERTY_MAP = {
    "title": "title",
    "year": "year",
    "season": "season",
    "episode": "episode",
    "video_codec": "video_codec",
    "audio_codec": "audio_codec",
    "audio_channels": "audio_channels",
    "screen_size": "screen_size",
    "source": "source",
    "container": "container",
    "release_group": "release_group",
    "edition": "edition",
    "type": "type",
    # Properties we map from guessit's "other" list.
    "other": "other",
}

# Properties hunch does NOT implement yet.
UNIMPLEMENTED = {
    "language", "subtitle_language", "country", "date",
    "episode_title", "streaming_service", "website",
    "video_profile", "audio_profile", "color_depth",
    "proper_count", "cd", "cd_count", "part",
    "film", "film_title", "bonus", "bonus_title",
    "episode_format", "episode_details", "disc", "week",
    "size", "aspect_ratio", "uuid", "crc32",
}


def parse_yaml_tests(filepath: Path) -> list[tuple[str, dict]]:
    """Parse a guessit YAML test file into (input, expected) pairs.

    guessit's format:
        ? input_string
        : key: value
          key2: value2

    Returns only positive test cases (skips `-` prefixed negatives
    and `__default__` entries).
    """
    with open(filepath) as f:
        raw = yaml.safe_load(f)

    if not isinstance(raw, dict):
        return []

    # Check for default type.
    default_type = None
    if "__default__" in raw:
        defaults = raw.pop("__default__")
        if isinstance(defaults, dict):
            default_type = defaults.get("type")

    tests = []
    for input_str, expected in raw.items():
        if input_str is None or not isinstance(expected, dict):
            continue
        # Skip negatives (start with -).
        if str(input_str).startswith("-"):
            continue
        # Skip entries with options (we don't support those yet).
        if "options" in expected:
            continue
        # Inject default type if not overridden.
        if default_type and "type" not in expected:
            expected["type"] = default_type
        tests.append((str(input_str), expected))
    return tests


def run_hunch(input_str: str) -> dict:
    """Run hunch CLI and return parsed JSON output."""
    try:
        result = subprocess.run(
            [str(HUNCH_BIN), input_str],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode != 0:
            return {}
        return json.loads(result.stdout.strip())
    except (subprocess.TimeoutExpired, json.JSONDecodeError):
        return {}


def normalize_value(val):
    """Normalize a value for comparison."""
    if isinstance(val, list):
        return sorted(str(v) for v in val)
    return str(val)


def compare(expected: dict, actual: dict) -> tuple[dict, dict, dict]:
    """Compare expected vs actual, returning (passed, failed, skipped) per property."""
    passed = {}
    failed = {}
    skipped = {}

    for prop, exp_val in expected.items():
        # Skip negated assertions (guessit uses "-property" prefix).
        if prop.startswith("-"):
            continue

        if prop in UNIMPLEMENTED:
            skipped[prop] = exp_val
            continue

        hunch_key = PROPERTY_MAP.get(prop)
        if hunch_key is None:
            skipped[prop] = exp_val
            continue

        act_val = actual.get(hunch_key)

        if act_val is None:
            failed[prop] = {"expected": exp_val, "actual": None}
            continue

        # Normalize for comparison.
        exp_norm = normalize_value(exp_val)
        act_norm = normalize_value(act_val)

        if exp_norm == act_norm:
            passed[prop] = exp_val
        else:
            failed[prop] = {"expected": exp_val, "actual": act_val}

    return passed, failed, skipped


def main():
    if not HUNCH_BIN.exists():
        print(f"ERROR: hunch binary not found at {HUNCH_BIN}")
        print("Run: cargo build --release")
        sys.exit(1)

    if not GUESSIT_TEST_DIR.exists():
        print(f"ERROR: guessit test dir not found at {GUESSIT_TEST_DIR}")
        sys.exit(1)

    # Test files to validate against.
    test_files = [
        "movies.yml",
        "episodes.yml",
        "various.yml",
        "rules/video_codec.yml",
        "rules/audio_codec.yml",
        "rules/screen_size.yml",
        "rules/source.yml",
        "rules/edition.yml",
        "rules/other.yml",
        "rules/release_group.yml",
        "rules/title.yml",
        "rules/episodes.yml",
    ]

    total_passed = 0
    total_failed = 0
    total_skipped = 0
    total_tests = 0
    prop_stats: dict[str, dict[str, int]] = {}
    failure_examples: list[tuple[str, str, dict]] = []  # (file, input, failures)

    for tf in test_files:
        filepath = GUESSIT_TEST_DIR / tf
        if not filepath.exists():
            print(f"  SKIP (not found): {tf}")
            continue

        tests = parse_yaml_tests(filepath)
        file_passed = 0
        file_failed = 0
        file_skipped = 0

        for input_str, expected in tests:
            total_tests += 1
            actual = run_hunch(input_str)
            passed, failed, skipped = compare(expected, actual)

            for prop in passed:
                prop_stats.setdefault(prop, {"passed": 0, "failed": 0})
                prop_stats[prop]["passed"] += 1
            for prop in failed:
                prop_stats.setdefault(prop, {"passed": 0, "failed": 0})
                prop_stats[prop]["failed"] += 1

            if failed:
                file_failed += 1
                total_failed += 1
                if len(failure_examples) < 30:
                    failure_examples.append((tf, input_str, failed))
            elif skipped and not passed:
                file_skipped += 1
                total_skipped += 1
            else:
                file_passed += 1
                total_passed += 1

        pct = (file_passed / max(len(tests), 1)) * 100
        print(f"  {tf}: {file_passed}/{len(tests)} passed ({pct:.0f}%) "
              f"[{file_failed} failed, {file_skipped} skipped]")

    # Summary.
    print("\n" + "=" * 70)
    print("OVERALL RESULTS")
    print("=" * 70)
    print(f"  Total test cases:    {total_tests}")
    print(f"  Passed (all props):  {total_passed}")
    print(f"  Failed (any prop):   {total_failed}")
    print(f"  Skipped (unimpl.):   {total_skipped}")
    if total_tests > 0:
        rate = (total_passed / total_tests) * 100
        print(f"  Pass rate:           {rate:.1f}%")

    # Per-property breakdown.
    print("\nPER-PROPERTY ACCURACY:")
    print(f"  {'Property':<20} {'Passed':>8} {'Failed':>8} {'Rate':>8}")
    print(f"  {'-'*20} {'-'*8} {'-'*8} {'-'*8}")
    for prop in sorted(prop_stats.keys()):
        s = prop_stats[prop]
        total = s["passed"] + s["failed"]
        rate = (s["passed"] / total * 100) if total > 0 else 0
        emoji = "✅" if rate >= 80 else "⚠️ " if rate >= 50 else "❌"
        print(f"  {emoji} {prop:<18} {s['passed']:>8} {s['failed']:>8} {rate:>7.1f}%")

    # Failure examples.
    if failure_examples:
        print("\nSAMPLE FAILURES (first 30):")
        for file, input_str, fails in failure_examples:
            short_input = input_str[:70] + ("..." if len(input_str) > 70 else "")
            print(f"\n  [{file}] {short_input}")
            for prop, detail in fails.items():
                print(f"    {prop}: expected={detail['expected']!r}, got={detail['actual']!r}")

    # Unimplemented properties summary.
    print("\nUNIMPLEMENTED PROPERTIES (skipped during validation):")
    for prop in sorted(UNIMPLEMENTED):
        print(f"  - {prop}")


if __name__ == "__main__":
    main()
