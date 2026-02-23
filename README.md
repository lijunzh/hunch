# hunch 🔮

**A Rust media filename parser — spiritual descendant of Python's [guessit](https://github.com/guessit-io/guessit).**

*"It's not a guess, it's a hunch."*

`hunch` extracts structured metadata from media filenames using regex-based pattern matching
and conflict resolution. It's designed for media organizers, scrapers, and anyone who needs
to parse filenames like `The.Matrix.1999.1080p.BluRay.x264-SPARKS.mkv` into structured data.

## Features

- 🎬 **14 property types**: title, year, season, episode, screen_size, source, video_codec,
  audio_codec, audio_channels, edition, container, release_group, other flags, media type
- ⚡ **Fast**: pure Rust with lazy-compiled regexes, processes thousands of filenames per second
- 🧠 **Smart conflict resolution**: priority-based overlap handling
- 📁 **Path-aware**: extracts titles from parent directories, detects seasons from paths
- 🎯 **41%+ guessit compatibility** on the full guessit test suite (976 cases)
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
| video_codec | 98.7% | H.264, H.265, Xvid, AV1 |
| year | 96.8% | 1999, 2024 |
| audio_codec | 94.3% | AAC, DTS-HD, Dolby Atmos |
| screen_size | 95.9% | 720p, 1080p, 2160p, 4K |
| edition | 94.1% | Director's Cut, Extended |
| source | 93.4% | Blu-ray, Web, HDTV, DVD |
| audio_channels | 91.7% | 5.1, 7.1, 2.0 |
| container | 91.0% | mkv, mp4, avi |
| season | 84.2% | S01, Season 1 |
| type | 78.0% | movie, episode |
| episode | 65.0% | E02, 1x03 |
| title | 63.9% | The Matrix |
| release_group | 61.5% | SPARKS, FGT |
| other | 51.1% | HDR10, Remux, Proper |

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
    ├── other.rs
    ├── release_group.rs
    ├── screen_size.rs
    ├── source.rs
    ├── title.rs
    ├── video_codec.rs
    └── year.rs
```

## Running Tests

```bash
# Rust unit tests (94 tests)
cargo test

# Integration tests against guessit test suite
python3 tests/validate_guessit.py
```

## License

MIT
