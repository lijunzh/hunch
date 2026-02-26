# Hunch v0.2.0 — The TOML Engine

This release is a **ground-up architectural redesign** of hunch's matching
engine. The parser is faster, safer, more accurate, and significantly
simpler to extend.

## The Big Picture

Hunch v0.1 was a direct port of Python's guessit — regex patterns running
against raw input strings, using `fancy_regex` for lookaround assertions.
It worked, but the architecture had three problems:

1. **Security**: `fancy_regex` enables backtracking, making ReDoS possible
2. **Complexity**: 21 legacy matchers with ~1,400 lines of regex patterns
3. **Extensibility**: adding a new codec meant writing Rust code

v0.2 solves all three.

## What Changed

### TOML-driven rule engine

Property patterns now live in **declarative TOML files** embedded at compile
time. Adding a new video codec is one line:

```toml
# rules/video_codec.toml
av1 = "AV1"
```

The engine supports:
- **Exact lookups**: case-sensitive and case-insensitive hash maps
- **Regex patterns**: with `{N}` capture-group templates (`1920x1080` → `1080p`)
- **Side effects**: one match → multiple properties (`DVDRip` → Source:DVD + Other:Rip)
- **Neighbor constraints**: `not_before`, `not_after`, `requires_after` for context-aware matching

19 TOML rule files now define the vocabulary for all non-algorithmic properties.

### `fancy_regex` removed entirely

All regex matching uses the standard `regex` crate — **linear-time, ReDoS-immune
by construction**. Lookaround assertions are replaced by:
- Token isolation (the tokenizer provides word boundaries for free)
- Post-match `BoundarySpec` checks (character class guards)
- TOML neighbor constraints (`not_before = ["tv", "dvd"]`)

Test execution time dropped ~50% from eliminating `fancy_regex` compilation overhead.

### Path-segment tokenizer

The tokenizer now processes **all path segments** (directories + filename),
not just the filename. Each segment is tagged with `SegmentKind` (Directory
vs Filename), and each TOML rule set declares a `SegmentScope`:

- **`AllSegments`**: unambiguous tech tokens (XviD, 720p, AAC) are matched
  in directory names too, recovering metadata that v0.1 could see
- **`FilenameOnly`**: ambiguous tokens (HD, TV, DV) skip directories to
  avoid false positives

Directory matches receive a priority penalty so filename matches always win.

### Title extraction overhaul

Three structural improvements to how titles are extracted:

1. **Boundary detection**: structural separators (` - `, `--`, `()`) now
   stop the title at subtitle/director content instead of consuming it
2. **Single-word handling**: bare inputs like `"tv"` are treated as titles,
   not property matches
3. **Deepest-first directory walk**: title fallback now prefers directories
   closest to the filename

### New properties

| Property | Description | Accuracy |
|----------|-------------|:--------:|
| `absolute_episode` | Anime-style absolute numbering (Episode 366) | 90% |
| `film_title` | Franchise title from `-fNN-` markers (James Bond) | 87.5% |
| `alternative_title` | Content after title boundary separators | 43.8% |

### Legacy matcher migration

4 legacy matchers fully retired to TOML-only:
- `frame_rate.rs` → `rules/frame_rate.toml`
- `container.rs` → `rules/container.toml` + pipeline PATH A
- `screen_size.rs` → `rules/screen_size.toml`
- `audio_codec.rs` → `rules/audio_codec.toml` + `rules/audio_channels.toml`

`language.rs` gutted from 213 to 95 lines — TOML handles tokens, Rust
handles only bracket/brace multi-language codes (`[ENG+RU+PT]`).

8 additional modules had dead `ValuePattern` code removed.

**Net: -827 lines of code removed.**

## Accuracy

| | v0.1.2 | v0.2.0 | Delta |
|---|:---:|:---:|:---:|
| **Overall** | **75.1%** (983) | **77.3%** (1,012) | **+29** |
| video_codec | 94.0% | 98.6% | +24 |
| screen_size | 93.7% | 98.4% | +20 |
| audio_codec | 91.2% | 97.8% | +15 |
| subtitle_language | 49.4% | 77.8% | +23 |
| title | 84.6% | 87.9% | +35 |
| language | 77.5% | 84.5% | +10 |
| absolute_episode | 0% | 90.0% | new |
| film_title | 0% | 87.5% | new |
| alternative_title | 0% | 43.8% | new |

12 properties at 100%. 17 properties above 95%. 5 properties above 90% that
were below 95% in v0.1.

## Dependencies

| Crate | Purpose | Change |
|-------|---------|--------|
| `regex` | Pattern matching (linear-time) | Kept |
| `fancy-regex` | Lookaround fallback | **Removed** |
| `serde` + `serde_json` | JSON output | Kept |
| `clap` | CLI parsing | Kept |
| `toml` | Rule file parsing | Kept |

## Road Ahead

### Near-term (v0.2.1)

- ✅ **`bit_rate` property** — detect `NNNKbps` and `NN.NMbps` patterns.
- ✅ **`episode_format` property** — detect "Minisode" / "Minisodes".
- ✅ **`week` property** — detect "Week NN" patterns in episode context.
- ✅ **Title hardening** — "The 100" pattern, trailing Ep/Episode/bonus
  markers, trailing punctuation.
- ✅ **Release group** — language prefix stripping (HUN-nIk, TrueFrench-).
- ⬜ **ZoneMap architecture** — anchor detection + zone-aware matching
  to replace match-then-prune disambiguation. See ARCHITECTURE.md D006.

### Near-term (v0.2.x)

- **Subtitle language** (77.8% → 90%+) — migrate remaining complex
  patterns from the 406-line legacy matcher
- **Release group** edge cases (89.1% → 93%+)
- **Episode title** improvements (70.6% → 80%+)

### Intentionally omitted guessit properties

- **`audio_bit_rate` / `video_bit_rate`** — hunch uses a single
  `bit_rate` property. Users already have codec properties to determine
  which stream the bitrate refers to.
- **`mimetype`** — trivially derived from `container`. Redundant.

### Medium-term (v0.3)

- **Retire `other.rs` and `source.rs`** — the last two legacy matchers
  using `ValuePattern` (now standard `regex`, but still raw-string scanners)
- **Remove `regex_utils.rs`** entirely once all matchers are TOML or
  algorithmic-only
- **Absolute episode** improvements for anime formats
- **Alternative title** from parenthesized content (43.8% → 80%+)

### Long-term

- **80%+ overall accuracy** — pattern grinding on remaining edge cases
- **Layer 2**: optional TMDB/TVDB integration for title validation
  (separate crate, not in core `hunch`)
- **crates.io publish** and stable API
- **Benchmark suite** — quantitative comparison with Python guessit

## Breaking Changes

- `fancy-regex` is no longer a dependency. If you were depending on it
  transitively through hunch, you'll need to add it directly.
- The `HunchResult` now lowercases language/subtitle_language values for
  case-insensitive deduplication. Values like `"French"` remain title-cased,
  but duplicate entries from multiple matchers (e.g., `"nl"` and `"NL"`)
  are now deduplicated.

## Thank You

This release represents a complete rethink of how media filename parsing
should work: data-driven rules over hardcoded patterns, structural
disambiguation over regex heuristics, and linear-time safety by design.

The architecture is now clean enough that contributors can add new codecs,
editions, or streaming services by editing a TOML file — no Rust knowledge
required.
