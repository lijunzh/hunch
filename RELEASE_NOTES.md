# Hunch v0.2.2 — Phase C: Accuracy & Refactors

This release focuses on **accuracy improvements** and **structural refactoring**,
pushing the guessit compatibility rate from 76.6% to **79.1%** (+33 test cases).

## The Numbers

```
v0.2.1  ████████████████████████████████████████░░░░░░░░░░░  76.6%  (1,003 / 1,309)
v0.2.2  ████████████████████████████████████████████░░░░░░░░  79.1%  (1,036 / 1,309)
                                                  ▲
                                               +33 cases
```

| Metric | v0.2.1 | v0.2.2 |
|---|---|---|
| Overall pass rate | 76.6% | **79.1%** |
| Properties at 100% | 15 | **16** |
| Properties at 90%+ | 29 | **30** |
| Single-property failures | 124 | **99** |

## Highlights

### 🎯 edition reaches 100%

The `edition` property now matches guessit perfectly across all 83 test
assertions. Key fix: the `Edition Collector` pattern (French reversed
form) and directory-level edition detection (AllSegments scope).

### 📈 title crosses 90%

Title extraction improved from 89.1% → 90.8% through:

- **Bracket group title boundaries** — `[Ayako] Infinite Stratos - IS`
  now correctly extracts just `Infinite Stratos` (stops at ` - `)
- **Year-as-anchor zone filtering** — `A.Common.Title.Special.2014`
  keeps `Special` as title content instead of matching it as metadata
- **Parent directory after-match extraction** — `S02 Some Series/E01.mkv`
  now extracts `Some Series` from the directory (after the season marker)

### 🧠 New engine capability: `requires_context`

TOML patterns can now declare `requires_context = true` to match only
when the filename contains recognized technical tokens (Tier 1/2 anchors).
This replaces fragile 90-token enumeration lists with a structural check:

```toml
# Before (884 characters!):
requires_before = ["season", "saison", "dvd", "bluray", ... 87 more ...]

# After (clean and automatic):
requires_context = true
requires_before = ["season", "saison", "temporada", "staffel", "serie", "series"]
```

When `requires_context` and `requires_before` are combined, the
`requires_before` acts as a fallback for anchor-less filenames.

### ♻️ release_group module split

The 626-line monolithic `release_group.rs` was split into a clean module:

- `release_group/mod.rs` (312 lines) — regex patterns + matching logic
- `release_group/known_tokens.rs` (190 lines) — token exclusion list,
  strip_trailing_metadata, expand_group_backwards, helper functions

Organized `is_known_token()` into categorized sections (containers,
video codecs, audio codecs, sources, quality, release tags, languages,
subtitle markers) for easier maintenance.

## Per-Property Improvements

| Property | v0.2.1 | v0.2.2 | Delta |
|---|---|---|---|
| edition | 97.6% | **100%** | +2.4% |
| source | 95.4% | **97.5%** | +2.1% |
| year | 96.1% | **96.5%** | +0.4% |
| title | 89.1% | **90.8%** | +1.7% |
| other | 81.7% | **84.5%** | +2.8% |
| language | 77.5% | **84.5%** | +7.0% |
| episode_title | 70.1% | **72.1%** | +2.0% |

## What's New

### Engine features

- **`requires_context`** — TOML constraint: match only when filename has
  tech anchors. Replaces fragile token-enumeration lists.
- **`requires_before`** — symmetric with `requires_after`: match only
  when the previous token is in the list.
- **Zone Rule 6** — source subsumption dedup: when both TV and HDTV
  exist, the generic TV is dropped automatically.
- **AmazonHD side_effects** — `AmazonHD` now emits both
  `streaming_service: Amazon Prime` and `other: HD`.

### TOML rule improvements

- `bd` → Source: Blu-ray (standalone BD detection)
- `scr` → Other: Screener (standalone SCR detection)
- `ultra` → Other: Ultra HD (standalone Ultra detection)
- `hq`, `ld` moved from zone_scope=tech_only to unrestricted
- `dubbed` → not_after constraint for language names
- Audio profile HQ now requires AAC prefix (standalone HQ → Other)
- Complete uses `requires_context` with season-word fallback
- Fix requires tech tokens on both sides via `requires_before`+`requires_after`
- FLEMISH → `nl-be` (Belgian Dutch)
- Edition Collector pattern (French reversed form)
- Fansub/fastsub added to release_group known tokens

### Zone & pipeline improvements

- Tier 2 anchor expansion: `dvd`, `dvdr`, `bd`, `pal`, `ntsc`, `secam`
- Year-as-anchor zone filtering (when title content ≥ 6 bytes)
- Date as episode_title anchor (date-based shows like Simply Red)
- Bracket group title boundary detection
- Parent directory title extraction after leading matches
- Year disambiguation: first parenthesized year wins
- Title-year overlap: range-based instead of exact position match
- Zone Rule 5: adjacency-based HQ/FanSub pruning near release groups
- Zone rules audited and renumbered (7 active, no gaps)

## Performance

Benchmarks are stable or improved compared to v0.2.1:

| Benchmark | v0.2.2 | vs v0.2.1 |
|---|---|---|
| movie_basic | 453 µs | -3.6% |
| movie_complex | 718 µs | ~0% |
| episode_sxxexx | 608 µs | ~0% |
| episode_with_path | 979 µs | ~0% |
| anime_bracket | 937 µs | ~0% |
| minimal | 387 µs | -3.4% |

## Breaking Changes

None. v0.2.2 is fully backwards-compatible with v0.2.1.

## What's Next

- **Phase E1**: Release group → post-resolution extraction
  (eliminate the 190-line `is_known_token` exclusion list)
- **Path B**: Sprint to 80% (12 more cases needed)
- **crates.io**: Consider publishing

## Install

```bash
cargo install hunch
# or
cargo add hunch
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete list of changes.
