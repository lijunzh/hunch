# Hunch vs guessit — Compatibility Report

Hunch is a **Rust port** of Python's [guessit](https://github.com/guessit-io/guessit).
This document tracks how closely hunch reproduces guessit's behavior, measured
by running hunch against guessit's own test suite (1,309 test cases across 22
YAML files).

> **Last updated:** 2026-03-20 (v1.1.5)

---

## Overall Results

| Metric | Value |
|---|---|
| Total test cases | 1,309 |
| Passed (all props correct) | 1,071 |
| Failed (any prop wrong) | 238 |
| **Pass rate** | **81.8%** |
| Properties implemented | 49 / 49 |
| Properties intentionally diverged | 3 |

guessit passes 100% of its own tests by definition. Hunch currently
reproduces 82.2% of those results identically.

---

## Pass Rate by Test File

| Test file | Passed | Total | Rate |
|---|---|---|---|
| rules/audio_codec.yml | 17 | 17 | **100%** |
| rules/edition.yml | 44 | 44 | **100%** |
| rules/language.yml | 9 | 9 | **100%** |
| rules/other.yml | 46 | 46 | **100%** |
| rules/part.yml | 9 | 9 | **100%** |
| rules/release_group.yml | 19 | 19 | **100%** |
| rules/screen_size.yml | 9 | 9 | **100%** |
| rules/size.yml | 3 | 3 | **100%** |
| rules/source.yml | 23 | 23 | **100%** |
| rules/video_codec.yml | 45 | 45 | **100%** |
| rules/common_words.yml | 155 | 156 | **99%** |
| rules/episodes.yml | 77 | 79 | 98% |
| rules/date.yml | 7 | 8 | 88% |
| movies.yml | 160 | 199 | 80% |
| rules/title.yml | 14 | 18 | 78% |
| various.yml | 89 | 124 | 72% |
| episodes.yml | 337 | 488 | 69% |
| rules/bonus.yml | 2 | 3 | 67% |
| rules/country.yml | 2 | 3 | 67% |
| rules/film.yml | 2 | 3 | 67% |
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
| screen_size | 421 | 7 | 98.4% |
| audio_codec | 220 | 6 | 97.3% |
| video_codec | 488 | 16 | 96.8% |
| year | 222 | 8 | 96.5% |
| source | 538 | 22 | 96.1% |
| crc32 | 24 | 1 | 96.0% |

### ✅ Good (90–95%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| audio_channels | 112 | 6 | 94.9% |
| container | 143 | 8 | 94.7% |
| season | 445 | 29 | 93.9% |
| type | 769 | 53 | 93.6% |
| title | 972 | 84 | 92.0% |
| website | 20 | 2 | 90.9% |
| episode | 503 | 52 | 90.6% |
| release_group | 487 | 52 | 90.4% |
| streaming_service | 28 | 3 | 90.3% |

### 🟡 Solid (80–90%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| other | 311 | 38 | 89.1% |
| film_title | 7 | 1 | 87.5% |
| uuid | 7 | 1 | 87.5% |
| video_profile | 12 | 2 | 85.7% |
| audio_profile | 29 | 5 | 85.3% |
| part | 16 | 3 | 84.2% |
| episode_details | 13 | 3 | 81.2% |
| language | 115 | 27 | 81.0% |
| subtitle_language | 65 | 16 | 80.2% |

### ⚠️ Needs Work (50–80%)

| Property | Passed | Failed | Rate |
|---|---|---|---|
| episode_title | 153 | 48 | 76.1% |
| country | 9 | 4 | 69.2% |
| bonus_title | 8 | 5 | 61.5% |
| absolute_episode | 6 | 4 | 60.0% |
| cd | 3 | 2 | 60.0% |
| alternative_title | 9 | 7 | 56.2% |
| cd_count | 2 | 2 | 50.0% |

### ❌ Intentionally diverged

| Property | Reason |
|---|---|
| audio_bit_rate | Hunch uses single `bit_rate` (see below) |
| video_bit_rate | Hunch uses single `bit_rate` (see below) |
| mimetype | Trivially derived from `container`; not implemented |

---

## Known Gaps & Future Work

See [GitHub Issues](https://github.com/lijunzh/hunch/issues) for tracked
improvements. Key P3-aligned work:

- [#52](https://github.com/lijunzh/hunch/issues/52) — Context-based episode detection (replace digit decomposition heuristic)
- [#53](https://github.com/lijunzh/hunch/issues/53) — Context-based year disambiguation

---

## Intentional Divergences

### 1. `bit_rate` (combined)

guessit splits bit rate into `audio_bit_rate` and `video_bit_rate`.
Hunch emits a single `bit_rate` because the filename alone rarely
contains enough context to disambiguate audio vs video bit rate.

### 2. `mimetype`

guessit derives MIME type from container extension (e.g., `mkv` →
`video/x-matroska`). This is a trivial lookup that belongs in the
consumer, not the parser.

---

## How to Reproduce

```bash
# Run the full compatibility report:
cargo test compatibility_report -- --ignored --nocapture

# Dump individual failures:
HUNCH_DUMP_FAILURES=50 cargo test compatibility_report -- --ignored --nocapture
```
