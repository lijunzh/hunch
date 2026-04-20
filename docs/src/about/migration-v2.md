# Migrating to v2.0.0

hunch v2.0.0 is the first major version bump since v1.0. It carries
two breaking API changes — both small, both summarized here in one
place. The full release notes live in the [Changelog](./changelog.md).

This page exists so library consumers don't have to scrape the
changelog: if your code compiles and runs against v1.x, the two
sections below tell you everything you need to update.

## 1. `Property::BitRate` is removed

`Property::BitRate` was deprecated mid-v1 wave in favor of two
unit-typed variants: `Property::AudioBitRate` (Kbps) and
`Property::VideoBitRate` (Mbps). The bit-rate matcher captures the
unit from the input and routes to one of the two specific variants;
the old combined variant has been unreachable from any parser path
since the split landed.

Removing it now under the v2.0.0 major bump avoids forcing a v3.0.0
just to delete one variant later.

**If your code matches on `Property::BitRate`,** switch to the
unit-typed variants. The `#[non_exhaustive]` annotation already
requires a wildcard arm, so the diff is usually a one-liner:

```rust
match prop {
    // Before:
    Property::BitRate       => handle_either(value),

    // After:
    Property::AudioBitRate  => handle_audio(value),
    Property::VideoBitRate  => handle_video(value),
    _ => {} // already required by #[non_exhaustive]
}
```

If you don't care about the unit distinction, you can collapse both
arms into one:

```rust
Property::AudioBitRate | Property::VideoBitRate => handle_either(value),
```

## 2. Deep module imports are gone — use crate-root re-exports

The `Options` module and various deep-path imports under
`hunch::pipeline::*`, `hunch::matcher::*`, and `hunch::properties::*`
are no longer part of the public API surface. Everything an external
caller needs is re-exported from the crate root.

**If you have deep imports,** switch to the crate-root re-exports:

```rust
// Before:
use hunch::pipeline::Pipeline;
use hunch::hunch_result::HunchResult;
use hunch::matcher::span::Property;

// After:
use hunch::{Pipeline, HunchResult, Property};
```

For the full list of public types, the
[Public API Surface](../reference/public-api.md) page is generated
directly from `cargo public-api` output and is the authoritative
reference.

## What hasn't changed

- `hunch()` and `hunch_with_context()` keep the same signatures.
- `HunchResult` accessors (`.title()`, `.season()`, `.year()`, etc.)
  are unchanged. v2.0.0 actually *adds* a few:
  `HunchResult::is_movie()`, `is_episode()`, `is_extra()`,
  `audio_bit_rate()`, `video_bit_rate()`, `mimetype()`.
- The CLI (`hunch <filename>`, `hunch --batch <dir> -r`,
  `hunch --context <dir> <file>`) is fully backwards-compatible —
  no flag renames, no output-format breakage.
- The compatibility-report contract (per-property pass rates) holds:
  v2.0.0 maintains or improves every property's accuracy versus v1.x.

## Why a major bump for so little?

Two reasons. **One:** SemVer requires it for any incompatible API
change, no matter how small. Removing one enum variant qualifies even
if it was effectively dead code. **Two:** both removals had been
deprecated for one or more minor releases already; bundling them under
a single major bump amortizes the upgrade cost (callers update once,
not twice).

If you find a v1.x integration point we missed, please
[open an issue](https://github.com/lijunzh/hunch/issues/new/choose) —
the goal is *no* surprise breakage.
