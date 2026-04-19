# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

<!--
Release prep checklist (per #179):
  1. Bump `version` in Cargo.toml
  2. Move "[Unreleased]" entries below into a new "[X.Y.Z] - YYYY-MM-DD" section
  3. Add the new tag to the RELEASE_TAGS array in
     docs/src/reference/release-trajectory.md (top of list)
  4. (Optional) Add a "### Performance" subsection to the new release with
     a one-liner like:
     - See <https://lijunzh.github.io/hunch/reference/release-trajectory.html>
       for bench numbers compared to vX.Y.Z-1.
  5. Tag + push: `git tag vX.Y.Z && git push origin vX.Y.Z`
  6. The Benchmarks workflow auto-publishes the snapshot to
     gh-pages/release-snapshots/vX.Y.Z.json (~3 min after the tag push)
-->

## [Unreleased]

### Added

- **`HunchResult::is_movie()`, `is_episode()`, `is_extra()` convenience
  methods.** Pure derived getters over the existing `media_type()` typed
  accessor. All three return `false` when media type is unknown rather
  than defaulting to a guess — callers needing to distinguish "definitely
  not X" from "unknown" should still use `media_type()` directly. (#156)
- **`Property::AudioBitRate`, `Property::VideoBitRate`, `Property::Mimetype`
  variants** with matching `HunchResult::audio_bit_rate()`,
  `video_bit_rate()`, `mimetype()` accessors. The bit-rate split is
  classified by unit (`Kbps` → audio, `Mbps` → video); mimetype is a
  pure derivation from container extension (mp4 → video/mp4, mkv →
  video/x-matroska, etc.; unknown → `None`, never fabricated). All three
  properties moved from 0% to 100% accuracy on the compatibility corpus.
  (#158, #165)
- **DVD region codes R0–R6** in the property exact-match table.
  Previously only R5 was recognized. R7–R9 are intentionally omitted to
  limit false positives on niche release-group tokens. (#156)

### Changed

- **⚠️ BREAKING: public enums now carry `#[non_exhaustive]`.** Affected
  enums: `Property`, `MediaType`, `Confidence`, `OutputFormat` (and any
  others reachable from `pub use src/lib.rs`). Downstream code that
  matches exhaustively on these enums **must** add a wildcard arm:

  ```rust
  match prop {
      Property::Title => ...,
      // ... existing arms ...
      _ => ...,  // ← now required
  }
  ```

  Why: this lets future minor releases add new variants (the bit-rate
  split in #165 was the immediate trigger) without re-breaking the API
  every time. (#172)

### Fixed

- **Website false-positives on country-code TLDs inside language
  abbreviations.** Filenames like `Community.s02e20.rus.eng.720p.mkv`
  no longer extract `s02e20.ru` as a website. The TLD alternation now
  requires a trailing word boundary, so `.ru` cannot match inside
  `.rus`, `.com` inside `.community`, etc. (#163, #167)
- **Anime-release bit-rate notation** (`kbit`, `mbits`) now parsed
  correctly via suffix alternation. (#165)
- **`DD5.1.448kbps`-style filenames** no longer mis-parse the leading
  digits as part of the bit-rate (regex bound tightened to `\d{1,2}`).
  (#165)

### Deprecated

- **`Property::BitRate` variant** — superseded by the
  `AudioBitRate`/`VideoBitRate` split in #165. The variant is retained
  for enum-API stability but no parser path produces it. Callers should
  migrate to the unit-typed variants.

### Internal / Infrastructure

This release lands a substantial CI and documentation investment
motivated by the project moving from "experimental, no users" to
"users filing real bug reports." None of the items below change
parser behavior, but they meaningfully improve the project's
ability to catch regressions before they ship:

- **Code coverage tracking** via `cargo-llvm-cov` (advisory). (#145, #168)
- **Mutation testing baseline** via `cargo-mutants` nightly, with
  29 surviving mutants triaged and killed across
  #175, #180, #181, #182, #183, #184, #185. (#146, #169, #170, #173)
- **Fuzz baseline** via `cargo-fuzz` with two targets (single-string
  parse + multi-segment path) and nightly CI. (#147, #174)
- **Public API surface tripwire** that fails CI on accidental changes
  to `pub use src/lib.rs` items. (#144, #171)
- **Continuous benchmarking** via `criterion` + `github-action-benchmark`,
  with PR-time regression gating (>120% threshold), per-commit history
  on a live dashboard, and per-release immutable snapshots. See the
  [Benchmarks reference](https://lijunzh.github.io/hunch/reference/benchmarks.html)
  and [Release Trajectory](https://lijunzh.github.io/hunch/reference/release-trajectory.html).
  (#148, #176, #177, #178, #179, #186, #189, #191, #192, #194)
- **Documentation portal** at <https://lijunzh.github.io/hunch/>
  built with mdbook. (#188, #190)
- **Release pipeline hardening** — PR-time CI now also runs on release
  branches; release workflow is more defensive. (#150, #151, #152, #159)
- **Misc test additions** pinning behaviors against future regressions:
  `TitleStrategy` fallback ordering (#154, #161), `cli_walk_dir`
  safety boundaries (#153, #162), parse-torrent-name corpus pins
  (#157, #164).

## [1.1.8] - 2026-04-17

### Changed

- **`--batch -r` now bounds recursion depth and skips symlinks.** Recursive
  directory walks (`hunch --batch <dir> -r`) cap at 32 levels deep and
  silently skip symbolic links — both regular files and directories.
  Defends against denial-of-service via deeply nested trees (stack
  overflow) and symlink loops (infinite recursion). Users with curated
  libraries that rely on symlinks (e.g., a `Movies/` directory built from
  NAS symlinks) will see fewer or zero results in v1.1.8 — either follow
  the symlinks before invoking hunch, or run hunch on the original
  directory tree. (#137)

### Fixed

- **Anime titles containing `" - "` and `"Part N"`** — in `[Group] Show - Sub
  Part 2 - 13 [tags]` style filenames, the title is now extracted as the full
  `Show - Sub Part 2`. Previously the parser truncated at the first `" - "`
  and incorrectly extracted `Part 2` as a standalone `part` property.
  (#124, #127)

### Refactored

- **Pipeline `rule_registry` extracted** from `pipeline/mod.rs` into its own
  module. Centralizes the legacy / TOML rule registration so the pipeline
  orchestration stays at the orchestration layer of abstraction. (#134)
- **Title `find_title_boundary` renamed** for clarity, with documented
  semantics and a pinned caveat preventing accidental re-introduction of the
  pre-rename behavior. (#128 Debt #4, #133)
- **Title fallback extractors unified** behind a new `TitleStrategy` trait.
  The 5–6 ad-hoc extractor functions are now first-class strategy types in
  `properties/title/strategies/`, registered in a single ordered fallback
  list. (#128 Debt #1, #132)
- **Part reclaimable when Episode present.** `Part N` matches in the same
  set as an `Episode` match are now marked reclaimable so the existing
  title-absorption step can fold them into the title uniformly. Replaces
  the bespoke `absorb_part_into_title` post-hoc corrector (in line with the
  D10 "no post-hoc correctors" tripwire). (#128 Debt #3, #131)
- **`clean_title` decomposed** into composable transforms (`strip_*`,
  `normalize_separators`, `trim_trailing_punct`, `strip_trailing_keywords`,
  `clean_title_preserve_dashes`, `DashPolicy`). Each transform is
  individually testable and composable; `clean_title` becomes a thin
  orchestrator. (#128 Debt #2, #130)
- **`mark_reclaimable_when_episode_present` visibility tightened** from
  `pub` to `pub(crate)`. Internal-only helper; never intended as part of
  the public API surface. (release-prep)

### Tests

- **Three regression scenarios pinned** as named tests in dedicated files:
  flat-batch warning hint, parent-context propagation, and wrong-type path
  inference. Prevents silent regression of behaviors that previously had
  only ad-hoc coverage. (#138)
- **`tests/cli_walk_dir_safety.rs`** added alongside #137 with four
  scenarios: deep-tree depth bound (40 levels, control file at depth 1);
  realistic-depth happy path (depth 6); `cfg(unix)` symlink-loop containment
  (counts occurrences to prove non-following); outside-root symlink-escape
  rejection. (#137)

### Docs

- **`SECURITY.md` added** at repo root with threat model, vulnerability
  reporting procedure (private GitHub Security Advisories), and explicit
  in-scope / out-of-scope categorization. (#139)
- **API Stability Policy** added to `CONTRIBUTING.md` documenting the hard
  vs. soft public-API contract: `hunch::Pipeline`, `HunchResult`, `MediaType`,
  `Confidence`, `Property`, and the top-level `hunch()` / `hunch_with_context()`
  functions are SemVer-stable; `properties::*` submodules are explicitly
  unstable. (#139)
- **`DESIGN.md` promoted** to a root-level document (was `docs/design.md`).
  Adds D10 "Refactor before accreting" with three concrete tripwire rules:
  no post-hoc correctors, no parallel matchers, no growing dispatchers.
  (#129, #135)
- **`docs/user_manual.md`** updated to document `-r` recursion behavior:
  symlinks are skipped (loop-safe), traversal stops at 32 levels deep.
  (release-prep, paired with #137)
- **Doc drift cleanup** — README, CONTRIBUTING, user_manual, and
  compatibility cross-references audited and refreshed against current
  source state. (#136)
- **Compatibility report** refreshed: 1072 / 1311 fixtures pass (81.8%),
  up from 1071 / 1309 in v1.1.7 (two fixtures added, one new pass).
  (release-prep)

### CI

- **`cargo-semver-checks` PR-time gate** added. Detects accidental
  SemVer-incompatible changes to the public Rust API by comparing PR head
  against the latest crates.io release. Blocks breaking changes within a
  major version line. (#142)
- **Cross-OS PR matrix** — `Check` and `Test` jobs now run on
  ubuntu-latest, macos-latest, and windows-latest. Catches
  platform-conditional compile errors and path-handling differences before
  release time. (#141)
- **Security hardening of CI workflows.** All third-party actions SHA-pinned
  with version comments (defends against tag-republishing supply-chain
  attacks). `cargo audit` now hard-fails on RUSTSEC vulnerabilities (was
  silenced by `|| true`). Dependabot auto-merge metadata-gated to
  patches-only and dev/CI-tooling minor bumps; major bumps and runtime-dep
  minor bumps now require manual review. Two yanked transitive dev-deps
  refreshed (`js-sys 0.3.88` → `0.3.95`, `wasm-bindgen 0.2.111` →
  `0.2.118`). Default `permissions: contents: read` on `ci.yml`. (#140)

### Repository governance

- **`.gitignore` hardened** with broad patterns for accidental secret /
  credential commits (`.env*`, `*.pem`, `*.key`, `id_rsa*`, `secrets*`,
  `credentials.json`, `service-account*.json`). (#139)

## [1.1.7] - 2026-03-23

### Fixed

- **Bracket metadata leakage** — bracketed metadata in CJK/anime filenames no
  longer leaks into `episode_title`, and release-group extraction now prefers
  the actual first bracket group instead of bracket fragments. (#92)
- **Generic category directories** — library/category directories like
  `English/`, `Japanese/`, `Anime/`, and CJK bonus folders are filtered more
  aggressively so they do not become titles. (#95)
- **Parent-context fallback in batch mode** — files in sparse extras/specials
  subdirectories now fall back to parent-directory context more reliably during
  recursive batch parsing. (#96)
- **Empty intermediate directory propagation** — recursive batch parsing now
  preserves useful parent context through empty/intermediate directory layers
  instead of dropping title hints. (#98)
- **Explicit movie signals override `tv/` path hints** — filenames and parent
  directories containing strong movie cues such as `The Movie`, `... Movie`,
  and `劇場版` now classify as `type=movie` even inside TV-oriented directory
  trees. (#99)
- **Natural-language first brackets** — filenames like
  `[Kimetsu no Yaiba Mugen Ressha Hen][JPN+ENG]...` now treat the first bracket
  as `title` when it looks like natural language instead of a release group.
  (#100)

### Docs

- Added a README **Known Limitations** section documenting the main remaining
  edge-case categories and their tradeoffs. (#103)

## [1.1.6] - 2026-03-22

### Added

- **`MediaType::Extra`** — new media type variant for supplementary content
  (NCED, NCOP, OP, ED, SP, PV, CM, OVA, OAD, ONA, Menu, Tokuten). Files
  with `episode_details` but no episode/season/date markers now return
  `type=extra` instead of `type=episode`. The specific marker remains
  accessible via `episode_details()`. (#89)
- **Recursive `--batch -r`** — new `-r`/`--recursive` flag walks the full
  directory tree and groups siblings per-directory. Enables cross-file title
  extraction for deeply nested libraries (`tv/Show/Season 1/01.mkv` →
  `title: "Show"`). (#66)
- **Library ergonomics** — `Property` re-exported at crate root
  (`use hunch::Property`); 10 new typed accessors on `HunchResult`
  (`episode_details()`, `language()`, `languages()`, `subtitle_language()`,
  `subtitle_languages()`, `bonus()`, `date()`, `film()`, `disc()`,
  `media_type()`); `MatchSpan::value` implements `AsRef<str>`. (#73)
- **Flat `--batch` warning** — when `--batch <dir>` is used without `-r`
  and subdirectories contain media files being skipped, hunch prints a hint
  to stderr suggesting `--batch -r`. (#74)

### Fixed

- **"Movie N" parsed as episode** — `Detective.Conan.Movie.10...` in a
  `movie/` directory now returns `type=movie`. Bare number matches at
  HEURISTIC priority lose to movie-directory path context; strong S/E
  markers still win. (#88)
- **Missing anime bonus markers** — SP, OVA, OAD, ONA, OP, ED, and MENU
  tokens now emit `episode_details`, fixing classification of common anime
  BD bonus content. (#68)
- **Batch mode parent dir fallback** — `--batch` now passes
  `parent_dir/filename` to the pipeline so `extract_title_from_parent()`
  has directory context. Fixes ~860 files that previously parsed without a
  title. (#62)
- **Batch siblings invariance** — siblings passed to the invariance engine
  now include the parent directory path so the invariant title text (e.g.,
  "Paw Patrol") is correctly identified and suppressed from episode titles.
  (#63)

### Changed

- **Named priority constants** — new `src/priority.rs` module exposes
  `STRUCTURAL`, `KEYWORD`, `VOCABULARY`, `DEFAULT`, `HEURISTIC`,
  `POSITIONAL` tiers (and others) as named constants. Replaces magic
  integers throughout the codebase. (#85)
- **Named zone rules** — zone rules are now referred to by descriptive
  names (e.g., `language_in_title_zone`) instead of numbers (Rule 1,
  Rule 2, …). (#86)

### Docs

- Added `--batch -r` flag to CLI help, README, and user manual. (#69)
- Added P5 principle (surface ambiguity) and updated D6 in design.md. (#76)
- Restructured design.md: separated principles, decisions, and boundaries
  into distinct sections. (#77, #78)
- Added Mission section to design.md — hunch is not a guessit port. (#79)
- Scoped D7 to reflect reality; acknowledged D9 matcher classes. (#84)

### Tests

- Added CLI integration tests for the flat-batch subdirectory warning. (#75)

## [1.1.5] - 2026-03-20

### Added

- **CJK episode markers** (`第N話`, `第N集`, `第N回`, `第N话`) — structural
  pattern recognition for Japanese and Chinese episode numbering. Full-width
  digit normalization (０-９ → 0-9) included. (#46)
- **Anime bonus vocabulary** — NCOP, NCED, PV, CM tokens emit
  `EpisodeDetails`, correctly classifying bonus content as episodes. (#46)
- **Path-based type inference** — directory names (`tv/`, `anime/`,
  `donghua/`, `Season N/`, `sN/`) force `MediaType::Episode` even when
  the filename alone lacks episode markers. (#46)
- **InvarianceReport** with year/episode signal detection — cross-file
  sequential analysis identifies bare numbers as episodes and suppresses
  invariant years from metadata. (#47, #48)
- **Source tagging** (`Structural`, `Context`, `Heuristic`) on all
  `MatchSpan`s — heuristic-only results cap confidence at Medium. (#47, #48)
- 28 new integration tests (370 → 386 total) covering CJK markers,
  path inference, invariance signals, cross-feature interactions, and
  panic safety edge cases.

### Changed

- **`find_invariant_text`** now returns `(usize, String)` — pre-computed
  byte offset eliminates fragile `input.find()` re-search that could match
  the wrong occurrence for short/repeated title strings.
- **`find_invariant_text`** accepts `&[&[UnclaimedGap]]` instead of
  cloning all gap Vecs (zero-copy).
- **Year signal expansion** sorts signals by `.start` before the loop,
  preventing non-adjacent text from being glued into titles.
- **Heuristic eviction guard** — `apply_invariance_signals` now checks
  for non-heuristic overlaps *before* evicting heuristic matches,
  preventing data loss when a codec or screen-size match occupies the
  same span.
- **Trailing Part regex** hoisted to `LazyLock<Regex>` (was compiled
  per-call in episode title extraction).
- **`is_episode_directory`** uses `strip_prefix('s')` instead of
  `component[1..]` byte indexing for safe UTF-8 handling.

### Fixed

- **`CODEC_NUMBERS` shared constant** (264, 265, 128) — extracted from
  duplicated checks in `invariance.rs` and `episodes/mod.rs`. (DRY)
- Stale SP comment orphan removed from `anime_bonus.toml`.
- Unused `_input` parameter removed from `apply_invariance_signals`.
- `.unwrap()` → `.expect()` on CJK regex capture groups.

## [1.1.4] - 2026-03-20

### Added

- **Cross-file context for title extraction** (`run_with_context`, `hunch_with_context`) —
  when sibling filenames are provided, hunch identifies the invariant text across
  files as the title. Dramatically improves CJK and non-standard filename parsing. (#47)
- **CLI `--context <dir>`** flag — use sibling files from a directory for
  improved title detection.
- **CLI `--batch <dir>`** flag — parse all media files in a directory with
  mutual cross-file context.
- **`Confidence` enum** on `HunchResult` — `High | Medium | Low` based on
  structural signals (tech anchors, title quality, cross-file context).
- Low-confidence CLI warning suggesting `--context` when results are uncertain.
- Architecture documentation for cross-file context design decisions. (#48)
- 10 matching constraint tests covering `not_before`, `not_after`,
  `requires_context`, `requires_nearby`, side effects, compound windows,
  zone scoping, and reclaimable matches.

### Changed

- **Pipeline refactored** into `pass1()` / `pass2()` for reuse by cross-file
  context. No behavior change for existing `run()` callers.
- **`Token::lower()` now cached** — lowercased text computed once at
  tokenization, eliminating 6+ redundant allocations per token in matching.
- **`trim_title_suffix` zero-alloc** — uses `&str` slices instead of cloning
  in a loop.
- **CLI deps feature-gated** — `clap` and `env_logger` now behind the `cli`
  feature (enabled by default). Library consumers no longer pull in CLI
  dependencies.
- `--batch` now properly conflicts with positional filename args.
- `list_media_files` signature: `&PathBuf` → `&Path` (idiomatic Rust).

### Fixed

- Stale doc-links pointing to `hunch` instead of `hunch_with_context`.
- `Pipeline` doc comment merged with `SegmentScope` doc (missing blank line).
- ARCHITECTURE.md pass rate updated to 81.8%.
- README.md: removed deleted `options.rs`, updated test count to 333.

## [1.1.3] - 2026-03-19

### Changed

- **Overall pass rate: 81.7% → 82.2%** (1,069 → 1,076 / 1,309).
- **Structure-aware neighbor-context disambiguation** — replaced fragile
  positional heuristics ("first half of title zone", "before the anchor",
  "unmatched bytes ratio") with principled structural reasoning based on
  what actually surrounds each token. New `token_context` module provides:
  - **Neighbor roles**: Score adjacent tokens as title words vs tech tokens.
  - **Peer reinforcement**: Adjacent tokens of the same property type
    (e.g., FRENCH next to ENGLISH) signal a metadata cluster.
  - **Structural separators**: Tokens after " - " or in brackets are
    metadata, not title content.
  - **Structural fallback**: Edge-of-segment tokens use position relative
    to first tech anchor as tiebreaker.
  - **Duplicate detection**: Same value in firm tech context elsewhere
    drops the title-zone instance.
- **Structure-aware episode title extraction** — episode title is now
  extracted from whichever path segment contains the episode anchor,
  not hardcoded to the leaf filename.
- **TOML-driven disambiguation** — new `requires_nearby` and
  `reclaimable` fields in TOML rules reduce Rust-side special-casing.

### Improved

- **language: 80.3% → 81.0%** — neighbor context + peer reinforcement.
- **title: 91.8% → 92.0%** — better language filtering.
- **episode_title: 73.6% → 76.1%** — parent-dir extraction, boundary fixes.
- **other: 88.8% → 89.1%** — TOML-driven `requires_nearby` for "Proper".

### Fixed

- Episode title extraction from parent directories when the leaf filename
  contains only a numeric code (e.g., `Bones.S12E02.The.Brain.In.The.Bot
  .1080p.WEB-DL/161219_06.mkv` → episode_title: "The Brain In The Bot").
- Language "FR" after " - " separator no longer dropped
  (`Love Gourou (Mike Myers) - FR` → language: French).
- Adjacent language tokens now reinforce each other as metadata
  (`QC.FRENCH.ENGLISH.NTSC` → both languages detected).
- JSON numeric coercion limited to semantically numeric properties.
- Added BDMux/BRMux/BDRipMux/BRRipMux source patterns.
- Multi-segment alternative_title with earliest-boundary fix.

### Refactored

- `Property` enum uses `define_properties!` macro (DRY).
- 8 positional args replaced with `MatchContext` struct.
- `known_tokens.rs` renamed to `validation.rs`.

### Removed

- `Options` struct, `hunch_with()`, `--type`/`--name-only` CLI flags.
  These were dead code from v1.0.0 (never wired into the pipeline).
- `src/options.rs` module deleted.

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

[1.1.8]: https://github.com/lijunzh/hunch/releases/tag/v1.1.8
[1.1.7]: https://github.com/lijunzh/hunch/releases/tag/v1.1.7
[1.1.6]: https://github.com/lijunzh/hunch/releases/tag/v1.1.6
[1.1.5]: https://github.com/lijunzh/hunch/releases/tag/v1.1.5
[1.1.4]: https://github.com/lijunzh/hunch/releases/tag/v1.1.4
[1.1.3]: https://github.com/lijunzh/hunch/releases/tag/v1.1.3
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
