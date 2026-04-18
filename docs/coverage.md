# Code Coverage

Hunch tracks line, function, and region coverage via [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).
The CI `Coverage` job runs on every PR and uploads an LCOV report as a build
artifact (`coverage-report`).

## Current baseline

Captured against `main` on **2026-04-18** (post v1.1.8, after PR #167):

| Dimension      | Coverage  | Total   | Missed |
|----------------|-----------|---------|--------|
| **Lines**      | **94.34%** | 15,030  | 851    |
| **Functions**  | **95.54%** | 1,054   | 47     |
| **Regions**    | **94.63%** | 8,571   | 460    |

Re-measure with:

```bash
cargo llvm-cov --workspace --summary-only
```

## Lowest-covered files (line %)

Useful targets for the next round of test-quality work (and for the upcoming
mutation-testing epic, [#146](https://github.com/lijunzh/hunch/issues/146)):

| File                                                | Line %  | Missed |
|-----------------------------------------------------|---------|--------|
| `src/properties/language.rs`                        | 79.67%  | 37     |
| `src/properties/date.rs`                            | 89.00%  | 55     |
| `src/properties/title/strategies/unclaimed_bracket.rs` | 90.91%  | 8      |
| `src/properties/part.rs`                            | 91.29%  | 37     |
| `src/properties/subtitle_language.rs`               | 91.99%  | 45     |
| `src/properties/website.rs`                         | 93.30%  | 14     |

Everything else is ≥ 94% line coverage. 273 of 282 unit tests pass on every
fixture; the missed lines are concentrated in a handful of long-tail edge
branches (rare locale codes, malformed date fragments, etc.).

## Running locally

Install once:

```bash
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview
```

Generate a quick summary:

```bash
cargo llvm-cov --workspace --summary-only
```

Generate a full HTML report (open in browser):

```bash
cargo llvm-cov --workspace --html --open
```

Generate the LCOV file CI uploads (for IDE coverage gutters or external tools):

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## Roadmap

This document captures the **first slice** of the [Code coverage CI epic
(#145)](https://github.com/lijunzh/hunch/issues/145). Deferred to follow-up PRs:

- **PR-time regression gate**: fail CI if total line coverage drops more than
  ~0.5% below this baseline. Deferred until the baseline has settled across a
  few merges (otherwise we'd be tuning the noise threshold against a moving
  target).
- **Codecov.io / Coveralls integration**: the LCOV artifact is already in the
  right shape; just needs the upload step + a token. Optional per the epic.
- **Per-file delta comments on PRs**: shows which newly-added lines aren't
  covered. Most useful with codecov.io's PR comment integration.
- **Branch coverage**: `cargo-llvm-cov` reports it but the issue scopes the
  initial work to line coverage.

## Notes

- **Why single-OS**: line/region coverage of pure-Rust code is platform-agnostic.
  Running coverage on the cross-OS matrix would triple CI time for zero
  additional signal. Platform-conditional branches (`cfg(unix)` symlink tests,
  Windows path code) are still exercised by the cross-OS `test` matrix.
- **Why not 100%**: parser code intentionally has permissive fallback branches
  (e.g., "we couldn't decide, return the empty result") that aren't worth
  contorting tests to hit. ≥ 94% is the realistic ceiling for this codebase.
