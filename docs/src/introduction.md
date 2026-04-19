# hunch

> A fast, accurate media-filename parser for Rust. Extracts 50 properties
> from movie/TV/anime release names with high accuracy on real-world
> libraries.

This site is the canonical home for hunch's user-facing documentation,
release-engineering reports, and contributor guides. The source lives in
[`docs/src/`](https://github.com/lijunzh/hunch/tree/main/docs/src) — every
page has an "edit this page" link in the top right.

## Where to start

| You are… | Start here |
|---|---|
| A user (CLI or library) | [User Manual](./user-guide/user-manual.md) |
| Evaluating accuracy vs guessit | [guessit Compatibility](./user-guide/compatibility.md) |
| Curious about performance | [Benchmarks](./reference/benchmarks.md) → [Live Dashboard](./reference/benchmark-dashboard.md) |
| Auditing the public API surface | [Public API Surface](./reference/public-api.md) |
| Contributing tests | [Mutation Testing](./contributor-guide/mutation-baseline.md), [Coverage](./contributor-guide/coverage.md) |
| Contributing crash-shape work | [Fuzzing](./contributor-guide/fuzzing.md) |

## How the quality stack fits together

| Layer | Catches | Where |
|---|---|---|
| **Coverage** ([#168](https://github.com/lijunzh/hunch/issues/168)) | Which lines are exercised at all | [coverage.md](./contributor-guide/coverage.md) |
| **Mutation testing** ([#146](https://github.com/lijunzh/hunch/issues/146)) | Whether tests actually catch bugs | [mutation-baseline.md](./contributor-guide/mutation-baseline.md) |
| **Fuzzing** ([#147](https://github.com/lijunzh/hunch/issues/147)) | Crash-shape bugs on adversarial inputs | [fuzzing.md](./contributor-guide/fuzzing.md) |
| **Public API surface** ([#145](https://github.com/lijunzh/hunch/issues/145)) | SemVer-relevant public-surface drift | [public-api.md](./reference/public-api.md) |
| **Performance** ([#148](https://github.com/lijunzh/hunch/issues/148)) | Parse-time regressions | [benchmarks.md](./reference/benchmarks.md) |

Each layer is independently honest: coverage tells you what code runs,
but a 100%-covered codebase can still have zero meaningful assertions —
that's what mutation testing exists for. Fuzzing finds the inputs that
the other three layers never thought to test.

## Project links

- 🐙 **Repository**: <https://github.com/lijunzh/hunch>
- 📦 **crates.io**: <https://crates.io/crates/hunch>
- 📚 **Rust API docs**: <https://docs.rs/hunch>
- 📝 **Changelog**: [CHANGELOG.md](https://github.com/lijunzh/hunch/blob/main/CHANGELOG.md)
- 🏗️ **Design notes**: [DESIGN.md](https://github.com/lijunzh/hunch/blob/main/DESIGN.md)
- 🔒 **Security policy**: [SECURITY.md](https://github.com/lijunzh/hunch/blob/main/SECURITY.md)
