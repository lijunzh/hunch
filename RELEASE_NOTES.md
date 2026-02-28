# Hunch v1.1.0 — Logging & Documentation

This release adds structured logging for debugging misparses and brings
Rustdoc coverage from 0% to 100% (zero `missing_docs` warnings).

No parsing behavior changes — all 1,069 guessit compatibility cases
still pass.

## What's New

### 🔍 Structured Logging

The full pipeline is now instrumented with `log` crate diagnostics.
See exactly how each filename is tokenized, matched, and resolved:

```bash
# Debug level — stage summaries
hunch -v "Movie.2024.1080p.BluRay.x264-GROUP.mkv"

# Trace level — every match span and conflict decision
RUST_LOG=hunch=trace hunch "Movie.2024.1080p.mkv"
```

Zero runtime cost when logging is disabled (the default).

### 📖 Comprehensive Rustdoc

- All 49 `Property` variants documented with example values
- `HunchResult`, `Options`, `Pipeline`, `MatchSpan` enriched with
  examples and cross-links
- `hunch_with()` fully documented with worked examples
- Crate-level docs expanded: 7 sections covering all usage patterns
- 15 doc-tests compiled and run as part of `cargo test`
- `#![warn(missing_docs)]` prevents future regressions

### 🛡️ Robustness

- ~30 bare `.unwrap()` → descriptive `.expect()` messages
- CLI JSON errors now reported to stderr (was silently swallowed)

## Compatibility (unchanged)

- **81.7%** guessit compatibility (1,069 / 1,309)
- **22 properties at 95%+**, 16 at 100%
- **295 tests** (225 unit + 23 regression + 32 integration + 15 doc-tests)

## Install / Upgrade

```bash
# Homebrew
brew upgrade hunch

# Cargo
cargo install hunch

# As a library
cargo add hunch@1.1.0
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history.
