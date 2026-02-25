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
│   • Exact-match hash lookups for known tokens               │
│   • Regex patterns for structured matches (S01E02, DTS-HD)  │
│   • Data-driven: patterns defined in TOML, embedded at      │
│     compile time via include_str!()                          │
│   • Offline, deterministic, fast (microseconds)              │
│   • Target: ~80% accuracy                                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Tokenizer + Zones (v0.2) — this crate              │
│                                                             │
│   • Split input into tokens at boundaries (. - _ space)     │
│   • Anchor tokens (S01E02, 720p) divide into zones:         │
│     TITLE ZONE | TECH ZONE                                  │
│   • Context-sensitive: "French" in title zone = title word,  │
│     "French" in tech zone = language                         │
│   • Eliminates all prune_* heuristics                        │
│   • Eliminates need for regex lookaround assertions          │
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

**Status**: Decided, implementing in v0.1

**Context**: We had ~486 regex patterns hardcoded across 20+ Rust files.
Adding a new codec meant editing Rust, recompiling, and navigating through
pattern-matching code mixed with detection logic.

**Decision**: Move simple property patterns into TOML rule files, embedded
at compile time via `include_str!()`. Keep complex algorithmic logic
(title extraction, episode parsing, release group heuristics) in Rust.

**Consequences**:
- Pattern definitions are readable and auditable in isolation
- The Rust engine becomes a generic rule loader + matcher
- Contributors can add patterns without deep Rust knowledge
- Single binary deployment preserved (TOML embedded at compile time)
- Regex validity checked by tests, not compiler (acceptable — same
  practical safety as current `LazyLock` + `unwrap()` approach)

### D002: `regex` crate only — no `fancy_regex` for TOML patterns

**Status**: Decided, implementing in v0.1

**Context**: We originally used `fancy_regex` for lookahead/lookbehind
assertions (`(?<![a-z])HDTV(?![a-z])`). This was necessary because Rust's
`regex` crate doesn't support lookaround. However, `fancy_regex` uses
backtracking, which makes it vulnerable to ReDoS (Regular Expression Denial
of Service) attacks from malicious patterns.

**Decision**: TOML-loaded patterns use the `regex` crate only (linear-time,
ReDoS-immune). Complex patterns that genuinely need lookaround stay as
hand-written Rust code using `fancy_regex` (audited, not user-editable).

**Why this is safe**: For v0.1, we use `\b` (word boundary) instead of
lookaround in TOML patterns. `\b` behaves slightly differently at
digit/punctuation boundaries, but this is acceptable because:

1. Most patterns are exact token matches (HashMap lookup, no regex at all)
2. The ~20% that need regex are simple keyword patterns where `\b` works
3. Edge cases where `\b` differs from lookaround (e.g., `DD+5.1`) stay
   in hand-written Rust
4. In v0.2, the tokenizer eliminates the need for word boundary assertions
   entirely — tokens are already isolated, so `"HDTV"` matches against the
   whole token, not a substring of a larger string

**Security boundary**: Nothing loaded from TOML can cause unbounded
computation. The `regex` crate guarantees linear-time matching structurally.

### D003: Tokenizer deferred to v0.2

**Status**: Decided

**Context**: The current architecture runs regex patterns across the entire
input string, then uses `prune_*` functions to remove false positives
(e.g., "French" in title position matched as a language). A tokenizer
would split the input first, establish title/tech zones, and eliminate
these heuristics.

**Decision**: Ship v0.1 with the current regex-over-full-string approach.
Add tokenizer in v0.2.

**Rationale**:
- v0.1 at ~75% accuracy is useful and shippable
- The tokenizer is a significant refactor of the matching layer
- TOML data-driven patterns (D001) can be done independently and will
  carry over to the tokenizer architecture
- The tokenizer will also eliminate the `fancy_regex` dependency for
  the remaining hand-written patterns (D002), since matching against
  isolated tokens doesn't need lookaround

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
  ├─ 1. Load rules from embedded TOML files
  │     ├─ Exact lookups (HashMap<token, value>)
  │     └─ Regex patterns (regex crate, linear-time)
  │
  ├─ 2. Property matchers scan input, produce Vec<MatchSpan>
  │     ├─ TOML-driven matchers (generic engine)
  │     └─ Rust-coded matchers (episodes, title, release_group, date)
  │
  ├─ 3. Conflict resolution
  │     └─ Overlapping spans: higher priority wins, then longer wins
  │
  ├─ 4. Pruning heuristics (to be eliminated in v0.2 by tokenizer)
  │     ├─ prune_language_in_title_zone
  │     ├─ prune_early_source_duplicates
  │     ├─ prune_redundant_hd_tags
  │     ├─ prune_early_episode_details
  │     └─ prune_other_overlapping_release_group
  │
  ├─ 5. Post-processing
  │     ├─ Title extraction (largest unclaimed region before tech tokens)
  │     ├─ Episode title extraction
  │     ├─ Media type inference
  │     └─ Proper count computation
  │
  └─ 6. Build HunchResult → JSON
```

## v0.2 Architecture (planned)

```
Input string
  │
  ├─ 1. Tokenize: split at separators (. - _ space), identify brackets
  │     → [Token { text, position, separator_type }]
  │
  ├─ 2. Anchor detection: classify tokens using TOML rules + regex
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

---

## File Organization

### v0.1 target structure

```
src/
├── lib.rs                   # Public API
├── main.rs                  # CLI
├── pipeline.rs              # Orchestration
├── hunch_result.rs          # Output type
├── options.rs               # Configuration
├── matcher/
│   ├── span.rs              # MatchSpan + Property enum
│   ├── engine.rs            # Conflict resolution
│   └── rule_loader.rs       # Generic TOML → matcher engine
└── properties/
    ├── title.rs             # Title extraction (Rust, algorithmic)
    ├── episodes.rs          # Episode parsing (Rust, algorithmic)
    ├── release_group.rs     # Release group (Rust, positional)
    ├── date.rs              # Date parsing (Rust, algorithmic)
    └── mod.rs               # Wires TOML-driven + Rust matchers

rules/                       # Data-driven pattern definitions
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

### TOML rule files (data layer)

- Embedded at compile time — no runtime file access
- `regex` crate only — linear-time matching guaranteed
- Schema-validated at load time (max pattern length, valid property names)
- Validated by test suite before any release
- Cannot cause unbounded computation, panics, or code execution

### Rust code (logic layer)

- Standard Rust safety guarantees (memory, type, thread safety)
- `fancy_regex` allowed only in hand-written, audited code
- Complex patterns are few (~30) and reviewed as code changes
- No `unsafe`, no FFI, no file I/O, no network
