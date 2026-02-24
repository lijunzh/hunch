# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Directory-aware release group** — abbreviated scene filenames
  (e.g., `wthd-cab.avi`) now correctly pull the group from the parent
  directory (e.g., `DVDRip.XviD-TheWretched/`).
- **Hyphenated release groups** — `D-Z0N3` and `MARINE-FORD` are now
  captured as full group names by expanding backwards past hyphens.
- **Multi-episode patterns** — `E02-03`, `S01E01+02`, `S01.E02.E03`
  now produce proper episode arrays.
- **Multi-season support** — `S01-S10` → `[1..10]`, `Season 1-3` →
  `[1,2,3]`, `Season 1&3` → `[1,3]`.
- **Bracket language parsing** — `[ENG+RU+PT]` now produces
  `[en, pt, ru]` via `lang_code_to_name()` with 30+ ISO 639 codes.
- **New language tags** — FLEMISH, Ukr, DUBLADO, Dual Audio.
- **New other tags** — HC (Hardcoded Subtitles), COMPLET (French).
- **Source improvements** — DVDSCR → source "DVD" (not "Screener"),
  DLMux → Web, Ultra HD Blu-ray patterns expanded.
- **Duplicate source pruning** — "Web" in title zone no longer eats
  title words when WEB-DL appears later.
- Single-property failure analysis expanded to cover all major properties.

### Changed

- **Overall pass rate: 57.4% → 61.6%** (751 → 806 / 1,309 test cases).
- **12 properties at 100%** (added edition to the perfect list).
- Title extraction: added generic directory names (mnt, nas, films,
  share, home), improved abbreviated filename detection.
- `.ts` file extension no longer false-positives as Telesync.
- Conflict resolver now allows same-span Season matches with different
  values (enabling multi-season output).
- Regression floors ratcheted up across all fixture files.

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

[0.1.1]: https://github.com/lijunzh/hunch/releases/tag/v0.1.1
[0.1.0]: https://github.com/lijunzh/hunch/releases/tag/v0.1.0
