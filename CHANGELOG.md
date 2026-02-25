# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.2] - 2026-02-24

### Added

- **ARCHITECTURE.md** — layered architecture design document with decision
  log (D001–D005) covering TOML rules, regex-only, tokenizer, and
  offline-only constraints.
- **VideoApi property** — DXVA (DirectX Video Acceleration) detection.
- **Proof detection** — standalone `PROOF` tag in Other flags.
- **DOKU support** — German `DOKU` now maps to "Documentary" (like `DOCU`).
- **Español Castellano** — combined pattern maps to Catalan correctly.
- **DTS.HD-MA** — dot-separated `DTS.HD-MA` now matches as DTS-HD.

### Changed

- **Overall pass rate: 61.6% → 75.1%** (806 → 983 / 1,309 test cases).
- **proper_count** — `REAL` keyword scanned case-insensitively but only
  in the technical zone (prevents false positives on titles like
  "Real Time With Bill Maher").
- All clippy warnings resolved (regex-in-loop, collapsible-if, char arrays).
- Updated ARCHITECTURE.md with architecture decisions and v0.2 roadmap.
- Updated README.md with current compatibility stats.

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
- 27 property matchers covering all 49 guessit properties.
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

[0.1.2]: https://github.com/lijunzh/hunch/releases/tag/v0.1.2
[0.1.1]: https://github.com/lijunzh/hunch/releases/tag/v0.1.1
[0.1.0]: https://github.com/lijunzh/hunch/releases/tag/v0.1.0
