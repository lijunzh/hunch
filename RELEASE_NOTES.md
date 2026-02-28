# Hunch v1.0.1 — Documentation Patch

Patch release to fix incorrect documentation shipped with v1.0.0.
No code changes — only documentation and compatibility numbers corrected.

## What Changed Since v1.0.0

- Fixed README compatibility numbers to match actual test results (81.7%)
- Fixed COMPATIBILITY.md version reference
- Updated RELEASE_NOTES.md for v1.0.1
- Added CHANGELOG entries for v1.0.0 and v1.0.1

## Highlights (unchanged from v1.0.0)

- **81.7% compatibility** with guessit's 1,309-case YAML test suite
  (1,069 / 1,309 cases pass)
- **22 properties at 95%+ accuracy**, 16 at 100%
- **All 49 properties implemented** (3 intentionally diverged)
- Zero-dependency on network, databases, or ML
- Single binary, TOML rules embedded at compile time

## Accuracy Summary

| Tier | Properties |
|------|------------|
| 100% | 16 properties (edition, date, proper_count, …) |
| 95–99% | video_codec (98.6%), screen_size (98.4%), audio_codec (97.8%), source (97.5%), year (96.5%), crc32 (96.0%) |
| 90–94% | season (93.9%), type (93.6%), title (91.7%), release_group (91.1%), episode (90.6%), … |
| 80–89% | other (87.7%), language (87.3%), subtitle_language (82.7%), … |

## Install

```bash
# Homebrew
brew install lijunzh/hunch/hunch

# Cargo
cargo install hunch

# As a library
cargo add hunch
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history.
