# hunch 🔮

**A Rust media filename parser — spiritual descendant of Python's [guessit](https://github.com/guessit-io/guessit).**

*"It's not a guess, it's a hunch."*

`hunch` extracts structured metadata from media filenames using regex-based pattern matching
and conflict resolution. It's designed for media organizers, scrapers, and anyone who needs
to parse filenames like `The.Matrix.1999.1080p.BluRay.x264-SPARKS.mkv` into structured data.

## Features

- 🎬 **15+ property types**: title, year, season, episode, screen_size, source, video_codec,
  audio_codec, audio_channels, edition, container, release_group, language,
  streaming_service, other flags, media type
- ⚡ **Fast**: pure Rust with lazy-compiled regexes, processes thousands of filenames per second
- 🧠 **Smart conflict resolution**: priority-based overlap handling
- 📁 **Path-aware**: extracts titles from parent directories, detects seasons from paths
- 🎯 **58% guessit compatibility** on the full guessit test suite (1330 cases)
  and **90%+ accuracy** on core properties (video_codec, audio_codec, screen_size,
  source, edition, year, audio_channels, container)

## Quick Start

```bash
cargo build --release
./target/release/hunch "The.Matrix.1999.1080p.BluRay.x264-SPARKS.mkv"
```

Output:
```json
{
  "container": "mkv",
  "release_group": "SPARKS",
  "screen_size": "1080p",
  "source": "Blu-ray",
  "title": "The Matrix",
  "type": "movie",
  "video_codec": "H.264",
  "year": 1999
}
```

## As a Library

```rust
use hunch::Pipeline;

let pipeline = Pipeline::default();
let guess = pipeline.run("Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv");

assert_eq!(guess.title(), Some("Breaking Bad"));
assert_eq!(guess.season(), Some(5));
assert_eq!(guess.episode(), Some(16));
assert_eq!(guess.video_codec(), Some("H.264"));
```

## Supported Properties

| Property | Accuracy | Examples |
|----------|----------|----------|
| video_codec | 98.6% | H.264, H.265, Xvid, AV1 |
| year | 96.8% | 1999, 2024 |
| edition | 96.4% | Director's Cut, Extended |
| source | 94.3% | Blu-ray, Web, HDTV, DVD |
| audio_codec | 94.2% | AAC, DTS-HD, Dolby Atmos |
| screen_size | 93.0% | 720p, 1080p, 2160p, 4K |
| audio_channels | 92.4% | 5.1, 7.1, 2.0 |
| type | 90.5% | movie, episode |
| container | 89.7% | mkv, mp4, avi |
| season | 86.9% | S01, Season 1 |
| release_group | 80.6% | SPARKS, FGT, [SubGroup] |
| episode | 78.5% | E02, 1x03, 501 |
| other | 71.6% | HDR10, Remux, Proper |
| title | 71.3% | The Matrix |

## Architecture

```
src/
├── lib.rs          # Public API
├── main.rs         # CLI binary
├── pipeline.rs     # Orchestration
├── guess.rs        # Result type
├── options.rs      # Configuration
├── matcher/
│   ├── engine.rs   # Conflict resolution
│   ├── span.rs     # Match spans & properties
│   └── regex_utils.rs
└── properties/     # One file per property type
    ├── audio_codec.rs
    ├── container.rs
    ├── edition.rs
    ├── episodes.rs
    ├── language.rs
    ├── other.rs
    ├── release_group.rs
    ├── screen_size.rs
    ├── source.rs
    ├── streaming_service.rs
    ├── title.rs
    ├── video_codec.rs
    └── year.rs
```

## Running Tests

```bash
# Rust unit tests (105 tests)
cargo test

# Integration tests against guessit test suite
python3 tests/validate_guessit.py
```

## License

MIT
