---
name: rust-release
description: |
  Formalized release workflow for the hunch Rust project. Use when the user wants to:
  (1) Cut a new release / version bump
  (2) Prepare release notes or changelog entries
  (3) Tag and push a release to GitHub
  (4) Run pre-release quality gates (fmt, clippy, tests, docs)
  Triggers on phrases like "release", "cut a release", "bump version", "tag a release",
  "prepare release notes", "publish", "ship it".
metadata:
  author: l0z05rg
sample-prompts:
  - 'cut a new release'
  - 'release v0.4.0'
  - 'bump version and tag'
  - 'prepare release notes for the next version'
arguments:
  - [version] - optional, target version (e.g. 0.4.0). If omitted, prompt the user.
  - [--dry-run] - optional, run all checks but skip git commit/tag/push.
---

# Rust Release Skill

This skill executes a formalized release process for the `hunch` Rust project.
It ensures every release passes quality gates before any git operations.

## Release Workflow

Execute these phases **in order**. Abort on any failure.

### Phase 1: Pre-flight Checks

1. Ensure working tree is clean: `git status --porcelain` must be empty.
   - If dirty, list the uncommitted files and ask the user whether to stash or abort.
2. Ensure on `main` branch: `git branch --show-current` must be `main`.
3. Determine the **target version**:
   - If provided as argument, use it.
   - Otherwise, read current version from `Cargo.toml` and suggest the next patch/minor/major.
   - Validate it follows semver (MAJOR.MINOR.PATCH).
4. Confirm the tag `v{version}` does not already exist: `git tag -l v{version}`.

### Phase 2: Quality Gates

Run all four gates. Do **not** skip any. Collect all failures before reporting.

```bash
cargo fmt --check          # Gate 1: formatting
cargo clippy --all-targets -- -D warnings   # Gate 2: lints
cargo test                 # Gate 3: tests
cargo doc --no-deps        # Gate 4: docs build
```

Set `RUSTFLAGS=-Dwarnings` and `RUSTDOCFLAGS=-Dwarnings` for gates 2-4.

If any gate fails, show the failure output and abort. Do NOT proceed to Phase 3.

### Phase 3: Version Bump & Documentation Updates

1. **Bump `Cargo.toml`** — Update the `version = "..."` field to the target version.
2. **Update `Cargo.lock`** — Run `cargo check` to regenerate it with the new version.
3. **Update `README.md`** — Update any version references (e.g. pass rate stats,
   compatibility percentages) only if the user provides new numbers. Do NOT fabricate stats.
4. **Update `CHANGELOG.md`** — Add a new `## [{version}] - {YYYY-MM-DD}` section at
   the top (below the header). Follow [Keep a Changelog](https://keepachangelog.com/) format.
   Populate it by inspecting `git log` since the last tag:
   ```bash
   git log $(git describe --tags --abbrev=0)..HEAD --oneline
   ```
   Categorize commits into: Added, Changed, Deprecated, Removed, Fixed, Security.
   Ask the user to review/edit the generated changelog before proceeding.
5. **Update `RELEASE_NOTES.md`** — Overwrite with a human-friendly summary for the
   GitHub Release page. Include:
   - A headline with the version and a short tagline
   - Key metrics comparison table (if applicable)
   - Highlights of the most important changes
   - Breaking changes (if any), called out prominently
   This file is consumed by `.github/workflows/release.yml` as the GitHub Release body.
6. **Update `ARCHITECTURE.md`** — Only if there are architectural changes in this release.
   Ask the user: "Were there architectural changes to document?" If yes, update.
   If no, skip.

### Phase 4: Git Commit, Tag & Push

If `--dry-run` was specified, stop here and report what would have been committed.

1. Stage all changed files:
   ```bash
   git add Cargo.toml Cargo.lock README.md CHANGELOG.md RELEASE_NOTES.md ARCHITECTURE.md
   ```
   Only stage files that actually changed (`git diff --name-only`).

2. Commit with a conventional message:
   ```bash
   git commit -m "chore: release v{version}"
   ```

3. Create an annotated tag:
   ```bash
   git tag -a v{version} -m "Release v{version}"
   ```

4. **Confirm before pushing.** Show the user:
   - The commit diff summary (`git show --stat HEAD`)
   - The tag name
   - The remote and branch that will be pushed to
   Ask: "Push commit and tag to origin? (y/n)"

5. Push:
   ```bash
   git push origin main
   git push origin v{version}
   ```

6. Inform the user:
   - The `v*` tag push triggers `.github/workflows/release.yml`
   - This will: verify CI ✅ → build binaries → publish to crates.io → create GitHub Release
   - Link: `https://github.com/lijunzh/hunch/actions`

### Phase 5: Post-Release

1. Remind the user to monitor the release workflow on GitHub Actions.
2. Suggest verifying the GitHub Release page has the correct release notes.
3. Suggest verifying the crate is published: `cargo search hunch`.

## Important Rules

- **Never fabricate test results or stats.** Only report what the commands actually output.
- **Never force-push.** Use `git push`, never `git push --force`.
- **Always confirm before pushing.** The user must explicitly approve.
- **Abort early on failures.** Do not skip failing gates.
- **Keep commits atomic.** One release = one commit + one tag.
