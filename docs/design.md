# Design — Hunch

> Mission, principles, architecture, and key decisions for
> contributors and maintainers.

---

## Mission

Hunch is a media filename parser built on Rust — not a port of
guessit, but a new tool with different goals.

guessit is a mature Python library with deep coverage of legacy
release conventions. Hunch respects that lineage but doesn't try
to replicate its outcomes. Instead, hunch is built for the future:

- **Match most of guessit's capabilities, not all its outputs.**
  guessit's test suite encodes years of edge cases, some of which
  reflect conventions that no longer exist or decisions we disagree
  with. Hunch aims for high coverage of real-world filenames, not
  test-for-test parity with guessit.

- **Evolve from real-world testing, not from a frozen fixture.**
  Hunch's test fixtures are living documents. When a real-world
  filename breaks expectations, the fixture grows. When a pattern
  turns out to be wrong, the fixture changes. Tests reflect what
  hunch *should* do, not what guessit *did* do.

- **Build for the future, not the past.** Reasonable backward
  compatibility matters, but it doesn't override correctness.
  When new evidence shows a better interpretation, hunch adopts
  it — with clear versioning and changelogs so users can adapt.

- **Rust as a platform choice, not a language preference.** Rust
  enables compile-time safety, single-binary deployment, and
  linear-time regex guarantees. These aren't nice-to-haves —
  they're structural advantages that shape the design (P2).

---

## Principles

Foundational beliefs that drive every design decision.

### P1: Predictable behavior

Same input, same output. Always.

Hunch is a deterministic function. Given the same filename, path, and
sibling context, it always produces the same result. When it can't be
confident, it says so honestly rather than guessing silently. Users
should always be able to understand *why* hunch produced a given
result and *what to do* when it's wrong.

A confident wrong answer is worse than an honest "I'm not sure."

### P2: Compile-time safety

Correctness is enforced before shipping, not at runtime.

No `unsafe` code, no runtime file loading, no external dependencies
at runtime. If it compiles, the binary is self-contained and the
regex engine is guaranteed linear-time. Runtime surprises are
structurally eliminated.

---

## Design Decisions

Each decision is derived from one or both principles. Some decisions
establish boundaries (library/CLI, data/code, engine/human); others
are standalone constraints.

### D1: Pure library, I/O-free (P1, P2)

The library (`hunch::hunch()`, `Pipeline::run()`) is a pure function:
filename, path, and sibling context in, metadata out. No network, no
database, no ML, no filesystem I/O. Deterministic by construction (P1).

The CLI is the only component that touches the filesystem: reading
directories for `--batch` and `--context`, printing to stdout/stderr.
This keeps the library embeddable, testable, and safe to call from
any context.

### D2: Vocabulary in TOML, logic in Rust (P1, P2)

Simple pattern recognition ("is `x264` a codec?") lives in TOML
lookup tables — readable, auditable, contributors can add patterns
without deep Rust knowledge:

```toml
[exact]
x264 = "H.264"
hevc = "H.265"
```

Control flow (episode parsing, date detection, title extraction)
lives in Rust. The boundary is: if it's a vocabulary lookup, it's
TOML; if it needs branching or state, it's Rust.

### D3: Single self-contained binary (P2)

All TOML rules are `include_str!`-ed at compile time. No runtime
config files, no data directories. `cargo install hunch` gives you
everything.

### D4: Linear-time regex only (P2)

The `regex` crate (not `fancy_regex`) ensures linear-time matching.
The tokenizer eliminates the need for lookaround by isolating tokens
before matching. ReDoS is structurally impossible.

### D5: Zero `unsafe` (P2)

The entire codebase is safe Rust. No `unsafe`, no FFI.

### D6: Dumb engine, smart context (P1)

The Rust engine is a simple pattern matcher — TOML lookups and regex,
nothing clever. When the engine can't decide (is "French" a language
or a title word?), it defers to **context**:

- **Directory structure:** `tv/`, `movie/`, `Season 1/` in the path
- **Sibling filenames:** cross-file invariance reveals titles
- **Token position:** relative to unambiguous anchors (SxxExx, 1080p)

Prefer context over heuristics. Heuristics are fragile; context is
structural. When context is also insufficient, surface the ambiguity
to the human (D7).

### D7: Surface ambiguity to the user (P1)

When multiple valid interpretations exist and neither the engine nor
available context can distinguish them, hunch is transparent about
the uncertainty rather than guessing.

Mechanism:
- **Confidence** drops when conflicting signals exist.
- **Conflicts** are surfaced in the output so callers can inspect
  and resolve them.
- The CLI prints **actionable hints** suggesting how the user can
  provide structural disambiguation.

**Example:** `Detective.Conan.Movie.10.mkv` — "Movie" followed by
a number is genuinely ambiguous. It could be the 10th movie in a
franchise (common in CJK media where movies and TV series coexist
in the same directory) or episode 10 of something with "Movie" in
the title. Adding a "if preceded by Movie, treat as Film" rule
just replaces one wrong guess with a different wrong guess. The
correct response: lower confidence, surface the conflict, let the
user organize files into `movie/` or `tv/` for unambiguous
classification.

Known ambiguity patterns:

| Pattern | Interpretations | User resolution |
|---|---|---|
| `Movie N` | Film #N vs. episode N | Organize into `movie/` or `tv/` |
| `YYYY` in title position | Year vs. title word | Cross-file context |
| Bare number after title | Episode vs. version vs. part | Use structural markers |
| CJK mixed collections | Movies + TV in same dir | Directory structure |

The escalation chain (D6 → D7):
```
Unambiguous pattern (S01E02)  →  High confidence, engine decides
Context resolves it (tv/ dir) →  High confidence, context decides
Heuristic guess (bare number) →  Medium confidence, engine guesses
Genuine ambiguity (Movie 10)  →  Low confidence, human decides
```

---

## Architecture Overview

The problem decomposes into three sub-problems:

| Sub-problem | Approach | Example |
|---|---|---|
| **Recognition** — is `x264` a codec? | TOML lookup tables + regex | `x264 → H.264` |
| **Disambiguation** — is `French` a language or title? | Zone inference | Position relative to tech anchors |
| **Extraction** — where does the title end? | Context-driven (gaps + siblings) | Unclaimed text between matches |

### Pipeline

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Tokenize     → ["The", "Walking", "Dead", "S05E03", "720p", ...]
  ├─ 2. Zone map     → title_zone: [0..3], tech_zone: [3..end]
  │
  ══ PASS 1: Match & Resolve ══════════════════════════════════
  ├─ 3. TOML rules   → match tokens against 20 rule files
  ├─ 4. Algorithmic  → episodes, dates, years (Rust code)
  ├─ 5. Conflicts    → priority + length tiebreaking
  ├─ 6. Zone filter  → suppress ambiguous matches in title zone
  │
  ══ PASS 2: Positional Extraction ════════════════════════════
  ├─ 7. Release group → "-DEMAND" (uses resolved match positions)
  ├─ 8. Title        → "The Walking Dead" (unclaimed title zone)
  ├─ 9. Episode title, media type, confidence
  │
  └─ 10. HunchResult → JSON
```

**Why two passes?** Release group and title extraction need to know
what's already been claimed by tech properties. Pass 1 resolves all
tech matches; Pass 2 uses those positions for structural extraction.

---

## Implementation Details

### No rebulk port

guessit uses `rebulk`, a Python pattern engine with chains, rules,
tags, formatters, handlers, and validators (~15 features). Hunch's
TOML engine has 5 features and expresses ~90% of rebulk's patterns:

| Feature | Rebulk | Hunch |
|---|---|---|
| Exact lookup | `string_match()` | `[exact]` HashMap |
| Regex | `regex_match()` | `[[patterns]]` |
| Side effects | Callbacks + chains | `side_effects = [...]` |
| Neighbor checks | `previous`/`next` callbacks | `not_before`/`not_after` |
| Zone scoping | Rule tags + validators | `zone_scope` field |

The remaining 10% (multi-span patterns with arbitrary gaps) are edge
cases where cross-file context is the principled solution, not
more clever Rust code.

### Zone map — anchors first, matching second

The v0.1 pipeline matched everything, then pruned mistakes. This lost
information (a pruned match can't be restored as title content).

The zone map inverts the flow:
1. Find unambiguous **anchors** (SxxExx, 1080p, x264, BluRay)
2. Derive **zones** (title zone = before first anchor, tech zone = after)
3. Match with **zone awareness** (ambiguous tokens suppressed in title zone)

**Anchor confidence tiers:**

| Tier | Examples | Confidence |
|---|---|---|
| 1: Structural | `S01E02`, `1080p`, `.mkv` | Always unambiguous |
| 2: Tech vocab | `x264`, `BluRay`, `DTS` | Almost always unambiguous |
| 3: Positional | Year-like numbers (1920–2039) | Ambiguous — use context |

Tier 1 and 2 anchors are unambiguous (D6). Tier 3 tokens like
year-like numbers are genuinely ambiguous — "2001" in
"2001.A.Space.Odyssey.1968" is title, not year. The engine uses basic
positional heuristics as a fallback, but the principled solution is
**cross-file context**: if siblings all share "2001" in the same
position, it's title. Confidence scoring signals when context
would help.

### Cross-file context

The title is the **invariant text** across sibling files:

```
(BD)十二国記 第01話「月の影 影の海　一章」(1440x1080 x264-10bpp flac).mkv
(BD)十二国記 第02話「月の影 影の海　二章」(1440x1080 x264-10bpp flac).mkv
     ^^^^^^^^ invariant = title
              ^^^^  variant = episode number
                    ^^^^^^^^^^^^^^^^ variant = episode title
```

**Algorithm:**
1. Run Pass 1 on target + each sibling
2. Find unclaimed text gaps (regions between resolved matches)
3. Compute common prefix of corresponding gaps → title
4. Run Pass 2 with resolved title

**Hard boundary:** The library takes sibling filenames as `&[&str]` —
caller-provided data, not filesystem access. The CLI reads directories
via `--context` and `--batch`.

### Confidence scoring

`HunchResult::confidence()` returns `High | Medium | Low`:

| Signal | Confidence |
|---|---|
| Cross-file context + title found | High |
| ≥3 tech anchors + title ≥2 chars | High |
| Some anchors, reasonable title | Medium |
| Conflicting interpretations (D7) | Low |
| No title or title ≤1 char | Low |

Confidence is honest about uncertainty (P1). When the engine can't
decide, it says so — and the CLI suggests using `--context` to
provide structural context instead of guessing harder.

When hunch detects conflicting interpretations (D7), it:

1. **Still produces a result** — picks the most common interpretation
   as the default (a best-effort answer is better than none).
2. **Drops confidence to Low** — signals that the result is uncertain.
3. **Surfaces conflicts** — includes machine-readable conflict
   descriptions so callers can decide how to handle them.

---

## TOML Rule Format

```toml
property = "video_codec"
zone_scope = "unrestricted"   # "unrestricted" | "tech_only" | "after_anchor"

[exact]                       # Case-insensitive exact token lookups
x264 = "H.264"
hevc = "H.265"

[exact_sensitive]              # Case-sensitive (ambiguous short tokens)
NZ = "NZ"

[[patterns]]                   # Regex patterns
match = '(?i)^[xh][-.]?265$'
value = "H.265"

[[patterns]]                   # Capture templates
match = '(?i)^(\d{3,4})x(\d{3,4})$'
value = "{2}p"                # Capture group 2 → "1080p"

[[patterns]]                   # Side effects
match = '(?i)^dvd[-. ]?rip$'
value = "DVD"
side_effects = [{ property = "other", value = "Rip" }]

[[patterns]]                   # Neighbor constraints
match = '(?i)^hd$'
value = "HD"
not_before = ["tv", "dvd", "cam", "rip"]
# Also: not_after, requires_after, requires_before, requires_nearby
```

Match order: case-sensitive exact → case-insensitive exact → regex
(first match wins).

---

## Module Map

```
src/
├── lib.rs              # Public API: hunch(), hunch_with_context()
├── main.rs             # CLI binary (behind "cli" feature)
├── hunch_result.rs     # HunchResult + Confidence + typed accessors
├── tokenizer.rs        # Input → TokenStream (separators, brackets)
├── zone_map.rs         # Anchor detection + zone boundaries
├── pipeline/
│   ├── mod.rs          # Two-pass orchestration
│   ├── matching.rs     # Token-level TOML rule matching
│   ├── context.rs      # Cross-file invariance detection
│   ├── token_context.rs # Structure-aware disambiguation
│   └── zone_rules.rs   # Post-match zone filtering
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (49 variants)
│   ├── engine.rs       # Conflict resolution (priority + length)
│   ├── rule_loader.rs  # TOML → RuleSet parser
│   └── regex_utils.rs  # BoundedRegex (strips lookarounds)
└── properties/         # 31 property matcher modules
    ├── episodes/       # S01E02, 1x03, ranges, anime (algorithmic)
    ├── title/          # Title extraction (algorithmic)
    ├── release_group/  # Positional heuristics (algorithmic)
    └── ...             # year, date, language, etc.

rules/                  # 20 TOML data files (compile-time embedded)
tests/                  # Integration + regression + constraint tests
benches/                # Criterion benchmarks
```

---

## Adding a New Property

1. Create `rules/<name>.toml` with `property`, `[exact]`, `[[patterns]]`.
2. Add a `LazyLock<RuleSet>` static in `pipeline/mod.rs`.
3. Register it in `toml_rules` with property + priority + segment scope.
4. Add `Property::YourProp` variant to `matcher/span.rs`.
5. Add integration tests.
6. Only create `properties/<name>.rs` if the property needs algorithmic
   logic that tokens can't express.

---

## Conflict Resolution

1. **Priority tiers:** Extension (10) > known tokens (0) > weak (-1/-2).
   Directory matches get a -5 penalty.
2. **Overlap:** Higher priority wins; ties broken by longer span.
3. **Multi-value:** Episode, Language, SubtitleLanguage, Other, Season,
   Disc support multiple values (serialized as JSON arrays).

---

## Security Model

- TOML rules embedded at compile time — no runtime file I/O
- `regex` crate only — linear-time, ReDoS structurally impossible
- Zero `unsafe`, zero FFI, zero network
- All patterns reviewed as code changes (TOML files are versioned)
- Bracket depth guard (max 3) prevents stack overflow from malicious input
