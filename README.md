# 🔍 Hunch

> 🤖 **This repository is entirely AI-generated with human guidance.** All code,
> tests, and documentation were produced by AI under the direction of a human
> collaborator.

**A Rust port of Python's [guessit](https://github.com/guessit-io/guessit)
for extracting media metadata from filenames.**

> ⚠️ **Work in progress.** Hunch currently passes **75.1%** of guessit's own
> 1,309-case test suite (983 / 1,309). Core properties like video codec,
> container, source, year, and screen size are 96–100% accurate. Title
> extraction, episode parsing, and release group detection are steadily
> improving. See [COMPATIBILITY.md](COMPATIBILITY.md) for the full breakdown.

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
implemented. We validate against guessit's own YAML test suite:

| | guessit (Python) | hunch (Rust) |
|---|---|---|
| Overall pass rate | 100% (by definition) | **75.1%** (983 / 1,309) |
| Properties implemented | 49 | 43 |
| Properties at 90%+ | 49 | 24 |
| Properties at 100% | 49 | 12 |

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

1. **Match** — 29 property matcher functions scan the input independently
   and produce `MatchSpan`s (start, end, property, value) with priorities.
2. **Resolve** — Overlapping spans are resolved by priority, then by
   length (longer matches win ties).
3. **Extract** — Title is inferred from the largest unclaimed region
   before the first technical property. Media type and proper count are
   computed as derived values and set directly on the result.

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Run 29 matcher functions → Vec<MatchSpan>
  ├─ 2. Resolve conflicts (priority, then length)
  ├─ 3. Extract title from unclaimed leading region
  ├─ 4. Set computed properties (media type, proper count)
  └─ 5. Build HunchResult (BTreeMap<Property, Vec<String>>)
```

## Project Structure

```
src/
├── lib.rs              # Public API: hunch(), hunch_with()
├── main.rs             # CLI binary (clap)
├── hunch_result.rs     # HunchResult type + JSON serialization
├── options.rs          # Configuration (media type hint, name-only mode)
├── pipeline.rs         # Orchestration: matchers → resolve → extract
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (42 variants)
│   ├── engine.rs       # Conflict resolution (free function)
│   └── regex_utils.rs  # ValuePattern helper for fancy_regex
└── properties/         # 29 matcher functions (one module each)
    ├── title.rs        # Title extraction + media type inference
    ├── episodes.rs     # Season/episode detection (S01E02, 1x03, etc.)
    ├── year.rs, source.rs, video_codec.rs, ...
    └── mod.rs          # Module re-exports

tests/
├── integration.rs      # 27 hand-written end-to-end tests
├── guessit_regression.rs # 22 regression suites + compatibility report
├── helpers/mod.rs      # Custom YAML fixture parser
└── fixtures/           # Copied from guessit (self-contained)

benches/
└── parse.rs            # Criterion benchmarks
```

## License

MIT
