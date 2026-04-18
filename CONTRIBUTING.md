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

## License

By contributing, you agree that your contributions will be licensed under
the [MIT License](LICENSE).
