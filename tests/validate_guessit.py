#!/usr/bin/env python3
"""Compatibility report generator: validate hunch against guessit's YAML test vectors.

Parses YAML test files from tests/fixtures/ (copied from guessit),
runs hunch CLI against each test case, and reports pass/fail rates
per property and per file. No external guessit repo needed.

Usage:
    cargo build --release
    uv run --with pyyaml python3 tests/validate_guessit.py
"""

import json
import subprocess
import sys
from pathlib import Path

import yaml

HUNCH_BIN = Path(__file__).parent.parent / "target" / "release" / "hunch"
FIXTURES_DIR = Path(__file__).parent / "fixtures"

# Properties hunch knows about (maps guessit names -> hunch JSON keys).
PROPERTY_MAP = {
    "title": "title",
    "year": "year",
    "season": "season",
    "episode": "episode",
    "episode_title": "episode_title",
    "video_codec": "video_codec",
    "video_profile": "video_profile",
    "audio_codec": "audio_codec",
    "audio_profile": "audio_profile",
    "audio_channels": "audio_channels",
    "screen_size": "screen_size",
    "source": "source",
    "container": "container",
    "release_group": "release_group",
    "edition": "edition",
    "type": "type",
    "proper_count": "proper_count",
    "streaming_service": "streaming_service",
    "color_depth": "color_depth",
    "subtitle_language": "subtitle_language",
    "country": "country",
    "date": "date",
    "crc32": "crc32",
    "website": "website",
    "episode_details": "episode_details",
    "part": "part",
    "disc": "disc",
    "cd": "cd",
    "cd_count": "cd_count",
    "film": "film",
    "film_title": "film_title",
    "bonus": "bonus",
    "bonus_title": "bonus_title",
    "size": "size",
    "uuid": "uuid",
    "aspect_ratio": "aspect_ratio",
    "week": "week",
    "episode_format": "episode_format",
    # Properties we map from guessit's "other" list.
    "other": "other",
}

# Properties hunch does NOT implement yet.
UNIMPLEMENTED = set()


def parse_yaml_tests(filepath: Path) -> list[tuple[str, dict]]:
    """Parse a guessit YAML test file into (input, expected) pairs.

    guessit's format supports chained keys:
        ? +input1
        ? +input2
        ? -negative_input
        : key: value
          key2: value2

    Multiple `?` keys can share one `:` value block.
    PyYAML collapses these, so we parse line-by-line for rules files,
    and use simple YAML for main test files.
    """
    text = filepath.read_text()

    # Detect if this is a chained-key file (rules files use `+`/`-` prefixes).
    if any(line.strip().startswith("? +") or line.strip().startswith("? -") for line in text.splitlines()[:20]):
        return _parse_chained_yaml(filepath)
    return _parse_simple_yaml(filepath)


def _parse_chained_yaml(filepath: Path) -> list[tuple[str, dict]]:
    """Parse YAML files with chained `? +key` / `? -key` patterns."""
    text = filepath.read_text()
    lines = text.splitlines()

    # Collect groups: list of (keys, yaml_block)
    tests = []
    current_keys: list[str] = []
    i = 0

    # Extract default type.
    raw = yaml.safe_load(text)
    default_type = None
    if isinstance(raw, dict) and "__default__" in raw:
        defaults = raw["__default__"]
        if isinstance(defaults, dict):
            default_type = defaults.get("type")

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped.startswith("? "):
            key = stripped[2:].strip()
            current_keys.append(key)
            i += 1
        elif stripped.startswith(": ") and current_keys:
            # Collect the YAML mapping block.
            block_lines = [stripped[2:]]  # First line after ":"
            i += 1
            while i < len(lines):
                nline = lines[i]
                if nline.startswith("?") or nline.strip() == "" or nline.startswith("#"):
                    break
                # Continuation of mapping (indented).
                block_lines.append(nline.strip())
                i += 1

            # Parse the block as YAML.
            block_yaml = "\n".join(block_lines)
            try:
                expected = yaml.safe_load(block_yaml)
            except Exception:
                current_keys = []
                continue

            if not isinstance(expected, dict):
                current_keys = []
                continue

            # Filter out `-` prefixed properties.
            clean_expected = {}
            for k, v in expected.items():
                if not str(k).startswith("-"):
                    clean_expected[k] = v

            # Skip entries with options.
            if "options" in clean_expected:
                current_keys = []
                continue

            # Inject default type.
            if default_type and "type" not in clean_expected:
                clean_expected["type"] = default_type

            # Emit test cases for each positive key.
            for key in current_keys:
                if key.startswith("-"):
                    continue
                clean_key = key.lstrip("+")
                tests.append((clean_key, dict(clean_expected)))

            current_keys = []
        else:
            i += 1
            current_keys = []

    return tests


def _parse_simple_yaml(filepath: Path) -> list[tuple[str, dict]]:
    """Parse standard guessit YAML test files."""
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
        # Strip positive prefix (+).
        clean_input = str(input_str)
        if clean_input.startswith("+"):
            clean_input = clean_input[1:]
        # Skip entries with options (we don't support those yet).
        if "options" in expected:
            continue
        # Inject default type if not overridden.
        if default_type and "type" not in expected:
            expected["type"] = default_type
        tests.append((clean_input, expected))
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
        return sorted(normalize_single(str(v)) for v in val)
    return normalize_single(str(val))


# Language normalization map: guessit may use full names, ISO 2-letter,
# or ISO 3-letter codes interchangeably. Normalize to a canonical form.
LANG_NORMALIZE = {
    "en": "en", "eng": "en", "english": "en",
    "fr": "fr", "fre": "fr", "fra": "fr", "french": "fr",
    "es": "es", "spa": "es", "spanish": "es",
    "de": "de", "ger": "de", "deu": "de", "german": "de",
    "it": "it", "ita": "it", "italian": "it",
    "pt": "pt", "por": "pt", "portuguese": "pt",
    "pt-br": "pt-br",
    "ja": "ja", "jpn": "ja", "japanese": "ja",
    "ko": "ko", "kor": "ko", "korean": "ko",
    "zh": "zh", "chi": "zh", "zho": "zh", "chinese": "zh",
    "ru": "ru", "rus": "ru", "russian": "ru",
    "ar": "ar", "ara": "ar", "arabic": "ar",
    "hi": "hi", "hin": "hi", "hindi": "hi",
    "nl": "nl", "dut": "nl", "nld": "nl", "dutch": "nl",
    "pl": "pl", "pol": "pl", "polish": "pl",
    "sv": "sv", "swe": "sv", "swedish": "sv",
    "no": "no", "nor": "no", "norwegian": "no",
    "da": "da", "dan": "da", "danish": "da",
    "fi": "fi", "fin": "fi", "finnish": "fi",
    "hu": "hu", "hun": "hu", "hungarian": "hu",
    "cs": "cs", "cze": "cs", "ces": "cs", "czech": "cs",
    "ro": "ro", "rum": "ro", "ron": "ro", "romanian": "ro",
    "el": "el", "gre": "el", "ell": "el", "greek": "el",
    "tr": "tr", "tur": "tr", "turkish": "tr",
    "he": "he", "heb": "he", "hebrew": "he",
    "uk": "uk", "ukr": "uk", "ukrainian": "uk",
    "bg": "bg", "bul": "bg", "bulgarian": "bg",
    "hr": "hr", "hrv": "hr", "croatian": "hr",
    "sr": "sr", "srp": "sr", "serbian": "sr",
    "sk": "sk", "slo": "sk", "slk": "sk", "slovak": "sk",
    "sl": "sl", "slv": "sl", "slovenian": "sl",
    "et": "et", "est": "et", "estonian": "et",
    "lv": "lv", "lav": "lv", "latvian": "lv",
    "lt": "lt", "lit": "lt", "lithuanian": "lt",
    "ca": "ca", "cat": "ca", "catalan": "ca",
    "mul": "mul", "multiple languages": "mul",
    "und": "und", "undetermined": "und",
}

# Properties that contain language values needing normalization.
LANG_PROPS = {"language", "subtitle_language", "country"}


def normalize_single(val):
    """Normalize a single string value."""
    return val


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

        # Apply language normalization for language properties.
        if prop in LANG_PROPS:
            if isinstance(exp_norm, list):
                exp_norm = sorted(LANG_NORMALIZE.get(v.lower(), v) for v in exp_norm)
                act_norm = sorted(LANG_NORMALIZE.get(v.lower(), v) for v in act_norm) if isinstance(act_norm, list) else [LANG_NORMALIZE.get(act_norm.lower(), act_norm)]
            else:
                exp_norm = LANG_NORMALIZE.get(exp_norm.lower(), exp_norm)
                act_norm = LANG_NORMALIZE.get(act_norm.lower(), act_norm) if isinstance(act_norm, str) else act_norm

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

    if not FIXTURES_DIR.exists():
        print(f"ERROR: fixtures dir not found at {FIXTURES_DIR}")
        print("Expected test fixtures in tests/fixtures/")
        sys.exit(1)

    # Test files to validate against (all bundled fixtures).
    test_files = [
        "movies.yml",
        "episodes.yml",
        "various.yml",
        "rules/audio_codec.yml",
        "rules/bonus.yml",
        "rules/cd.yml",
        "rules/common_words.yml",
        "rules/country.yml",
        "rules/date.yml",
        "rules/edition.yml",
        "rules/episodes.yml",
        "rules/film.yml",
        "rules/language.yml",
        "rules/other.yml",
        "rules/part.yml",
        "rules/release_group.yml",
        "rules/screen_size.yml",
        "rules/size.yml",
        "rules/source.yml",
        "rules/title.yml",
        "rules/video_codec.yml",
        "rules/website.yml",
    ]

    total_passed = 0
    total_failed = 0
    total_skipped = 0
    total_tests = 0
    prop_stats: dict[str, dict[str, int]] = {}
    failure_examples: list[tuple[str, str, dict]] = []  # (file, input, failures)

    for tf in test_files:
        filepath = FIXTURES_DIR / tf
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
