# Fuzzing

Hunch uses [`cargo-fuzz`](https://rust-fuzz.github.io/book/) to drive
the public parser surface (`hunch::hunch`, `hunch::hunch_with_context`)
with arbitrary mutator-generated inputs, hunting for panics, integer
overflows, out-of-bounds slice indexing, and other crash-shape bugs
that fixture-based tests can't reach.

This complements the existing testing layers:

| Layer | Catches | Where |
|---|---|---|
| **Unit + integration tests** (~525 fixtures) | Correctness for known inputs | `tests/`, `src/**/tests` |
| **Coverage** (#168) | Which lines are exercised at all | [Coverage](./coverage.md) |
| **Mutation testing** (#146) | Whether assertions actually catch bugs | [Mutation Testing](./mutation-baseline.md) |
| **Fuzzing** (this doc) | Crash-shape bugs on adversarial inputs | `fuzz/` + this doc |

## Fuzz targets

Two targets in `fuzz/fuzz_targets/`:

| Target | Entry point | What it stresses |
|---|---|---|
| `parse_filename` | `hunch::hunch(input: &str)` | Single-string parse with arbitrary UTF-8 |
| `parse_with_context` | `hunch::hunch_with_context(input, &siblings)` | Multi-input invariance analysis with up to 16 siblings |

Both reject non-UTF-8 input at the harness layer (the public API takes `&str`,
so non-UTF-8 isn't a reachable failure mode). `parse_with_context` uses the
[`arbitrary`](https://docs.rs/arbitrary/) derive macro to give libfuzzer
structured input it can mutate per-field — better coverage feedback than
manually splitting a single `&[u8]` blob.

## How it runs

[`.github/workflows/fuzz.yml`](https://github.com/lijunzh/hunch/blob/main/.github/workflows/fuzz.yml) defines two jobs:

| Job | Trigger | Time | What |
|---|---|---|---|
| `Fuzz harness build` | every PR + push | ~30 s | Just `cargo fuzz build` — confirms targets compile against current API |
| `Nightly fuzz` | 04:14 UTC + manual | ~12 min total (5 min × 2 targets) | Real fuzzing, corpus persistence via `actions/cache`, crash artifacts uploaded |

The PR-time job is intentionally **build-only**, not actual fuzzing.
A 60-second per-target fuzz on every PR (the epic's [#147 DoD](https://github.com/lijunzh/hunch/issues/147))
adds noise without high signal — the corpus barely grows in 60 s, and
nightly catches the same bugs at higher coverage. We can revisit if PR-time
fuzzing produces useful findings; for now build-check is the daily signal.

### What the nightly does

```bash
cargo +nightly fuzz run <target> -- \
  -max_total_time=300   # 5 min wall-clock
  -timeout=10           # per-input cap (catches slow-input DoS too)
  -rss_limit_mb=2048    # treat unbounded allocation as a crash
```

The corpus directory `fuzz/corpus/<target>/` is restored from `actions/cache`
at the start of each run and saved at the end, so coverage compounds across
nights instead of restarting from the ~10-input seed each time.

Crashes (if any) land in `fuzz/artifacts/<target>/` and are uploaded as
the `fuzz-artifacts-<target>` workflow artifact (30-day retention).

## Local usage

One-time install:

```bash
rustup toolchain install nightly --profile minimal
cargo install cargo-fuzz --locked
```

Build the harness:

```bash
cd fuzz
cargo +nightly fuzz build
```

Run a single target for 60 seconds (good iteration loop):

```bash
cd fuzz
cargo +nightly fuzz run parse_filename -- -max_total_time=60
```

Both targets in parallel for the same time the nightly uses:

```bash
cd fuzz
cargo +nightly fuzz run parse_filename       -- -max_total_time=300 &
cargo +nightly fuzz run parse_with_context   -- -max_total_time=300 &
wait
```

## Seed corpus

Located at `fuzz/corpus/<target>/`, **committed to git**. Each file is
one input. Curated to give libfuzzer's coverage feedback a wide initial
spread rather than 100 near-duplicate movie filenames.

Current `parse_filename` seeds (10 entries):

| File | What it covers |
|---|---|
| `movie_basic` | Standard `Title.Year.Resolution.Source.Codec-Group.ext` |
| `episode_sxxexx` | `SxxExx` marker + episode title |
| `anime_brackets` | `[Group]` + `[CRC32]` brackets, `HEVC AAC` codec stack |
| `path_with_dirs` | Multi-segment path with parenthesised year in directory |
| `multi_episode` | `S01E01-E03` range syntax |
| `unicode_title` | Multi-byte UTF-8 (Japanese title) |
| `empty` | Zero-byte input |
| `just_year` | 4-digit-only input |
| `long_dotted` | Pathological 26-token dotted name |
| `extension_only` | Just `.mkv` |

When you discover a new corpus-worthy input (e.g., a real-world
filename that previously broke parsing), add it to the corpus under a
descriptive name. The default `fuzz/.gitignore` ignores `corpus/` to
keep mutator-discovered hex-named files out of git noise, so commit
named seeds with `git add -f`:

```bash
echo 'My.New.Filename.shape.mkv' > fuzz/corpus/parse_filename/regression_165
git add -f fuzz/corpus/parse_filename/regression_165
```

## Triage protocol — when fuzzing finds a crash

A crash artifact in `fuzz/artifacts/<target>/` is a 1-input file that
makes the harness panic. Reproduce + minimize + fix workflow:

### 1. Reproduce locally

```bash
cd fuzz
cargo +nightly fuzz run <target> artifacts/<target>/<crash-id>
```

This re-runs the harness against just that one input. You should see
the panic output identical to the nightly's failure log.

### 2. Minimize

`cargo fuzz tmin` shrinks the input to the smallest reproduction:

```bash
cd fuzz
cargo +nightly fuzz tmin <target> artifacts/<target>/<crash-id>
```

The minimized input lands at `fuzz/artifacts/<target>/minimized-from-<crash-id>`.
A 200-byte input often shrinks to <20 bytes, which makes both the bug
report and the eventual unit test much clearer.

### 3. Classify the bug

| Pattern | Category | Action |
|---|---|---|
| `unwrap()` / `expect()` / `panic!()` reachable from public API | **Defensive bug** | Convert to `Result` return; add a unit test pinning the input → Err mapping |
| Integer overflow / underflow | **Arithmetic bug** | Use `checked_*` / `saturating_*` arithmetic; assert with `debug_assert!` if the invariant should hold |
| Out-of-bounds slice on UTF-8 boundary | **Encoding bug** | Use `str::char_indices` / `str::is_char_boundary` instead of byte slicing |
| Regex catastrophic backtracking (manifests as fuzzer timeout, not panic) | **DoS bug** | Rewrite the regex to be linear-time (typically by removing nested quantifiers) |
| Genuinely intentional invariant violation | **Documentation bug** | Add `debug_assert!` with comment, document the precondition |

### 4. Fix + add the minimized input as a regression test

The minimized crash input goes into either:
- `tests/fixtures/regression.yml` (if it should produce a sensible parse)
- `tests/regression_panics.rs` (if it should produce a non-panicking error)

Then commit the minimized input to `fuzz/corpus/<target>/regression-<issue-num>`
so future fuzz runs always cover the regression shape.

### 5. File an issue (if not already)

Use the `fuzz-finding` label. Include:
- The minimized input (hex if non-printable)
- The panic message + stack trace
- Your bug-classification verdict per step 3
- The fix PR link

## Initial baseline (2026-04-18)

First local smoke runs against current `main` (on this PR's branch):

| Target | Runs | Time | Crashes | Throughput |
|---|---|---|---|---|
| `parse_filename` | 28,668 | 26 s | **0** ✅ | ~1100 inputs/s |
| `parse_with_context` | 67,941 | 26 s | **0** ✅ | ~2600 inputs/s |

Zero crashes from ~96K combined runs is a strong starting baseline. The
public API surface is robust against adversarial single-input mutation
out of the gate. The interesting findings (if any) will come from the
*nightly* runs once the corpus accumulates coverage over weeks.

## Roadmap (deferred to follow-up PRs)

- [ ] **Differential fuzz target** vs guessit-rs — hunt for behavioural
      divergence between the two parsers. Separate epic; not in #147.
- [ ] **OSS-Fuzz integration application** (the epic's stretch goal).
      Google-funded continuous fuzzing for OSS Rust crates. Application
      template: <https://github.com/google/oss-fuzz/tree/master/projects>.
- [ ] **Promote PR-time job from build-only → 60s smoke fuzz** if the
      nightly produces useful findings worth catching earlier.
- [ ] **Add the guessit-rs corpus** as additional seeds (the epic's
      DoD mentions it; we seeded only from `tests/fixtures/` here).

## References

- [`cargo-fuzz` book](https://rust-fuzz.github.io/book/)
- [`libfuzzer-sys`](https://docs.rs/libfuzzer-sys/) — the underlying
  harness
- [`arbitrary`](https://docs.rs/arbitrary/) — structured-input derive
  used by `parse_with_context`
- [SECURITY.md](https://github.com/lijunzh/hunch/blob/main/SECURITY.md) — fuzzing maps to "DoS via crafted
  filenames" in the threat model
- Sibling docs: [Coverage](./coverage.md),
  [Mutation Testing](./mutation-baseline.md),
  [Public API](../reference/public-api.md)
