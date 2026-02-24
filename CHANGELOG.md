# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- 10 new rule fixture files from guessit: bonus, cd, common_words, country,
  date, film, language, part, size, website.
- All 22 fixture files now wired into Rust regression tests (was 12).
- `!!null` assertion support in regression test checker.
- Language normalization in regression tests (ISO 2/3-letter, full names).
- Compatibility report: `cargo test compatibility_report -- --ignored --nocapture`
  for full per-property and per-file accuracy breakdown.
- 192 total Rust tests (140 unit + 23 regression + 27 integration + 2 doc-tests).

### Changed

- YAML fixture parser now strips surrounding quotes from values and keys.
- Regression floors tightened to (actual − 2%) across all fixture files.

### Removed

- `tests/validate_guessit.py` — replaced by Rust-native compatibility report.
- Dependency on external `../guessit` repository. Everything is self-contained.

## [0.1.1] - 2026-02-22

### Added

- Pre-built binaries for 5 platforms in GitHub Releases.
- `cargo-binstall` support — install without compiling.

### Fixed

- All clippy warnings resolved.
- `cargo fmt` applied consistently.
- CI workflow now callable as reusable workflow.

## [0.1.0] - 2026-02-22

### Added

- Initial release — Rust port of Python's guessit.
- 27 property matchers covering all 39 guessit properties.
- Span-based conflict resolution engine.
- CLI binary (`hunch "filename.mkv"`) with JSON output.
- Library API: `hunch()` and `hunch_with()` entry points.
- 140 unit tests + doc-tests.
- Validation against guessit's 1,309-case test suite (53.6% pass rate).
- 191 Rust tests (140 unit + 22 regression + 27 integration + 2 doc-tests).
- Benchmark suite (`benches/parse.rs`).

#### Properties at 95%+ accuracy

video_codec, container, aspect_ratio, year, edition, crc32, website,
source, audio_codec, screen_size, audio_channels, date.

#### Properties at 100% accuracy

color_depth, streaming_service, bonus, episode_details, film.

[0.1.1]: https://github.com/lijunzh/hunch/releases/tag/v0.1.1
[0.1.0]: https://github.com/lijunzh/hunch/releases/tag/v0.1.0
