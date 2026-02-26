# ARCHITECTURE.md — Hunch

> **Decision log, architectural rationale, and developer guide for the hunch
> media filename parser.** This is the single source of truth for how the
> project works and why.

---

## Overview

Hunch parses media filenames like `The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv`
into structured metadata (title, season, episode, codec, etc.).

| Field       | Value                                                        |
| ----------- | ------------------------------------------------------------ |
| Crate name  | `hunch`                                                      |
| Language    | Rust (edition 2024)                                          |
| License     | MIT                                                          |
| Ancestor    | Python `guessit` (LGPLv3) — patterns/knowledge ported, engine rewritten |
| Goal        | Fast, correct, offline, deterministic media filename parsing |

The problem decomposes into three sub-problems, each favoring a different approach:

1. **Recognition** — Is `x264` a video codec? → Lookup tables + regex (TOML rules)
2. **Disambiguation** — Is `French` a language or title word? → Zone inference
3. **Extraction** — Where does the title end? → Positional/algorithmic (Rust code)

---

## Current Status

**Overall: 78.2%** (1,023 / 1,309 guessit test cases). `regex`-only (no
`fancy_regex`). TOML-driven rule engine with side effects, neighbor
constraints, and path-segment awareness.

| Tier | Properties |
|------|------------|
| ✅ 100% | video_api, season_count, disc, aspect_ratio, proper_count, version, bonus, film, size, frame_rate, date, episode_count, episode_format, week |
| ✅ 95–99% | video_codec (98.6%), screen_size (98.4%), audio_codec (97.8%), edition, source, color_depth, crc32, year |
| 🟡 90–94% | container, season, type, absolute_episode (90%), website, streaming_service, episode, audio_channels |
| 🟡 85–89% | release_group (89.1%), title (89.0%), film_title (87.5%), uuid, other, audio_profile |
| 🟡 77–84% | subtitle_language (77.8%), language |
| ⚠️ 60–70% | episode_title (70.6%), bonus_title, part, country |
| ⚠️ <60% | alternative_title (43.8%), cd, cd_count |

Properties: 49/49 implemented (3 intentionally diverged — see COMPATIBILITY.md).

Highest-ROI targets: title (18 single-prop fails), release_group (19),
episode_title (14), other (8).

---

## Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 0–1: Tokenizer + TOML Rules + regex-only (this crate) │
│                                                             │
│   • Split input into tokens at boundaries (. - _ space)     │
│   • Match tokens against TOML rule files (embedded at       │
│     compile time via include_str!())                         │
│   • Algorithmic matchers for episodes, dates, titles,       │
│     release groups (Rust code)                               │
│   • Zone-based disambiguation (structural, not heuristic)   │
│   • regex crate only — linear-time, ReDoS-immune            │
│   • Offline, deterministic, fast (microseconds)             │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Database Lookup (future — NOT this crate)          │
│   • TMDB/IMDB/TVDB to validate titles                      │
│   • Resolves "2001 A Space Odyssey" — DB knows 2001 is     │
│     part of the title, not the year                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: LLM Fallback (future — NOT this crate)             │
│   • For truly ambiguous filenames                            │
└─────────────────────────────────────────────────────────────┘
```

**Hard boundary**: Hunch is a **pure, offline, deterministic library**. No
network, no database, no ML. Layers 2–3 belong in downstream consumers.

---

## v0.2 Pipeline

```
Input string
  │
  ├─ 1. Tokenize: split on separators, extract extension, detect brackets
  │     └─ TokenStream { tokens: [{text, start, end, separator, in_brackets}...], extension }
  │
  ├─ 2a. TOML rules: iterate tokens + multi-token windows (1–3, longest first)
  │      └─ 19 TOML rule files (exact lookups + regex + {N} capture templates)
  │
  ├─ 2b. Legacy matchers: regex against raw input (transitional, being removed)
  │
  ├─ 2c. Extension → Container span (priority 10)
  │
  ├─ 3. Conflict resolution (sort by priority desc, length desc; sweep overlaps)
  │
  ├─ 4. Zone-based disambiguation (5 active rules in zone_rules.rs)
  │     ├─ Rule 1: Language in title zone → drop (uses ZoneMap)
  │     ├─ Rule 2: Duplicate source in title zone → drop early
  │     ├─ Rule 3: Redundant HD tags with UHD source → drop
  │     ├─ Rule 4: RETIRED (episode_details.toml zone_scope="tech_only")
  │     ├─ Rule 5: Other overlapping ReleaseGroup → drop ambiguous
  │     └─ Rule 6: Language contained within tech span → drop
  │
  ├─ 5. Post-processing: title extraction, episode_title, media_type, proper_count
  │
  └─ 6. Build HunchResult → JSON
```

### v0.2 pipeline limitations

The v0.2 pipeline follows a **match-everything-then-prune** pattern. Every
ambiguous token generates a match first; zone rules try to undo mistakes
after the fact. This has three structural problems:

1. **Lost information**: once a token is claimed (e.g., "Proof" → Other),
   title extraction sees it as consumed. Removing the Other match doesn't
   restore it as title content — the title extractor already ran.
2. **Scattered zone logic**: six zone rules in `apply_zone_rules()` each
   reconstruct a partial view of the filename's structure. Title extraction,
   episode title extraction, and release group extraction each independently
   re-derive zone boundaries.
3. **No positional awareness during matching**: TOML rules match tokens
   without knowing whether the token is in the title zone or the tech zone.
   The same token ("Proof", "French", "3D") should match in tech zones
   but be suppressed in title zones.

---

## v0.2.1 Pipeline: Zone Map

v0.2.1 introduces a **ZoneMap** — a structural analysis of the filename
that identifies zones *before* full matching runs. This inverts the
control flow from "match then prune" to "know zones, then match
appropriately."

```
Input string
  │
  ├─ 1. Tokenize (unchanged)
  │     └─ TokenStream { segments, extension }
  │
  ├─ 2. Anchor detection (NEW — two-phase, no TOML rules)
  │     Phase 1: Find high-confidence markers (Tier 1 + Tier 2):
  │     ├─ Structural:  S01E02, 1x03, 1080p, .mkv  (Tier 1)
  │     ├─ Tech vocab:  x264, BluRay, DTS, HDTV     (Tier 2)
  │     └─ → Establishes `tech_zone_start` (first Tier 1/2 token)
  │
  │     Phase 2: Disambiguate position-dependent tokens (Tier 3):
  │     ├─ Year-like numbers: use tech_zone_start + position + count
  │     ├─ Leading brackets:  [Group] at start
  │     └─ Trailing group:    -GROUP.ext at end
  │
  ├─ 3. Zone map construction (NEW)
  │     From anchors, derive zone boundaries per filename segment:
  │
  │     [Group] Title.Words.Year.SxxExx.Ep_Title.Source.Codec-Group.ext
  │      ──┬──  ───┬───── ─┬── ───┬─── ────┬───── ──────┬────── ─┬─── ─┬─
  │     release title     anchor          ep_title    tech          release ext
  │     (lead)  zone      (year/ep)       zone        zone          (trail)
  │
  │     ZoneMap {
  │       title_zone:      fn_start .. first_anchor,
  │       tech_zone:       first_anchor .. group_start,
  │       ep_title_zone:   ep_end .. first_tech_after_ep,  // if episode exists
  │       release_zone:    group_start .. ext_start,        // if group detected
  │     }
  │
  ├─ 4. Zone-aware matching (CHANGED)
  │     TOML rules declare a `zone_scope`:
  │     ├─ Unrestricted: match everywhere (default, backwards-compatible)
  │     │   VideoCodec, AudioCodec, ScreenSize (with p/i suffix), Source, ...
  │     ├─ TechOnly: suppress in title zone (ambiguous tokens)
  │     │   Other (Proof, HDR, 3D), Language, Edition, EpisodeDetails
  │     └─ TechOrAfterAnchor: match in tech zone + after any anchor
  │         Country, SubtitleLanguage
  │
  ├─ 4b. Legacy matchers (unchanged, transitional)
  │
  ├─ 5. Conflict resolution (unchanged)
  │
  ├─ 6. Zone-informed disambiguation (REPLACES apply_zone_rules)
  │     Most zone rules become unnecessary because matching was already
  │     zone-aware. Remaining rules handle cross-zone conflicts only.
  │
  ├─ 7. Post-processing (SIMPLIFIED)
  │     Title extraction uses zone boundaries directly instead of
  │     re-deriving them from match positions.
  │
  └─ 8. Build HunchResult → JSON
```

### Anchor confidence tiers

Not all tokens are equally unambiguous. Anchors exist on a **confidence
spectrum**, and zone construction must account for this.

#### Tier 1: Structural anchors (always unambiguous)

These tokens have built-in structural markers (prefix, suffix, or
position) that make them unambiguous regardless of where they appear.
They never appear as title words.

| Anchor | Signal | Examples |
|---|---|---|
| Season/Episode | `S`/`E` prefix + digits | `S01E02`, `S03-E01` |
| NxN episode | Digit-x-digit pattern | `1x03`, `5x09` |
| Suffixed resolution | Digits + `p`/`i` suffix | `1080p`, `720i`, `2160p` |
| Extension | Last `.xxx` position | `.mkv`, `.avi`, `.mp4` |

#### Tier 2: Unambiguous vocabulary (high confidence)

These tokens have unique vocabulary that virtually never appears in
titles. They are safe to use as zone boundary markers.

| Anchor | Examples |
|---|---|
| Video codec | `x264`, `x265`, `H.264`, `XviD`, `HEVC`, `AV1` |
| Audio codec | `DTS`, `AAC`, `AC3`, `FLAC`, `Atmos`, `TrueHD` |
| Source | `BluRay`, `WEB-DL`, `HDTV`, `DVDRip`, `BDRip` |
| Container (inline) | `MKV`, `AVI` when not in extension position |

#### Tier 3: Position-dependent (need disambiguation)

These tokens can be either metadata or title content. Their meaning
depends on **absolute position** (where in the filename) and **relative
position** (what's around them).

| Token | As metadata | As title | Disambiguation |
|---|---|---|---|
| Year-like (1920–2039) | Release year | Movie title | See below |
| `[Group]` at start | Release group | Subtitle tag | Anime heuristics |
| `-GROUP` at end | Release group | Hyphenated word | Tech tokens must precede |
| `HD`, `3D` | Other / ScreenSize | Title word | Zone position |

### Year disambiguation strategy

Year-like numbers (1920–2039) are the most important position-dependent
anchor. Notable title-as-year examples:

- `1917.2019.1080p.BluRay` — "1917" is title, "2019" is year
- `2001.A.Space.Odyssey.1968.1080p` — "2001" is title, "1968" is year
- `2012.2009.720p.BluRay` — "2012" is title, "2009" is year
- `1922.2017.WEB-DL` — "1922" is title, "2017" is year

**Key insight**: tech tokens define the zone boundary; years are
disambiguated *using* that boundary, not the other way around.

The algorithm:

1. **Find `tech_zone_start`** from Tier 1 + Tier 2 anchors (the first
   structural anchor or unambiguous tech token in the filename). This
   does NOT use year-like numbers.

2. **Classify each year candidate** using `tech_zone_start`:

   | Position | Context | Classification |
   |---|---|---|
   | Parenthesized `(NNNN)` | Any | **Year** (very high confidence) |
   | After `tech_zone_start` | In tech zone | **Year** |
   | Before `tech_zone_start` | Only year candidate | **Year** (also zone boundary) |
   | Before `tech_zone_start` | Multiple candidates; this is first | **Title** |
   | Before `tech_zone_start` | Multiple candidates; this is last | **Year** |
   | At filename start (pos 0) | Followed by non-year words | **Title** |
   | At filename start (pos 0) | Immediately before tech | **Ambiguous** (could be either) |

3. **Refine `title_zone`**: if the year candidate that survives as
   the actual year is before `tech_zone_start`, it becomes the new
   zone boundary. If the year is title content, the boundary stays
   at `tech_zone_start`.

Example walkthrough:
```
2001.A.Space.Odyssey.1968.HDDVD.1080p.DTS.x264.mkv
 │                     │    │     │    │    │
 │                     │    └─────┴────┴────┴── Tier 2 tech tokens
 │                     │
 ├─ Year candidate #1  ├─ Year candidate #2
 │  pos=0 (start)       │  pos=before tech zone
 │  followed by words   │  immediately before HDDVD (Tier 2)
 │                      │
 └─ → Title content     └─ → Actual year

tech_zone_start = position of "HDDVD" (Tier 2, first tech token)
title_zone = [0 .. "1968") → "2001 A Space Odyssey"
year = 1968
```

### Zone scopes for TOML rules

Each TOML rule file can declare a `zone_scope` (defaults to `"unrestricted"`
for backwards compatibility):

```toml
property = "other"
zone_scope = "tech_only"     # NEW: suppress in title zone

[exact]
proof    = "Proof"
hdr      = "HDR"
hdr10    = "HDR10"
```

| Scope | Behavior | Use for |
|---|---|---|
| `unrestricted` | Match in all zones (default) | Unambiguous tech: codecs, resolutions |
| `tech_only` | Suppress matches in title zone | Ambiguous tokens: Other, Edition |
| `after_anchor` | Match only after first anchor | Language, SubtitleLanguage, Country |

The pipeline passes the `ZoneMap` to `match_tokens_in_segment()`. If a
token falls in the title zone and the rule's scope is `tech_only`, the
match is silently skipped — no span is emitted, no conflict to resolve.

### What ZoneMap solves

| Problem | v0.2 (match-then-prune) | v0.2.1 (zones-first) |
|---|---|---|
| "Proof" at start → Other | Other claims it, title gets nothing | title_zone → Other suppressed → title |
| "French" before year | Language claims it, zone rule partial fix | title_zone → Language suppressed → title |
| "3D" in movie title | ScreenSize claims it | title_zone → ambiguous ScreenSize suppressed |
| "LiNE" in title | Other (Line Audio) claims it | title_zone → Other suppressed → title |
| "Edition" in title | Edition claims it before title runs | title_zone → Edition suppressed → title |
| "2001" as title | Both 2001 and 1968 claimed as year | Year disambiguation: first=title, last=year |
| Episode title boundary | Independently re-derives zones | Uses ep_title_zone from ZoneMap |
| Path segment selection | Title picks wrong dir | Per-segment zone analysis → best structure |

### What ZoneMap does NOT solve

These problems are orthogonal to zones:

- **Multi-token compounds** ("Edition Collector") — needs TOML engine
  multi-token window enhancement, not zone awareness.
- **Compound release groups** ("Tigole QxR") — needs release_group.rs
  logic for merging across brackets, not zones.
- **Layer 2 disambiguation** ("2001 A Space Odyssey") — needs external
  title DB. Out of scope for this crate (D004).

### Incremental implementation path

No big-bang rewrite. Each step is a separate commit, tested independently:

1. **Add `ZoneMap` struct + `detect_anchors()`** — non-breaking new code.
   Two-phase: (a) find Tier 1+2 tech tokens → `tech_zone_start`,
   (b) disambiguate Tier 3 year candidates using that boundary.
   Compute zones, log to debug. (~150 lines)
2. **Add `zone_scope` field to TOML `RuleSet`** — additive, defaults to
   `Unrestricted`. Parse `zone_scope` from TOML files. (~30 lines)
3. **Pass `ZoneMap` to `match_tokens_in_segment()`** — add filtering
   alongside existing matching code. Tokens in title zone skip
   `TechOnly` rules. (~20 lines)
4. **Tag ambiguous TOML rules** with `zone_scope = "tech_only"` —
   start with `other.toml`, `edition.toml`. Each file is one commit.
5. **Retire `apply_zone_rules()` heuristics** one at a time as zone
   scopes make them redundant.
6. **Simplify `extract_title()`** — use `title_zone` boundaries
   instead of re-scanning for first-match positions.
7. **Integrate year disambiguation** into the zone map so title
   extraction naturally includes year-as-title numbers.

### Why not rebulk?

guessit uses `rebulk`, a generic Python pattern-matching engine with match
chaining, conflict resolution, and rule-based post-processing. We do NOT
port rebulk. Instead:

| Rebulk Feature | Hunch Equivalent | Why |
|---------------|-----------------|-----|
| Chain execution (A → B) | Flat pipeline, order-independent | Avoids execution-order bugs |
| Rule DSL + callbacks | TOML files + Rust closures | Simpler, auditable |
| Runtime reconfiguration | Compile-time embedding | YAGNI, faster |
| Match tagging system | Typed `Property` enum | Stronger types |
| Backtracking regex | `regex` crate (linear-time) | ReDoS-immune |

---

## v0.2 Audit Findings (2025-02)

Thorough review of every engine component, all 18 TOML files, 21 legacy
matchers, the pipeline, tokenizer, and test infrastructure.

### What's solid (don't change)

1. **Tokenizer-first pipeline** — structurally position-aware, eliminates
   rebulk's execution-order sensitivity.
2. **TOML data-driven rules** — clean, auditable, compile-time embedded.
   Contributors can add codecs without deep Rust knowledge.
3. **Zone-based disambiguation** — structural (derived from token positions),
   not procedural (dependent on execution order). Better than guessit.
4. **Algorithmic matchers in Rust** — episodes (800+ lines), title, release_group,
   date. These are inherently algorithmic and should NEVER be TOML.
5. **Flat `Vec<MatchSpan>` + conflict resolution** — simpler than rebulk's
   chain/tree, priority + length tiebreaking is sufficient.

### Gap 1: Side-effect rules ~~(BLOCKER for legacy removal)~~ ✅ IMPLEMENTED

The single most important gap. Pattern:
```
"DVDRip" → Source:DVD + Other:Rip        (2 properties)
"BRRip"  → Source:Blu-ray + Other:Rip + Other:Reencoded  (3 properties!)
"DVDSCR" → Source:DVD + Other:Screener   (2 properties)
```

**Implemented**: `side_effects` in TOML pattern entries:
```toml
[[patterns]]
match = '(?i)^dvd[-. ]?rip$'
value = "DVD"
side_effects = [
    { property = "other", value = "Rip" }
]
```

This is NOT rebulk chains. It's a flat, declarative side-effect list. One
match → N outputs. No callbacks, no execution order, no rule dependencies.

### Gap 2: Context-dependent matching ✅ IMPLEMENTED

Some tokens are ambiguous and need neighbor-awareness:
- `"HD"` → Other:HD, but NOT before `tv`, `dvd`, `cam`, `rip`
- `"DV"` → Dolby Vision in tech zone vs ignored elsewhere
- `"TS"` → Telesync source vs `.ts` file extension

**Implemented**: Token neighbor checks (NOT regex lookahead):
```toml
[[patterns]]
match = '(?i)^hd$'
value = "HD"
not_before = ["tv", "dvd", "cam", "rip", "tc", "ts"]
```

Supports `not_before`, `not_after`, and `requires_after`. Uses the
tokenizer — checks adjacent token text, not regex lookahead.
Linear time, no backtracking, structurally sound.

### Gap 3: Path-segment awareness ✅ IMPLEMENTED

The tokenizer now tokenizes ALL path segments with `SegmentKind`
(Directory vs Filename). Each TOML rule set declares a `SegmentScope`:

- **`FilenameOnly`**: Tech properties (source, codec, screen_size, etc.)
  skip directory tokens to avoid false positives like "TV Shows" → Source:TV.
- **`AllSegments`**: Contextual properties that benefit from directory metadata
  get a priority penalty (-5) so filename matches always win in conflicts.

Currently all rules use FilenameOnly. AllSegments requires per-segment
zone detection (future work) to avoid title-word false positives in dirs.

### The fancy_regex removal path

| Component | Uses fancy_regex? | Removal path |
|-----------|:-:|---|
| TOML rule_loader | ❌ | Already clean |
| BoundedRegex (episodes, date) | ❌ | Strips lookarounds → standard `regex` |
| ValuePattern (source, language, other) | ⚠️ Fallback | Blocked by legacy matchers |

`fancy_regex` lives ONLY in `ValuePattern` (regex_utils.rs). Once legacy
matchers are removed, `regex_utils.rs` (380 lines), `ValuePattern`, and
`fancy_regex` all die together.

### The dual-pipeline problem

Every vocabulary property is currently matched twice (TOML + legacy). This:
- Required Zone Rule 6 (TOML Source catches "WEB-DL", legacy Language catches "DL")
- Causes ~1-2% regression in common_words and movies tests
- Increases conflict resolution work

This is fine as a transitional state but must be resolved by removing legacy
matchers incrementally.

---

## Execution Plan

### Phase A: Close engine gaps ~~(unblock legacy removal)~~ ✅ DONE
1. ✅ Add `side_effects` to TOML rule engine + rule_loader
2. ✅ Add `not_before` / `not_after` / `requires_after` neighbor checks
3. ✅ Path-segment tokenizer with `SegmentKind` (Directory vs Filename)
4. ✅ Property-scoped `SegmentScope` (FilenameOnly vs AllSegments)
5. ✅ `Property::from_name()` for side-effect property mapping
6. ✅ Extract `match_tokens_in_segment()` for clarity
7. ✅ tokenizer.rs coherent single unit (698 lines) — no split needed

### Phase A.1: Zone map (v0.2.1) — ✅ DONE
1. ✅ `ZoneMap` struct + two-phase `detect_anchors()` in `zone_map.rs`
2. ✅ `zone_scope` field in TOML `RuleSet` + parser (`rule_loader.rs`)
3. ✅ `ZoneMap` passed to `match_tokens_in_segment()` for filtering
4. ✅ Tagged ambiguous TOML rules:
   - `other_weak.toml`: `zone_scope = "tech_only"`
   - `episode_details.toml`: `zone_scope = "tech_only"`
   - `edition.toml`, `other.toml`: intentionally unrestricted (unambiguous)
   - `language.toml`: handled by zone_rules Rule 1 (needs legacy matcher retirement first)
5. ✅ Retired `apply_zone_rules()` heuristics (Rule 4 → TOML zone_scope)
6. ✅ Simplified `extract_title()` — uses ZoneMap for year disambiguation
7. ✅ Year disambiguation integrated into zone map + pipeline Step 2b

### Phase B: Remove legacy matchers (incremental)
Retire one legacy matcher at a time, in order of coverage:
1. **Already TOML-only**: color_depth, audio_profile, other_weak, video_api
2. **Fully covered by TOML** (after Phase A): video_codec, edition,
   streaming_service, video_profile, episode_details, country
3. **Partially covered**: source, screen_size, container, frame_rate,
   audio_codec, language, subtitle_language, other
4. **Never TOML** (algorithmic): episodes, title, release_group, date,
   year, crc32, uuid, website, size, part, bonus, version, aspect_ratio

After step 3: remove `regex_utils.rs` + `fancy-regex` dependency.

### Phase C: Accuracy improvements
1. Subtitle language (49% → 80%+) — highest ROI
2. Title extraction (84% → 90%+) — most single-prop failures
3. Episode title (70% → 80%+)
4. Add missing properties: film_title, absolute_episode, bit_rates
5. Release group edge cases

### Phase D: Polish
- Bump version to 0.2.0
- Update README + CHANGELOG
- `cargo clippy` clean, no warnings
- Benchmark comparison with guessit (Python)
- Consider crates.io publish

---

## Decision Log

### D001: Data-driven patterns (TOML) over hardcoded Rust

**Status**: In progress (v0.2)

Move simple property patterns into TOML rule files, embedded at compile time
via `include_str!()`. Keep complex algorithmic logic (title, episodes,
release_group, date) in Rust.

**Consequences**:
- Pattern definitions are readable and auditable in isolation
- The Rust engine becomes a generic rule loader + matcher
- Contributors can add patterns without deep Rust knowledge
- Single binary deployment preserved (TOML embedded at compile time)

### D002: `regex` crate only — drop `fancy_regex`

**Status**: In progress (blocked by legacy matchers)

The tokenizer eliminates the need for lookaround because patterns match
against isolated tokens, not substrings of the full input:
```
Before (needs lookaround):  (?<![a-z])HDTV(?![a-z])  on "Movie.HDTV.x264"
After  (token isolation):   (?i)^HDTV$               on token "HDTV"
```

**Security benefit**: `regex` guarantees linear-time matching. ReDoS is
structurally impossible.

### D003: Tokenizer + TOML + regex-only as bundled change

These three are interdependent:
- TOML without tokenizer still needs `fancy_regex` for boundaries
- regex-only without tokenizer breaks ~30 patterns
- Tokenizer enables both TOML and regex-only cleanly

### D004: No network, database, or ML in this crate

**Status**: Decided, permanent. See Layered Architecture above.

### D005: No rebulk port

**Status**: Decided, permanent. See "Why not rebulk?" above.

### D006: Zone map for disambiguation (anchors first, zones second, matching third)

**Status**: In progress (v0.2.1)

The v0.2 pipeline matches all tokens against all rules, then prunes
mistakes via post-hoc zone rules. This loses information (a pruned match
can't be restored as title content) and scatters zone logic across
multiple components.

v0.2.1 inverts the flow: detect unambiguous anchors first, construct a
zone map from those anchors, then run TOML rules with zone-awareness so
ambiguous tokens in the title zone are never matched in the first place.

**Key insight 1**: Anchors are not binary — they have confidence tiers.
Structural markers (SxxExx, 1080p, .mkv) are always unambiguous.
Tech vocabulary (x264, BluRay) is almost always unambiguous. But
position-dependent tokens like year-like numbers (1920–2039) need
contextual disambiguation.

**Key insight 2**: Tech tokens define the zone boundary; years are
disambiguated *using* that boundary, not the other way around. The
first unambiguous tech token establishes `tech_zone_start`. Year
candidates before it may be title content (e.g., "2001" in
"2001.A.Space.Odyssey.1968"); year candidates at or after it are
metadata.

**Consequences**:
- Eliminates the "match-then-prune" anti-pattern for most disambiguation
- Zone logic lives in one place (`ZoneMap`) instead of scattered heuristics
- Title extraction becomes simpler (uses zone boundaries, not re-scanning)
- Year-as-title cases (1917, 2001, 2012) are handled structurally
- Incremental: each TOML rule file can opt-in to zone scoping independently
- Backwards compatible: default scope is `unrestricted` (no behavior change)

**Does NOT solve**: multi-token compounds, compound release groups,
title DB lookups (Layer 2). These are orthogonal problems.

---

## Module Map

```
src/
├── lib.rs                  # Public API: parse()
├── main.rs                 # CLI binary (clap)
├── hunch_result.rs         # HunchResult type + typed accessors + JSON
├── options.rs              # Options / configuration
├── pipeline.rs             # v0.2 pipeline orchestration
├── tokenizer.rs            # Input → TokenStream (separators, brackets, extension)
├── zone_map.rs             # (v0.2.1) Anchor detection + zone boundary computation
├── matcher/
│   ├── mod.rs              # Re-exports
│   ├── span.rs             # MatchSpan, Property enum (55 variants)
│   ├── engine.rs           # Conflict resolution (priority + length sweep)
│   ├── regex_utils.rs      # ValuePattern + BoundedRegex (LEGACY — to be removed)
│   └── rule_loader.rs      # TOML rule engine: exact + regex + {N} templates + zone_scope
└── properties/             # 31 property matcher modules
    ├── title.rs             # Title/episode_title extraction (algorithmic, stays)
    ├── episodes/            # S01E02, 1x03, ranges, anime, week (algorithmic, stays)
    ├── release_group.rs     # Positional heuristics (stays)
    ├── bit_rate.rs          # Bit rate detection (v0.2.1)
    ├── date.rs              # Date parsing (algorithmic, stays)
    ├── year.rs              # Year detection (stays)
    └── ...                  # ~25 legacy matchers (being migrated to TOML)

rules/                      # 20 TOML data files (compile-time embedded)
├── video_codec.toml        # H.264, H.265, AV1, Xvid, …
├── audio_codec.toml        # AAC, DTS, Dolby, FLAC, Opus, …
├── source.toml             # Blu-ray, WEB-DL, HDTV, DVDRip, …
├── screen_size.toml        # 720p, 1080p, 4K, WxH ({N} templates)
├── container.toml          # mkv, mp4, avi, srt, …
├── frame_rate.toml         # 24fps, 120fps ({N} templates)
├── language.toml           # English, French, Multi, VFF, …
├── subtitle_language.toml  # VOSTFR, NLsubs, SubForced, …
├── edition.toml            # Director's Cut, Extended, Unrated, …
├── other.toml              # HDR, Remux, Proper, Repack, 3D, …
├── other_weak.toml         # Low-priority Other matches
├── streaming_service.toml  # AMZN, NF, HMAX, DSNP, …
├── video_profile.toml      # Hi10P, HP, SVC, …
├── audio_profile.toml      # Atmos, DTS:X, TrueHD, …
├── audio_channels.toml     # 5.1, 7.1, 2ch, …
├── color_depth.toml        # 10-bit, 8-bit, 12-bit, …
├── country.toml            # US, UK, GB, CA, AU, NZ
├── episode_details.toml    # Special, Pilot, Unaired, Final
├── episode_format.toml     # Minisode (v0.2.1)
└── video_api.toml          # DXVA, D3D11, CUDA, …
```
tests/
├── guessit_regression.rs   # 22 fixture files, ratchet-pattern floors
├── integration.rs          # 27 hand-written end-to-end tests
├── helpers/mod.rs           # YAML loader, test case struct
└── fixtures/               # Self-contained test vectors (from guessit)
```

---

## TOML Rule File Format

```toml
property = "video_codec"
zone_scope = "unrestricted"  # (v0.2.1) "unrestricted" | "tech_only" | "after_anchor"

[exact]                    # Case-insensitive exact token lookups
x264 = "H.264"
hevc = "H.265"

[exact_sensitive]          # Case-sensitive (for ambiguous short tokens)
NZ = "NZ"

[[patterns]]               # Regex with optional {N} capture templates
match = '(?i)^[xh][-.]?265$'
value = "H.265"            # Static value

[[patterns]]
match = '(?i)^(\d{3,4})x(\d{3,4})$'
value = "{2}p"             # Dynamic: capture group 2 → "1080p"

[[patterns]]               # Side effects: one match → multiple properties
match = '(?i)^dvd[-. ]?rip$'
value = "DVD"
side_effects = [{ property = "other", value = "Rip" }]

[[patterns]]               # Neighbor constraints: context-aware matching
match = '(?i)^hd$'
value = "HD"
not_before = ["tv", "dvd", "cam"]
# Also available: not_after, requires_after
```

Matching order: case-sensitive exact → case-insensitive exact → regex (first match wins).
All regex uses `regex` crate only (linear-time, ReDoS-immune).

---

## Developer Guidelines

### Code style

- **Idiomatic Rust**: ownership, strong types, exhaustive matches, `clippy` clean.
- **DRY**: shared helpers in `matcher/`. Don't duplicate patterns between TOML and Rust.
- **YAGNI**: don't build Phase D infra during Phase A.
- **Files under 600 lines**: split by responsibility if growing beyond this.
- **Tests in each module**: `#[cfg(test)] mod tests` blocks.

### Adding a new property (v0.2)

1. Create `rules/<name>.toml` with `property`, `[exact]`, and `[[patterns]]`.
2. Add a `LazyLock<RuleSet>` static in `pipeline.rs`.
3. Register it in the `toml_rules` vector with appropriate property + priority.
4. Add `Property::YourProp` variant to `src/matcher/span.rs` (if new).
5. Add unit tests in rule_loader and integration tests.
6. Only create `src/properties/<name>.rs` if the property needs algorithmic
   parsing (episodes, year, etc.) that tokens can't express.

### Testing strategy

1. **Unit tests** in each property matcher (`#[cfg(test)]` blocks).
2. **Integration tests** (`tests/integration.rs`) — hand-written end-to-end.
3. **Regression tests** (`tests/guessit_regression.rs`) — 22 fixture files with
   ratchet-pattern minimum pass rates. Floors only go up.
4. **Compatibility report** — `cargo test compatibility_report -- --ignored --nocapture`
5. **Benchmarks** (`benches/parse.rs`) — Criterion benchmarks.
6. All fixtures self-contained in `tests/fixtures/` (no external repos needed).

### Conflict resolution

1. **Priority tiers**: Extension (10) > known tokens (0) > weak/positional (-1/-2).
2. **Overlap**: higher priority wins; ties broken by longer span.
3. **Multi-value**: Episode, Language, SubtitleLanguage, Other, Season, Disc
   support multiple values on the same span (serialized as JSON arrays).

---

## Dependencies

| Crate | Purpose | Status |
|-------|---------|--------|
| `regex` | Pattern matching (linear-time) | Permanent |
| `fancy-regex` | Lookaround fallback | **Removing** (blocked by legacy matchers) |
| `serde` + `serde_json` | JSON output | Permanent |
| `clap` | CLI argument parsing | Permanent |
| `toml` | TOML rule parsing | Permanent |

---

## Guessit Source Map

For porting patterns, find originals in
[guessit](https://github.com/guessit-io/guessit) `guessit/rules/properties/`:

| hunch module | guessit source |
|-------------|----------------|
| `episodes/` | `episodes.py` (~900 lines) |
| `title.rs` | `title.py` |
| `release_group.rs` | `release_group.py` |
| `source.rs` → `source.toml` | `source.py` |
| `audio_codec.rs` → `audio_codec.toml` | `audio_codec.py` |
| `video_codec.rs` → `video_codec.toml` | `video_codec.py` |
| `language.rs` → `language.toml` | `language.py` |
| `other.rs` → `other.toml` | `other.py` |

---

## Security Model

- TOML rule files embedded at compile time — no runtime file access
- `regex` crate only (target) — linear-time, ReDoS structurally impossible
- Schema-validated at load time (max pattern length, valid property names)
- No `unsafe`, no FFI, no file I/O, no network
- All patterns reviewed as code changes (TOML files are versioned)
