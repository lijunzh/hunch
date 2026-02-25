# Architecture: Hunch

> **Decision log and architectural rationale for the hunch media filename parser.**
>
> This document captures the *why* behind design decisions, not just the *what*.
> It's intended for contributors and future-us who wonder "why did we do it
> this way?"

---

## Overview

Hunch parses media filenames like `The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv`
into structured metadata (title, season, episode, codec, etc.).

The problem decomposes into three sub-problems:

1. **Recognition** — Is `x264` a video codec? Is `720p` a screen size?
2. **Disambiguation** — Is `French` a language or part of the title?
3. **Extraction** — What region of the string is the title?

Each sub-problem favors a different approach. Our architecture layers them.

---

## Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: Lookup Tables + Regex (v0.1) — this crate         │
│                                                             │
│   • fancy_regex for word-boundary assertions (lookaround)   │
│   • Patterns hardcoded in Rust (ValuePattern + LazyLock)    │
│   • Offline, deterministic, fast (microseconds)             │
│   • Target: ~75% accuracy                                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Tokenizer + TOML Rules + regex-only (v0.2) — this crate  │
│                                                             │
│   • Split input into tokens at boundaries (. - _ space)     │
│   • Anchor tokens (S01E02, 720p) divide into zones:         │
│     TITLE ZONE | TECH ZONE                                  │
│   • Context-sensitive: "French" in title zone = title word,  │
│     "French" in tech zone = language                         │
│   • Patterns move to TOML files (embedded at compile time)   │
│   • Drop fancy_regex entirely — tokenization eliminates      │
│     the need for lookaround assertions                       │
│   • regex crate only — linear-time, ReDoS-immune            │
│   • Eliminates all prune_* heuristics                        │
│   • Target: ~90% accuracy                                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Database Lookup (future) — downstream consumer     │
│                                                             │
│   • Query TMDB/IMDB/TVDB to validate parsed titles          │
│   • Resolves "2001 A Space Odyssey" — DB knows 2001 is      │
│     part of the title, not the year                          │
│   • Cache in SQLite for offline use                          │
│   • NOT part of this crate — belongs in plex_organizer       │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: LLM Fallback (future) — downstream consumer        │
│                                                             │
│   • For truly ambiguous filenames where Layers 0-2 have      │
│     low confidence                                           │
│   • NOT part of this crate — belongs in plex_organizer       │
└─────────────────────────────────────────────────────────────┘
```

### Boundary: what lives in hunch vs. downstream

Hunch is a **pure, offline, deterministic library**. It takes a string in and
returns structured data out. No network, no database, no ML model. This is a
hard constraint — it keeps the crate fast, testable, and embeddable anywhere
(CLI, server, WASM, embedded).

Layers 2-3 (database lookup, LLM) require network access and external state.
They belong in downstream consumers like `plex_organizer`, not in this crate.

---

## Decision Log

### D001: Data-driven patterns (TOML) over hardcoded Rust

**Status**: Deferred to v0.2 (bundled with tokenizer)

**Context**: We have ~486 regex patterns hardcoded across 20+ Rust files.
Adding a new codec means editing Rust, recompiling, and navigating through
pattern-matching code mixed with detection logic.

**Decision**: In v0.2, move simple property patterns into TOML rule files,
embedded at compile time via `include_str!()`. Keep complex algorithmic logic
(title extraction, episode parsing, release group heuristics) in Rust.

**Why deferred**: TOML migration is most valuable when combined with the
tokenizer (D003) and regex-only (D002) changes. Doing TOML alone in v0.1
would require keeping `fancy_regex` for word boundary assertions, which
defeats the security benefit (D002). All three changes are interdependent
and should ship together.

**Consequences (when implemented)**:
- Pattern definitions are readable and auditable in isolation
- The Rust engine becomes a generic rule loader + matcher
- Contributors can add patterns without deep Rust knowledge
- Single binary deployment preserved (TOML embedded at compile time)
- Regex validity checked by tests, not compiler (acceptable — same
  practical safety as current `LazyLock` + `unwrap()` approach)

### D002: `regex` crate only — drop `fancy_regex` entirely

**Status**: Deferred to v0.2 (requires tokenizer first)

**Context**: We use `fancy_regex` for lookahead/lookbehind assertions
(`(?<![a-z])HDTV(?![a-z])`). This is necessary because Rust's `regex` crate
doesn't support lookaround. However, `fancy_regex` uses backtracking, which
makes it theoretically vulnerable to ReDoS.

**Decision**: In v0.2, drop `fancy_regex` entirely. The tokenizer (D003)
eliminates the need for lookaround because patterns match against isolated
tokens, not substrings of the full input.

**Why deferred**: Dropping `fancy_regex` without the tokenizer would break
~30 complex patterns (episode parsing, date detection, release groups) and
drop compatibility from 75% to ~50%. The tokenizer must come first.

**Why tokenizer solves lookaround**:
```
Current (full-string matching, needs lookaround):
  Input: "Movie.HDTV.x264"     Pattern: (?<![a-z])HDTV(?![a-z])
  Must assert no letter before/after HDTV in the full string.

With tokenizer (isolated token matching, no lookaround needed):
  Tokens: ["Movie", "HDTV", "x264"]    Pattern: (?i)^HDTV$
  Token is already bounded. No surrounding context to worry about.
```

**Security benefit**: The `regex` crate guarantees linear-time matching,
making ReDoS structurally impossible. This is enforced by the engine,
not by convention.

### D003: Tokenizer in v0.2 (bundled with D001 + D002)

**Status**: Planned for v0.2

**Context**: The current architecture runs regex patterns across the entire
input string, then uses `prune_*` functions to remove false positives
(e.g., "French" in title position matched as a language). A tokenizer
would split the input first, establish title/tech zones, and eliminate
these heuristics.

**Decision**: Ship v0.1 with the current regex-over-full-string approach
(including `fancy_regex`). In v0.2, implement the tokenizer together with
TOML rules (D001) and regex-only (D002) as a single coordinated change.

**Rationale**:
- v0.1 at ~75% accuracy is useful and shippable
- D001 (TOML), D002 (regex-only), and D003 (tokenizer) are interdependent:
  - TOML without tokenizer still needs `fancy_regex` for boundaries
  - regex-only without tokenizer breaks ~30 patterns
  - Tokenizer enables both TOML and regex-only cleanly
- Shipping all three together avoids intermediate regression
- The current `fancy_regex` patterns carry over as reference for
  what the tokenizer needs to handle

**What the tokenizer solves**:
- Eliminates all `prune_*` functions (currently 5 and growing)
- Makes title extraction trivial ("title zone tokens" instead of
  "largest unclaimed gap before first tech token")
- Removes need for lookaround assertions (tokens are already bounded)
- Makes position-dependent disambiguation structural, not heuristic

### D004: No network, database, or ML in this crate

**Status**: Decided, permanent

**Context**: Database lookups (TMDB/IMDB) and LLM APIs could significantly
improve accuracy for ambiguous cases (e.g., "2001 A Space Odyssey" where
2001 is part of the title, not the year).

**Decision**: Hunch is and will remain a pure, offline, deterministic
library. Network-dependent features belong in downstream consumers.

**Rationale**:
- A parsing library should be fast, predictable, and embeddable
- Network dependencies break offline use, add latency, and introduce
  failure modes
- Downstream tools (plex_organizer) can layer database/LLM validation
  on top of hunch's output
- The layered architecture (see above) makes this separation clean

### D005: No rebulk port

**Status**: Decided, permanent

**Context**: guessit uses `rebulk`, a generic Python pattern-matching
engine with match chaining, conflict resolution, and rule-based
post-processing. rebulk is powerful but deeply Pythonic and complex.

**Decision**: Do not port rebulk. Use a simpler span-based architecture
with flat conflict resolution.

**Rationale**:
- rebulk's complexity is a maintenance burden even in guessit
- A flat `Vec<MatchSpan>` + sort-and-sweep conflict resolution is
  simpler, faster, and easier to reason about
- The TOML rule files give us the data-driven benefits of rebulk's
  configuration system without the engine complexity

---

## v0.1 Architecture (current)

```
Input string
  │
  ├─ 1. Property matchers scan input, produce Vec<MatchSpan>
  │     ├─ ValuePattern matchers (fancy_regex + canonical values)
  │     └─ Algorithmic matchers (episodes, title, release_group, date)
  │
  ├─ 2. Conflict resolution
  │     └─ Overlapping spans: higher priority wins, then longer wins
  │
  ├─ 3. Pruning heuristics (to be eliminated in v0.2 by tokenizer)
  │     ├─ prune_language_in_title_zone
  │     ├─ prune_early_source_duplicates
  │     ├─ prune_redundant_hd_tags
  │     ├─ prune_early_episode_details
  │     └─ prune_other_overlapping_release_group
  │
  ├─ 4. Post-processing
  │     ├─ Title extraction (largest unclaimed region before tech tokens)
  │     ├─ Episode title extraction
  │     ├─ Media type inference
  │     └─ Proper count computation
  │
  └─ 5. Build HunchResult → JSON
```

## v0.2 Architecture (planned — tokenizer + TOML + regex-only)

```
Input string
  │
  ├─ 1. Tokenize: split at separators (. - _ space), identify brackets
  │     → [Token { text, position, separator_type }]
  │
  ├─ 2. Anchor detection: classify tokens using TOML rules
  │     ├─ Exact lookups (HashMap from TOML [exact] sections)
  │     ├─ Regex patterns (regex crate only, linear-time, from TOML)
  │     └─ Algorithmic matchers (episodes, dates — still in Rust)
  │     → S01E02 = Episode, 720p = ScreenSize, mkv = Container
  │
  ├─ 3. Zone inference: anchors divide token stream
  │     → TITLE ZONE (before first anchor) | TECH ZONE (after)
  │
  ├─ 4. Context-sensitive classification
  │     → "French" in title zone = title word (ignored)
  │     → "French" in tech zone = Language
  │     → No prune_* functions needed
  │
  ├─ 5. Title = concatenation of unmatched title-zone tokens
  │
  └─ 6. Build HunchResult → JSON
```

### Key v0.2 changes from v0.1

| Aspect | v0.1 | v0.2 |
|--------|------|------|
| Pattern storage | Hardcoded Rust | TOML files (embedded) |
| Regex engine | `fancy_regex` (backtracking) | `regex` only (linear-time) |
| Word boundaries | Lookaround assertions | Token isolation (structural) |
| Disambiguation | 5 prune_* heuristics | Zone-based (structural) |
| Title extraction | Gap detection in byte offsets | Unmatched title-zone tokens |
| Security | Patterns audited as Rust code | ReDoS structurally impossible |

---

## File Organization

### v0.1 structure (current)

```
src/
├── lib.rs                   # Public API
├── main.rs                  # CLI
├── pipeline.rs              # Orchestration + prune_* heuristics
├── hunch_result.rs          # Output type
├── options.rs               # Configuration
├── matcher/
│   ├── span.rs              # MatchSpan + Property enum
│   ├── engine.rs            # Conflict resolution
│   └── regex_utils.rs       # ValuePattern (fancy_regex + value)
└── properties/              # ~20 matcher modules (hardcoded patterns)
    ├── title.rs             # Title extraction (algorithmic)
    ├── episodes.rs          # Episode parsing (algorithmic)
    ├── release_group.rs     # Release group (positional)
    ├── date.rs              # Date parsing (algorithmic)
    ├── video_codec.rs       # Hardcoded ValuePattern lists
    ├── audio_codec.rs       # Hardcoded ValuePattern lists
    ├── ...                  # ~15 more property modules
    └── mod.rs
```

### v0.2 target structure

```
src/
├── lib.rs                   # Public API
├── main.rs                  # CLI
├── pipeline.rs              # Orchestration (no more prune_*)
├── hunch_result.rs          # Output type
├── options.rs               # Configuration
├── tokenizer.rs             # NEW: input → token stream + zones
├── matcher/
│   ├── span.rs              # MatchSpan + Property enum
│   ├── engine.rs            # Conflict resolution
│   └── rule_loader.rs       # NEW: generic TOML → matcher engine
└── properties/
    ├── title.rs             # Simplified: title = unmatched title-zone tokens
    ├── episodes.rs          # Episode parsing (algorithmic, regex-only)
    ├── release_group.rs     # Release group (positional)
    ├── date.rs              # Date parsing (algorithmic, regex-only)
    └── mod.rs               # Wires TOML-driven + Rust matchers

rules/                       # NEW: data-driven pattern definitions
├── video_codec.toml
├── audio_codec.toml
├── source.toml
├── screen_size.toml
├── container.toml
├── edition.toml
├── other.toml
├── language.toml
├── subtitle_language.toml
├── streaming_service.toml
├── country.toml
├── audio_profile.toml
├── video_profile.toml
├── color_depth.toml
└── frame_rate.toml
```

---

## Security Model

### v0.1 (current)

- All patterns are hardcoded in Rust source — no external data files
- `fancy_regex` is used for lookaround (backtracking possible)
- All patterns are reviewed as code changes
- No `unsafe`, no FFI, no file I/O, no network
- ReDoS risk is low: patterns are authored by maintainers, not user input

### v0.2 (planned)

- TOML rule files embedded at compile time — no runtime file access
- `regex` crate only — linear-time matching guaranteed, ReDoS structurally
  impossible even for malicious patterns
- Schema-validated at load time (max pattern length, valid property names)
- Validated by test suite before any release
- `fancy_regex` dependency removed entirely
- No `unsafe`, no FFI, no file I/O, no network
