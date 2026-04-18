# Contributing to Hunch

Thanks for helping improve hunch! 🔍

## Reporting Failed Parses

The easiest way to contribute is reporting filenames that hunch gets wrong.

### Option 1: Open an Issue

1. Go to [Issues → New Issue](https://github.com/lijunzh/hunch/issues/new/choose)
2. Select **🎬 Failed Parse Report**
3. Fill in the filename, expected properties, and (optionally) actual output

We'll add your case to the community test suite and fix the parser.

### Option 2: Submit a PR

Add your test case directly to `tests/fixtures/community.yml`:

```yaml
? Your.Movie.Title.2024.1080p.BluRay.x264-GROUP.mkv
: type: movie
  title: Your Movie Title
  year: 2024
  screen_size: 1080p
  source: Blu-ray
  video_codec: H.264
  release_group: GROUP
  container: mkv
```

**Format rules:**
- `?` line: the filename (or full path)
- `:` block: expected properties, one per line
- Only include properties you care about
- Use the same values as `hunch` output (run `hunch "filename"` to see)
- List properties are comma-separated: `language: english, french`

**Quick check before submitting:**

```bash
# See what hunch currently produces
hunch "Your.Movie.Title.2024.1080p.BluRay.x264-GROUP.mkv"

# Run the community tests
cargo test community -- --nocapture
```

## Development

```bash
# Run all tests
cargo test

# Run guessit compatibility report
cargo test compatibility_report -- --ignored --nocapture

# Run benchmarks
cargo bench

# Run clippy
cargo clippy -- -D warnings
```

## Code Style

- `cargo fmt` before committing
- `cargo clippy` with zero warnings
- Follow the design principles in [DESIGN.md](DESIGN.md)
- Prefer context over heuristics (Principle 3)

## Releases

Maintainer-only. The standard release flow auto-extracts release notes
from the matching `## [X.Y.Z]` section of `CHANGELOG.md`.

### Optional: per-release notes override

For a one-off release (e.g., a hotfix that needs an executive summary or
an upgrade-guide blurb that shouldn't bloat the CHANGELOG), drop a
`RELEASE_NOTES.md` file at the repo root **before tagging**. The release
workflow will use it verbatim instead of the CHANGELOG extract.

**Important:** delete `RELEASE_NOTES.md` after the release ships,
otherwise every subsequent release will reuse the same stale notes.
`RELEASE_NOTES.md` is intentionally **not** in `.gitignore` because the
release workflow needs to read it from a clean checkout.

## API Stability Policy

`hunch` follows [Semantic Versioning](https://semver.org/) on its
**Rust public API** — anything reachable from `pub use` in `src/lib.rs`.
Within the `1.x` line, breaking changes to that surface require a
major-version bump.

What counts as a breaking change to the **Rust API**:

- Removing or renaming a `pub` item (function, type, variant, field)
- Changing the signature of a `pub` function (parameter / return types)
- Adding a non-defaulted variant to a `pub` enum or a non-defaulted
  field to a `pub` struct (callers' exhaustive matches break)
- Tightening a trait bound on a `pub` item
- Changing a public re-export's source path in a way that breaks
  downstream `use` statements

What does **not** count ("soft API" — free to change in a minor):

- The exact **parsed output** for a given filename. Property extractors
  (title cleaner, type voter, edition detector, etc.) improve over
  time. We may produce a different `title` / `episode_title` / `type`
  for the same input across minor versions — that's a feature, not a
  contract.
- Confidence scores. The numeric values are heuristic and subject to
  re-tuning. Consumers should treat them as ordinal, not absolute.
- The set of properties returned for a given filename (we may newly
  detect a property we previously missed).
- Internal module structure (`src/properties/`, `src/pipeline/`, etc.).
  Anything not re-exported from `src/lib.rs` is implementation detail.
- CLI human-readable output formatting (column widths, wording of
  hints, color choices).
- The contents of `tests/fixtures/*.yml` and the
  `docs/compatibility.md` numbers — these are diagnostic, not API.

What is **soft-but-still-careful**:

- The **JSON output schema** of `hunch -j` is a documented integration
  point. Field renames or removals will be called out in the changelog
  under a "CLI output" heading and rolled with care, but they do not
  by themselves trigger a major-version bump.
- New JSON fields may appear in any minor release; consumers should
  ignore unknown fields.

When in doubt, file an issue describing your use case before relying
on a behavior that isn't on the Rust API surface — we'll either
promote it to a stable contract or document it as soft.

## Reporting Security Issues

See [SECURITY.md](SECURITY.md) for the private reporting channel and
response timeline. Please **do not** file security vulnerabilities as
public GitHub issues.

## License

By contributing, you agree that your contributions will be licensed under
the [MIT License](LICENSE).
