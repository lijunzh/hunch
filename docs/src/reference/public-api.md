# Public API Surface

Hunch's public Rust API is the contract that downstream library consumers
depend on. SemVer-incompatible changes (removing/renaming `pub` items,
changing signatures, adding non-`#[non_exhaustive]` enum variants, etc.)
must be deliberate, not accidental.

Two complementary tools watch this contract:

- **`cargo-semver-checks`** (in [`ci.yml`](https://github.com/lijunzh/hunch/blob/main/.github/workflows/ci.yml))
  — compares the PR head's API against the latest release on crates.io.
  Catches *semantic* SemVer breaks (signature changes, trait-bound
  tightening, etc.). Advisory pre-1.0; will become a release-prep gate.
- **`cargo-public-api`** (this doc + the `Public API Surface` CI job)
  — produces a flat text inventory of every `pub` item. Diffed against
  the committed snapshot in [`public-api.txt`](https://github.com/lijunzh/hunch/blob/main/docs/src/reference/public-api.txt) on every
  PR. Catches *additive* surface drift (new `pub` items that probably
  shouldn't be exposed) that semver-checks doesn't flag because adding
  is SemVer-minor, not major.

## Current baseline

Captured against `main` on **2026-04-18** (post #170):

| Metric | Count |
|---|---|
| Total API lines | **853** |
| Public modules | 40 |
| Public functions | 199 |
| Public structs | 17 |
| Public enums | 11 |

### Top exposure offenders (lines per module path)

| Module path | API lines | Comment |
|---|---|---|
| `hunch::matcher::*` | 88 | Regex helpers, span types — mostly internal scaffolding |
| `hunch::HunchResult::*` | 44 | Public result type — keep, but field accessors warrant audit |
| `hunch::properties::*` | 54 (32 mods + 22 fns) | Every property is a `pub mod` exposing its `find_matches` — almost certainly should be `pub(crate)` |
| `hunch::tokenizer::*` | 25 | Internal — should be `pub(crate)` |
| `hunch::zone_map::*` | 10 | Internal — should be `pub(crate)` |
| `hunch::Confidence::*` | 6 | Public, intentional |
| `hunch::Pipeline::*` | 5 | Public, intentional |

The intentional public surface is roughly: `hunch()`, `hunch_with_context()`,
`Pipeline`, `HunchResult`, `Confidence`. Everything else is incidental
exposure that grew organically and was never audited.

This is exactly what the [Public-API visibility audit epic
(#144)](https://github.com/lijunzh/hunch/issues/144) is for.

## How the CI tripwire works

The `Public API Surface` job in [`ci.yml`](https://github.com/lijunzh/hunch/blob/main/.github/workflows/ci.yml):

1. Installs Rust nightly (`cargo-public-api` requires the unstable
   rustdoc-JSON output).
2. Installs `cargo-public-api` via `taiki-e/install-action` (prebuilt
   binary; matches the pattern set by the coverage and mutation jobs).
3. Generates the current API listing.
4. `diff -u`s it against [`docs/src/reference/public-api.txt`](https://github.com/lijunzh/hunch/blob/main/docs/src/reference/public-api.txt).
5. Posts the diff to the GitHub Job Summary.

The job is **advisory** (`continue-on-error: true`), matching the
existing `semver-checks` advisory job. It will *not* block merging a
PR — but the diff in the Job Summary makes any drift impossible to
miss in code review.

## When the diff fires

| Diff content | What to do |
|---|---|
| New `pub` items | **Audit**: should they be `pub(crate)` instead? If yes, demote in the same PR. If genuinely public, regenerate the snapshot (below) and document the addition in the PR body. |
| Removed `pub` items | This is a SemVer-major change. The `semver-checks` job should also be flagging it. Confirm intent, regenerate the snapshot, and bump the version per [CONTRIBUTING.md → API Stability Policy](https://github.com/lijunzh/hunch/blob/main/CONTRIBUTING.md). |
| Signature changes | Same as removed — SemVer-major. Confirm with `semver-checks`. |
| Reordered lines (no real diff) | The snapshot is sorted by `cargo-public-api`'s internal logic; reordering shouldn't happen in normal use. If you see this, regenerate. |

## Regenerating the baseline

Required when an intentional API change lands.

```bash
# One-time install (uses Walmart proxy if needed):
rustup toolchain install nightly --profile minimal
cargo install cargo-public-api --locked

# Capture the current public API:
cargo public-api --simplified > docs/public-api.txt

# Verify the diff matches what you intended:
git diff docs/public-api.txt
```

Commit `docs/public-api.txt` together with the API change in the same PR.
The diff in the PR review should make the API delta easy for reviewers
to scan.

## Roadmap

This document captures the **first slice** of the [API audit epic
(#144)](https://github.com/lijunzh/hunch/issues/144). Deferred to follow-up
PRs:

- **Triage pass**: classify every `pub` item as Keep / Demote / Deprecate
  per the epic's definition-of-done. Each demotion lands in its own PR
  to make the visibility change reviewable in isolation.
- **Reverse-dep check**: query crates.io for downstream users of items
  marked Demote/Deprecate before actually pulling the trigger.
- **`pub use` cleanup**: collapse the obvious "should never have been
  exposed" cases (`tokenizer`, `zone_map`, `matcher::engine`'s internals,
  the property modules) en masse.
- **`#[non_exhaustive]`** on the public enums (`Confidence`,
  `HunchResult` field types) so future variant additions aren't
  SemVer-major. _Partially landed in #172: `Property`, `MediaType`,
  `Source`, `Separator`, `BracketKind`, `ZoneScope`, `CharClass` now
  bear the attribute. `Confidence` and `SegmentKind` deliberately
  excluded — they're conceptually saturated (Low/Med/High and
  Directory/Filename respectively)._
- **Promote the CI job from advisory → blocking** once the surface has
  stabilised post-audit.

## References

- [`cargo-public-api`](https://github.com/Enselic/cargo-public-api)
- [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
  (sibling tool, already in CI)
- [CONTRIBUTING.md → API Stability Policy](https://github.com/lijunzh/hunch/blob/main/CONTRIBUTING.md)
- Sibling docs: [Coverage](../contributor-guide/coverage.md) (line coverage),
  [Mutation Testing](../contributor-guide/mutation-baseline.md) (assertion quality)
