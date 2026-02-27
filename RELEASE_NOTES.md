# Hunch v0.3.0 — Two-Pass Pipeline & 80% Milestone

This is a **major architectural release** that introduces a two-pass parsing
pipeline, eliminates the 130-token exclusion list, and crosses the **80%
guessit compatibility threshold**.

## The Numbers

```
v0.2.2  ████████████████████████████████████████████░░░░░░░░  79.0%  (1,034 / 1,309)
v0.3.0  ████████████████████████████████████████████████░░░░  80.0%  (1,047 / 1,309)
                                                      ▲
                                                   +13 cases
```

| Metric | v0.2.2 | v0.3.0 |
|---|---|---|
| Overall pass rate | 79.0% | **80.0%** |
| Properties at 100% | 16 | **16** |
| Properties at 90%+ | 27 | **31** |
| Single-property failures | 99 | **63** |

## 🏗️ Architecture: Two-Pass Pipeline

The biggest change in this release is the **two-pass pipeline**. Release group
extraction now runs AFTER conflict resolution, using resolved match positions
instead of a manually-maintained exclusion list.

```
BEFORE (v0.2.x single pass):         AFTER (v0.3.0 two-pass):
┌─────────────────────────┐         ┌─── PASS 1 ────────────────┐
│ TOML rules              │         │ TOML rules                │
│ Legacy matchers         │         │ Legacy matchers (no RG)   │
│   └─ release_group     │         │ Conflict resolution       │
│ Conflict resolution     │         │ Zone disambiguation       │
│ Zone rules              │         │  → resolved_tech_matches  │
│ Title extraction        │         ├─── PASS 2 ────────────────┤
└─────────────────────────┘         │ release_group(resolved)   │
                                    │ Title extraction          │
                                    │ Episode title             │
                                    └─────────────────────────┘
```

### What this unlocks

- **`is_known_token` (130 tokens) → DELETED** — No more maintaining a
  parallel exclusion list that duplicates TOML rules. Release group
  validation now uses `is_position_claimed()` against resolved matches.
- **Small curated list** — Only ~20 non-group tokens remain (subtitle
  markers, containers not covered by TOML).
- **Zone Rule 5 post-RG** — HQ/HR/FanSub adjacency pruning now runs
  after release group extraction, so it can actually see the group.

## 🏗️ Architecture: Tokenizer Bracket Model

The tokenizer now extracts **structured bracket groups** from the input:

```rust
BracketGroup {
    kind: BracketKind::Square,  // Round, Square, or Curly
    open: 42,                   // Position of opening bracket
    close: 48,                  // Position of closing bracket
    content: "1080p",           // Content between brackets
    segment_idx: 1,             // Which path segment
}
```

This enables future bracket-aware parsing for compound release groups
like `(Tigole) [QxR]`, subtitle language codes like `{Fr-Eng}`, and
CRC32 checksums like `[DEADBEEF]`.

## 🏗️ Architecture: Per-Directory Zone Maps

Zone filtering now works for **directory segments**, not just the filename.
Each directory component gets its own `SegmentZone` with title/tech
boundaries. TOML rules with `zone_scope = "tech_only"` are now properly
suppressed in directory title zones.

## 🏗️ Architecture: TokenStream in Pass 2

All Pass 2 extractors (release_group, title, episode_title, film_title,
alternative_title) now receive the full `TokenStream`. This provides
access to:
- Structured bracket groups
- Per-segment token positions
- Path segment information
- Extension detection

## 📊 Per-Property Improvements

| Property | v0.2.2 | v0.3.0 | Delta |
|---|---|---|---|
| title | 90.1% | **91.6%** | +1.5% |
| release_group | 89.1% | **90.2%** | +1.1% |
| other | 83.7% | **84.8%** | +1.1% |
| episode_title | 70.1% | **74.1%** | +4.0% |

### Key property milestones

- **title crosses 91%** — leading codec handling, language dedup,
  asterisk stripping, year-as-title improvements
- **release_group crosses 90%** — post-resolution extraction, SC/SDH
  context-dependent matching, Zone Rule 5 post-RG
- **episode_title gains 4%** — EpisodeCount boundaries, show title
  separator splitting, suspicious Other detection, trailing Part stripping

## What's New

### Engine features

- **Two-pass pipeline** — Pass 1 resolves tech properties, Pass 2
  extracts positional properties using resolved match positions.
- **Position-based release group validation** — `is_position_claimed()`
  replaces the 130-token `is_known_token` exclusion list.
- **Bracket group model** — `BracketGroup` struct in tokenizer for
  structured bracket content parsing.
- **Per-directory zone maps** — `SegmentZone` provides title/tech
  boundaries for each path segment.
- **Suspicious Other detection** — `Other:Proper` in episode titles is
  recognized as title content when followed by non-tech words.
- **Episode title separator splitting** — ` - ShowTitle - EpTitle`
  patterns are correctly split.
- **Trailing Part stripping** — "Part N" at the end of episode titles
  is stripped (Part is extracted as a separate property).

### TOML rule improvements

- **video_profile.toml** — SC/SCH/SDH now require a preceding codec
  token (`requires_before`). Prevents false positives where SC is a
  release group or SDH means subtitles.
- **video_codec.toml** — HEVC suffix regex tightened from `hevc.+` to
  `hevc[a-zA-Z0-9_]+` to prevent multi-token window over-matching.

### Zone & pipeline improvements

- **Zone Rule 1 enhanced** — drops duplicate language in title zone
  when the same language appears in the tech zone.
- **Zone Rule 5 moved to post-RG** — HQ/HR/FanSub adjacency pruning
  now runs after release group extraction.
- **Title: leading tech skip** — when filename starts with codec tokens
  (e.g., `h265 - HEVC Riddick...`), title extraction skips to the next gap.
- **Title: asterisk stripping** — `*` treated as separator character.

## Breaking Changes

### API

- `release_group::find_matches()` signature changed — now takes
  `(input, resolved_matches, zone_map, token_stream)` instead of
  just `(input)`.
- `title::extract_title()` and all secondary title extractors now
  take an additional `token_stream` parameter.
- These are library-internal functions; the public API (`hunch()`,
  `Pipeline::run()`) is unchanged.

### Semantic

- Release group detection may differ slightly from v0.2.2 in edge cases
  where position-based overlap detection behaves differently from the
  old text-based exclusion list.

## Performance

No measurable performance regression. The two-pass pipeline adds minimal
overhead since Pass 2 operates on already-resolved matches.

## What's Next

- Episode title: directory-based extraction (Bones, Scrubs cases)
- Release group: compound bracket merging using bracket model
- Subtitle language: bracket-based `{Fr-Eng}` parsing
- Per-directory zone filtering for Language/Other in dir names
- Sprint to 85%

## Install

```bash
cargo install hunch
# or
cargo add hunch
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete list of changes.
