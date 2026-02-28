# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.1.2] - 2026-02-28

### Fixed

- **docs.rs build** — added `rust-version = "1.85"` and
  `[package.metadata.docs.rs]` to `Cargo.toml`. Edition 2024 requires
  Rust 1.85+; docs.rs needs this hint to select a compatible toolchain.
  Versions 1.0.0–1.1.1 failed to build on docs.rs for this reason.

## [1.1.1] - 2026-02-28

### Fixed

- **`cargo fmt`** — applied rustfmt to all files modified in v1.1.0.
  No logic changes; line wrapping only.

## [1.1.0] - 2026-02-28

### Added

- **Structured logging** — integrated the `log` crate with `debug!` and
  `trace!` instrumentation across the full pipeline. Each stage (tokenize,
  zone map, matching, conflict resolution, zone disambiguation, title
  extraction) emits diagnostic messages. Zero runtime cost when no
  subscriber is attached.
- **`--verbose` / `-v` CLI flag** — enables `hunch=debug` logging via
  `env_logger`. Users can also set `RUST_LOG=hunch=trace` for per-match
  detail.
- **`env_logger` dependency** — powers CLI log output.
- **`#![warn(missing_docs)]`** — compiler lint prevents future doc
  regressions.
- **15 new doc-tests** — all rustdoc examples are compiled and run as
  part of `cargo test` (total: 295 tests).

### Changed

- **Comprehensive Rustdoc coverage** — 81 missing-doc warnings → 0:
  - All 49 `Property` enum variants documented with example values.
  - `HunchResult`, `Options`, `Pipeline`, `MatchSpan`, `MediaType`
    enriched with usage examples and cross-links.
  - `hunch_with()` fully documented with two worked examples.
  - Crate-level docs (`lib.rs`) expanded: Quick Start, Options,
    Property access, Multi-valued, JSON output, Logging, Architecture.
  - All 15 `find_matches()` functions documented.
  - `SideEffect`, `BoundedRegex`, `TitleYear` fields documented.
  - Internal modules (`matcher`, `properties`) marked with stability notes.
- **README.md** — added Logging section, `--verbose` flag, `Options`
  example, API Documentation section with docs.rs links, updated test
  count (295).
- **CLI error handling** — JSON serialization errors now print to stderr
  and exit(1) instead of silently producing empty output.

### Fixed

- **~30 bare `.unwrap()` calls** replaced with descriptive `.expect()`
  messages across `zone_map.rs`, `bit_rate.rs`, `size.rs`, `uuid.rs`,
  `crc32.rs`, `year.rs`, `version.rs`, `proper_count.rs`,
  `release_group/mod.rs`, `episodes/mod.rs`, `episodes/patterns.rs`.
- **O(n²) comment** added to `resolve_conflicts()` documenting
  algorithmic complexity and future optimization path.
- **`#[allow(dead_code)]` on `Options`** annotated with TODO explaining
  planned `media_type` / `expected_title` wiring.

## [1.0.1] - 2026-02-28

### Fixed

- **Documentation patch** — v1.0.0 shipped with incorrect compatibility
  numbers in README. This release corrects all documentation to match
  actual test results (81.7%, 1,069 / 1,309).
- Updated COMPATIBILITY.md version reference to v1.0.1.
- Added missing CHANGELOG entries for v1.0.0 and v1.0.1.

## [1.0.0] - 2026-02-28

### Changed

- **Stable release** — first non-pre-release version.
- Removed "in progress" / "developing" warnings from all documentation.
- Updated all compatibility numbers to match current test results.
- CLI description updated.

### Summary

- **81.7% compatibility** with guessit's 1,309-case YAML test suite.
- **22 properties at 95%+ accuracy**, 16 at 100%.
- **All 49 properties implemented** (3 intentionally diverged).
- Zero-dependency on network, databases, or ML.
- Single binary, TOML rules embedded at compile time.

## [0.3.1] - 2026-02-27

### Fixed

- **Language/subtitle_language disambiguation** — Add zone Rule 8 to
  suppress Language matches contained within SubtitleLanguage spans.
  Fixes cases like `ENG.-.FR Sub` where `FR` was incorrectly detected
  as both language and subtitle_language.
- **Subtitle language 2-letter codes** — Add ISO 639-1 codes (FR, SV,
  DE, etc.) to the `LANG SUBS` regex. Patterns like `FR Sub` and
  `SV Sub` now correctly produce subtitle_language matches.
- **Bracket subtitle over-matching** — Tighten the `SUB_LANG` regex
  separator class to exclude `)}]`, preventing greedy matches that
  consumed content past closing brackets (e.g., `St{Fr-Eng}.Chaps]`).
  Multi-language bracket patterns like `St{Fr-Eng}` now correctly
  extract both languages.
- **Remove unused `is_episode_property`** — Dead code cleanup.

### Changed

- **language.yml pass rate** — 66.7% → 100% (ratcheted to 98%).
- **Enable Language rules in directory segments** — Language TOML
  matching now applies to directory components with per-directory
  zone filtering.
- **LC-AAC audio profile** — Added Low Complexity pattern.
- **Space-separated episode numbers** — Zero-padded episode numbers
  with spaces are now detected.
- **Spanish season keyword** — `Temp` recognized as Temporada.
- **Bonus without film/year** — Implies episode media type.
- **Portuguese 'pt' code** — Added ISO 639-1 code for language matching.
- **Multi-dot release groups** — Names like `YTS.LT` are merged.
- **Mid-filename bracket release groups** — Detection improved.
- **Bracket trailing strip** — Metadata cleanup for release groups.
- **Episode title paren fix** — Don't truncate at parens with digits.
- **Bracket '/' skip** — Skip bracket groups with slashes in RG detection.
- **Episode title separator** — Strip leading separators.
- **Per-directory Other rules** — Other property matching with zone filtering.
- **Compound bracket groups** — Tokenizer model improvements.

## [0.3.0] - 2026-02-26

### Added

- **Two-pass pipeline** — Release group extraction runs after conflict
  resolution (Pass 2), using resolved match positions instead of a
  130-token exclusion list.
- **Position-based release group validation** — `is_position_claimed()`
  checks candidate spans against resolved tech matches. Replaces the
  DRY-violating `is_known_token()` function.
- **Bracket group model** — `BracketGroup` struct in tokenizer tracks
  matched bracket pairs (Square, Round, Curly) with positions and content.
- **Per-directory zone maps** — `SegmentZone` provides title/tech zone
  boundaries for directory segments. TOML zone-scope filtering now works
  for directory tokens.
- **TokenStream in Pass 2** — All positional extractors (release_group,
  title, episode_title, film_title, alternative_title) receive the full
  TokenStream for bracket-aware and path-aware parsing.
- **Suspicious Other detection** — `Other:Proper` in episode titles is
  treated as title content when the original token text is not a release
  tag and the next word is not a tech token.
- **Episode title separator splitting** — show title repetition after
  ` - ` is correctly split from the actual episode title.
- **Trailing Part stripping** — "Part N" at the end of episode titles
  is stripped (Part is extracted as a separate property).
- **EpisodeCount/SeasonCount boundary** — episode title extraction
  starts after episode_count matches, not just episode matches.
- **Title: leading tech skip** — when filename starts with codec tokens,
  title extraction skips to the first non-tech gap.
- **Zone Rule 1 duplicate language detection** — drops language in
  title zone when the same language appears in the tech zone.

### Changed

- **Overall pass rate: 79.0% → 80.0%** (1,034 → 1,047 / 1,309).
- **title: 90.1% → 91.6%** — leading codec, language dedup, asterisks.
- **release_group: 89.1% → 90.2%** — post-resolution, SC/SDH context.
- **episode_title: 70.1% → 74.1%** — boundaries, Part strip, suspicious Other.
- **other: 83.7% → 84.8%** — Zone Rule 5 post-RG, HQ adjacency.
- **`release_group::find_matches()`** signature changed to accept
  `(input, resolved_matches, zone_map, token_stream)`.
- **All Pass 2 extractors** now accept `token_stream` parameter.
- **Zone Rule 5** moved to `apply_post_release_group_rules()` so it
  can see release group positions.

### Fixed

- **video_codec.toml**: HEVC suffix regex `hevc.+` → `hevc[a-zA-Z0-9_]+`
  to prevent multi-token window over-matching (e.g., HEVC.Atmos-GROUP).
- **video_profile.toml**: SC/SCH/SDH require preceding codec token
  (`requires_before`). Prevents false positives where SC is a release
  group name or SDH means subtitle tag.
- **Title asterisk stripping**: `*` treated as separator character.
- **Episode title REPACK/REAL**: checks original input text, not just
  the Other match value, to distinguish metadata from title content.

### Removed

- **`is_known_token()`** — 130-token exclusion list replaced by
  position-based overlap detection + 20-token curated non-group list.

## [0.2.2] - 2026-02-26

### Added

- **`requires_before` constraint** in TOML rule engine — symmetric with
  `requires_after`. A match is rejected unless the previous token
  (lowercased) is in the list.
- **Zone Rule 8: Source subsumption dedup** — when both a generic
  source (TV) and a specific source (HDTV) exist, the generic is dropped.
- **AmazonHD side_effect** — `AmazonHD` now emits both
  `streaming_service:Amazon Prime` and `other:HD`.
- **Tier 2 anchor expansion** — `dvd`, `dvdr`, `bd`, `pal`, `ntsc`,
  `secam` added as unambiguous tech vocabulary for zone boundary detection.
- **Year-as-anchor for zone filtering** — when title content before a
  year is ≥6 bytes, the year enables zone filtering even without Tier 1/2
  anchors. Fixes titles like `A.Common.Title.Special.2014`.

### Changed

- **Overall pass rate: 76.6% → 79.1%** (1,003 → 1,036 / 1,309).
- **edition: 97.6% → 100%** on per-property accuracy.
- **source: 95.4% → 97.5%** — BD standalone, source dedup.
- **title: 89.1% → 90.8%** — bracket group boundary detection,
  year-as-anchor zone filtering, Edition Collector pattern,
  parent dir after-match extraction.
- **other: 81.7% → 84.5%** — HQ/LD unrestricted, Complete context,
  SCR screener, FanSub pruning, Dubbed not_after.
- **language: 77.5% → 84.5%** — FLEMISH nl-be, Tier 2 anchor improvements.
- **episode_title: 70.1% → 72.1%** — Date-based anchoring, Part exclusion.
- **year: 96.1% → 96.5%** — first-paren disambiguation.
- **release_group module** split into `mod.rs` + `known_tokens.rs`
  (626 lines → 312 + 190).

### Fixed

- **HQ standalone** → Other:High Quality (was audio_profile:High Quality).
  AudioProfile HQ now requires AAC prefix.
- **LD/HQ** moved from tech_only to unrestricted zone scope
  (fixes detection when appearing before the first Tier 2 tech token).
- **Dubbed** no longer emits Other:Dubbed after language names
  (GERMAN.DUBBED → just language, not Other).
- **Complete** now requires contextual preceding token (season, language,
  number, source) to avoid false-positive matching on title words.
- **Fix** requires tech tokens on both sides (`requires_before` +
  `requires_after`) per guessit semantics.
- **Edition Collector** 2-token pattern added (French reversed form).
- **Bracket group titles** now apply find_title_boundary
  (`[Ayako] Infinite Stratos - IS` → `Infinite Stratos`).
- **Episode titles** no longer stop at Part matches
  (`Elements.Part.1.Skyhooks` → full episode title).
- **Zone Rule 5** extended with adjacency gap and Fan Subtitled value.

## [0.2.1] - 2026-02-26

### Added

- **`bit_rate` property** — detects audio/video bit rates from filename
  patterns (`320Kbps`, `19.1Mbps`, `1.5Mbps`). Emitted as a single
  `bit_rate` (not split into audio/video — see COMPATIBILITY.md).
- **`episode_format` property** — detects "Minisode" / "Minisodes".
- **`week` property** — detects "Week 45" in episode context.
- **Zone map (ZoneMap)** — two-phase anchor detection for structural
  filename analysis. Tier 1+2 anchors establish tech_zone_start;
  Tier 3 year disambiguation uses that boundary.
- **`zone_scope` in TOML rules** — `tech_only` and `after_anchor`
  scopes suppress ambiguous tokens in the title zone at match time.
- **Source side-effects in TOML** — `source.toml` now emits Other:Rip,
  Other:Screener, Other:Reencoded via declarative side_effects.
- **Zone Rule 7** — promotes Blu-ray → Ultra HD Blu-ray when UHD/4K/2160p
  signals exist elsewhere in the filename.

### Changed

- **Overall pass rate: 78.2% → 76.6%** (1,023 → 1,003 / 1,309).
  Slight regression from eliminating dual-pipeline overlap; source-specific
  accuracy improved (91% → 100%). See architecture notes below.
- **Source: 91.3% → 100%** on rules/source.yml fixture.
- **Year: 95.2% → 96.1%** — improved boundary handling.

### Architecture

- **Phase A + A.1 complete** — ZoneMap, zone_scope filtering, year
  disambiguation all integrated into pipeline.
- **Dual-pipeline eliminated** — source.rs retired to TOML-only;
  subtitle_language.rs trimmed to algorithmic-only (no TOML overlap);
  language.rs already cooperative (bracket codes only).
- **ValuePattern retired** — year.rs uses plain Regex; ValuePattern
  struct and related code deleted from regex_utils.rs.
- **Dead legacy code removed** — other.rs gutted (282→75 lines);
  source.rs gutted (288→80 lines).
- **File splits for clarity** —
  - `pipeline.rs` (808 lines) → `pipeline/` module: mod.rs (600),
    zone_rules.rs (165), proper_count.rs (68)
  - `title.rs` (1043 lines) → `title/` module: mod.rs (365),
    clean.rs (266), secondary.rs (253)
  - `episodes/mod.rs` find_matches (640-line function) → 25-line
    orchestrator + 6 named category functions
- **Renamed** `other_weak.toml` → `other_positional.toml` for clarity.
- **`episode_details.toml`** tagged with `zone_scope = "tech_only"`,
  retiring zone Rule 4.
- **Zone Rule 1** (language in title zone) now uses ZoneMap boundaries
  directly instead of re-deriving from match positions.
- **cargo clippy** clean — zero warnings.

### Fixed

- **Title: "The 100" pattern** — absolute episode candidates before the
  first S/E span are now skipped.
- **Title: trailing keywords** — strip trailing `Episode`/`Ep` words
  and `-xNN` bonus markers.
- **Title: trailing punctuation** — strip trailing colons, hyphens,
  commas, semicolons.
- **Title: year-as-title** — uses ZoneMap year disambiguation for
  structural handling (e.g., "2001.A.Space.Odyssey.1968").
- **Release group: language prefixes** — `HUN-nIk` → `nIk`,
  `TrueFrench-Scarface45` → `Scarface45`.
- **Episode title: Part boundary** — `Property::Part` stops extraction.

### Intentional divergences (documented)

- **`audio_bit_rate` / `video_bit_rate`**: single `bit_rate` property.
- **`mimetype`**: trivially derived from `container`; redundant.

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

[1.1.2]: https://github.com/lijunzh/hunch/releases/tag/v1.1.2
[1.1.1]: https://github.com/lijunzh/hunch/releases/tag/v1.1.1
[1.1.0]: https://github.com/lijunzh/hunch/releases/tag/v1.1.0
[1.0.1]: https://github.com/lijunzh/hunch/releases/tag/v1.0.1
[1.0.0]: https://github.com/lijunzh/hunch/releases/tag/v1.0.0
[0.3.1]: https://github.com/lijunzh/hunch/releases/tag/v0.3.1
[0.3.0]: https://github.com/lijunzh/hunch/releases/tag/v0.3.0
[0.2.2]: https://github.com/lijunzh/hunch/releases/tag/v0.2.2
[0.2.1]: https://github.com/lijunzh/hunch/releases/tag/v0.2.1
[0.2.0]: https://github.com/lijunzh/hunch/releases/tag/v0.2.0
[0.1.2]: https://github.com/lijunzh/hunch/releases/tag/v0.1.2
[0.1.1]: https://github.com/lijunzh/hunch/releases/tag/v0.1.1
[0.1.0]: https://github.com/lijunzh/hunch/releases/tag/v0.1.0
