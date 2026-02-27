# Hunch vs guessit — Compatibility Report

Hunch is a **Rust port** of Python's [guessit](https://github.com/guessit-io/guessit).
This document tracks how closely hunch reproduces guessit's behavior, measured
by running hunch against guessit's own test suite (1,309 test cases across 22
YAML files).

> **Last updated:** 2026-02-26 (v0.2.2)

---

## Overall Results

| Metric | Value |
|---|---|
| Total test cases | 1,309 |
| Passed (all props correct) | 1,036 |
| Failed (any prop wrong) | 273 |
| **Pass rate** | **79.1%** |
| Properties implemented | 49 / 49 |
| Properties intentionally diverged | 3 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 79.1% of those results identically.

---

## Pass Rate by Test File

| Test file | Passed | Total | Rate |
|---|---|---|---|
| rules/audio_codec.yml | 17 | 17 | **100%** |
| rules/edition.yml | 44 | 44 | **100%** |
| rules/other.yml | 46 | 46 | **100%** |
| rules/part.yml | 9 | 9 | **100%** |
| rules/screen_size.yml | 9 | 9 | **100%** |
| rules/size.yml | 3 | 3 | **100%** |
| rules/source.yml | 23 | 23 | **100%** |
| rules/video_codec.yml | 45 | 45 | **100%** |
| rules/common_words.yml | 154 | 156 | **99%** |
| rules/episodes.yml | 75 | 79 | 95% |
| rules/date.yml | 7 | 8 | 88% |
| rules/release_group.yml | 15 | 19 | 79% |
| rules/language.yml | 7 | 9 | 78% |
| rules/title.yml | 14 | 18 | 78% |
| movies.yml | 149 | 199 | 75% |
| various.yml | 86 | 124 | 69% |
| rules/bonus.yml | 2 | 3 | 67% |
| rules/country.yml | 2 | 3 | 67% |
| rules/film.yml | 2 | 3 | 67% |
| episodes.yml | 325 | 488 | 67% |
| rules/cd.yml | 1 | 2 | 50% |
| rules/website.yml | 1 | 2 | 50% |

---

## Pass Rate by Property

Each row shows how often hunch produces the correct value for that
property, across all test cases that assert it.

### ✅ Perfect (100%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| aspect_ratio | 2 | 0 | **100.0%** |
| bonus | 13 | 0 | **100.0%** |
| color_depth | 28 | 0 | **100.0%** |
| date | 26 | 0 | **100.0%** |
| disc | 6 | 0 | **100.0%** |
| edition | 83 | 0 | **100.0%** |
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

### ✅ Excellent (95%+)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| video_codec | 497 | 7 | 98.6% |
| screen_size | 421 | 7 | 98.4% |
| audio_codec | 221 | 5 | 97.8% |
| source | 546 | 14 | 97.5% |
| year | 222 | 8 | 96.5% |
| crc32 | 24 | 1 | 96.0% |

### ✅ Good (90–95%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| audio_channels | 112 | 6 | 94.9% |
| container | 143 | 8 | 94.7% |
| season | 444 | 30 | 93.7% |
| type | 767 | 55 | 93.3% |
| title | 959 | 97 | 90.8% |
| website | 20 | 2 | 90.9% |
| streaming_service | 28 | 3 | 90.3% |
| episode | 501 | 54 | 90.3% |

### 🟡 Solid (80–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| release_group | 480 | 59 | 89.1% |
| film_title | 7 | 1 | 87.5% |
| uuid | 7 | 1 | 87.5% |
| video_profile | 12 | 2 | 85.7% |
| audio_profile | 29 | 5 | 85.3% |
| other | 295 | 54 | 84.5% |
| language | 120 | 22 | 84.5% |
| part | 16 | 3 | 84.2% |
| episode_details | 13 | 3 | 81.2% |

### ⚠️ Developing (50–80%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| subtitle_language | 62 | 19 | 76.5% |
| episode_title | 145 | 56 | 72.1% |
| country | 9 | 4 | 69.2% |
| bonus_title | 8 | 5 | 61.5% |
| absolute_episode | 6 | 4 | 60.0% |
| cd | 3 | 2 | 60.0% |
| cd_count | 2 | 2 | 50.0% |
| alternative_title | 7 | 9 | 43.8% |

### ❌ Intentionally diverged

| Property | Reason |
|---|---|
| audio_bit_rate | Hunch uses single `bit_rate` (see below) |
| video_bit_rate | Hunch uses single `bit_rate` (see below) |
| mimetype | Derived from `container`; redundant |

---

## Intentional Divergences

### `bit_rate` (single property)

Hunch emits a single `bit_rate` property instead of guessit's split
`audio_bit_rate` / `video_bit_rate`. Users already have codec properties
for stream context. The split adds complexity without value for the
primary use case (file organization).

### `mimetype`

Trivially derived from `container` (`mkv` → `video/x-matroska`).
Redundant for a filename parser. Users can derive it if needed.

---

## Architecture Notes

### v0.2.2 Pipeline

```
Input → Tokenize → ZoneMap → TOML Rules + Legacy Matchers
     → Conflict Resolution → Zone Disambiguation → Title Extraction → Result
```

- **TOML-driven**: 20 rule files for vocabulary-based properties
- **Algorithmic**: Rust code for episodes, title, release_group, dates
- **Zone-aware**: ZoneMap provides structural boundaries for disambiguation
- **No dual-pipeline**: All TOML/legacy overlap eliminated in v0.2.1
- **regex-only**: No `fancy_regex`, linear-time, ReDoS-immune

See ARCHITECTURE.md for full design documentation.
