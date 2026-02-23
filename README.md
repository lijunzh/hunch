# 🔍 Hunch

**A Rust library for guessing media metadata from filenames.**

Hunch is a fast, opinionated media filename parser — a spiritual descendant
of Python's [guessit](https://github.com/guessit-io/guessit), rewritten from
scratch for Rust. It extracts title, year, season, episode, resolution, codec,
language, and 35+ other properties from messy media filenames.

## Quick Start

```bash
cargo add hunch
```

### As a library

```rust
use hunch::parse;

fn main() {
    let result = parse("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
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

## Supported Properties

### Core Properties (95%+ accuracy)

| Property | Examples | Accuracy |
|---|---|---|
| video_codec | x264, H.265, HEVC, Xvid, AV1 | 98.6% |
| container | mkv, mp4, avi, srt | 98.6% |
| aspect_ratio | Computed from WxH resolution | 97.6% |
| year | 2024, (1999) | 96.8% |
| edition | Director's Cut, Extended, Unrated | 96.4% |
| crc32 | [1A2B3C4D] | 96.0% |
| website | [site.com], www.domain.com | 95.2% |

### Strong Properties (90–95% accuracy)

| Property | Examples | Accuracy |
|---|---|---|
| source | BluRay, WEB-DL, HDTV, DVDRip | 94.3% |
| audio_codec | DTS, AAC, FLAC, AC3, EAC3 | 94.2% |
| screen_size | 720p, 1080p, 4K, 2160p | 93.0% |
| audio_channels | 5.1, 7.1, 2.0, Stereo | 92.4% |
| date | 2017-06-22, 20021107, 03-29-2012 | 92.0% |
| type | movie, episode | 91.4% |
| country | US, UK, GB | 90.0% |

### Good Properties (80–90% accuracy)

| Property | Examples | Accuracy |
|---|---|---|
| season | S05, Season 3, Saison 2 | 88.6% |
| uuid | Standard & compact UUIDs | 87.5% |
| release_group | DEMAND, SPARKS, FGT | 82.7% |
| subtitle_language | French, eng, VOSTFR, NLsubs | 82.6% |
| episode | E03, 5x03, Ep.12, E01-E03 | 81.1% |

### Developing Properties (60–80% accuracy)

| Property | Examples | Accuracy |
|---|---|---|
| title | Movie / show name (inferred) | 78.5% |
| audio_profile | Master Audio, Atmos, DTS:X | 77.5% |
| proper_count | Proper, REAL.PROPER | 75.0% |
| other | HDR, Remux, Proper, Repack, 3D | 73.7% |
| episode_title | Inferred from position | 62.1% |

### Perfect Properties (100% accuracy)

| Property | Examples |
|---|---|
| color_depth | 10-bit, 8-bit, 12-bit |
| streaming_service | AMZN, NF, HMAX, DSNP, ATVP |
| bonus | x01, x02 extras |
| episode_details | Special, Pilot, Unaired, Final |
| film | f01, f21 (collections) |

*All accuracy numbers measured against guessit's 1,330-case test suite.*

## Design

Hunch uses a **span-based architecture**:

1. **Match** — 27 property matchers each scan the input independently
   and produce `MatchSpan`s (start, end, property, value) with priorities.
2. **Resolve** — Overlapping spans are resolved using priority +
   specificity rules (longer matches win ties).
3. **Extract** — Title is inferred from the largest unclaimed region
   before the first technical property.

This is fundamentally different from guessit's rebulk-based approach.
Instead of a complex rule engine, Hunch uses simple regex matchers
composed via a trait (`PropertyMatcher`) and resolved with span logic.

```
Input: "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
  │
  ├─ 1. Pre-process: strip path, extract extension
  ├─ 2. Run 27 property matchers → Vec<MatchSpan>
  ├─ 3. Resolve conflicts (priority, then length)
  ├─ 4. Extract title from unclaimed leading region
  ├─ 5. Infer media type (episode vs movie)
  └─ 6. Build JSON output (BTreeMap)
```

## Performance

Hunch compiles all regex patterns at startup via `lazy_static` and
runs each parse in microseconds. The CLI adds ~85ms of process startup
overhead.

## Compatibility with guessit

Hunch targets **feature parity** with guessit's test suite:

- **Overall pass rate: 58.5%** (778 / 1,330 test cases)
- **20 properties at 90%+** accuracy
- **5 properties at 100%** accuracy
- **0 properties skipped** — all 39 guessit properties are implemented

The remaining gaps are mostly in complex title extraction heuristics,
episode title inference, and edge cases in `other` flags.

## Project Structure

```
src/
├── lib.rs              # Public API: parse()
├── main.rs             # CLI binary
├── guess.rs            # GuessResult type + JSON serialization
├── options.rs          # Configuration
├── pipeline.rs         # Orchestration: matchers → resolve → extract
├── matcher/
│   ├── span.rs         # MatchSpan + Property enum (39 variants)
│   ├── engine.rs       # Conflict resolution
│   └── regex_utils.rs  # ValuePattern helper
└── properties/         # 27 matcher modules
    ├── title.rs, episodes.rs, year.rs, ...
    └── mod.rs          # PropertyMatcher trait
```

## License

MIT
