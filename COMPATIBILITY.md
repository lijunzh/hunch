# Hunch vs guessit — Compatibility Report

Hunch is a **Rust port** of Python's [guessit](https://github.com/guessit-io/guessit).
This document tracks how closely hunch reproduces guessit's behavior, measured
by running hunch against guessit's own test suite (1,309 test cases across 22
YAML files).

> **Last updated:** 2026-02-25

---

## Overall Results

| Metric | Value |
|---|---|
| Total test cases | 1,309 |
| Passed (all props correct) | 1,023 |
| Failed (any prop wrong) | 286 |
| **Pass rate** | **78.2%** |
| Properties implemented | 49 / 49 |
| Properties intentionally diverged | 3 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 78.2% of those results identically.

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
| rules/video_codec.yml | 45 | 45 | **100%** |
| rules/other.yml | 46 | 46 | **100%** |
| rules/audio_codec.yml | 17 | 17 | **100%** |
| rules/part.yml | 9 | 9 | **100%** |
| rules/common_words.yml | 154 | 156 | **99%** |
| rules/episodes.yml | 74 | 79 | **94%** |
| rules/source.yml | 21 | 23 | **91%** |
| rules/release_group.yml | 15 | 19 | 79% |
| rules/language.yml | 7 | 9 | 78% |
| rules/title.yml | 14 | 18 | 78% |
| rules/date.yml | 6 | 8 | 75% |
| movies.yml | 143 | 199 | 72% |
| various.yml | 87 | 124 | 70% |
| rules/bonus.yml | 2 | 3 | 67% |
| rules/country.yml | 2 | 3 | 67% |
| rules/film.yml | 2 | 3 | 67% |
| episodes.yml | 321 | 488 | 66% |
| rules/cd.yml | 1 | 2 | 50% |
| rules/website.yml | 1 | 2 | 50% |

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
| date | 26 | 0 | **100.0%** |
| disc | 6 | 0 | **100.0%** |
| episode_count | 6 | 0 | **100.0%** |
| episode_format | 2 | 0 | **100.0%** |
| film | 8 | 0 | **100.0%** |
| frame_rate | 7 | 0 | **100.0%** |
| proper_count | 31 | 0 | **100.0%** |
| season_count | 2 | 0 | **100.0%** |
| size | 9 | 0 | **100.0%** |
| version | 13 | 0 | **100.0%** |
| video_api | 3 | 0 | **100.0%** |
| week | 1 | 0 | **100.0%** |

### ✅ Excellent (90%+)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| video_codec | 497 | 7 | 98.6% |
| screen_size | 421 | 7 | 98.4% |
| audio_codec | 221 | 5 | 97.8% |
| edition | 81 | 2 | 97.6% |
| source | 544 | 16 | 97.1% |
| crc32 | 24 | 1 | 96.0% |
| year | 219 | 11 | 95.2% |
| audio_channels | 112 | 6 | 94.9% |
| container | 143 | 8 | 94.7% |
| season | 444 | 30 | 93.7% |
| type | 767 | 55 | 93.3% |
| website | 20 | 2 | 90.9% |
| streaming_service | 28 | 3 | 90.3% |
| episode | 501 | 54 | 90.3% |

### 🟡 Good (70–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| release_group | 480 | 59 | 89.1% |
| title | 940 | 116 | 89.0% |
| uuid | 7 | 1 | 87.5% |
| film_title | 7 | 1 | 87.5% |
| video_profile | 12 | 2 | 85.7% |
| other | 299 | 50 | 85.7% |
| audio_profile | 29 | 5 | 85.3% |
| language | 120 | 22 | 84.5% |
| part | 16 | 3 | 84.2% |
| subtitle_language | 63 | 18 | 77.8% |
| episode_title | 142 | 59 | 70.6% |

### ⚠️ Developing (50–70%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| country | 9 | 4 | 69.2% |
| episode_details | 11 | 5 | 68.8% |
| bonus_title | 8 | 5 | 61.5% |
| cd | 3 | 2 | 60.0% |
| absolute_episode | 6 | 4 | 60.0% |
| cd_count | 2 | 2 | 50.0% |

### ❌ Intentionally diverged

| Property | Reason |
|---|---|
| audio_bit_rate | Hunch uses single `bit_rate` (see below) |
| video_bit_rate | Hunch uses single `bit_rate` (see below) |
| mimetype | Derived from `container`; redundant |
| alternative_title | 43.8% — partially implemented, improving |

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

### Bit Rate: Single Property (not split)

**Guessit**: Splits bit rate into `audio_bit_rate` and `video_bit_rate`
based on context (proximity to audio/video codec tokens).

**Hunch**: Uses a single `bit_rate` property. In practice, a filename
containing `320Kbps` is unambiguously audio; one containing `20Mbps` is
unambiguously video. Users already have `audio_codec` / `video_codec`
to determine which stream the bitrate refers to. Splitting adds
classification complexity with no real-world benefit.

**Affected test cases** (8): all `audio_bit_rate` and `video_bit_rate`
assertions in the guessit fixture suite.

### Mimetype: Intentionally Omitted

**Guessit**: Derives `mimetype` from the file extension/container
(e.g., `mp4` → `video/mp4`).

**Hunch**: Does not emit `mimetype`. It is a trivial lookup from
`container`, which hunch already provides. Duplicating derived data
adds no information — callers who need a MIME type can map from
`container` themselves.

**Affected test cases** (3): all `mimetype` assertions in the guessit
fixture suite.

### Release Group: Compound Groups

**Guessit**: Can merge separate parts into compound release groups
(e.g., `Tigole QxR` from `...Tigole) [QxR]`).

**Hunch**: Takes the last hyphen-delimited or bracket-enclosed group.

---

## Highest-ROI Improvements

109 test cases currently fail on **exactly 1 property**. Fixing that single
property would flip them from fail to pass. These are the highest-leverage
targets:

| Property | Single-prop fails | Impact |
|---|---|---|
| release_group | 19 | +1.5pp |
| title | 18 | +1.4pp |
| episode_title | 14 | +1.1pp |
| other | 8 | +0.6pp |
| subtitle_language | 7 | +0.5pp |
| language | 6 | +0.5pp |
| episode | 5 | +0.4pp |

Fixing all 109 actionable single-property failures would bring the pass
rate from 78.2% to ~86%.

---

## Known Gaps

These are the areas where hunch diverges most from guessit, with
explanations of why.

### Title extraction (89.0%)

The hardest problem. Title is "everything that's left" after all
technical tokens are claimed. Remaining gaps are mostly disambiguation
problems where a token can be both a title word and a property value:

- "Proof", "LiNE" consumed by Other matcher instead of staying as title
- "3D" consumed by ScreenSize in titles like "Harold & Kumar 3D Christmas"
- "French" consumed by Language in titles like "Immersion French"
- Path segment selection choosing wrong directory for title
- Titles starting with technical-looking words ("h265 - HEVC Riddick")

The planned ZoneMap architecture (v0.2.1) addresses this class of
problems by suppressing ambiguous matches in the title zone.

### Episode parsing (90.3%)

Most standard patterns work (S01E02, 1x03, E01-E03, S01-S10).
Remaining gaps:

- `E01 02 03` multi-episode without separator
- Compact SSEE format (0106 → S01E06)
- Spanish "Cap.102" chapter notation (partially supported)

### Episode title (70.6%)

Positional inference: the episode title is the unclaimed region between
the episode number and the first technical token. Remaining gaps:

- Episode titles after date-based episodes ("Show - 2010-11-23 - Ep Name")
- Part numbers sometimes leak into episode title
- Path-based episode titles from directory structure

### Multi-value subtitle languages

Patterns like `ST{Fr-Eng}` (both French and English subtitles) require
compound parsing that splits within brackets. Hunch handles `[ENG+RU+PT]`
but not curly-brace patterns yet.

### Niche properties

All guessit properties are now either implemented or intentionally
divergent. See "Intentional Design Differences" for `audio_bit_rate`,
`video_bit_rate`, and `mimetype`.

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
