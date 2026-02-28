# Hunch v1.0.0 — Stable Release

The first **stable release** of Hunch — a fast, offline media filename parser
for Rust. All 49 guessit properties are implemented. The engine uses a
tokenizer-first, two-pass, TOML-driven architecture with linear-time regex
only (ReDoS-immune).

## Highlights

- **81.7% compatibility** with guessit's 1,309-case YAML test suite
  (1,069 / 1,309 cases pass)
- **22 properties at 95%+ accuracy**, 16 at 100%
- **All 49 properties implemented** (3 intentionally diverged)
- Zero-dependency on network, databases, or ML
- Single binary, TOML rules embedded at compile time

## Accuracy Summary

| Tier | Properties |
|------|------------|
| 100% | 16 properties (edition, date, proper_count, …) |
| 95–99% | video_codec (98.6%), screen_size (98.4%), audio_codec (97.8%), source (97.5%), year (96.5%), crc32 (96.0%) |
| 90–94% | season (93.9%), type (93.6%), title (91.7%), release_group (91.1%), episode (90.6%), … |
| 80–89% | other (87.7%), language (87.3%), subtitle_language (82.7%), … |

## What Changed Since v0.3.1

- Version bumped to 1.0.0 (stable)
- Removed "in progress" / "developing" warnings from all documentation
- Updated all compatibility numbers to match current test results
- CLI description updated

## Install

```bash
# Homebrew
brew install lijunzh/hunch/hunch

# Cargo
cargo install hunch

# As a library
cargo add hunch
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history.
