# 🔍 Hunch

[![CI](https://img.shields.io/github/actions/workflow/status/lijunzh/hunch/ci.yml?branch=main&label=CI&logo=github)](https://github.com/lijunzh/hunch/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/hunch.svg)](https://crates.io/crates/hunch)
[![Docs](https://img.shields.io/docsrs/hunch)](https://docs.rs/hunch)
[![License](https://img.shields.io/crates/l/hunch.svg)](LICENSE)

**A fast, offline media filename parser for Rust — extract title, year,
season, episode, codec, language, and 53 properties from messy filenames.**

A Rust rewrite of Python's [guessit](https://github.com/guessit-io/guessit).
Pure, deterministic, single-binary, linear-time regex only (ReDoS-immune).

## Project status: maintenance mode

hunch is feature-complete: I'm not planning new features of my own.
I now do most media renaming with AI agents, so a dedicated renaming tool
matters less to me, and probably to others. hunch is still useful as a
fast, offline, deterministic parser that programs and agents can call,
so it stays maintained. What to expect:

- **Security fixes** ship as patch releases. A weekly `cargo audit` of the
  lockfile flags new advisories, and Dependabot applies the fixes.
- **Dependency updates** land monthly. A release follows when a fix
  reaches users, mainly through the prebuilt binaries and Homebrew. As a
  library, hunch picks up compatible fixes in your own `cargo update`
  without a new hunch release.
- **guessit parity** is checked monthly. When a guessit release adds test
  cases, an issue tracks them, and misses that affect real filenames get
  fixed in a minor release.
- **Bug reports and pull requests** are welcome. Review may be slow, and
  new parsing features beyond guessit's are unlikely to be merged.

The 2.x API is stable, and no breaking release is planned. See
[SECURITY.md](SECURITY.md) to report a vulnerability.

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

**From scripts and AI agents:** `hunch -j` prints one JSON object per
line, with `_confidence` (`high`, `medium` or `low`) so the caller knows
when to retry with `--context` or check the result. Plex/Jellyfin tags
such as `{tmdb-12345}` and `{imdb-tt0133093}` come back as `tmdb_id` and
`imdb_id`. See
[Scripts and AI agents](https://lijunzh.github.io/hunch/user-guide/user-manual.html#scripts-and-ai-agents).

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
| [**User Manual**](https://lijunzh.github.io/hunch/user-guide/user-manual.html) | Install, CLI, library API, all 53 properties, batch parsing, cross-file context |
| [**guessit Compatibility**](https://lijunzh.github.io/hunch/user-guide/compatibility.html) | Pass rates per property, methodology |
| [**Known Limitations**](https://lijunzh.github.io/hunch/user-guide/known-limitations.html) | Edge cases that remain difficult to handle |
| [**Migrating to v2.0.0**](https://lijunzh.github.io/hunch/about/migration-v2.html) | Breaking-change guide |
| [**Design**](https://lijunzh.github.io/hunch/about/design.html) | Principles, architecture, key decisions |
| [**API Reference**](https://docs.rs/hunch) | Full Rust API docs |
| [**Changelog**](CHANGELOG.md) | Version history |

## Accuracy

hunch passes **78.2%** of guessit v4.5.5's 1,631 test cases (November
2026); per-property rates and the one intentional divergence are in the
[compatibility report](https://lijunzh.github.io/hunch/user-guide/compatibility.html).
A monthly workflow compares that copy with guessit's latest release, and
`cargo test compatibility_report --release -- --ignored --nocapture`
reproduces the numbers. Edge cases that remain difficult are documented
under
[Known Limitations](https://lijunzh.github.io/hunch/user-guide/known-limitations.html).
