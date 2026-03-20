# 🔍 Hunch

**A fast, offline media filename parser for Rust — extract title, year, season,
episode, codec, language, and 49 properties from messy filenames.**

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
      --context <DIR>  Use sibling files from directory for better title detection
      --batch <DIR>    Parse all media files in a directory (mutual context)
  -j, --json           Output compact JSON (default is pretty-printed)
  -v, --verbose        Enable debug logging (see Logging below)
```

Cross-file context for CJK and tricky filenames:

```bash
# Single file with context directory
hunch --context ./Season1/ "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv"

# Batch mode: parse all files in a directory
hunch --batch ./Season1/ --json
```

### Library

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    assert_eq!(result.title(), Some("The Walking Dead"));
    assert_eq!(result.season(), Some(5));
    assert_eq!(result.episode(), Some(3));
    assert_eq!(result.source(), Some("Blu-ray"));
    assert_eq!(result.video_codec(), Some("H.264"));
    assert_eq!(result.release_group(), Some("DEMAND"));
    assert_eq!(result.container(), Some("mkv"));
}
```

Cross-file context for improved title detection:

```rust
use hunch::hunch_with_context;

fn main() {
    // When sibling files are available, use them for better title detection
    let result = hunch_with_context(
        "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv",
        &[
            "(BD)十二国記 第01話「月の影 影の海　一章」(1440x1080 x264-10bpp flac).mkv",
            "(BD)十二国記 第02話「月の影 影の海　二章」(1440x1080 x264-10bpp flac).mkv",
        ],
    );
    assert_eq!(result.title(), Some("十二国記"));
}
```

## Logging

Hunch uses the [`log`](https://docs.rs/log) crate for structured diagnostic
output. This is invaluable for debugging misparses — you can see exactly
which pipeline stage matched, dropped, or promoted each property.

```bash
# CLI: --verbose enables debug-level logging
hunch -v "Movie.2024.1080p.BluRay.x264-GROUP.mkv"

# Fine-grained control via RUST_LOG
RUST_LOG=hunch=trace hunch "Movie.2024.1080p.mkv"
```

| Level | What it shows |
|---|---|
| `debug` | Pipeline stage transitions, match counts, title/release group decisions |
| `trace` | Every individual match span, conflict resolution evictions, zone rule filtering |

In library usage, attach any `log`-compatible subscriber (e.g., `env_logger`,
`tracing-log`). When no subscriber is attached, all log calls compile to
no-ops — **zero runtime cost**.

## API Documentation

Full API docs are available on **[docs.rs/hunch](https://docs.rs/hunch)**.

All 49 [`Property`](https://docs.rs/hunch/latest/hunch/matcher/span/enum.Property.html)
variants are documented with example values. The
[`HunchResult`](https://docs.rs/hunch/latest/hunch/struct.HunchResult.html)
type provides typed accessors plus generic `first()`/`all()` methods.

## guessit Compatibility

Hunch validates against guessit's own 1,309-case YAML test suite.
All 49 guessit properties are implemented (3 intentionally diverged).

| | guessit (Python) | hunch (Rust) |
|---|---|---|
| Overall pass rate | 100% | **82.2%** (1,076 / 1,309) |
| Properties implemented | 49 | 49 (3 intentionally diverged) |
| Properties at 95%+ | 49 | 22 |
| Properties at 100% | 49 | 16 |

**96–100% accurate:** year, video_codec, container, source, screen_size,
audio_codec, crc32, color_depth, streaming_service, edition, frame_rate,
aspect_ratio, size, version, date, proper_count, and more.

**90%+ accurate:** title, release_group, episode, season, audio_channels,
type, website, film_title.

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
├── lib.rs              # Public API: hunch()
├── main.rs             # CLI binary (clap)
├── hunch_result.rs     # HunchResult type + JSON serialization
├── zone_map.rs         # Structural zone analysis
├── tokenizer.rs        # Input → TokenStream
├── pipeline/           # Two-pass orchestration + logging
├── matcher/            # Conflict resolution + TOML rule engine
└── properties/         # 31 property matcher modules

rules/                  # 20 TOML data files (compile-time embedded)
tests/                  # Integration tests + guessit regression suite
```

## Contributing

```bash
cargo test              # Run all tests (333 tests)
cargo test -- --ignored # Run guessit compatibility report
cargo bench             # Run benchmarks
cargo doc --open        # Build and browse API docs locally
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

[MIT](LICENSE)
