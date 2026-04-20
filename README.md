# 🔍 Hunch

[![Coverage](https://img.shields.io/badge/coverage-94.34%25-brightgreen)](https://lijunzh.github.io/hunch/contributor-guide/coverage.html)

**A fast, offline media filename parser for Rust — extract title, year,
season, episode, codec, language, and 49 properties from messy filenames.**

A Rust rewrite of Python's [guessit](https://github.com/guessit-io/guessit).
Pure, deterministic, single-binary, linear-time regex only (ReDoS-immune).

## Quick Start

```bash
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

For batch parsing across a media library and cross-file context for
ambiguous CJK / anime filenames, see the
[User Manual](https://lijunzh.github.io/hunch/user-guide/user-manual.html).

### Library

```rust
use hunch::hunch;

let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
assert_eq!(result.title(), Some("The Walking Dead"));
assert_eq!(result.season(), Some(5));
assert_eq!(result.episode(), Some(3));
```

## Documentation

📖 **Full documentation site:** <https://lijunzh.github.io/hunch>

| Document | What's there |
|---|---|
| [**User Manual**](https://lijunzh.github.io/hunch/user-guide/user-manual.html) | Install, CLI, library API, all 49 properties, batch parsing, cross-file context |
| [**guessit Compatibility**](https://lijunzh.github.io/hunch/user-guide/compatibility.html) | Pass rates per property, methodology |
| [**Known Limitations**](https://lijunzh.github.io/hunch/user-guide/known-limitations.html) | Edge cases that remain difficult to handle |
| [**Migrating to v2.0.0**](https://lijunzh.github.io/hunch/about/migration-v2.html) | Breaking-change guide |
| [**Design**](https://lijunzh.github.io/hunch/about/design.html) | Principles, architecture, key decisions |
| [**API Reference**](https://docs.rs/hunch) | Full Rust API docs |
| [**Changelog**](CHANGELOG.md) | Version history |

## Real-world accuracy

Validated against guessit's upstream test suite — see the
[compatibility report](https://lijunzh.github.io/hunch/user-guide/compatibility.html)
for the live pass rate, regenerated from
`cargo test -- --ignored guessit_compat` so it can't drift.

In one real-world library audit of 7,838 files, hunch achieved **99.8%
accuracy** across a mixed Anime / English / Japanese / Kids collection.
The remaining edge cases are documented under
[Known Limitations](https://lijunzh.github.io/hunch/user-guide/known-limitations.html).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The easiest contribution is
[reporting a failed parse](https://github.com/lijunzh/hunch/issues/new/choose).

```bash
cargo test              # full suite
cargo test -- --ignored # guessit compatibility report
```

## License

[MIT](LICENSE)
