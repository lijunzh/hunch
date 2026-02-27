# 🔍 Hunch

> 🤖 **This repository is entirely AI-generated with human guidance.** All code,
> tests, and documentation were produced by AI under the direction of a human
> collaborator.

**A Rust port of Python's [guessit](https://github.com/guessit-io/guessit)
for extracting media metadata from filenames.**

> ⚠️ **Work in progress.** Hunch currently passes **79.1%** of guessit's own
> 1,309-case test suite (1,036 / 1,309). All 49 guessit properties are
> implemented (3 intentionally diverged). Core properties like video codec,
> screen size, source, audio codec, edition, and year are 96–100% accurate.
> Title (91%), episode (90%), and 40+ other properties are steadily improving.
> All regex is linear-time via the `regex` crate (ReDoS-immune).
> See [COMPATIBILITY.md](COMPATIBILITY.md) for the full breakdown.

Hunch extracts title, year, season, episode, resolution, codec, language,
and 40+ other properties from messy media filenames — the same job guessit
does, rewritten from scratch for Rust.

## Quick Start

```bash
cargo add hunch
```

### As a library

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    println!("{:#?}", result);
    // GuessResult {
    //   title: Some("The Walking Dead"),
    //   season: Some(5),
    //   episode: Some(3),
    //   screen_size: Some("720p"),
    //   source: Some("Blu-ray"),
    //   video_codec: Some("H.264"),
    //   release_group: Some("DEMAND"),
    //   container: Some("mkv"),
    //   media_type: Episode,
    //   ...
    // }
}
```

### As a CLI tool

```bash
$ hunch "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
{
  "container": "mkv",
  "episode": 3,
  "release_group": "DEMAND",
  "screen_size": "720p",
  "season": 5,
  "source": "Blu-ray",
  "title": "The Walking Dead",
  "type": "episode",
  "video_codec": "H.264"
}
```

## guessit Compatibility

Hunch is a port of guessit. All 49 of guessit's properties are
implemented (3 intentionally diverged — see COMPATIBILITY.md). We validate
against guessit's own YAML test suite:

| | guessit (Python) | hunch (Rust) |
|---|---|---|
| Overall pass rate | 100% (by definition) | **79.1%** (1,036 / 1,309) |
| Properties implemented | 49 | 49 (3 diverged) |
| Properties at 90%+ | 49 | 30 |
| Properties at 100% | 49 | 16 |

**Where hunch matches guessit** (96–100% accuracy):
year, video_codec, container, source, screen_size, audio_codec, crc32,
color_depth, streaming_service, bonus, film, aspect_ratio, size, edition,
episode_details, version, frame_rate, episode_count, season_count,
proper_count, date, disc, episode_format, week, video_api.

**Where hunch is developing** (70–95%):
title (91%), release_group (89%), episode (90%), season (94%),
audio_channels (95%), language (85%), other (85%),
episode_title (72%), subtitle_language (77%),
audio_profile (85%).

For per-property breakdowns, per-file results, and known gaps,
see **[COMPATIBILITY.md](COMPATIBILITY.md)**.

## Design

Hunch does **not** port guessit's `rebulk` engine. Instead it uses a
**tokenizer-first, TOML-driven architecture**:

1. **Tokenize** — Split input on separators (. - _ space), extract
   extension, detect brackets, handle path segments.
2. **Match** — TOML rule files (20 files, embedded at compile time) match
   tokens via exact lookup, regex, and capture-group templates.
   Algorithmic matchers handle complex patterns (episodes, title, dates).
3. **Resolve** — Overlapping spans resolved by priority, then length.
4. **Disambiguate** — Zone-based rules suppress false positives
   (e.g., language words in title zone).
5. **Extract** — Title inferred from largest unclaimed region before
   first technical property. Episode title, media type, proper count
   computed as derived values.

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Tokenize → TokenStream (segments, tokens, extension)
  ├─ 2. Match: TOML rules + algorithmic matchers → Vec<MatchSpan>
  ├─ 3. Resolve conflicts (priority, then length)
  ├─ 4. Zone-based disambiguation
  ├─ 5. Extract title, episode_title, media_type
  └─ 6. Build HunchResult (BTreeMap<Property, Vec<String>>)
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design, decision log,
and module map.

## Project Structure

```
src/
├── lib.rs              # Public API: hunch(), hunch_with()
├── main.rs             # CLI binary (clap)
├── hunch_result.rs     # HunchResult type + JSON serialization
├── options.rs          # Configuration
├── zone_map.rs         # ZoneMap: structural filename zone analysis
├── tokenizer.rs        # Input → TokenStream (segments, brackets, extension)
├── pipeline/           # Orchestration: tokenize → zones → match → resolve
│   ├── mod.rs          # Pipeline struct + run/match_all
│   ├── zone_rules.rs   # Post-match zone disambiguation (7 rules)
│   └── proper_count.rs # PROPER/REPACK count computation
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (55 variants)
│   ├── engine.rs       # Conflict resolution
│   ├── regex_utils.rs  # BoundedRegex + boundary checking
│   └── rule_loader.rs  # TOML rule engine: exact + regex + templates + zone_scope
└── properties/         # 31 property matcher modules
    ├── title/          # Title extraction (mod.rs + clean.rs + secondary.rs)
    ├── episodes/       # Season/episode (mod.rs + patterns.rs + tests.rs)
    ├── release_group/ # Positional release group heuristics
    │   ├── mod.rs       # Regex patterns + matching logic
    │   └── known_tokens.rs # Token exclusion list + helpers
    └── ...             # year, date, source, language, etc.

rules/                  # 20 TOML data files (compile-time embedded)
├── source.toml         # Source patterns with Rip/Screener side_effects
├── other.toml          # Other flags (unambiguous)
├── other_positional.toml # Position-dependent Other (zone_scope=tech_only)
└── ...                 # video_codec, audio_codec, language, edition, etc.
```
tests/
├── integration.rs      # 32 hand-written end-to-end tests
├── guessit_regression.rs # 22 regression suites + compatibility report
└── fixtures/           # Copied from guessit (self-contained)
```

## License

MIT
