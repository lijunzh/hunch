# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-02-25

### Added

- **TOML side effects** — one pattern match can emit multiple properties
  (e.g., `DVDRip` → Source:DVD + Other:Rip). Declarative, no callbacks.
- **Neighbor constraints** — `not_before`, `not_after`, `requires_after`
  for context-aware TOML matching.
- **Path-segment tokenizer** — tokenizes all path segments with
  `SegmentKind` (Directory vs Filename).
- **Property-scoped `SegmentScope`** — each TOML rule set declares
  whether it matches directory tokens (`AllSegments` for unambiguous
  tech properties, `FilenameOnly` for ambiguous ones).
- **`absolute_episode`** property — detects absolute episode numbers
  (anime-style) when both S/E markers and standalone ranges coexist.
  0% → 90%.
- **`film_title`** property — extracts franchise title from `-fNN-`
  patterns (e.g., James Bond). 0% → 87.5%.
- **`alternative_title`** property — extracts content after title
  boundary separators (` - `, `--`, `(`). 0% → 43.8%.
- **Title boundary detection** — structural separators (` - `, `--`,
  `()`) stop title extraction at subtitle/director content.
- **Single-word input handling** — bare words without path/extension
  are treated as title.
- **Italian `Stagione`** season keyword support.
- **`audio_channels.toml`** — standalone channel count detection
  (5.1, 7.1, 2ch, mono, stereo).
- **Subtitle language capture groups** — `SUB.FR` / `FR-SUB` patterns
  extract the language code via `{1}` template.

### Changed

- **Overall pass rate: 75.1% → 77.3%** (983 → 1,012 / 1,309 test cases).
- **`fancy_regex` removed entirely** — all regex is now standard `regex`
  crate only (linear-time, ReDoS-immune). 🎉
- **4 legacy matchers fully retired** to TOML-only: frame_rate,
  container, screen_size, audio_codec.
- **`language.rs` gutted** — TOML handles tokens, Rust handles only
  bracket/brace multi-language codes (`[ENG+RU+PT]`, `{Fr-Eng}`).
- **8 dead modules cleaned** — removed vestigial `ValuePattern` code
  from video_codec, audio_profile, color_depth, country, edition,
  episode_details, streaming_service, video_profile.
- **Directory selection** — title extraction now walks directories
  deepest-first (closest to filename preferred).
- **Language zone rule** improved — fixes "The Italian Job" case where
  "Italian" was matched as language instead of title word.
- **Case-insensitive dedup** for language/subtitle_language values.
- All clippy warnings resolved.

### Property improvements

| Property | v0.1.2 | v0.2.0 |
|----------|:------:|:------:|
| video_codec | 94.0% | 98.6% |
| screen_size | 93.7% | 98.4% |
| audio_codec | 91.2% | 97.8% |
| title | 84.6% | 87.9% |
| subtitle_language | 49.4% | 77.8% |
| language | 77.5% | 84.5% |
| episode_title | 69.7% | 70.6% |
| absolute_episode | 0% | 90.0% |
| film_title | 0% | 87.5% |
| alternative_title | 0% | 43.8% |

### Dependencies

- Removed: `fancy-regex` (was fallback for lookaround patterns)
- All regex matching is now guaranteed linear-time via `regex` crate

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

[0.2.0]: https://github.com/lijunzh/hunch/releases/tag/v0.2.0
[0.1.2]: https://github.com/lijunzh/hunch/releases/tag/v0.1.2
[0.1.1]: https://github.com/lijunzh/hunch/releases/tag/v0.1.1
[0.1.0]: https://github.com/lijunzh/hunch/releases/tag/v0.1.0
