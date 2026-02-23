# Gap Analysis: hunch vs guessit Test Vectors

**Date**: 2026-02-22
**Test runner**: `tests/validate_guessit.py`
**Test corpus**: 976 test cases from guessit's YAML files

---

## Executive Summary

| Metric | Value |
|---|---|
| Total test cases | 976 |
| Passed (all props) | 223 (22.8%) |
| Failed (any prop) | 750 |
| Skipped | 3 |

The strict pass rate (ALL properties match) is 22.8%, but per-property
accuracy tells a more useful story — core detectors score 85-91%.

---

## Per-Property Accuracy

| Property | Passed | Failed | Rate | Status |
|---|---|---|---|---|
| audio_codec | 207 | 20 | 91.2% | ✅ Strong |
| container | 131 | 13 | 91.0% | ✅ Strong |
| video_codec | 411 | 41 | 90.9% | ✅ Strong |
| screen_size | 402 | 41 | 90.7% | ✅ Strong |
| year | 193 | 28 | 87.3% | ✅ Strong |
| source | 446 | 81 | 84.6% | ✅ Strong |
| type | 576 | 202 | 74.0% | ⚠️ Medium |
| season | 293 | 118 | 71.3% | ⚠️ Medium |
| episode | 304 | 179 | 62.9% | ⚠️ Medium |
| release_group | 298 | 208 | 58.9% | ⚠️ Medium |
| title | 452 | 354 | 56.1% | ⚠️ Medium |
| edition | 18 | 33 | 35.3% | ❌ Weak |
| audio_channels | 39 | 81 | 32.5% | ❌ Weak |
| other | 26 | 295 | 8.1% | ❌ Weak |

---

## Root Cause Analysis

### 1. Duplicate Matches from Directory Paths (HIGH IMPACT)

**Affects**: year, source, video_codec, audio_codec, release_group

When the same property appears in both the directory path AND filename,
hunch returns an array `["Blu-ray", "Blu-ray"]` instead of deduplicating.

```
Input:  Movies/Sin City (BluRay) (2005)/Sin.City.2005.BDRip.720p.x264.AC3-SEPTiC.mkv
Expect: source = "Blu-ray"
Got:    source = ["Blu-ray", "Blu-ray"]
```

**Fix**: Deduplicate matches with identical property+value. Also, when
the same property appears in both directory and filename, prefer the
filename match (guessit behavior).

### 2. "Rip" Not Tracked as Separate `other` Flag (HIGH IMPACT)

**Affects**: other (>150 failures)

guessit treats `DVDRip` as TWO properties: `source: DVD` + `other: Rip`.
hunch only extracts the source. Almost every "Rip" source variant fails.

**Fix**: When source patterns match `*Rip`, also emit
`MatchSpan { property: Other, value: "Rip" }`.

### 3. Title Extraction from Multi-Path Inputs (HIGH IMPACT)

**Affects**: title (~40% of title failures)

hunch uses only the last path component. guessit has complex rules:
- Prefer directory name for title when filename is abbreviated
- Handle parenthesized alternative titles
- Strip bracketed prefixes like `[XCT]`

```
Input:  Movies/Alice in Wonderland DVDRip.XviD-DiAMOND/dmd-aw.avi
Expect: title = "Alice in Wonderland"
Got:    title = "dmd"
```

**Fix**: Implement path-aware title extraction that considers parent
directories when the filename appears to be an abbreviated scene name.

### 4. Missing "other" Flags (MEDIUM IMPACT)

**Affects**: other, type

guessit tracks many `other` values we don't detect:
- `Rip` (see #2)
- `Region 5`, `Region C`
- `Reencoded` / `re-enc`
- `HD`, `Full HD`, `Ultra HD`
- `Widescreen` / `ws`
- `Audio Fixed`, `Sync Fixed`
- `Fan Subtitled`, `Fast Subtitled`
- `Preair`
- `PAL`, `NTSC`, `SECAM`
- `Low Definition`, `Line Dubbed`, `Mic Dubbed`
- `High Quality`, `High Resolution`
- `Micro HD`
- `Upscaled`
- `BT.2020`
- `2in1`
- `Converted` (vs our existing "Convert")

### 5. Edition Detection Gaps (MEDIUM IMPACT)

**Affects**: edition

Missing editions:
- `Collector` (separate from "Special Edition")
- `Deluxe`
- `Alternative Cut`
- `Limited`
- `Director's Definitive Cut` (DDC)
- `Fan` (vs our "Fan Edit")
- Combined editions: `[Ultimate, Collector]`

Also, guessit uses "Special" not "Special Edition" for
`Special Edition` input.

### 6. Audio Channels Pattern Gaps (MEDIUM IMPACT)

**Affects**: audio_channels

Missing patterns:
- `5ch` / `6ch` → 5.1
- `7ch` / `8ch` → 7.1
- `2ch` → 2.0
- `DD5.1` / `DD51` → Dolby Digital + 5.1 (combined)
- `True-HD51` → Dolby TrueHD + 5.1 (combined)
- `AAC2.0` → AAC + 2.0 (combined)

### 7. Release Group Edge Cases (MEDIUM IMPACT)

**Affects**: release_group

Missing patterns:
- Bracket prefix: `[ABC] Title.mkv` → group = ABC
- Bracket suffix with CRC: `Artik[SEDG]`
- `@` in group names: `HiS@SiLUHD`
- Groups after `by.`: `by.Artik[SEDG]`
- Multi-part groups: `JBENT TAoE` from `(... - JBENT)[TAoE]`
- Scene group from parent dir when filename is abbreviated

### 8. Episode Detection Edge Cases (LOW IMPACT)

**Affects**: episode, season

Missing patterns:
- `501` → S05E01 (positional 3-digit)
- `S03E01E02` → multi-episode array [1, 2]
- `E01-E21` → episode range expansion
- `S01-S04` / `S01 to S04` → multi-season array
- `Season Two` / `Season II` (word/roman numeral)
- `S01EP01` format
- Disc numbering: `S01D02`
- Week numbering: `Week 45`

### 9. Screen Size Edge Cases (LOW IMPACT)

**Affects**: screen_size

- `720hd` / `720pHD` → 720p
- `720p24` / `720p60` → 720p (framerate suffix)
- `480px` → 480p
- Non-standard aspect ratio handling
- `500x480` → "500x480" (non-standard kept as-is)

---

## Unimplemented Properties (26)

These properties exist in guessit but are NOT in hunch yet:

| Property | Complexity | Priority |
|---|---|---|
| episode_title | Medium | P1 |
| language | High | P1 |
| subtitle_language | High | P1 |
| streaming_service | Medium | P1 |
| date | Medium | P2 |
| video_profile | Low | P2 |
| audio_profile | Low | P2 |
| color_depth | Low | P2 |
| proper_count | Low | P2 |
| country | Medium | P2 |
| website | Medium | P3 |
| cd / cd_count | Low | P3 |
| part | Low | P3 |
| film / film_title | Low | P3 |
| bonus / bonus_title | Low | P3 |
| episode_format | Low | P3 |
| episode_details | Low | P3 |
| disc | Low | P3 |
| week | Low | P3 |
| size | Low | P3 |
| aspect_ratio | Medium | P3 |
| crc32 / uuid | Low | P4 |

---

## Recommended Fix Priority (for maximum pass-rate gain)

### Phase 1: Quick Wins (est. +25-30% overall pass rate)
1. **Deduplicate matches** from directory+filename
2. **Track "Rip" as `other` flag** alongside source
3. **Add missing audio channel patterns** (5ch, 6ch, combined)
4. **Add missing edition values** (Collector, Deluxe, Limited, DDC)
5. **Add missing other flags** (Region, Reencoded, HD, Widescreen, etc.)

### Phase 2: Title & Structure (est. +15-20%)
6. **Path-aware title extraction** (prefer parent dir over abbreviated filenames)
7. **Strip bracketed prefixes** from titles (`[XCT]`, `[阿维达]`)
8. **Handle parenthesized alternative titles**
9. **Release group from bracket prefix** `[ABC]`

### Phase 3: New Properties (est. +10-15%)
10. **Episode title extraction** (text between episode and next known property)
11. **Language / subtitle_language detection**
12. **Streaming service detection** (Netflix, Amazon, etc.)
13. **Date detection** (YYYY.MM.DD, DD.MM.YYYY)

### Phase 4: Advanced Episode Handling (est. +5%)
14. **Multi-episode ranges** (E01-E21 → array)
15. **Multi-season** (S01-S04 → array)
16. **3-digit episode codes** (501 → S05E01)
17. **Roman numerals / word numbers** for seasons

---

## Test Infrastructure

The integration test runner lives at `tests/validate_guessit.py`.
Run it with:

```bash
.venv/bin/python3 tests/validate_guessit.py
```

It parses guessit's YAML test files, runs each case through `hunch` CLI,
and reports per-property accuracy with sample failures.
