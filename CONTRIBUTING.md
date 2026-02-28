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
- Keep files under 600 lines — split into modules if needed
- Follow the existing TOML-first architecture (see ARCHITECTURE.md)

## License

By contributing, you agree that your contributions will be licensed under
the [MIT License](LICENSE).
