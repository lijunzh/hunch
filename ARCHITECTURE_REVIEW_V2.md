# Architecture Review v2: Context-Awareness Audit

> **Date**: 2026-02-26
> **Question**: Do we have the right tools for context-aware parsing?
> Is the two-pass approach complete or partial?

---

## TL;DR

The two-pass pipeline is **real but narrow**. Only **release_group** benefits
from Pass 2 today. Every other positional extractor (title, episode_title,
alternative_title) runs in Pass 2 but doesn't actually USE resolved match
positions — they just look at `all_matches` which includes everything from
Pass 1. The path/directory handling is **fragmented across 4 independent
implementations** with no shared abstraction.

---

## Component-by-Component Audit

### 1. Tokenizer (`tokenizer.rs`)

**What it provides:**
- Path splitting into `PathSegment`s with `SegmentKind` (Directory vs Filename)
- Token positions (byte offsets) within each segment
- Extension detection
- Dot-acronym preservation (S.H.I.E.L.D)
- Bracket detection (`in_brackets` flag)
- `filename_start` offset

**What it DOESN'T provide:**
- No per-segment zone analysis (zones are filename-only)
- No structural relationship between segments
  (e.g., "this directory looks like it contains the same info as the filename")
- No bracket-pair matching (only marks tokens as `in_brackets`, doesn't
  track which bracket group they belong to)

**Assessment**: Good foundation. The segment model is there but underutilized.

---

### 2. ZoneMap (`zone_map.rs`)

**What it provides:**
- Title zone vs tech zone boundary for the **filename only**
- Year disambiguation (title-years vs metadata-years)
- `has_anchors` flag for confidence

**What it DOESN'T provide:**
- **No per-directory zone analysis** — Zone map is built exclusively from
  `input[fn_start..]` (the filename). Directories are invisible to zones.
- **No episode_title zone** — ARCHITECTURE.md mentions `ep_title_zone` in
  the v0.2.1 design but it was never implemented.
- **No release zone** — Similarly designed but not built.
- **Single ZoneMap** — There's exactly one, for the filename. No per-segment
  zone maps for directory components.

**Assessment**: The ZoneMap is a filename-only tool. It's the right abstraction
but needs to be generalized to handle path segments.

---

### 3. TOML Rule Engine (`rule_loader.rs` + `match_tokens_in_segment()`)

**What it provides:**
- Per-segment matching with `SegmentScope` (FilenameOnly vs AllSegments)
- Directory priority penalty (`DIR_PRIORITY_PENALTY = -5`)
- Zone scope filtering (`tech_only`, `after_anchor`, `unrestricted`)
- Neighbor constraints (`not_before`, `not_after`, `requires_before`, `requires_after`)
- Side effects (one match → multiple properties)

**What it DOESN'T provide:**
- **Zone filtering only works for filename segments** — The ZoneMap is
  filename-only, so directory segments get NO zone filtering even when
  scope is `AllSegments`. A token like "Proof" in a directory name would
  match Other:Proof even though it's clearly a title word.
- **No cross-segment awareness** — A match in segment A can't influence
  matching in segment B. E.g., "Complete" in a directory can't be related
  to the season info in the filename.

**Assessment**: The engine is capable but zone filtering is filename-only.
The `SegmentScope` mechanism is good but incomplete without per-segment zones.

---

### 4. Legacy Matchers (15 `fn(&str) -> Vec<MatchSpan>` functions)

**Signature**: `fn(input: &str) -> Vec<MatchSpan>`

Every legacy matcher receives the **raw input string** (full path) and
parses it independently. Each one re-derives filename boundaries:

| Matcher | Path-Aware? | How? |
|---|---|---|
| `episodes` | ✅ Yes | `input.rfind('/')` for path-based seasons |
| `year` | ❌ No | Searches full input indiscriminately |
| `date` | ❌ No | Searches full input |
| `language` | ⚠️ Partial | Searches full input (bracket codes) |
| `subtitle_language` | ⚠️ Partial | Extension-based (.eng.srt) + bracket codes |
| `release_group` | ✅ Yes* | `rfind('/')`, parent dir fallback. *Uses resolved matches |
| `title` | ✅ Yes | Parent dir fallback, abbreviated detection |
| `episode_title` | ❌ No | Filename only |
| `crc32` | ❌ No | Searches full input |
| `uuid` | ❌ No | Searches full input |
| `website` | ❌ No | Searches full input |
| `part` | ❌ No | Searches full input |
| `bonus` | ❌ No | Searches full input |
| `version` | ❌ No | Searches full input |
| `size` | ❌ No | Searches full input |

**Assessment**: Most legacy matchers are path-unaware. They search the full
input string including directory names, which causes false positives
(e.g., "2004" in a directory path matched as a year).

---

### 5. Two-Pass Pipeline (`pipeline/mod.rs`)

**What Pass 2 provides:**

| Component | Uses resolved matches? | Uses ZoneMap? | Uses TokenStream? |
|---|---|---|---|
| `release_group` | ✅ Yes (position overlap) | ✅ Yes (has_anchors) | ❌ No |
| `title` | ⚠️ Partial (match positions only) | ✅ Yes (year disambiguation) | ❌ No |
| `episode_title` | ⚠️ Partial (gap detection) | ❌ No | ❌ No |
| `film_title` | ⚠️ Partial (gap detection) | ❌ No | ❌ No |
| `alternative_title` | ⚠️ Partial (separator detection) | ❌ No | ❌ No |
| `zone_rules` (post-RG) | ✅ Yes (RG adjacency) | ❌ No | ❌ No |

**What "uses resolved matches" means in practice:**
- `release_group`: Actually checks `is_position_claimed()` against resolved spans. **True post-resolution.**
- `title`: Looks at match positions to find the "gap before first match". Uses matches as boundaries but doesn't check overlap or property types. **Not truly post-resolution.**
- `episode_title`: Finds the gap between episode markers and tech properties. Same boundary-based approach. **Not truly post-resolution.**

**Assessment**: The two-pass architecture exists but only release_group
actually leverages it. Title and episode_title were already running after
matching in v0.2 — they just moved to the "Pass 2" label without gaining
new capabilities.

---

## Gap Analysis

### Gap 1: No per-directory ZoneMap

**Impact**: High. Directory segments carry real metadata:
```
Movies/The Matrix (1999)/                    ← title + year in dir
Series/Mad Men Season 1 Complete/            ← title + season + "Complete" in dir
Season 06/e01.1080p.bluray.x264-wavey.mkv    ← season in dir, title is "wavey"?
```

Currently, TOML rules with `AllSegments` scope match directory tokens
without zone filtering. This means:
- "Proof" in a dir name → matched as Other:Proof
- "French" in a dir name → matched as Language:French
- "Complete" in a dir name → NOT matched (Other is FilenameOnly)

**Fix**: Build a ZoneMap per path segment. Directory zones would be simpler
(no release group zone, just title vs metadata boundary).

### Gap 2: Title/episode_title don't use resolved positions

**Impact**: Medium. The "match-then-prune" information loss:
- `Other:Proper` claims "Proper" → episode_title stops at "Downward Dogs and"
  instead of "Downward Dogs and Proper Pigs"
- `Language:French` claims "French" → title becomes "Immersion" instead of
  "Immersion French" (partially fixed by Zone Rule 1 duplicate check)

**Fix**: Title/episode_title could check if matches within their zone are
"suspicious" (e.g., Other:Proper surrounded by non-tech words → likely
title content, not metadata).

### Gap 3: Legacy matchers bypass tokenizer

**Impact**: Medium. 15 matchers receive raw `&str` and re-derive boundaries.
This means:
- No benefit from the tokenizer's bracket detection
- No benefit from segment-level analysis
- Each matcher independently implements `input.rfind('/')`
- Path-based false positives (year in dir, CRC in dir, etc.)

**Fix**: Change legacy matcher signature to
`fn(&str, &TokenStream, &ZoneMap) -> Vec<MatchSpan>`. This requires
refactoring each matcher but gives them tokenized input and zone context.

### Gap 4: No bracket-group model

**Impact**: Medium. Bracket content like `[1080p AMZN Webrip x265 - JBENT]`
or `(Tigole) [QxR]` needs structured parsing. Currently:
- Tokenizer marks tokens as `in_brackets` but doesn't track which bracket group
- Release group's `find_compound_bracket_group()` does ad-hoc bracket scanning
- No shared bracket model for subtitle_language `{Fr-Eng}` or CRC `[DEADBEEF]`

**Fix**: Add a `BracketGroup` struct to the tokenizer:
```rust
struct BracketGroup {
    kind: BracketKind,    // Round, Square, Curly
    start: usize,         // Position of opening bracket
    end: usize,           // Position of closing bracket
    tokens: Vec<Token>,   // Tokens inside the bracket
}
```

### Gap 5: No episode_title zone

**Impact**: Medium. ARCHITECTURE.md v0.2.1 design includes:
```
ep_title_zone: ep_end .. first_tech_after_ep
```
But this was never implemented in ZoneMap. Episode title extraction currently
re-derives this boundary in `secondary.rs`.

**Fix**: Add `ep_title_zone: Option<Range<usize>>` to ZoneMap, populated
when episode markers are detected as anchors.

---

## What "Complete" Two-Pass Would Look Like

```
Input
 │
 ├─ 1. Tokenize (segments, tokens, brackets, extension)
 ├─ 2. Per-segment ZoneMap (title zone, tech zone, ep_title zone)
 │
 ═══ PASS 1: Tech Property Resolution ═══════════════════════
 │
 ├─ 3a. TOML rules (zone-aware, per-segment, bracket-aware)
 ├─ 3b. Legacy matchers (receive TokenStream + ZoneMap)
 ├─ 4. Conflict resolution
 ├─ 5. Zone disambiguation
 │     └─ Output: resolved_tech_matches
 │
 ═══ PASS 2: Positional Extraction ══════════════════════════
 │
 ├─ 6a. release_group(resolved_matches, zone_map, token_stream)
 │      └─ Uses: position overlap, bracket groups, parent dir
 ├─ 6b. title(resolved_matches, zone_map, token_stream)
 │      └─ Uses: zone boundaries, position gaps, per-dir zones
 ├─ 6c. episode_title(resolved_matches, zone_map)
 │      └─ Uses: ep_title_zone, suspicious match detection
 ├─ 6d. alternative_title(resolved_matches, zone_map)
 │
 ├─ 7. Post-RG zone rules
 └─ 8. Build HunchResult
```

### Key differences from current state:

1. **Per-segment ZoneMap** — directories get zone analysis too
2. **Bracket model** — structured bracket groups available to all extractors
3. **Legacy matchers receive context** — TokenStream + ZoneMap, not just raw str
4. **Title/ep_title use resolved matches** — suspicious match detection
5. **Episode title zone in ZoneMap** — structural boundary, not re-derived

---

## Recommendation: Incremental Path

Don't rewrite everything at once. Each step is independently valuable:

### Step 1: Pass TokenStream to Pass 2 extractors (Low effort)
Release group, title, episode_title already run in Pass 2 but don't
receive the TokenStream. Wire it through. This unblocks bracket-aware
extraction without changing any matcher logic.

### Step 2: Bracket group model (Medium effort)
Add `BracketGroup` to TokenStream. Use it in release_group for compound
bracket merging. Use it in subtitle_language for `{Fr-Eng}` parsing.
Use it in CRC32 for `[DEADBEEF]` detection.

### Step 3: Per-directory ZoneMap (Medium effort)
Extend `build_zone_map()` to produce a `Vec<ZoneMap>` (one per segment).
Use in TOML `match_tokens_in_segment()` for directory zone filtering.
Fixes "Complete" in dir, "Proof" in dir, "French" in dir.

### Step 4: Episode title zone (Low effort)
Add `ep_title_zone` to ZoneMap. Simplifies episode_title extraction.

### Step 5: Legacy matcher context (High effort, optional)
Change signature from `fn(&str) → Vec<MatchSpan>` to
`fn(&str, &TokenStream, &ZoneMap) → Vec<MatchSpan>`. Requires touching
every legacy matcher. Only worth doing for matchers where path awareness
matters (episodes, year, language).
