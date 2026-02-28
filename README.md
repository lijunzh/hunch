# 🔍 Hunch

**A fast, offline media filename parser for Rust — extract title, year, season,
episode, codec, language, and 40+ other properties from messy filenames.**

Hunch is a Rust rewrite of Python's [guessit](https://github.com/guessit-io/guessit).
All 49 guessit properties are implemented. The engine uses a tokenizer-first,
two-pass, TOML-driven architecture with linear-time regex only (ReDoS-immune).

## Install

### Homebrew (macOS / Linux)

```bash
brew install lijunzh/hunch/hunch
```

### Cargo (from source)

```bash
cargo install hunch
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/lijunzh/hunch/releases).
Also supports [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall hunch
```

### As a library

```bash
cargo add hunch
```

## Usage

### CLI

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

Multiple files at once:

```bash
hunch "Movie.2024.1080p.mkv" "Show.S01E01.mkv"
```

Options:

```
-t, --type <TYPE>    Hint media type: "movie" or "episode"
-n, --name-only      Treat input as name only (no path separators)
-j, --json           Output compact JSON (default is pretty-printed)
```

### Library

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    println!("{:#?}", result);
    // HunchResult {
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

## guessit Compatibility

Hunch validates against guessit's own 1,309-case YAML test suite.

| | guessit (Python) | hunch (Rust) |
|---|---|---|
| Overall pass rate | 100% | **80.0%** (1,047 / 1,309) |
| Properties implemented | 49 | 49 (3 intentionally diverged) |
| Properties at 95%+ | 49 | 22 |
| Properties at 100% | 49 | 16 |

**96–100% accurate:** year, video_codec, container, source, screen_size,
audio_codec, crc32, color_depth, streaming_service, edition, frame_rate,
aspect_ratio, size, version, date, proper_count, and more.

**90%+ accurate:** title, release_group, episode, season, audio_channels,
type, website, film_title.

**Developing:** episode_title (74%), subtitle_language (77%),
alternative_title (44%).

For per-property breakdowns see **[COMPATIBILITY.md](COMPATIBILITY.md)**.

### Intentional divergences

| Property | Reason |
|---|---|
| `audio_bit_rate` / `video_bit_rate` | Hunch uses a single `bit_rate` |
| `mimetype` | Trivially derived from `container`; redundant |

## Design

Hunch does **not** port guessit's `rebulk` engine. Instead:

1. **Tokenize** — split on separators, extract extension, detect brackets
2. **Zone map** — detect anchors (SxxExx, 720p, x264) to establish
   title zone vs tech zone boundaries
3. **Pass 1: Match & Resolve** — 20 TOML rule files (embedded at compile
   time) + algorithmic matchers. Conflict resolution by priority then length.
4. **Pass 2: Extract** — release group, title, and episode title run
   with access to resolved match positions from Pass 1.
5. **Result** — `HunchResult` with 49 typed property accessors

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design and decision log.

## Project Structure

```
src/
├── lib.rs              # Public API: hunch(), hunch_with()
├── main.rs             # CLI binary (clap)
├── hunch_result.rs     # HunchResult type + JSON serialization
├── options.rs          # Configuration
├── zone_map.rs         # Structural zone analysis
├── tokenizer.rs        # Input → TokenStream
├── pipeline/           # Two-pass orchestration
├── matcher/            # Conflict resolution + TOML rule engine
└── properties/         # 31 property matcher modules

rules/                  # 20 TOML data files (compile-time embedded)
tests/                  # Integration tests + guessit regression suite
```

## Contributing

```bash
cargo test              # Run all tests
cargo test -- --ignored # Run guessit compatibility report
cargo bench             # Run benchmarks
```

## License

[MIT](LICENSE)
