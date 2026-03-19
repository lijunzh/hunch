## What does this PR do?
<!-- One-paragraph summary. Link to issue if applicable: Fixes #123 -->


## Changes
<!-- List the key changes. Be specific about files and functions. -->

- 
- 

## Type
- [ ] Bug fix
- [ ] New feature
- [ ] Refactor (no behavior change)
- [ ] Docs / tests only
- [ ] Performance

## Testing
<!-- How did you verify this works? -->

- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Manually tested: <!-- describe -->

## Checklist
- [ ] No `unwrap()` in non-test code (use `?` or `.unwrap_or`)
- [ ] CHANGELOG.md updated (if user-facing)
- [ ] Guessit regression floors maintained (run `cargo test compatibility_report -- --ignored`)
