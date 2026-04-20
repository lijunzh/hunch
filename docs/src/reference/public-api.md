# Public API Surface

Hunch's public Rust API is the contract that downstream library consumers
depend on. SemVer-incompatible changes (removing/renaming `pub` items,
changing signatures, adding non-`#[non_exhaustive]` enum variants, etc.)
must be deliberate, not accidental.

Two complementary tools watch this contract:

- **`cargo-semver-checks`** (in [`ci.yml`](https://github.com/lijunzh/hunch/blob/main/.github/workflows/ci.yml))
  — compares the PR head's API against the latest release on crates.io.
  Catches *semantic* SemVer breaks (signature changes, trait-bound
  tightening, etc.). Runs as an **advisory** CI job (non-blocking).
- **`cargo-public-api`** (this doc + the snapshot at [`public-api.txt`](https://github.com/lijunzh/hunch/blob/main/docs/src/reference/public-api.txt))
  — produces a flat text inventory of every `pub` item. Run **locally**
  during release prep to verify the snapshot still matches the actual
  surface; commit any intentional drift in the same PR. Catches
  *additive* surface drift (new `pub` items that probably shouldn't be
  exposed) that semver-checks doesn't flag because adding is
  SemVer-minor, not major.

> The dedicated "Public API Surface" CI job that previously diffed the
> snapshot on every PR was removed in #216 as part of trimming
> over-engineered CI for a hobby-scale crate. The contract still holds;
> the verification step just moved from "every PR" to "release prep".

## Current baseline

Captured against `main` at the v2.0.0 release tag (post #197/#198):

| Metric | Count |
|---|---|
| Total API lines | **201** |
| Public modules | 1 (`hunch`) |
| Public functions | 70 |
| Public structs | 2 (`HunchResult`, `Pipeline`) |
| Public enums | 3 (`Confidence`, `MediaType`, `Property`) |

The intentional public surface is: `hunch()`, `hunch_with_context()`,
`Pipeline`, `HunchResult`, `Confidence`, `MediaType`, `Property`. The
v2.0.0 audit (#144 / #197) demoted the `matcher`, `properties`,
`tokenizer`, and `zone_map` modules from `pub mod` to `pub(crate) mod`,
shrinking the surface from 853 → 201 lines (76% reduction). See the
[v2.0.0 migration guide](../about/migration-v2.md) for the migration
path for downstream code that was using deep imports.

## Verifying the snapshot during release prep

Required when an intentional API change lands.

```bash
# One-time install:
rustup toolchain install nightly --profile minimal
cargo install cargo-public-api --locked

# Capture the current public API:
cargo +nightly public-api --simplified 2>/dev/null > docs/src/reference/public-api.txt

# Verify the diff matches what you intended:
git diff docs/src/reference/public-api.txt
```

Commit `docs/src/reference/public-api.txt` together with the API change
in the same PR. The diff in PR review should make the API delta easy
for reviewers to scan.

## Interpreting a diff

| Diff content | What to do |
|---|---|
| New `pub` items | **Audit**: should they be `pub(crate)` instead? If yes, demote in the same PR. If genuinely public, regenerate the snapshot and document the addition in the PR body. |
| Removed `pub` items | This is a SemVer-major change. The `semver-checks` job should also be flagging it. Confirm intent, regenerate the snapshot, and bump the major version. |
| Signature changes | Same as removed — SemVer-major. Confirm with `semver-checks`. |

## Public enum policy

All public enums carry `#[non_exhaustive]` as of v2.0.0 (#172, #196):
`Property`, `MediaType`, `Confidence`. Downstream code must include a
wildcard arm (`_ => …`) when matching on any of these. This lets
future minor releases add new variants without re-breaking the API.

## References

- [`cargo-public-api`](https://github.com/Enselic/cargo-public-api)
- [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
  (sibling tool, advisory CI job)
- [v2.0.0 migration guide](../about/migration-v2.md) — what the surface
  shrink means for callers
- Sibling docs: [Coverage](../contributor-guide/coverage.md) (run locally),
  [Mutation Testing](../contributor-guide/mutation-baseline.md) (run locally)
