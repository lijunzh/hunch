# 🔍 Hunch

**A Rust library for guessing media metadata from filenames.**

Hunch is a fast, opinionated media filename parser — a spiritual descendant
of Python's [guessit](https://github.com/guessit-io/guessit), rewritten from
scratch for Rust. It extracts title, year, season, episode, resolution, codec,
language, and 30+ other properties from messy media filenames.

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

| Property | Examples | Accuracy |
|---|---|---|
| title | Movie name / show name | 78% |
| year | 2024, (1999) | 97% |
| season | S05, Season 3 | 89% |
| episode | E03, 5x03, Ep.12 | 81% |
| episode_title | Inferred from position | 62% |
| screen_size | 720p, 1080p, 4K, 2160p | 93% |
| source | BluRay, WEB-DL, HDTV | 94% |
| video_codec | x264, H.265, HEVC, Xvid | 99% |
| audio_codec | DTS, AAC, FLAC, AC3 | 94% |
| audio_channels | 5.1, 7.1, 2.0 | 92% |
| audio_profile | Master Audio, Atmos | 78% |
| release_group | DEMAND, SPARKS, FGT | 83% |
| container | mkv, mp4, avi | 99% |
| edition | Director's Cut, Extended | 96% |
| streaming_service | AMZN, NF, HMAX | 100% |
| other | Proper, Repack, HDR | 74% |
| subtitle_language | French, eng, VOSTFR | 83% |
| language | French, Multi, VOSTFR | 91% |
| color_depth | 10-bit, 8-bit | 100% |
| video_profile | High, HEVC, AVCHD | 64% |
| date | 2017-06-22, 20021107 | 92% |
| country | US, UK, GB | 90% |
| crc32 | [1A2B3C4D] | 96% |
| website | [site.com], www.x.com | 95% |
| uuid | Standard & compact UUIDs | 88% |
| aspect_ratio | Computed from resolution | 98% |
| bonus / bonus_title | x01 extras | 100% / 60% |
| part / disc / cd | Part 2, Disc 1, CD1 | 67% |
| size | 700MB, 1.4GB | 67% |

*Accuracy measured agit's test suite (1,330 test cases).*

## Design

Hunch uses a **span-based architecture**:

1. **Match** — Each property matcher scans the input and produces
   `MatchSpan`s (start, end, property, value) with priorities.
2. **Resolve** — Overlapping spans are resolved using priority and
   specificity rules (longer matches win ties).
3. **Extract** — Title is inferred from the largest unclaimed region
   before the first technical property.

This is fundamentally different from guessit's rebulk-based approach.
Instead of a complex rule engine, Hunch uses simple regex matchers
composed via a trait (`PropertyMatcher`) and resolved with span logic.

## Performance

Hunch compiles all regex patterns at startup via `lazy_static` and
runs each parse in microseconds. The CLI adds ~85ms of process startup
overhead.

## Compatibility with guessit

Hunch targets **feature parity** with guessit's test suite:

- **Overall pass rate: 58.5%** (778/1,330 test cases)
- **20 properties at 90%+** accuracy
- **7 properties at 100%** accuracy

The remaining gaps are mostly in complex title extraction heuristics,
episode title inference, and edge cases in `other` flags. These are
actively being improved.

## License

MIT
