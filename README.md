# 🔍 Hunch

**A Rust port of Python's [guessit](https://github.com/guessit-io/guessit)
for extracting media metadata from filenames.**

> ⚠️ **Work in progress.** Hunch currently passes **57.4%** of guessit's own
> 1,309-case test suite. Core properties like video codec, container, source,
> year, and screen size are 96–100% accurate, but title extraction and episode
> title inference are still maturing. See
> [COMPATIBILITY.md](COMPATIBILITY.md) for the full breakdown.

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

Hunch is a port of guessit. All 39+ of guessit's properties are
implemented. We validate against guessit's own YAML test suite:

| | guessit (Python) | hunch (Rust) |
|---|---|---|
| Overall pass rate | 100% (by definition) | **57.4%** (751 / 1,309) |
| Properties implemented | 39 | 42 |
| Properties at 90%+ | 39 | 23 |
| Properties at 100% | 39 | 11 |

**Where hunch matches guessit** (96–100% accuracy):
year, video_codec, container, source, screen_size, crc32, color_depth,
streaming_service, bonus, film, aspect_ratio, size, edition,
episode_details, version, frame_rate, episode_count, season_count.

**Where hunch diverges** (<70% accuracy):
episode_title (62%), language (67%), video_profile (57%),
bonus_title (62%), alternative_title (0%).

For per-property breakdowns, per-file results, and known gaps,
see **[COMPATIBILITY.md](COMPATIBILITY.md)**.

## Design

Hunch does **not** port guessit's `rebulk` engine. Instead it uses a
simpler **span-based architecture**:

1. **Match** — 30 property matchers scan the input independently and
   produce `MatchSpan`s (start, end, property, value) with priorities.
2. **Resolve** — Overlapping spans are resolved by priority, then by
   length (longer matches win ties).
3. **Extract** — Title is inferred from the largest unclaimed region
   before the first technical property.

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Pre-process: strip path, extract extension
  ├─ 2. Run 30 property matchers → Vec<MatchSpan>
  ├─ 3. Resolve conflicts (priority, then length)
  ├─ 4. Extract title from unclaimed leading region
  ├─ 5. Infer media type (episode vs movie)
  └─ 6. Build JSON output (BTreeMap)
```

## Project Structure

```
src/
├── lib.rs              # Public API: parse()
├── main.rs             # CLI binary
├── guess.rs            # GuessResult type + JSON serialization
├── options.rs          # Configuration
├── pipeline.rs         # Orchestration: matchers → resolve → extract
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (42 variants)
│   ├── engine.rs       # Conflict resolution
│   └── regex_utils.rs  # ValuePattern helper
└── properties/         # 30 property matcher modules
    ├── title.rs, episodes.rs, year.rs, version.rs, ...
    └── mod.rs          # PropertyMatcher trait

tests/
├── integration.rs      # 27 hand-written end-to-end tests
├── guessit_regression.rs # 22 regression suites + compatibility report
├── helpers/mod.rs      # Custom YAML fixture parser
└── fixtures/           # Copied from guessit (self-contained)
    ├── movies.yml, episodes.yml, various.yml
    └── rules/          # 19 per-property test files

benches/
└── parse.rs            # Criterion benchmarks
```

## License

MIT
