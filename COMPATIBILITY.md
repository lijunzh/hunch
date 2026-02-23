# Hunch vs guessit — Compatibility Report

Hunch is a **Rust port** of Python's [guessit](https://github.com/guessit-io/guessit).
This document tracks how closely hunch reproduces guessit's behavior, measured
by running hunch against guessit's own test suite (1,330 test cases across 12
YAML files).

> **Last updated:** 2026-02-22

---

## Overall Results

| Metric | Value |
|---|---|
| Total test cases | 1,330 |
| Passed (all props correct) | 778 |
| Failed (any prop wrong) | 552 |
| **Pass rate** | **58.5%** |
| Properties implemented | 39 / 39 |
| Properties skipped | 0 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 58.5% of those results identically.

---

## Pass Rate by Test File

guessit's tests are split across general-purpose files and per-property
rule files. Hunch performs strongest on isolated property tests and
weaker on full-filename tests that require many properties to be correct
simultaneously.

| Test file | Passed | Total | Rate |
|---|---|---|---|
| rules/edition.yml | 43 | 44 | **98%** |
| rules/other.yml | 67 | 70 | **96%** |
| rules/audio_codec.yml | 61 | 65 | **94%** |
| rules/video_codec.yml | 39 | 45 | **87%** |
| rules/source.yml | 97 | 128 | **76%** |
| rules/screen_size.yml | 58 | 82 | **71%** |
| rules/release_group.yml | 10 | 15 | 67% |
| rules/episodes.yml | 47 | 92 | 51% |
| movies.yml | 95 | 194 | 49% |
| episodes.yml | 211 | 455 | 46% |
| various.yml | 46 | 122 | 38% |
| rules/title.yml | 4 | 18 | 22% |

---

## Pass Rate by Property

Each row shows how often hunch produces the correct value for that
property, across all test cases that assert it. guessit scores 100%
on all of these by definition.

### ✅ Excellent (90%+)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| bonus | 10 | 0 | **100.0%** |
| color_depth | 27 | 0 | **100.0%** |
| episode_details | 15 | 0 | **100.0%** |
| film | 5 | 0 | **100.0%** |
| streaming_service | 30 | 0 | **100.0%** |
| container | 144 | 2 | 98.6% |
| video_codec | 482 | 7 | 98.6% |
| aspect_ratio | 40 | 1 | 97.6% |
| year | 214 | 7 | 96.8% |
| edition | 80 | 3 | 96.4% |
| crc32 | 24 | 1 | 96.0% |
| website | 20 | 1 | 95.2% |
| source | 598 | 36 | 94.3% |
| audio_codec | 244 | 15 | 94.2% |
| screen_size | 453 | 34 | 93.0% |
| audio_channels | 121 | 10 | 92.4% |
| date | 23 | 2 | 92.0% |
| type | 714 | 67 | 91.4% |
| country | 9 | 1 | 90.0% |

### ⚠️ Good (70–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| season | 405 | 52 | 88.6% |
| uuid | 7 | 1 | 87.5% |
| release_group | 421 | 88 | 82.7% |
| subtitle_language | 57 | 12 | 82.6% |
| episode | 430 | 100 | 81.1% |
| title | 666 | 184 | 78.4% |
| audio_profile | 31 | 9 | 77.5% |
| proper_count | 24 | 8 | 75.0% |
| other | 306 | 109 | 73.7% |

### ⚠️ Developing (50–70%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| cd | 2 | 1 | 66.7% |
| cd_count | 2 | 1 | 66.7% |
| part | 6 | 3 | 66.7% |
| size | 4 | 2 | 66.7% |
| video_profile | 9 | 5 | 64.3% |
| episode_title | 121 | 74 | 62.1% |
| bonus_title | 6 | 4 | 60.0% |

### ❌ Weak (<50%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| disc | 2 | 3 | 40.0% |
| episode_format | 0 | 2 | 0.0% |
| film_title | 0 | 5 | 0.0% |
| week | 0 | 1 | 0.0% |

---

## Known Gaps

These are the areas where hunch diverges most from guessit, with
explanations of why.

### Title extraction (78.4%)

The hardest problem. Title is "everything that's left" after all
technical tokens are claimed. guessit uses multi-pass rules and
title-specific heuristics that hunch hasn't fully replicated:

- Parent-directory fallback (path-based inputs)
- Titles containing year-like numbers ("2001: A Space Odyssey")
- Titles with colons, hyphens, or dots that look like separators
- Anime titles with brackets and group tags

### Episode title (62.1%)

Positional inference: the episode title is the unclaimed region between
the episode number and the first technical token. guessit applies
post-processing rules to clean up edge cases that hunch doesn't yet
handle.

### Multi-value subtitle languages

Patterns like `ST{Fr-Eng}` (both French and English subtitles) require
compound parsing that splits within brackets. Hunch currently extracts
only the first language in these cases.

### Niche properties

`episode_format`, `film_title`, and `week` are rare properties with
very few test cases. They aren't yet implemented because the patterns
are unusual and low-priority.

---

## How This Is Measured

The validation script (`tests/validate_guessit.py`) does the following:

1. Loads guessit's YAML test vectors from `../guessit/guessit/test/`.
2. Runs `hunch` (CLI) against each filename.
3. Compares every expected property value against hunch's JSON output.
4. A test "passes" only if **all** asserted properties match exactly.
5. Language values are normalized (ISO 2-letter, 3-letter, and full
   names are treated as equivalent, e.g. `fr` = `fre` = `French`).

To run locally:

```bash
# Requires guessit repo at ../guessit
cargo build --release
python3 tests/validate_guessit.py
```
