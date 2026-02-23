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

## Architecture Overview

### How guessit works (Python)

1. **rebulk** — a generic pattern-matching engine (string + regex + functional
   matchers) with conflict resolution, match chaining, and rule-based
   post-processing.
2. **Property builders** — ~25 modules (`video_codec.py`, `source.py`, etc.)
   each register patterns on a `Rebulk` instance.
3. **Rules** — Python classes that inspect the full match set and
   add/remove/rename matches (e.g., "if there's no season but there is an
   episode, promote year to season").
4. **Config** — `options.json` carries default pattern lists so users can
   override without touching code.

### How hunch works (Rust — our design)

We do **not** port rebulk. Instead we build a simpler, faster, Rust-native
matching pipeline:

```
Input string
  │
  ├─ 1. Pre-process: split by path separators, strip extension
  │
  ├─ 2. Property matchers (each impl `PropertyMatcher`)
  │     └─ Container, VideoCodec, AudioCodec, Source, ScreenSize,
  │        Episodes, Year, Edition, ReleaseGroup, Other, …
  │     Each returns Vec<MatchSpan>
  │
  ├─ 3. Conflict resolution (MatchEngine::resolve_conflicts)
  │     └─ Overlapping spans: higher priority wins, then longer wins
  │
  ├─ 4. Post-processing rules
  │     └─ Title extraction ("whatever's left")
  │     └─ Media type inference (has season/episode → Episode, else Movie)
  │     └─ Release group extraction
  │
  └─ 5. Build Guess (BTreeMap<String, Vec<String>>)
```

### Key design decisions

| Decision | Rationale |
| -------- | --------- |
| No rebulk port | rebulk is deeply Pythonic (closures, monkey-patching). A flat Vec<MatchSpan> + sort-and-sweep is simpler and faster. |
| Patterns in code, not JSON | Rust's `lazy_static!` + `Regex` gives us compile-time validation. Config override can come later as a feature. |
| `PropertyMatcher` trait | Each property module is self-contained and testable in isolation. |
| `BTreeMap` output | Deterministic key ordering for JSON output and tests. |
| Edition 2024 | Latest Rust edition for modern syntax and features. |

---

## Module Map

```
src/
├── lib.rs                  # Public API: guess(), guess_with()
├── main.rs                 # CLI binary (clap)
├── guess.rs                # Guess result type + typed accessors
├── options.rs              # Options / configuration
├── pipeline.rs             # Orchestrates matchers → conflicts → rules → Guess
├── matcher/
│   ├── mod.rs              # Re-exports
│   ├── span.rs             # MatchSpan, Property enum
│   └── engine.rs           # Conflict resolution
└── properties/
    ├── mod.rs              # PropertyMatcher trait
    ├── container.rs        # File extension (.mkv, .mp4, …)
    ├── video_codec.rs      # H.264, H.265, AV1, …
    ├── audio_codec.rs      # AAC, DTS, Dolby, FLAC, …
    ├── source.rs           # Blu-ray, WEB-DL, HDTV, …
    ├── screen_size.rs      # 720p, 1080p, 2160p/4K, …
    ├── episodes.rs         # S01E02, 1x03, season/episode
    ├── year.rs             # 4-digit year detection
    ├── edition.rs          # Director's Cut, Extended, …
    ├── release_group.rs    # Group name (usually last, after "-")
    ├── other.rs            # HDR, Remux, Proper, Repack, …
    └── title.rs            # Title extraction (positional / leftover)
```

---

## Implementation Plan

### Phase 1 — MVP (core engine + most-used properties) ✦ CURRENT

- [x] Project scaffold (Cargo.toml, module structure)
- [x] `MatchSpan` + `Property` enum
- [x] `MatchEngine::resolve_conflicts`
- [x] `Guess` result type with typed accessors
- [x] `Options` struct
- [x] `ContainerMatcher`
- [x] `VideoCodecMatcher`
- [x] `AudioCodecMatcher`
- [x] `SourceMatcher`
- [x] `ScreenSizeMatcher`
- [x] `YearMatcher`
- [x] `EpisodeMatcher` (S01E02, 1x03, etc.)
- [x] `EditionMatcher`
- [x] `OtherMatcher` (HDR, Remux, Proper, …)
- [x] `ReleaseGroupMatcher`
- [x] `TitleExtractor` (post-processing rule)
- [x] `Pipeline` (orchestration)
- [x] CLI binary (`hunch "filename.mkv"`)
- [ ] Integration tests against guessit's YAML test vectors
- [x] Git init + .gitignore

### Phase 2 — Feature parity

- [ ] Language / subtitle language detection
- [ ] Country detection
- [ ] Streaming service detection
- [ ] Date parsing (beyond year)
- [ ] Episode title extraction
- [ ] CRC detection (`[DEADBEEF]`)
- [ ] Size / bit rate detection
- [ ] Bonus / film / part / CD
- [ ] Configurable pattern overrides (TOML config file)
- [ ] `expected_title` hints
- [ ] `name_only` mode

### Phase 3 — Polish & ecosystem

- [ ] Benchmarks vs. guessit (Python) via shared test vectors
- [ ] WASM target for browser use
- [ ] `#![no_std]` core (optional)
- [ ] Publish to crates.io
- [ ] Integration with `plex-media-organizer`

---

## Property Reference

These are the properties we aim to extract, ported from guessit's property
list. Priority order reflects typical filename structure:

| Property            | Example values                          | Phase |
| ------------------- | --------------------------------------- | ----- |
| `title`             | The Matrix, Breaking Bad                | 1     |
| `year`              | 1999, 2024                              | 1     |
| `season`            | 1, 2, 3                                 | 1     |
| `episode`           | 1, 13, 1-3                              | 1     |
| `video_codec`       | H.264, H.265, AV1, Xvid                | 1     |
| `audio_codec`       | AAC, DTS, DTS-HD, Dolby Digital         | 1     |
| `source`            | Blu-ray, WEB-DL, HDTV, DVD             | 1     |
| `screen_size`       | 720p, 1080p, 2160p, 4K                  | 1     |
| `container`         | mkv, mp4, avi                           | 1     |
| `release_group`     | YTS, SPARKS, FGT                        | 1     |
| `edition`           | Director's Cut, Extended, Unrated       | 1     |
| `other`             | HDR, Remux, Proper, Repack              | 1     |
| `episode_title`     | Pilot, The One Where…                   | 2     |
| `audio_channels`    | 5.1, 7.1, 2.0                           | 1     |
| `audio_profile`     | HD, HD-MA, HE                           | 2     |
| `video_profile`     | High, Main, Baseline                    | 2     |
| `streaming_service` | Netflix, AMZN, DSNP                     | 2     |
| `language`          | English, French                         | 2     |
| `subtitle_language` | English, Spanish                        | 2     |
| `country`           | US, GB, AU                              | 2     |
| `date`              | 2024-01-15                              | 2     |
| `color_depth`       | 10-bit, 8-bit                           | 2     |
| `frame_rate`        | 23.976fps, 60fps                        | 2     |
| `type`              | movie, episode                          | 1     |
| `crc`               | DEADBEEF                                | 2     |
| `size`              | 1.4GB                                   | 2     |
| `bit_rate`          | 128kbps                                 | 2     |

---

## Developer Guidelines

### Code style

- **Zen of Python applies**: simple > complex, explicit > implicit, flat > nested.
- **DRY**: shared regex helpers go in `matcher/` or a `utils.rs`.
- **YAGNI**: don't build Phase 2 infra in Phase 1.
- **Files under 600 lines**. If a file grows past that, split it.
- **Tests in each module** (`#[cfg(test)] mod tests`).

### Testing strategy

1. **Unit tests** in each property matcher (does this regex find what it should?).
2. **Integration tests** in `tests/` against real-world filenames.
3. **Compatibility tests** ported from guessit's YAML test vectors
   (`../guessit/guessit/test/*.yml`) — these are the ground truth.

### Adding a new property matcher

1. Create `src/properties/<name>.rs`.
2. Define a struct implementing `PropertyMatcher`.
3. Use `lazy_static!` for compiled regexes.
4. Add unit tests in the same file.
5. Register in `src/properties/mod.rs` and `src/pipeline.rs`.
6. Update this file's checklist.

### Separators & word boundaries

guessit defines separators as: ` [](){}+*|=-_~#/\\.,;:`

We use regex word boundaries (`\b`) where possible but fall back to
`(?<![a-zA-Z0-9])` / `(?![a-zA-Z0-9])` for patterns that include digits or
hyphens (where `\b` misbehaves).

### Conflict resolution strategy

1. **Priority tiers**: Extension (10) > known tokens (0) > weak/positional (-1).
2. **Overlap rule**: higher priority wins; ties broken by longer span.
3. **Same-property rule**: keep the first occurrence in the most significant
   file-path segment (innermost directory or filename).

---

## Dependencies

| Crate         | Purpose                        |
| ------------- | ------------------------------ |
| `regex`       | Pattern matching (no look-around) |
| `fancy-regex`  | Pattern matching with look-around  |
| `lazy_static` | Compile regexes once           |
| `serde`       | Serialization for Guess output |
| `serde_json`  | JSON output for CLI            |
| `clap`        | CLI argument parsing           |

### Intentionally NOT using

| What           | Why                                                     |
| -------------- | ------------------------------------------------------- |
| `babelfish`    | No Rust equivalent yet; language detection deferred to Phase 2 |
| `rebulk`       | Python-specific; we build our own simpler engine        |
| `python-dateutil` | Rust has `chrono`; will add when date parsing lands  |

---

## Reference: guessit source map

For anyone porting patterns, here's where to find them in `../guessit/`:

| hunch module       | guessit source                               |
| ------------------ | -------------------------------------------- |
| `container.rs`     | `rules/properties/container.py` + `config/options.json[container]` |
| `video_codec.rs`   | `rules/properties/video_codec.py`            |
| `audio_codec.rs`   | `rules/properties/audio_codec.py` + `config/options.json[audio_codec]` |
| `source.rs`        | `rules/properties/source.py`                 |
| `screen_size.rs`   | `rules/properties/screen_size.py` + `config/options.json[screen_size]` |
| `episodes.rs`      | `rules/properties/episodes.py` (largest file, ~900 lines) |
| `title.rs`         | `rules/properties/title.py`                  |
| `release_group.rs` | `rules/properties/release_group.py`          |
| `edition.rs`       | `rules/properties/edition.py` + `config/options.json[edition]` |
| `other.rs`         | `rules/properties/other.py` + `config/options.json[other]` |
| `year.rs`          | `rules/common/date.py` (valid_year function) |
