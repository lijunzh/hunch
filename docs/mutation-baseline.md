# Mutation Testing Baseline

Hunch uses [`cargo-mutants`](https://mutants.rs/) to measure **assertion
quality**, not just code coverage. Mutation testing mutates the source
(flips `==` to `!=`, replaces `+` with `-`, etc.) and runs the test suite
against each mutated build. A mutation that **survives** all tests means
no test would actually catch that bug — the line might be 100% covered
yet still fail to detect a real regression.

This complements [code coverage (#145)](./coverage.md): coverage tells us
which lines run; mutation testing tells us which lines have **strong
assertions**.

## How it runs

A nightly GitHub Actions workflow ([`.github/workflows/mutants.yml`](../.github/workflows/mutants.yml))
runs `cargo mutants` against the highest-value targets at 03:14 UTC and
publishes results as a Job Summary + downloadable `mutants-out` artifact.

The job is **advisory** (`|| true` on the run step) — surviving mutants
do not fail CI. They get triaged here and addressed in follow-up PRs.

You can also trigger it on demand from the Actions UI via
**Run workflow** → **Mutation Tests**.

## Scope (first slice)

The full crate has ~2,876 mutants and would take ~10 hours single-threaded.
This first slice scopes the nightly run to two highest-value targets
identified in the [Mutation testing epic (#146)](https://github.com/lijunzh/hunch/issues/146):

| File | Mutants | Why |
|---|---|---|
| `src/pipeline/mod.rs` | ~57 | Orchestration core — every property runs through here |
| `src/properties/title/clean.rs` | ~99 | Busiest property module; PR-C #138 added kitchen-sink coverage |

Combined run with `--jobs 4` on a GitHub-hosted ubuntu runner: **~12–15 min**.

## Roadmap

Sequenced expansion (each is its own PR within the #146 epic):

1. **Add the `Pipeline` neighbours**: `pipeline/context.rs`, `pipeline/invariance.rs`,
   `pipeline/mod.rs`'s sibling helpers.
2. **Add the `properties/title/` strategies**: `secondary.rs`, `mod.rs`,
   the four `strategies/*.rs` modules.
3. **Add the matcher core**: `src/matcher/`, `src/zone_map.rs`, `src/tokenizer.rs`.
4. **Opt-in PR-time check**: a `mutation-test` label triggers a diff-only
   mutants run on the PR (cheap subset). Requires #146's epic-level decision
   on labelling.
5. **Hard kill-rate gate**: fail the nightly if kill rate drops more than
   N% below baseline. Deferred until baseline has settled across enough
   nightly runs to characterise noise.

## Local usage

Install once (note: requires `--locked` so the version matches CI):

```bash
cargo install cargo-mutants --locked
```

Run against one file (~5 min for a small file):

```bash
cargo mutants --file src/properties/year.rs --no-shuffle
```

Run against the same scope CI uses:

```bash
cargo mutants --no-shuffle --jobs 4 \
  --file src/pipeline/mod.rs \
  --file src/properties/title/clean.rs
```

Outputs land in `./mutants.out/`:

| File | Contents |
|---|---|
| `outcomes.json` | Machine-readable per-mutant results + counts |
| `missed.txt`    | Surviving mutants (the interesting ones) |
| `caught.txt`    | Killed mutants (good — your tests work) |
| `timeout.txt`   | Tests that hung — usually infinite-loop mutations |
| `unviable.txt`  | Mutants that didn't compile (rare, ignorable) |

`mutants.out/` is gitignored.

## Worked example: `src/properties/year.rs`

A pre-PR smoke run on `year.rs` (20 mutants, ~5 min) produced **3 surviving
mutants** that demonstrate the categories we'll see in nightly results:

### Equivalent mutation (accepted survival)

```text
src/properties/year.rs:19:15: replace < with <= in find_matches
```

```rust
let mut pos = 0;
while pos < input.len() {       // mutation: pos <= input.len()
    let Some(m) = YEAR_RE.find_at(input, pos) else {
        break;
    };
```

When `pos == input.len()`, `Regex::find_at` returns `None` and the loop
exits via the `else` branch on the next line — so `<` and `<=` produce
identical observable behaviour. **Equivalent mutation; document and
move on.**

### Real test gaps (backlog — file as follow-up issues)

```text
src/properties/year.rs:26:22: replace > with < in find_matches
src/properties/year.rs:29:20: replace < with > in find_matches
```

```rust
// Boundary: no digit before or after.
if m.start() > 0 && bytes[m.start() - 1].is_ascii_digit() {  // L26
    continue;
}
if m.end() < bytes.len() && bytes[m.end()].is_ascii_digit() { // L29
    continue;
}
```

Both mutations bypass the boundary check (the inverted comparison
short-circuits via `&&` so the check never runs). They survive because
**no test exercises a year touching the start or end of the input string**.
Trivial fix: add fixtures like `2020` (year alone), `12020.mkv` (digit
prefix), `20201.mkv` (digit suffix) and assert the boundary rejection.

These two are not fixed in this PR — that's deliberate. This PR sets up
the *infrastructure* to find findings; fixing them is the next loop.

## Triage protocol

When the nightly job posts a Job Summary with surviving mutants:

1. **Equivalent mutation?** (the mutation produces identical observable
   behaviour) → add a one-line entry to the "Accepted equivalents" table
   below with the mutation string + a one-sentence rationale.
2. **Real test gap?** → file a `tech-debt` issue with the mutation string
   in the title, link the surviving-mutants table from the most recent
   nightly run, and assign to the next coverage-improvement loop.
3. **Tool bug / unviable mis-classification?** → file upstream at
   <https://github.com/sourcefrog/cargo-mutants>.

## Accepted equivalents

| Mutation | Why it's equivalent | Accepted on |
|---|---|---|
| `src/properties/year.rs:19:15: replace < with <= in find_matches` | `find_at(input, input.len())` returns `None`; `<` and `<=` produce identical loop behaviour. | 2026-04-18 (smoke run) |

(Future entries get appended as they're triaged.)

## References

- [`cargo-mutants` book](https://mutants.rs/)
- Epic [#146](https://github.com/lijunzh/hunch/issues/146)
- Sibling: code coverage [#145](https://github.com/lijunzh/hunch/issues/145) /
  [`docs/coverage.md`](./coverage.md)
- Industry benchmark: 80% kill rate is the rough north star for parser
  code (mature mutation-tested Rust crates land 75–90%).
