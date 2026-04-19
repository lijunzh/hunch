# Benchmarks

Hunch uses [criterion](https://bheisler.github.io/criterion.rs/book/)
to track parsing performance via `benches/parse.rs`. The
[Benchmarks workflow](https://github.com/lijunzh/hunch/blob/main/.github/workflows/benchmark.yml) runs them
weekly + on every push to `main` to accumulate a performance history.

## How this fits in the quality stack

| Layer | Catches | Where |
|---|---|---|
| **Coverage** (#168) | Which lines are exercised at all | [Coverage](../contributor-guide/coverage.md) |
| **Mutation testing** (#146) | Whether tests actually catch bugs | [Mutation Testing](../contributor-guide/mutation-baseline.md) |
| **Fuzzing** (#147) | Crash-shape bugs on adversarial inputs | [Fuzzing](../contributor-guide/fuzzing.md) |
| **Public API surface** (#145) | SemVer-relevant public-surface drift | [Public API](./public-api.md) |
| **Performance** (this doc, #148) | Parse-time regressions | `benches/` + this doc |

## Bench coverage

Six benchmarks in `benches/parse.rs` cover the main parse paths:

| Bench | Input | What it stresses |
|---|---|---|
| `minimal` | `movie.mkv` | The parser fast path; baseline for "do nothing useful" cost |
| `movie_basic` | `The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv` | Standard movie filename |
| `movie_complex` | `Blade.Runner.2049.2017.2160p.UHD.BluRay.REMUX.HDR.HEVC.DTS-HD.MA.7.1.Atmos-EPSiLON.mkv` | Loaded movie with many tags (codec, source, audio, HDR, atmos) |
| `episode_sxxexx` | `The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv` | Standard episode with `SxxExx` marker |
| `episode_with_path` | `Series/Californication/Season 2/Californication.2x05.Vaginatown.HDTV.XviD-0TV.avi` | Multi-segment path with directory context |
| `anime_bracket` | `[SubGroup] Anime Title - 01 [720p] [ABCD1234].mkv` | Anime-style brackets + CRC32 checksum |

Coverage matches what [PR #138](https://github.com/lijunzh/hunch/pull/138) pinned for correctness — the same shapes get exercised for both correctness and perf.

## Initial baseline (2026-04-18)

Captured on local M-class hardware, criterion default (100 samples, 5s collection):

| Bench | Median time | Iterations to fill 5 s |
|---|---|---|
| `minimal` | **11.5 µs** | 444k |
| `anime_bracket` | **62.5 µs** | 81k |
| `movie_basic` | **74.8 µs** | 67k |
| `episode_with_path` | **78.5 µs** | 64k |
| `episode_sxxexx` | **83.4 µs** | 60k |
| `movie_complex` | **183 µs** | 27k |

Total cargo-bench wall clock: ~70 s on local hardware.

GitHub-hosted ubuntu-latest runners are **~2-3× slower** with significantly noisier per-iteration variance. Expect CI numbers to be roughly:

| Bench | CI estimate (rough) |
|---|---|
| `minimal` | 25-40 µs |
| `movie_complex` | 400-600 µs |
| Others | 150-250 µs |

These ranges will tighten once we have a multi-month CI history to characterize the noise floor.

## How it runs in CI

[`.github/workflows/benchmark.yml`](https://github.com/lijunzh/hunch/blob/main/.github/workflows/benchmark.yml) — triggered on:

| Trigger | Behavior |
|---|---|
| Pull request (paths: `src/`, `benches/`, `Cargo.*`, workflow file) | Compare vs latest `main` baseline; **comment delta table** on PR; **fail if any bench >120% of baseline** |
| Push to `main` (same paths) | Append results to `gh-pages/dev/bench/data.js`; no comment, no fail |
| Weekly Sunday 05:14 UTC | Produce artifact only (legacy belt) |
| Manual dispatch | Produce artifact only |

Output on every run: `bench-output/parse.txt` (full criterion log) + `bench-output/parse-summary.txt` (one bencher-format line per bench), uploaded as the `bench-results-<sha>` artifact with **90-day retention** — independent backstop in case the gh-pages history gets corrupted.

### Threshold rationale

- **`alert-threshold: '120%'`** — fail when any bench is **>20% slower** than the latest main baseline.
- Deliberately permissive to filter the typical 5–10% noise floor on shared GitHub-hosted runners. A tighter threshold without statistical-significance handling would flake constantly.
- Tighten once we have ~4 weeks of weekly-run data to characterize real variance — tracked in [#178](https://github.com/lijunzh/hunch/issues/178)'s comments.

### Triage when the gate fires

1. **Don't immediately revert.** Confirm by re-running the bench job (CPU jitter on shared runners is real). If the second run still flags it, you have a real signal.
2. Pull the bench artifact from both the PR run and the most recent `main` run; eyeball the absolute numbers (the action's comment shows %-delta, but absolute µs tells you whether it's a hot-path concern or a 100ns blip on a 10µs bench).
3. If real, profile locally with `cargo bench --bench parse -- --profile-time 10` (criterion's built-in flamegraph mode) or `samply` for deeper drill-down.
4. **Override path**: if the regression is intentional (perf traded for correctness or feature), document the rationale in `CHANGELOG.md` and use the `[skip-bench-gate]` label (TODO once we hit the first justified override) — for now, accept the failure and discuss in PR review.

## History storage — decision (#177)

**Choice: [`github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark) committing data to a `gh-pages` branch.**

Three real options were considered (full tradeoff table in [#177](https://github.com/lijunzh/hunch/issues/177)):

| Option | UX | Trust model | Cost | Picked? |
|---|---|---|---|---|
| **A: bencher.dev** (cloud SaaS) | Best (prebuilt charts, alerts) | External account + API token | Free for OSS | ❌ |
| **B: github-action-benchmark** (gh-pages) | Adequate (Chart.js dashboard) | In-repo, no external service | Free | ✅ |
| **C: Self-hosted runner** | N/A (storage-orthogonal) | Internal | High (someone owns a machine) | ❌ |

### Why Option B

1. **In-repo, no external deps** — matches the project's stewardship ethos (same as how [Mutation Testing](../contributor-guide/mutation-baseline.md) and [Fuzzing](../contributor-guide/fuzzing.md) keep their data in-tree).
2. **No vendor lock-in** — `data.js` on `gh-pages` is just a JSON-array-shaped file; if we outgrow it, exporting to bencher.dev later is straightforward.
3. **No org-secret friction** — zero coordination with Walmart secret-management to get a token provisioned per-fork.
4. **Negligible maintenance** — the action auto-commits to `gh-pages` on every push to `main`; no manual touch.
5. **PR comments are built-in** — the action ships a `comment-on-alert` flag that posts deltas back to PRs once we wire it up in [#178](https://github.com/lijunzh/hunch/issues/178).

### Infrastructure created by this decision

- **`gh-pages` branch** seeded with a placeholder `index.html` (commit `2916df7`). The action will populate it with `data.js` + an interactive Chart.js dashboard on the next push to `main`.
- The dashboard will be live at `https://lijunzh.github.io/hunch/` once GitHub Pages is enabled in repo settings (Settings → Pages → Source: `gh-pages` branch / `/` root). **Manual one-time toggle required.**

### Migration path if Option B disappoints

If the dashboard UX or noise-handling proves inadequate after a few months of real-world use, migrating to bencher.dev is mechanical:

1. Enable bencher.dev account, generate API token, store as repo secret.
2. Replace the `github-action-benchmark` step in `benchmark.yml` with `bencherdev/bencher@v0.x`.
3. Optionally backfill: `data.js` on `gh-pages` is a JSON array; bencher.dev supports CSV import.

No code change to the bench harness itself (`benches/parse.rs`) is needed in either direction.

## Local usage

Run the full bench suite:

```bash
cargo bench --bench parse
```

(Takes ~70 s on M-class hardware; longer on CI.)

Run just one bench:

```bash
cargo bench --bench parse -- minimal
```

Use criterion's baseline-comparison mode to detect regressions across two commits:

```bash
# Save numbers from the current state as 'before'
cargo bench --bench parse -- --save-baseline before

# Make your change, then compare
cargo bench --bench parse -- --baseline before
```

Criterion will print "Performance has improved" / "Performance has regressed" with confidence intervals. Anything with `p < 0.05` is statistically significant; anything > 5% delta is usually visible above the local-hardware noise floor.

## Roadmap (deferred to follow-up PRs within #148)

This first slice intentionally does **not** implement the full epic [DoD](https://github.com/lijunzh/hunch/issues/148). Why each piece is deferred:

- [x] **Decision: bencher.dev (cloud) vs github-action-benchmark (gh-pages) vs self-hosted runner** — resolved in [#177](https://github.com/lijunzh/hunch/issues/177). See "History storage — decision" above.
- [x] **PR-time comparison + comment + regression gate** — resolved in [#178](https://github.com/lijunzh/hunch/issues/178). See "How it runs in CI" above.
- [ ] **Tighten alert threshold from 120% to something stricter** — needs ~4 weeks of post-#178 data to characterize the noise floor on shared runners. Tracked in #178's comments.
- [ ] **Per-release baseline snapshot committed to the repo** ([#179](https://github.com/lijunzh/hunch/issues/179)): useful for showing perf trajectory in `CHANGELOG.md` per release. Trivial follow-up now that the storage substrate exists.
- [ ] **Differential vs guessit**: out of epic scope per #148 ("interesting but apples-to-oranges").
- [ ] **Memory profiling**: out of epic scope per #148 (criterion is wall-clock).

## Triage protocol — manual deep-dive when the gate fires

The automated triage steps live in "Triage when the gate fires" above. This section covers the deeper-dive workflow once you've decided a regression is real.

1. **Reproduce locally** with `cargo bench --bench parse` — confirm it's not just CI noise (re-run the workflow if numbers look wild).
2. **Bisect** with `cargo bench --bench parse -- --save-baseline X` at suspect commits, then `--baseline X` at the next one to find the introducing SHA.
3. **Profile** with `cargo bench --bench parse -- --profile-time 10` (criterion's built-in flag) or `samply` / `flamegraph` for deeper drill-down.
4. **Fix or revert**: if the regression is acceptable (e.g., new feature traded perf for correctness), document it in `CHANGELOG.md`. Otherwise, fix or revert.

## References

- [criterion.rs book](https://bheisler.github.io/criterion.rs/book/) — methodology, statistical model
- [`benches/parse.rs`](https://github.com/lijunzh/hunch/blob/main/benches/parse.rs) — the bench harness itself
- Sibling docs: [Coverage](../contributor-guide/coverage.md), [Mutation Testing](../contributor-guide/mutation-baseline.md), [Fuzzing](../contributor-guide/fuzzing.md), [Public API](./public-api.md)
- [#148](https://github.com/lijunzh/hunch/issues/148) — the parent epic
- [#138](https://github.com/lijunzh/hunch/pull/138) — established the property-extractor coverage shape this doc mirrors
- [#140](https://github.com/lijunzh/hunch/pull/140) — refreshed criterion's transitive dev-deps
