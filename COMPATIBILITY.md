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
| Passed (all props correct) | 806 |
| Failed (any prop wrong) | 503 |
| **Pass rate** | **61.6%** |
| Properties implemented | 46 / 49 |
| Properties skipped | 0 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 61.6% of those results identically.

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
| rules/edition.yml | 44 | 44 | **100%** |
| rules/other.yml | 44 | 46 | **96%** |
| rules/common_words.yml | 146 | 156 | **94%** |
| rules/video_codec.yml | 41 | 45 | **91%** |
| rules/audio_codec.yml | 15 | 17 | **88%** |
| rules/release_group.yml | 14 | 19 | 74% |
| rules/bonus.yml | 2 | 3 | 67% |
| rules/date.yml | 5 | 8 | 63% |
| rules/source.yml | 13 | 23 | 57% |
| rules/part.yml | 5 | 9 | 56% |
| rules/episodes.yml | 41 | 79 | 52% |
| movies.yml | 110 | 199 | 55% |
| episodes.yml | 246 | 488 | 50% |
| rules/cd.yml | 1 | 2 | 50% |
| rules/website.yml | 1 | 2 | 50% |
| rules/title.yml | 8 | 18 | 44% |
| various.yml | 54 | 124 | 44% |
| rules/country.yml | 1 | 3 | 33% |
| rules/language.yml | 3 | 9 | 33% |
| rules/film.yml | 0 | 3 | 0% |

---

## Pass Rate by Property

Each row shows how often hunch produces the correct value for that
property, across all test cases that assert it. guessit scores 100%
on all of these by definition.

### ✅ Perfect (100%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| aspect_ratio | 2 | 0 | **100.0%** |
| bonus | 13 | 0 | **100.0%** |
| color_depth | 28 | 0 | **100.0%** |
| edition | 83 | 0 | **100.0%** |
| episode_count | 6 | 0 | **100.0%** |
| episode_details | 16 | 0 | **100.0%** |
| film | 8 | 0 | **100.0%** |
| frame_rate | 7 | 0 | **100.0%** |
| season_count | 2 | 0 | **100.0%** |
| size | 9 | 0 | **100.0%** |
| streaming_service | 31 | 0 | **100.0%** |
| version | 13 | 0 | **100.0%** |

### ✅ Excellent (90%+)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| video_codec | 501 | 3 | 99.4% |
| screen_size | 422 | 6 | 98.6% |
| container | 146 | 5 | 96.7% |
| crc32 | 24 | 1 | 96.0% |
| source | 536 | 24 | 95.7% |
| year | 219 | 11 | 95.2% |
| audio_codec | 213 | 13 | 94.2% |
| proper_count | 29 | 2 | 93.5% |
| type | 762 | 60 | 92.7% |
| season | 432 | 42 | 91.1% |
| website | 20 | 2 | 90.9% |
| audio_channels | 107 | 11 | 90.7% |

### 🟡 Good (70–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| date | 23 | 3 | 88.5% |
| uuid | 7 | 1 | 87.5% |
| release_group | 458 | 81 | 85.0% |
| title | 867 | 189 | 82.1% |
| subtitle_language | 65 | 16 | 80.2% |
| episode | 444 | 111 | 80.0% |
| other | 270 | 79 | 77.4% |
| country | 10 | 3 | 76.9% |
| audio_profile | 26 | 8 | 76.5% |
| language | 100 | 42 | 70.4% |

### ⚠️ Developing (50–70%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| part | 12 | 7 | 63.2% |
| bonus_title | 8 | 5 | 61.5% |
| episode_title | 123 | 78 | 61.2% |
| cd | 3 | 2 | 60.0% |
| video_profile | 8 | 6 | 57.1% |
| disc | 3 | 3 | 50.0% |
| cd_count | 2 | 2 | 50.0% |

### ❌ Not Yet Implemented (<50%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| alternative_title | 0 | 16 | 0.0% |
| absolute_episode | 0 | 10 | 0.0% |
| film_title | 0 | 8 | 0.0% |
| audio_bit_rate | 0 | 4 | 0.0% |
| video_bit_rate | 0 | 4 | 0.0% |
| video_api | 0 | 3 | 0.0% |
| mimetype | 0 | 3 | 0.0% |
| episode_format | 0 | 2 | 0.0% |
| week | 0 | 1 | 0.0% |

---

## Guessit Fixture Audit

> Audit date: 2026-02-24. All 23 YAML fixture files reviewed
> (1,309 test cases, 6,925 expected properties).

### Duplicate Test Cases with Conflicting Expectations

These are upstream fixture bugs where the **same filename** appears
multiple times with **different** expected values.

| Filename | Lines | Conflict |
|----------|-------|----------|
| `FooBar.7.PDTV-FlexGet` | episodes.yml L856 vs L889 | L856: episode=7, type=episode (from `__default__`). L889: title="FooBar 7", type=movie. Second is auto-detect behavior. |
| `A Bout Portant (The Killers).PAL.Multi.DVD-R-KZ` | movies.yml L761 vs L773 | L761 omits `other: PAL`. L773 includes it. Second is corrected. |
| `Breaking.Bad.S01E01...WMV-NOVO` | episodes.yml L1729 vs L1828 | `container: WMV` vs `container: wmv` (casing difference). |

### Redundant Duplicates (Same Expectations)

These filenames appear 2-3 times with identical expected values.
They don't cause test failures but add unnecessary noise.

- `FooBar.07.PDTV-FlexGet` (2x), `FooBar.0307.PDTV-FlexGet` (2x)
- `Test.13.HDTV-Ignored` (3x)
- `Elementary.S01E01.Pilot.DVDSCR.x264.PREAiR-NoGRP` (2x, different YAML formatting)
- `Hyena.Road.2015...`, `Maze.Runner...OM...`, `How To Steal A Dog...` (2x each)
- `Cosmos...PROPER-LOL`, `Show Name 3x18...`, `Show.Name.S05...Belex` (2x each)
- `Greys.Anatomy.S07D1...`, `FlexGet.S01E02.TheName...` (2x each)
- Several in `rules/episodes.yml` (2x each)

### `__default__` Type Context

Guessit's fixture files use `__default__` blocks:
- `movies.yml` → `type: movie`
- `episodes.yml` → `type: episode`

Some cases in `episodes.yml` override `type: movie`, meaning they test
auto-detection behavior. Since hunch doesn't support a `-t` flag, we
follow auto-detection in all cases. Cases with explicit `options:` blocks
(e.g., `-t episode`, `advanced_config`) are correctly **skipped** by
our test harness.

---

## Intentional Design Differences

These are places where hunch **deliberately** diverges from guessit.

### Year-as-Season (`S2013E14`, `1940x01`)

**Guessit**: Supports 4-digit seasons. `S2013E14` → season=2013, episode=14.
`1940x01` → season=1940, episode=1, year=1940.

**Hunch**: Limits SxxExx seasons to 3 digits (`\d{1,3}`) and NNxNN seasons
to 2 digits (`\d{1,2}`). Year-like numbers (1920–2029) are parsed as years.

**Rationale**: Year-based seasons are rare and create dangerous ambiguity.
For general-purpose use, treating 2013 as a year is safer than as a season.

**Affected test cases** (~10):
`Looney Tunes 1940x01...`, `Eyes.Of.Dawn.1991.E01...`,
`FlexGet.US.S2013E14...`, `Panorama.S2013E25...`,
`Pawn.Stars.S2014E18...`, `Our.World.S2014E11...`,
`Storyville.S2016E08...`, `MotoGP.2016x03...`,
`FlexGet.Series.2013.14.of.21...`, `Show.Name.E02.S2010.mkv`

### Screen Size Normalization

**Guessit**: Preserves raw resolution strings like `1444x866`.

**Hunch**: Normalizes non-standard resolutions to the vertical component
(e.g., `1444x866` → `866p`). Standard resolutions (`1920x1080`,
`3840x2160`) map to their common names (`1080p`, `2160p`).

### `!!map {}` (Empty Expected)

**Guessit**: Uses `!!map {}` in `date.yml` for inputs `1919` and `2030`
to mean "no properties should be detected."

**Hunch**: May still produce `year: 2030` for input `2030`. Our test
harness passes these trivially (empty expected = nothing to verify).
We accept this as a known, minor difference.

### Release Group: Compound Groups

**Guessit**: Can merge separate parts into compound release groups
(e.g., `Tigole QxR` from `...Tigole) [QxR]`).

**Hunch**: Takes the last hyphen-delimited or bracket-enclosed group.

---

## Highest-ROI Improvements

267 test cases currently fail on **exactly 1 property**. Fixing that single
property would flip them from fail to pass. These are the highest-leverage
targets:

| Property | Single-prop fails | Impact |
|---|---|---|
| title | 59 | +4.5pp |
| episode | 39 | +3.0pp |
| release_group | 38 | +2.9pp |
| other | 28 | +2.1pp |
| episode_title | 22 | +1.7pp |
| language | 18 | +1.4pp |
| season | 13 | +1.0pp |
| source | 6 | +0.5pp |

Fixing all 267 would bring the pass rate from 61.6% to ~82%.

---

## Known Gaps

These are the areas where hunch diverges most from guessit, with
explanations of why.

### Title extraction (82.1%)

The hardest problem. Title is "everything that's left" after all
technical tokens are claimed. guessit uses multi-pass rules and
title-specific heuristics that hunch hasn't fully replicated:

- Titles containing dots as acronyms (S.H.I.E.L.D., S.W.A.T.)
- Titles with "Final", "Game", "Web" eaten by other matchers
- Anime titles with brackets and group tags
- Film numbering conventions (f17, f21)

### Episode parsing (80.0%)

Most standard patterns work (S01E02, 1x03, E01-E03, S01-S10).
Remaining gaps:

- 3-digit anime episodes (One\_Piece\_679 → decomposed as S6E79)
- Compact SSEE format (0106 → S01E06)
- Spanish "Cap.102" chapter notation

### Episode title (61.2%)

Positional inference: the episode title is the unclaimed region between
the episode number and the first technical token. guessit applies
post-processing rules to clean up edge cases that hunch doesn't yet
handle.

### Multi-value subtitle languages

Patterns like `ST{Fr-Eng}` (both French and English subtitles) require
compound parsing that splits within brackets. Hunch handles `[ENG+RU+PT]`
but not curly-brace patterns yet.

### Niche properties at 0%

| Property | Fixture Count | Description |
|----------|--------------|-------------|
| `alternative_title` | ~16 | Shows with alternate names |
| `absolute_episode` | ~10 | Anime absolute numbering |
| `film_title` | ~8 | Film numbering titles |
| `audio_bit_rate` | ~4 | Audio bitrate |
| `video_bit_rate` | ~4 | Video bitrate |
| `video_api` | ~3 | DXVA, etc. |
| `mimetype` | ~3 | File MIME type |
| `episode_format` | ~2 | Minisode format |
| `week` | ~1 | Week-based episode numbering |

### Partial Coverage Properties

| Property | Known Gaps |
|----------|------------|
| `language` | Codes like `de-CH`, `spa` (3-letter), `mul`/`Multiple languages` |
| `other` | Niche values: `2in1`, `Line Audio`, `Mux`, `OAR`, `XXX` |
| `edition` | `Festival`, `Uncensored` not yet recognized |

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
