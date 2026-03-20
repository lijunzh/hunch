# 🔍 Hunch

**A fast, offline media filename parser for Rust — extract title, year, season,
episode, codec, language, and 49 properties from messy filenames.**

Hunch is a Rust rewrite of Python's [guessit](https://github.com/guessit-io/guessit).
Pure, deterministic, single-binary, linear-time regex only (ReDoS-immune).

## Quick Start

```bash
# Install
brew install lijunzh/hunch/hunch   # macOS/Linux
cargo install hunch                 # from source
cargo binstall hunch                # pre-built binary
```

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

### Library

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    assert_eq!(result.title(), Some("The Walking Dead"));
    assert_eq!(result.season(), Some(5));
    assert_eq!(result.episode(), Some(3));
}
```

### Cross-file context

For CJK, anime, or ambiguous filenames:

```bash
hunch --context ./Season1/ "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv"
hunch --batch ./Season1/ --json
```

## Documentation

| Document | Audience | Content |
|---|---|---|
| [**User Manual**](docs/user_manual.md) | Users | Install, CLI, library API, all 49 properties, FAQ |
| [**Design**](docs/design.md) | Contributors | Principles, architecture, key decisions |
| [**Compatibility**](docs/compatibility.md) | Everyone | guessit test suite pass rates by property |
| [**API Reference**](https://docs.rs/hunch) | Developers | Full Rust API docs |
| [**Changelog**](CHANGELOG.md) | Everyone | Version history |

## guessit Compatibility

All 49 guessit properties implemented. Validated against guessit's
1,309-case test suite.

| Metric | Value |
|---|---|
| Pass rate | **81.8%** (1,071 / 1,309) |
| Properties at 95%+ | 22 |
| Properties at 100% | 16 |

See [docs/compatibility.md](docs/compatibility.md) for per-property breakdowns.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The easiest contribution is
[reporting a failed parse](https://github.com/lijunzh/hunch/issues/new/choose).

```bash
cargo test              # 295 tests
cargo test -- --ignored # guessit compatibility report
cargo bench             # benchmarks
```

## License

[MIT](LICENSE)
