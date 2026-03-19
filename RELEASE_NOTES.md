# Hunch v2.0.0 — Remove dead Options API

## Breaking Changes

The `Options` struct, `hunch_with()` function, and `--type`/`--name-only` CLI
flags have been removed. These were shipped in v1.0.0 but **never wired into
the parsing pipeline** — every field was silently ignored. A user passing
`--name-only` or `--type movie` received identical behavior to a plain
`hunch()` call.

Rather than retroactively change behavior that people may have relied on
(even if that reliance was based on a misunderstanding), we're being honest:
this was dead code from day one. When media-type hinting or name-only mode
are actually implemented, they'll return as a properly tested API.

### Migration guide

```rust
// Before (v1.x):
use hunch::{Options, hunch_with};
let r = hunch_with("file.mkv", Options::new().with_type("movie"));

// After (v2.0):
use hunch::hunch;
let r = hunch("file.mkv");  // identical behavior — Options was always ignored
```

CLI users: remove `--type` and `--name-only` flags. They had no effect.

## Compatibility (unchanged)

- **81.7%** guessit compatibility (1,069 / 1,309)
- **295 tests** passing

## Install / Upgrade

```bash
brew upgrade hunch
cargo install hunch
cargo add hunch@2.0.0
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history.
