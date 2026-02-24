# Hunch vs guessit — Compatibility Report

Hunch is a **Rust port** of Python's [guessit](https://github.com/guessit-io/guessit).
This document tracks how closely hunch reproduces guessit's behavior, measured
by running hunch against guessit's own test suite (1,309 test cases across 22
YAML files).

> **Last updated:** 2026-02-23

---

## Overall Results

| Metric | Value |
|---|---|
| Total test cases | 1,309 |
| Passed (all props correct) | 702 |
| Failed (any prop wrong) | 607 |
| **Pass rate** | **53.6%** |
| Properties implemented | 39 / 39 |
| Properties skipped | 0 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 53.6% of those results identically.

---

## Pass Rate by Test File

guessit's tests are split across general-purpose files and per-property
rule files. Hunch performs strongest on isolated property tests and
weaker on full-filename tests that require many properties to be correct
simultaneously.

| Test file | Passed | Total | Rate |
|---|---|---|---|
| rules/screen_size.yml | 9 | 9 | **100%** |
| rules/size.yml | 3 | 3 | **100%** |
| rules/other.yml | 44 | 46 | **96%** |
| rules/common_words.yml | 146 | 156 | **94%** |
| rules/audio_codec.yml | 15 | 17 | **88%** |
| rules/video_codec.yml | 39 | 45 | **87%** |
| rules/edition.yml | 36 | 44 | **82%** |
| rules/release_group.yml | 14 | 19 | 74% |
| rules/bonus.yml | 2 | 3 | 67% |
| rules/date.yml | 5 | 8 | 63% |
| rules/source.yml | 14 | 23 | 61% |
| rules/part.yml | 5 | 9 | 56% |
| rules/cd.yml | 1 | 2 | 50% |
| rules/website.yml | 1 | 2 | 50% |
| rules/episodes.yml | 36 | 79 | 46% |
| rules/title.yml | 8 | 18 | 44% |
| episodes.yml | 200 | 488 | 41% |
| movies.yml | 74 | 199 | 37% |
| various.yml | 46 | 124 | 37% |
| rules/country.yml | 1 | 3 | 33% |
| rules/language.yml | 3 | 9 | 33% |
| rules/film.yml | 0 | 3 | 0% |

---

## Pass Rate by Property

Each row shows how often hunch produces the correct value for that
property, across all test cases that assert it. guessit scores 100%
on all of these by definition.

### ✅ Excellent (90%+)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| aspect_ratio | 2 | 0 | **100.0%** |
| bonus | 13 | 0 | **100.0%** |
| color_depth | 28 | 0 | **100.0%** |
| film | 8 | 0 | **100.0%** |
| size | 9 | 0 | **100.0%** |
| streaming_service | 31 | 0 | **100.0%** |
| year | 230 | 0 | **100.0%** |
| video_codec | 498 | 6 | 98.8% |
| screen_size | 420 | 8 | 98.1% |
| container | 146 | 5 | 96.7% |
| crc32 | 24 | 1 | 96.0% |
| source | 532 | 28 | 95.0% |
| audio_codec | 208 | 18 | 92.0% |
| type | 753 | 69 | 91.6% |
| website | 20 | 2 | 90.9% |
| audio_channels | 107 | 11 | 90.7% |
| season | 428 | 46 | 90.3% |

### ⚠️ Good (70–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| date | 23 | 3 | 88.5% |
| uuid | 7 | 1 | 87.5% |
| episode_details | 14 | 2 | 87.5% |
| release_group | 449 | 90 | 83.3% |
| audio_profile | 28 | 6 | 82.4% |
| title | 862 | 194 | 81.6% |
| edition | 67 | 16 | 80.7% |
| subtitle_language | 65 | 16 | 80.2% |
| country | 10 | 3 | 76.9% |
| episode | 424 | 131 | 76.4% |
| proper_count | 23 | 8 | 74.2% |
| other | 248 | 101 | 71.1% |

### ⚠️ Developing (50–70%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| part | 12 | 7 | 63.2% |
| language | 89 | 53 | 62.7% |
| episode_title | 124 | 77 | 61.7% |
| bonus_title | 8 | 5 | 61.5% |
| cd | 3 | 2 | 60.0% |
| video_profile | 8 | 6 | 57.1% |
| disc | 3 | 3 | 50.0% |
| cd_count | 2 | 2 | 50.0% |

### ❌ Weak (<50%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| alternative_title | 2 | 14 | 12.5% |
| version | 0 | 13 | 0.0% |
| absolute_episode | 0 | 10 | 0.0% |
| film_title | 0 | 8 | 0.0% |
| frame_rate | 0 | 7 | 0.0% |
| episode_count | 0 | 6 | 0.0% |
| audio_bit_rate | 0 | 4 | 0.0% |
| video_bit_rate | 0 | 4 | 0.0% |
| video_api | 0 | 3 | 0.0% |
| mimetype | 0 | 3 | 0.0% |
| episode_format | 0 | 2 | 0.0% |
| season_count | 0 | 2 | 0.0% |
| week | 0 | 1 | 0.0% |

---

## Known Gaps

These are the areas where hunch diverges most from guessit, with
explanations of why.

### Title extraction (81.6%)

The hardest problem. Title is "everything that's left" after all
technical tokens are claimed. guessit uses multi-pass rules and
title-specific heuristics that hunch hasn't fully replicated:

- Parent-directory fallback (path-based inputs)
- Titles containing year-like numbers ("2001: A Space Odyssey")
- Titles with colons, hyphens, or dots that look like separators
- Anime titles with brackets and group tags

### Episode title (61.7%)

Positional inference: the episode title is the unclaimed region between
the episode number and the first technical token. guessit applies
post-processing rules to clean up edge cases that hunch doesn't yet
handle.

### Multi-value subtitle languages

Patterns like `ST{Fr-Eng}` (both French and English subtitles) require
compound parsing that splits within brackets. Hunch currently extracts
only the first language in these cases.

### Niche properties

`alternative_title`, `version`, `absolute_episode`, `frame_rate`,
`film_title`, `episode_count`, `audio_bit_rate`, `video_bit_rate`,
`video_api`, `mimetype`, `episode_format`, `season_count`, and `week`
are rare properties at 0% or near-0%. They aren't yet implemented
because the patterns are unusual and low-priority.

---

## How This Is Measured

The regression test suite (`tests/guessit_regression.rs`) does the following:

1. Loads guessit's YAML test vectors from `tests/fixtures/` (bundled, self-contained).
2. Runs `hunch()` as a library against each filename.
3. Compares every expected property value against hunch's output.
4. A test "passes" only if **all** asserted properties match exactly.
5. Language values are normalized (ISO 2-letter, 3-letter, and full
   names are treated as equivalent, e.g. `fr` = `fre` = `French`).

To run the full compatibility report:

```bash
cargo test compatibility_report -- --ignored --nocapture
```

To run the regression guards (CI):

```bash
cargo test --test guessit_regression
```
