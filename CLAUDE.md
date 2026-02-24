# CLAUDE.md — Hunch Developer Guide

> **Hunch**: A media filename parser for Rust — spiritual descendant of Python's
> [guessit](https://github.com/guessit-io/guessit).

---

## Project Identity

| Field       | Value                                                        |
| ----------- | ------------------------------------------------------------ |
| Crate name  | `hunch`                                                      |
| Language    | Rust (edition 2024)                                          |
| License     | MIT                                                          |
| Ancestor    | Python `guessit` (LGPLv3) — patterns/knowledge ported, engine rewritten |
| Goal        | Fast, correct, zero-copy-friendly media filename parsing     |

---

## Current Status

**Pass rate: 53.6%** (702 / 1,309 guessit test cases) with all 39 properties
implemented (zero skipped).

### Accuracy Tiers

| Tier | Properties | Count |
|------|-----------|-------|
| ✅ 100% | year, aspect_ratio, bonus, color_depth, film, size, streaming_service | 7 |
| ✅ 95–99% | video_codec, screen_size, container, crc32, source | 5 |
| ✅ 90–95% | audio_codec, type, website, audio_channels, season | 5 |
| 🟡 80–90% | date, uuid, episode_details, release_group, audio_profile, title, edition, subtitle_language | 8 |
| ⚠️ 60–80% | country, episode, proper_count, other, part, language, episode_title, bonus_title, cd, video_profile | 10 |
| ❌ <60% | disc, cd_count, alternative_title, version, absolute_episode, frame_rate, film_title, episode_count, + 5 more | 15 |

---

## Architecture Overview

### How guessit works (Python)

1. **rebulk** — generic pattern-matching engine with conflict resolution,
   match chaining, and rule-based post-processing.
2. **Property builders** — ~25 modules register patterns on a `Rebulk` instance.
3. **Rules** — Python classes that inspect the full match set and
   add/remove/rename matches.
4. **Config** — `options.json` carries default pattern lists.

### How hunch works (Rust)

We do **not** port rebulk. Instead we use a simpler, faster pipeline:

```
Input string
  │
  ├─ 1. Pre-process: split by path separators, strip extension
  │
  ├─ 2. Property matchers (27 modules, each impl `PropertyMatcher`)
  │     Each returns Vec<MatchSpan> with priority levels
  │
  ├─ 3. Conflict resolution (MatchEngine::resolve_conflicts)
  │     Overlapping spans: higher priority wins, then longer wins
  │
  ├─ 4. Post-processing rules
  │     ├─ Title extraction (largest unclaimed region before tech tokens)
  │     ├─ Episode title extraction (unclaimed region after episode number)
  │     ├─ Release group extraction (after last separator)
  │     └─ Media type inference (has season/episode → Episode, else Movie)
  │
  └─ 5. Build Guess (BTreeMap<String, Vec<String>>) → JSON
```

### Key design decisions

| Decision | Rationale |
| -------- | --------- |
| No rebulk port | rebulk is deeply Pythonic. A flat `Vec<MatchSpan>` + sort-and-sweep is simpler and faster. |
| Patterns in code, not JSON | Rust's `lazy_static!` + `Regex` gives compile-time validation. Config override can come later. |
| `PropertyMatcher` trait | Each property module is self-contained and testable in isolation. |
| `BTreeMap` output | Deterministic key ordering for JSON output and tests. |
| `fancy_regex` for look-arounds | Rust's `regex` crate doesn't support look-behind/ahead; `fancy_regex` fills that gap. |
| `ValuePattern` helper | Pairs a compiled regex with a canonical output value for clean pattern tables. |
| Edition 2024 | Latest Rust edition for modern syntax. |

---

## Module Map

```
src/
├── lib.rs                  # Public API: parse()
├── main.rs                 # CLI binary (clap)
├── guess.rs                # GuessResult type + typed accessors + JSON serialization
├── options.rs              # Options / configuration
├── pipeline.rs             # Orchestrates matchers → conflicts → rules → Guess
├── matcher/
│   ├── mod.rs              # Re-exports
│   ├── span.rs             # MatchSpan, Property enum (39 variants)
│   ├── engine.rs           # Conflict resolution (priority + length)
│   └── regex_utils.rs      # ValuePattern: compiled regex + canonical value
└── properties/             # 27 property matcher modules
    ├── mod.rs              # PropertyMatcher trait definition
    ├── title.rs            # Title extraction (positional / leftover) ~560 lines
    ├── episodes.rs         # S01E02, 1x03, season/episode, multi-ep ~599 lines
    ├── year.rs             # 4-digit year detection
    ├── container.rs        # File extension (.mkv, .mp4, .srt, …)
    ├── video_codec.rs      # H.264, H.265, AV1, Xvid, …
    ├── audio_codec.rs      # AAC, DTS, Dolby, FLAC, Opus, …
    ├── source.rs           # Blu-ray, WEB-DL, HDTV, DVDRip, …
    ├── screen_size.rs      # 720p, 1080p, 2160p/4K, 480i, …
    ├── edition.rs          # Director's Cut, Extended, Unrated, …
    ├── release_group.rs    # Group name (after last "-", brackets, etc.)
    ├── other.rs            # HDR, Remux, Proper, Repack, 3D, …
    ├── language.rs         # Audio language: English, French, Multi, …
    ├── subtitle_language.rs # VOSTFR, NLsubs, SubForced, sub.FR, … ~400 lines
    ├── streaming_service.rs # AMZN, NF, HMAX, DSNP, ATVP, …
    ├── audio_profile.rs    # Master Audio, Atmos, DTS:X, DDP, …
    ├── video_profile.rs    # HEVC, AVCHD, Hi10P, HP, SVC, …
    ├── color_depth.rs      # 10-bit, 8-bit, 12-bit, Hi10, HEVC10
    ├── aspect_ratio.rs     # Computed from WxH resolution
    ├── date.rs             # YYYY-MM-DD, YYYYMMDD, MM-DD-YYYY, …
    ├── country.rs          # US, UK, GB, CA, AU, NZ
    ├── crc32.rs            # [DEADBEEF] hex checksums
    ├── website.rs          # [site.com], www.domain.com, inline
    ├── uuid.rs             # Standard UUIDs + 32-char no-dash compact
    ├── bonus.rs            # x01/x02 extras, bonus titles
    ├── part.rs             # Part/Disc/CD/Film numbering
    ├── size.rs             # 700MB, 1.4GB file sizes
    └── episode_details.rs  # Special, Pilot, Unaired, Final
```

---

## Implementation Status

### Phase 1 — Core Engine + Most-used Properties ✅ COMPLETE

- [x] Project scaffold (Cargo.toml, module structure)
- [x] `MatchSpan` + `Property` enum (39 variants)
- [x] `MatchEngine::resolve_conflicts`
- [x] `GuessResult` type with typed accessors
- [x] `Options` struct
- [x] All core matchers: Container, VideoCodec, AudioCodec, Source,
      ScreenSize, Year, Episodes, Edition, Other, ReleaseGroup
- [x] `TitleExtractor` (post-processing rule)
- [x] Integration tests (`tests/integration.rs` — 27 tests)
- [x] `Pipeline` (orchestration)
- [x] CLI binary (`hunch "filename.mkv"`)
- [x] Rust regression suite (`tests/guessit_regression.rs` — 22 fixture files with ratchet floors)
- [x] Benchmark suite (`benches/parse.rs`)
- [x] Self-contained test fixtures in `tests/fixtures/` (no external `../guessit` repo needed)
- [x] Git init + .gitignore

### Phase 2 — Feature Parity ✅ COMPLETE

- [x] Language detection (French, Multi, VOSTFR, …)
- [x] Subtitle language detection (VOSTFR, NLsubs, SubForced, sub.FR, …)
- [x] Country detection (US, UK, GB, CA, AU, NZ)
- [x] Streaming service detection (AMZN, NF, HMAX, DSNP, …)
- [x] Date parsing (YYYY-MM-DD, YYYYMMDD, MM-DD-YYYY, YYYYxMM.DD)
- [x] Episode title extraction
- [x] CRC32 detection (`[DEADBEEF]`)
- [x] File size detection (700MB, 1.4GB)
- [x] Bonus / film / part / disc / CD numbering
- [x] Audio profile (Master Audio, Atmos, DTS:X, DDP)
- [x] Video profile (HEVC, AVCHD, Hi10P, HP, SVC)
- [x] Color depth (10-bit, 8-bit, 12-bit)
- [x] Aspect ratio (computed from WxH resolution)
- [x] UUID detection (standard + compact)
- [x] Website detection (brackets, inline, multi-part TLD)
- [x] Episode details (Special, Pilot, Unaired, Final)
- [ ] Configurable pattern overrides (TOML config file)
- [ ] `expected_title` hints
- [ ] `name_only` mode

### Phase 3 — Polish & Ecosystem (TODO)

- [ ] Benchmarks vs. guessit (Python) via shared test vectors
- [ ] WASM target for browser use
- [ ] `#![no_std]` core (optional)
- [ ] Publish to crates.io
- [ ] Integration with `plex-media-organizer`

---

## Known Gaps & Improvement Areas

### Title extraction (81.6%)

The hardest problem. Title is "everything that's left" after all technical
tokens are claimed. Key challenges:
- Parent directory title fallback (path-based inputs)
- Titles with colons, hyphens, or dots that look like separators
- Titles containing year-like numbers (e.g., "2001: A Space Odyssey")
- Anime titles with brackets and group tags

### Episode title (61.7%)

Requires positional awareness: the episode title is typically the unclaimed
region between the episode number and the first technical token. Tricky
because it overlaps with the release group zone.

### Other flags (71.1%)

Many niche patterns remain: OAD, OAR, PROOFFIX, various Screener variants,
FanSub markers, etc. Each is a small regex addition.

### Multi-value subtitle_language

Patterns like `ST{Fr-Eng}` (both French and English subtitles) need
compound parsing that splits within brackets. Currently extracts only
the first language.

---

## Developer Guidelines

### Code style

- **Zen of Python applies**: simple > complex, explicit > implicit, flat > nested.
- **DRY**: shared regex helpers go in `matcher/regex_utils.rs` (`ValuePattern`).
- **YAGNI**: don't build Phase 3 infra now.
- **Files under 600 lines**. If a file grows past that, split it.
  `episodes.rs` (599) and `title.rs` (560) are at the limit.
- **Tests in each module** (`#[cfg(test)] mod tests`).

### Testing strategy

1. **Unit tests** in each property matcher (`#[cfg(test)]` blocks) — 140 tests.
2. **Integration tests** (`tests/integration.rs`) — 27 hand-written end-to-end tests.
3. **Regression tests** (`tests/guessit_regression.rs`) — 22 fixture files with
   ratchet-pattern minimum pass rates. Floors are set to (actual − 2%) and should
   only go up. Includes language normalization (ISO codes).
4. **Compatibility report** — run `cargo test compatibility_report -- --ignored --nocapture`
   for a full per-property and per-file accuracy breakdown.
5. **Benchmarks** (`benches/parse.rs`) — Criterion benchmarks for parse performance.
6. All fixtures are self-contained in `tests/fixtures/` (no external repos needed).
   22 fixture files: `movies.yml`, `episodes.yml`, `various.yml`,
   `rules/{audio_codec,bonus,cd,common_words,country,date,edition,episodes,film,
   language,other,part,release_group,screen_size,size,source,title,video_codec,
   website}.yml`.

### Adding a new property matcher

1. Create `src/properties/<name>.rs`.
2. Define a struct implementing `PropertyMatcher`.
3. Use `lazy_static!` for compiled regexes (or `ValuePattern` for simple cases).
4. Add unit tests in the same file.
5. Add `Property::YourProp` variant to `src/matcher/span.rs`.
6. Register in `src/pipeline.rs` (add to matcher list + map to output key).
7. Update this file.

### Regex conventions

- **Word boundaries**: Use `(?<![a-zA-Z])` / `(?![a-zA-Z])` (requires `fancy_regex`).
  Standard `\b` misbehaves with digits and hyphens.
- **Case insensitive**: Prefix patterns with `(?i)` where needed.
- **ValuePattern**: For simple keyword → value mappings, use `ValuePattern::new(regex, value)`.
  It pairs a compiled `fancy_regex` with a canonical output string.

### Conflict resolution strategy

1. **Priority tiers**: Extension (10) > known tokens (0) > weak/positional (-1/-2).
2. **Overlap rule**: higher priority wins; ties broken by longer span.
3. **Same-property rule**: keep the first occurrence in the most significant
   file-path segment (innermost directory or filename).
4. **Multi-value**: Some properties support multiple values (episode, language,
   subtitle_language, other). These are serialized as JSON arrays.

---

## Dependencies

| Crate         | Purpose                           |
| ------------- | --------------------------------- |
| `regex`       | Pattern matching (no look-around) |
| `fancy-regex` | Pattern matching with look-around |
| `lazy_static` | Compile regexes once at startup   |
| `serde`       | Serialization for Guess output    |
| `serde_json`  | JSON output for CLI               |
| `clap`        | CLI argument parsing              |

---

## Reference: guessit source map

For porting patterns, find the originals in the
[guessit repo](https://github.com/guessit-io/guessit) under `guessit/rules/properties/`:

| hunch module           | guessit source                                        |
| ---------------------- | ----------------------------------------------------- |
| `container.rs`         | `rules/properties/container.py` + `config/options.json` |
| `video_codec.rs`       | `rules/properties/video_codec.py`                     |
| `audio_codec.rs`       | `rules/properties/audio_codec.py`                     |
| `source.rs`            | `rules/properties/source.py`                          |
| `screen_size.rs`       | `rules/properties/screen_size.py`                     |
| `episodes.rs`          | `rules/properties/episodes.py` (~900 lines)           |
| `title.rs`             | `rules/properties/title.py`                           |
| `release_group.rs`     | `rules/properties/release_group.py`                   |
| `edition.rs`           | `rules/properties/edition.py`                         |
| `other.rs`             | `rules/properties/other.py`                           |
| `language.rs`          | `rules/properties/language.py`                        |
| `subtitle_language.rs` | `rules/properties/language.py` (subtitle patterns)    |
| `streaming_service.rs` | `rules/properties/streaming_service.py`               |
| `audio_profile.rs`     | `rules/properties/audio_codec.py` (profile section)   |
| `video_profile.rs`     | `rules/properties/video_codec.py` (profile section)   |
