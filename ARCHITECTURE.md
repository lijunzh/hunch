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

**Overall: 75.8%** (992 / 1,309 guessit test cases). Phase C accuracy
improvements in progress. `fancy_regex` removed, `regex`-only.

| Tier | Properties |
|------|------------|
| ✅ 100% | video_api, season_count, disc, aspect_ratio, proper_count, version, bonus, film, size, frame_rate, date, episode_count |
| ✅ 95–99% | edition, source, color_depth, crc32, year |
| 🟡 90–94% | container, video_codec, screen_size, season, type, audio_codec, website, streaming_service, episode |
| 🟡 85–89% | release_group, title, uuid, video_profile, other, audio_profile |
| 🟡 80–84% | language, part, audio_channels |
| ⚠️ 60–78% | subtitle_language (78%), episode_title, country, episode_details, bonus_title, cd |
| ❌ <50% | cd_count, absolute_episode, film_title, alternative_title (0%) |

Highest-ROI targets: title (19 failures), subtitle_language (18),
release_group (18), episode_title (16).

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
  │      └─ 18 TOML rule files (exact lookups + regex + {N} capture templates)
  │
  ├─ 2b. Legacy matchers: regex against raw input (transitional, being removed)
  │
  ├─ 2c. Extension → Container span (priority 10)
  │
  ├─ 3. Conflict resolution (sort by priority desc, length desc; sweep overlaps)
  │
  ├─ 4. Zone-based disambiguation (6 rules, replaces v0.1 prune_* heuristics)
  │     ├─ Rule 1: Language in title zone → drop
  │     ├─ Rule 2: Duplicate source in title zone → drop early
  │     ├─ Rule 3: Redundant HD tags with UHD source → drop
  │     ├─ Rule 4: EpisodeDetails before episode marker → drop
  │     ├─ Rule 5: Other overlapping ReleaseGroup → drop ambiguous
  │     └─ Rule 6: Language contained within tech span → drop
  │
  ├─ 5. Post-processing: title extraction, episode_title, media_type, proper_count
  │
  └─ 6. Build HunchResult → JSON
```

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
7. ⬜ Split tokenizer.rs (684 lines, over 600-line limit)

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
├── matcher/
│   ├── mod.rs              # Re-exports
│   ├── span.rs             # MatchSpan, Property enum (46 variants)
│   ├── engine.rs           # Conflict resolution (priority + length sweep)
│   ├── regex_utils.rs      # ValuePattern + BoundedRegex (LEGACY — to be removed)
│   └── rule_loader.rs      # TOML rule engine: exact + regex + {N} templates
└── properties/             # 30 property matcher modules
    ├── title.rs             # Title/episode_title extraction (algorithmic, stays)
    ├── episodes/            # S01E02, 1x03, ranges, anime (algorithmic, stays)
    ├── release_group.rs     # Positional heuristics (stays)
    ├── date.rs              # Date parsing (algorithmic, stays)
    ├── year.rs              # Year detection (stays)
    └── ...                  # ~25 legacy matchers (being migrated to TOML)

rules/                      # 18 TOML data files (compile-time embedded)
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
├── color_depth.toml        # 10-bit, 8-bit, 12-bit, …
├── country.toml            # US, UK, GB, CA, AU, NZ
├── episode_details.toml    # Special, Pilot, Unaired, Final
└── video_api.toml          # DXVA, D3D11, CUDA, …

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
