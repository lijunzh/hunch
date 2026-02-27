# 🔍 Hunch

> 🤖 **This repository is entirely AI-generated with human guidance.** All code,
> tests, and documentation were produced by AI under the direction of a human
> collaborator.

**A Rust port of Python's [guessit](https://github.com/guessit-io/guessit)
for extracting media metadata from filenames.**

> ⚠️ **Work in progress.** Hunch currently passes **80.0%** of guessit's own
> 1,309-case test suite (1,047 / 1,309). All 49 guessit properties are
> implemented (3 intentionally diverged). Core properties like video codec,
> screen size, source, audio codec, edition, and year are 96–100% accurate.
> Title (92%), release group (90%), episode (90%), and 40+ other properties
> are steadily improving. Uses a two-pass pipeline with post-resolution
> extraction. All regex is linear-time via the `regex` crate (ReDoS-immune).
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
| Overall pass rate | 100% (by definition) | **80.0%** (1,047 / 1,309) |
| Properties implemented | 49 | 49 (3 diverged) |
| Properties at 90%+ | 49 | 31 |
| Properties at 100% | 49 | 16 |

**Where hunch matches guessit** (96–100% accuracy):
year, video_codec, container, source, screen_size, audio_codec, crc32,
color_depth, streaming_service, bonus, film, aspect_ratio, size, edition,
episode_details, version, frame_rate, episode_count, season_count,
proper_count, date, disc, episode_format, week, video_api.

**Where hunch is developing** (70–95%):
title (92%), release_group (90%), episode (90%), season (94%),
audio_channels (95%), language (85%), other (85%),
episode_title (74%), subtitle_language (77%),
audio_profile (85%).

For per-property breakdowns, per-file results, and known gaps,
see **[COMPATIBILITY.md](COMPATIBILITY.md)**.

## Design

Hunch does **not** port guessit's `rebulk` engine. Instead it uses a
**tokenizer-first, two-pass, TOML-driven architecture**:

1. **Tokenize** — Split input on separators (. - _ space), extract
   extension, detect bracket groups, handle path segments.
2. **Zone map** — Detect structural anchors (SxxExx, 720p, x264) to
   establish title zone vs tech zone boundaries, per directory and filename.
3. **Pass 1: Match & Resolve** — TOML rule files (20 files, embedded at
   compile time) match tokens via exact lookup, regex, and templates.
   Algorithmic matchers handle complex patterns (episodes, dates).
   Conflicts resolved by priority, then length. Zone disambiguation.
4. **Pass 2: Extract** — Release group, title, and episode title run
   with access to resolved match positions from Pass 1.
5. **Result** — HunchResult with 49 typed property accessors.

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Tokenize → TokenStream (segments, tokens, brackets, extension)
  ├─ 2. Zone map → title zone, tech zone, year disambiguation
  │
  ═══ Pass 1: Tech Resolution ═════════════════════════════════
  ├─ 3. TOML rules + legacy matchers → Vec<MatchSpan>
  ├─ 4. Conflict resolution (priority, then length)
  ├─ 5. Zone disambiguation → resolved_tech_matches
  │
  ═══ Pass 2: Positional Extraction ═══════════════════════════
  ├─ 6. Release group (sees resolved positions)
  ├─ 7. Title, episode title, alternative title
  └─ 8. Build HunchResult
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
├── zone_map.rs         # ZoneMap: structural zone analysis (filename + per-dir)
├── tokenizer.rs        # Input → TokenStream (segments, brackets, extension)
├── pipeline/           # Two-pass orchestration
│   ├─ mod.rs          # Pipeline: Pass 1 (tech) → Pass 2 (positional)
│   ├─ zone_rules.rs   # Zone disambiguation (pre-RG + post-RG rules)
│   └─ proper_count.rs # PROPER/REPACK count computation
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (55 variants)
│   ├── engine.rs       # Conflict resolution
│   ├── regex_utils.rs  # BoundedRegex + boundary checking
│   └── rule_loader.rs  # TOML rule engine: exact + regex + templates + zone_scope
└── properties/         # 31 property matcher modules
    ├── title/          # Title extraction (mod.rs + clean.rs + secondary.rs)
    ├── episodes/       # Season/episode (mod.rs + patterns.rs + tests.rs)
    ├── release_group/  # Post-resolution release group extraction (Pass 2)
    │   ├── mod.rs       # Regex patterns + matching logic
    │   └── known_tokens.rs # Position-based validation + helpers
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
