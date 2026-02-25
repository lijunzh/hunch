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

**Pass rate: ~73%** (957 / 1,309 guessit test cases) with 43 properties
implemented. Currently in v0.2 TOML migration — running TOML + legacy
matchers in parallel. ~1-2% temporary regression during migration; floors
adjusted to accommodate. Will recover once legacy matchers are trimmed.

### Accuracy Tiers

| Tier | Properties | Count |
|------|-----------|-------|
| ✅ 100% | aspect_ratio, bonus, color_depth, edition, film, size, streaming_service, episode_details, version, frame_rate, episode_count, season_count | 12 |
| ✅ 95–99% | video_codec, screen_size, container, crc32, source, year | 6 |
| ✅ 90–95% | audio_codec, proper_count, type, season, website, audio_channels | 6 |
| 🟡 80–90% | date, uuid, release_group, title, subtitle_language, episode | 6 |
| ⚠️ 60–80% | other, country, audio_profile, language, part, bonus_title, episode_title, cd, video_profile | 9 |
| ❌ <60% | disc, cd_count, alternative_title, absolute_episode, film_title, + 4 more | 9+ |

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

### v0.2 Pipeline Architecture

```
Input string
  │
  ├─ 1. Tokenize: split on separators, extract extension
  │     └─ TokenStream { tokens: [{text, start, end}...], extension }
  │
  ├─ 2a. TOML rules: iterate tokens + multi-token windows (1–3)
  │      └─ 18 TOML rule files (exact lookups + regex with {N} capture templates)
  │
  ├─ 2b. Legacy matchers: regex against raw input (to be removed)
  │
  ├─ 2c. Extension → Container span (priority 10)
  │
  ├─ 3. Conflict resolution (sort by priority desc, length desc; sweep)
  │
  ├─ 4. Zone-based disambiguation (6 rules)
  │     ├─ Rule 1: Language in title zone → drop
  │     ├─ Rule 2: Duplicate source in title zone → drop early
  │     ├─ Rule 3: Redundant HD tags with UHD source → drop
  │     ├─ Rule 4: EpisodeDetails before episode marker → drop
  │     ├─ Rule 5: Other overlapping ReleaseGroup → drop ambiguous
  │     └─ Rule 6: Language contained within tech span → drop (replaces fancy_regex lookbehind)
  │
  ├─ 5. Post-processing: title, episode_title, media_type, proper_count
  │
  └─ 6. Build HunchResult
```

### v0.2 TOML Migration Status

| TOML Rule File | Property | Capture Groups | Legacy Matcher Still Needed? |
|---------------|----------|:-:|---|
| video_codec.toml | VideoCodec | ✗ | Yes (compound patterns like DLx264) |
| audio_codec.toml | AudioCodec | ✗ | Yes (DD5.1 combined codec+channel) |
| color_depth.toml | ColorDepth | ✗ | No — TOML-only |
| streaming_service.toml | StreamingService | ✗ | Yes (Amazon lookahead) |
| video_profile.toml | VideoProfile | ✗ | Yes (case-sensitive HP/SC) |
| episode_details.toml | EpisodeDetails | ✗ | Yes (compound patterns) |
| edition.toml | Edition | ✗ | Yes (DDC/DC/SE lookaround) |
| country.toml | Country | ✗ | Yes (NZ boundary) |
| audio_profile.toml | AudioProfile | ✗ | No — TOML-only |
| other.toml | Other | ✗ | Yes (many compound patterns) |
| other_weak.toml | Other (weak) | ✗ | No — TOML-only |
| video_api.toml | VideoApi | ✗ | No — TOML-only |
| source.toml | Source | ✗ | Yes (Rip/Screener side-effects) |
| screen_size.toml | ScreenSize | ✓ | Yes (bare res before Hi10p) |
| container.toml | Container | ✗ | Yes (extension regex, standalone) |
| frame_rate.toml | FrameRate | ✓ | Yes (decimal fps, res-attached) |
| language.toml | Language | ✗ | Yes (bracket multi-lang) |
| subtitle_language.toml | SubtitleLanguage | ✗ | Yes (compound LANG SUBS) |

**Engine features:**
- ✅ Exact lookups (case-insensitive + case-sensitive)
- ✅ Regex patterns (linear-time `regex` crate only)
- ✅ Capture-group value templates (`{N}` syntax)
- ✅ Multi-token window matching (1-3 tokens, longest first)
- ✅ Extension → Container path (priority 10)
- ✅ Zone Rule 6: Language containment (replaces fancy_regex lookbehind)
- 🚧 Side-effect rules (one match → multiple properties)
- 🚧 Context-dependent matching (next-token lookahead)

### Key design decisions

| Decision | Rationale |
| -------- | --------- |
| No rebulk port | rebulk is deeply Pythonic. A flat `Vec<MatchSpan>` + sort-and-sweep is simpler and faster. |
| Patterns in Rust code (v0.1) | `LazyLock` + `fancy_regex` gives compile-time validation. Moving to TOML in v0.2. |
| `fancy_regex` for lookaround (v0.1) | Rust's `regex` crate doesn't support look-behind/ahead. Will be eliminated by tokenizer in v0.2. |
| Tokenizer + TOML + regex-only in v0.2 | Bundled change: tokenizer eliminates lookaround need, enabling regex-only (ReDoS-immune) + TOML data files. See `ARCHITECTURE.md` D001-D003. |
| No network/DB/ML | Hunch is pure, offline, deterministic. Layers 2-3 belong in downstream consumers. See `ARCHITECTURE.md` D004. |
| `BTreeMap` output | Deterministic key ordering for JSON output and tests. |
| `std::sync::LazyLock` | Compile regexes once at startup (std, no external dep). |
| Edition 2024 | Latest Rust edition for modern syntax. |

> **Full architecture rationale**: See [ARCHITECTURE.md](ARCHITECTURE.md) for the
> layered architecture, decision log, and v0.2 tokenizer plan.

---

## Module Map

```
src/
├── lib.rs                  # Public API: parse()
├── main.rs                 # CLI binary (clap)
├── hunch_result.rs         # HunchResult type + typed accessors + JSON serialization
├── options.rs              # Options / configuration
├── pipeline.rs             # v0.2 pipeline: tokenize → TOML+legacy match → zones → title
├── tokenizer.rs            # Input tokenizer: separators, brackets, extension stripping
├── matcher/
│   ├── mod.rs              # Re-exports
│   ├── span.rs             # MatchSpan, Property enum (42 variants)
│   ├── engine.rs           # Conflict resolution (priority + length)
│   ├── regex_utils.rs      # ValuePattern: compiled regex + canonical value (legacy)
│   └── rule_loader.rs      # TOML rule engine: exact + regex + {N} capture templates
└── properties/             # 30 property matcher modules (legacy, being migrated)
rules/                      # 18 TOML data files defining property patterns
├── video_codec.toml        # H.264, H.265, AV1, Xvid, …
├── audio_codec.toml        # AAC, DTS, Dolby, FLAC, Opus, …
├── source.toml             # Blu-ray, WEB-DL, HDTV, DVDRip, …
├── screen_size.toml        # 720p, 1080p, 4K, WxH ({N} templates)
├── container.toml          # mkv, mp4, avi, srt, …
├── frame_rate.toml         # 24fps, 120fps ({N} templates)
├── language.toml           # English, French, Multi, VFF, …
├── subtitle_language.toml  # VOSTFR, NLsubs, SubForced, …
├── edition.toml            # Director's Cut, Extended, Unrated, …
├── other.toml              # HDR, Remux, Proper, Repack, 3D, …
├── other_weak.toml         # Low-priority Other matches
├── streaming_service.toml  # AMZN, NF, HMAX, DSNP, …
├── video_profile.toml      # Hi10P, HP, SVC, …
├── audio_profile.toml      # Atmos, DTS:X, TrueHD, …
├── color_depth.toml        # 10-bit, 8-bit, 12-bit, …
├── country.toml            # US, UK, GB, CA, AU, NZ
├── episode_details.toml    # Special, Pilot, Unaired, Final
└── video_api.toml          # DXVA, D3D11, CUDA, …
```

---

## Implementation Status

### Phase 1 — Core Engine + Most-used Properties ✅ COMPLETE

- [x] Project scaffold (Cargo.toml, module structure)
- [x] `MatchSpan` + `Property` enum (46 variants)
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

### Title extraction (82.1%)

The hardest problem. Title is "everything that's left" after all technical
tokens are claimed. Key challenges:
- Parent directory title fallback (path-based inputs)
- Titles with colons, hyphens, or dots that look like separators
- Titles containing year-like numbers (e.g., "2001: A Space Odyssey")
- Anime titles with brackets and group tags

### Episode title (61.2%)

Requires positional awareness: the episode title is typically the unclaimed
region between the episode number and the first technical token. Tricky
because it overlaps with the release group zone.

### Other flags (77.4%)

Many niche patterns remain: OAD, OAR, PROOFFIX, various Screener variants,
FanSub markers, etc. Each is a small regex addition. BRRip/BDRip Reencoded
logic is now correct.

### Multi-value subtitle_language

Patterns like `ST{Fr-Eng}` (both French and English subtitles) need
compound parsing that splits within brackets. Bracket-delimited
multi-language codes like `[ENG+RU+PT]` are now supported, but
curly-brace patterns like `ST{Fr-Eng}` are not yet handled.

---

## Developer Guidelines

### Code style

- **Idiomatic Rust**: follow standard Rust conventions — ownership, strong
  types, exhaustive matches, `clippy` clean.
- **DRY**: shared regex helpers go in `matcher/regex_utils.rs` (`ValuePattern`).
- **YAGNI**: don't build Phase 3 infra now.
- **Split by responsibility, not line count**: keep each file focused on one
  concern. If a file handles multiple distinct responsibilities, split it.
  A long file that's cohesive (e.g., all episode parsing) is fine.
- **Tests in each module** (`#[cfg(test)] mod tests`).

### Testing strategy

1. **Unit tests** in each property matcher (`#[cfg(test)]` blocks) — 153 tests.
2. **Integration tests** (`tests/integration.rs`) — 27 hand-written end-to-end tests.
3. **Regression tests** (`tests/guessit_regression.rs`) — 22 fixture files with
   ratchet-pattern minimum pass rates. Floors are set to (actual − 2%) and should
   only go up. Includes language normalization (ISO codes).
4. **Compatibility report** — run `cargo test compatibility_report -- --ignored --nocapture`
   for a full per-property and per-file accuracy breakdown.
   Includes single-property failure analysis for prioritization.
5. **Benchmarks** (`benches/parse.rs`) — Criterion benchmarks for parse performance.
6. All fixtures are self-contained in `tests/fixtures/` (no external repos needed).
   22 fixture files: `movies.yml`, `episodes.yml`, `various.yml`,
   `rules/{audio_codec,bonus,cd,common_words,country,date,edition,episodes,film,
   language,other,part,release_group,screen_size,size,source,title,video_codec,
   website}.yml`.

### Adding a new property matcher (v0.2)

1. Create `rules/<name>.toml` with `property`, `[exact]`, and `[[patterns]]`.
2. Add a `LazyLock<RuleSet>` static in `pipeline.rs`.
3. Register it in the `toml_rules` vector with appropriate property + priority.
4. Add `Property::YourProp` variant to `src/matcher/span.rs` (if new).
5. Add unit tests in rule_loader and integration tests.
6. Only create a legacy `src/properties/<name>.rs` if the property needs
   structural parsing (episodes, year, etc.) that tokens can't express.
7. Update this file.

### TOML rule file format

```toml
property = "video_codec"

[exact]         # Case-insensitive exact token lookups
x264 = "H.264"
hevc = "H.265"

[exact_sensitive]  # Case-sensitive (for ambiguous short tokens)
NZ = "NZ"          # Country codes, etc.

[[patterns]]           # Regex with optional {N} capture templates
match = '(?i)^[xh][-.]?265$'
value = "H.265"        # Static value

[[patterns]]
match = '(?i)^(\d{3,4})x(\d{3,4})$'
value = "{2}p"         # Dynamic: capture group 2 → "1080p"
```

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
| `fancy-regex` | Pattern matching with look-around (to be removed in v0.2) |
| `serde`       | Serialization for HunchResult output    |
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
