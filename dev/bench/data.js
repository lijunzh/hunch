window.BENCHMARK_DATA = {
  "lastUpdate": 1776648746684,
  "repoUrl": "https://github.com/lijunzh/hunch",
  "entries": {
    "hunch criterion benches": [
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6c08a03891f285f20f9433245b6beeb6b9a3584e",
          "message": "feat(ci): wire up bench regression gate + PR comments (#178) (#189)\n\nSecond slice of the perf regression CI epic (#148, after #176 + #177).\nBuilds on #186's storage strategy decision (gh-pages branch, dev/bench\nsubdir) and the gh-pages orphan branch seeded there.\n\n## What changes in benchmark.yml\n\n  1. Add `pull_request` trigger (same path filter as push: source/\n     bench/Cargo paths only). Without it, the gate has no PRs to gate.\n\n  2. Add per-job `permissions:` override:\n       - contents: write     (push results to gh-pages on main)\n       - pull-requests: write (comment delta table on PRs)\n     Default workflow permissions stay `contents: read`.\n\n  3. Tighten `concurrency.group` to `benchmark-${{ github.ref }}` so\n     PR runs don't fight over a global lock with the main pipeline.\n     `cancel-in-progress` is now true for PRs (latest commit wins),\n     false for main pushes (every push appends to history).\n\n  4. Add new step \"Compare vs baseline + gate\" using\n     benchmark-action/github-action-benchmark@v1.22.0 (SHA-pinned\n     per project convention). Configured via `tool: cargo` to consume\n     the existing bench-output/parse.txt directly.\n\n## How the gate behaves per trigger\n\n  - Pull request: comment delta table; FAIL if any bench >120% of\n    baseline (= >20% slower). Does NOT push to gh-pages.\n  - Push to main: APPEND results to gh-pages/dev/bench/data.js. No\n    comment, no fail (regression that lands shouldn't block subsequent\n    main commits).\n  - Schedule (weekly Sunday) / workflow_dispatch: skip the comparison\n    step entirely. Artifact upload still happens as before\n    (90-day backstop).\n\n## Threshold rationale (from the issue)\n\n  - alert-threshold: 120% (= fail at >20% slower)\n  - Deliberately permissive to filter the 5-10% noise floor on shared\n    GitHub-hosted runners. A tighter threshold without statistical-\n    significance handling would flake constantly.\n  - Tighten once we have ~4 weeks of data to characterize real\n    variance. Tracked in the Roadmap section of docs/benchmarks.md.\n\n## docs/benchmarks.md updates\n\n  - \"How it runs in CI\": new per-trigger behavior table; new sections\n    \"Threshold rationale\" + \"Triage when the gate fires\".\n  - Roadmap: mark #178 as done; add \"tighten threshold\" follow-up;\n    update remaining items with current state.\n  - \"Triage protocol\" section repurposed as the manual deep-dive\n    workflow (now that the immediate triage steps live in the gate\n    section).\n  - Removes the \"Currently advisory only\" caveat \\xe2\\x80\\x94 the workflow now\n    has teeth.\n\n## Pre-merge note: stacked on #186\n\nThis PR is stacked on top of `docs/177-bench-storage-decision`\n(PR #186) since both edit docs/benchmarks.md. Merge order: #186 first,\nthen this. Rebase-on-merge is automatic via GitHub.\n\n## Verification done locally\n\n  - cargo bench --bench parse --no-run: compiles cleanly\n  - cargo test --lib: 339 passed (no source changes; sanity check)\n  - YAML validated with PyYAML: triggers/permissions/steps shape correct\n  - SHA for benchmark-action/github-action-benchmark@v1.22.0 confirmed\n    via GitHub API: a60cea5bc7b49e15c1f58f411161f99e0df48372\n\n## Verification deferred to first real run\n\nThe DoD-required verification scenarios from #178 will run on the\n*first PR after this merges* (chicken-and-egg \\xe2\\x80\\x94 the gate doesn't\nexist yet on main). Suggested followup checklist for the next perf-\nadjacent PR:\n\n  - [ ] Open a deliberately-regressing test PR (e.g., add\n        `std::thread::sleep(Duration::from_micros(500))` to parse())\n        and confirm the gate fires.\n  - [ ] Open a no-op PR and confirm no false-positive failure.\n  - [ ] Open a perf-improving PR and confirm the comment shows the win.\n\nCloses #178\nRefs #148\nRefs #177",
          "timestamp": "2026-04-19T10:05:14-05:00",
          "tree_id": "d86b7f17b8a4da8282eb9819f942633fb6a57bad",
          "url": "https://github.com/lijunzh/hunch/commit/6c08a03891f285f20f9433245b6beeb6b9a3584e"
        },
        "date": 1776611199866,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 99013,
            "range": "± 1877",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 232042,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 107400,
            "range": "± 5046",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 103391,
            "range": "± 725",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 84666,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "minimal",
            "value": 16588,
            "range": "± 57",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "97667d1b9d1c2bde8d87963715598b78b54c434d",
          "message": "refactor(docs): port docs/ to mdbook with three-section layout (#188) (#190)\n\n* refactor(docs): port docs/ to mdbook with three-section layout (#188)\n\nCloses #188.\n\n## Why\n\nHunch's docs/ had grown to 7 markdown files / ~1,256 lines covering\nboth end-user content (user_manual, compatibility) and contributor-\nfacing infra (mutation-baseline, fuzzing, coverage, benchmarks,\npublic-api). Rendered only as raw GitHub markdown — no search, no\nnested navigation, no thematic grouping.\n\nSister project koda recently adopted mdbook for the same surface and\nthe result is significantly nicer. ~3-hour port estimated; this commit\ndelivers it.\n\n## What changes\n\n### New mdbook scaffolding\n\n  - docs/book.toml             \\xe2\\x80\\x94 mdbook config (theme, edit URLs, search)\n  - docs/src/SUMMARY.md         \\xe2\\x80\\x94 sidebar nav (3 sections)\n  - docs/src/introduction.md    \\xe2\\x80\\x94 landing page with audience-table\n  - docs/src/reference/benchmark-dashboard.md \\xe2\\x80\\x94 NEW Pattern B page\n    that loads /dev/bench/data.js (committed by the benchmark workflow)\n    and renders Chart.js line charts per bench, per commit. Unifies\n    the perf dashboard into the docs site instead of forking a second\n    Chart.js page at /dev/bench/index.html.\n\n### Existing docs moved (git mv \\xe2\\x86\\x92 history preserved)\n\n  docs/user_manual.md         \\xe2\\x86\\x92 docs/src/user-guide/user-manual.md\n  docs/compatibility.md       \\xe2\\x86\\x92 docs/src/user-guide/compatibility.md\n  docs/benchmarks.md          \\xe2\\x86\\x92 docs/src/reference/benchmarks.md\n  docs/public-api.md          \\xe2\\x86\\x92 docs/src/reference/public-api.md\n  docs/public-api.txt         \\xe2\\x86\\x92 docs/src/reference/public-api.txt\n  docs/coverage.md            \\xe2\\x86\\x92 docs/src/contributor-guide/coverage.md\n  docs/mutation-baseline.md   \\xe2\\x86\\x92 docs/src/contributor-guide/mutation-baseline.md\n  docs/fuzzing.md             \\xe2\\x86\\x92 docs/src/contributor-guide/fuzzing.md\n\nCross-refs rewritten:\n  - Repo-root files (../.github/workflows/*, ../CONTRIBUTING.md,\n    ../SECURITY.md, ../benches/parse.rs) \\xe2\\x86\\x92 absolute github.com URLs\n    so they work in BOTH the rendered HTML and any GitHub-direct view.\n  - Sibling-doc refs (./other.md) \\xe2\\x86\\x92 mdbook-relative paths (e.g.\n    ../contributor-guide/coverage.md from a reference/ page).\n\n### Backwards-compat stubs\n\nOld top-level docs/*.md paths preserved as one-line stubs pointing\nto the new deployed-site URLs. Every issue, PR, crates.io page, and\nexternal link to the old paths still resolves. Slated for removal\nonce inbound traffic dies down (per #188 DoD).\n\n### CI: new docs deploy workflow\n\n  .github/workflows/docs.yml \\xe2\\x80\\x94 builds mdbook on push-to-main when\n  docs/** or the workflow itself changes; deploys to gh-pages branch\n  via peaceiris/actions-gh-pages with KEEP_FILES: TRUE.\n\n  The keep_files flag is critical: the benchmark workflow ALSO pushes\n  to gh-pages (into /dev/bench/). Without keep_files, every docs deploy\n  would wipe the bench dashboard data. With it, the two workflows\n  co-exist cleanly \\xe2\\x80\\x94 mdbook owns the site root + section dirs;\n  benchmark-action owns /dev/bench/.\n\n### Touch-ups\n\n  - README.md: doc table now points to the deployed mdbook URLs;\n    coverage badge link updated; new \"Benchmark Dashboard\" row added.\n  - src/properties/title/clean.rs: two doc comments updated to point\n    at the new docs/src/contributor-guide/mutation-baseline.md path.\n  - .gitignore: ignore docs/book/ (mdbook output); update two comments.\n\n## Verification done locally\n\n  - mdbook v0.5.2 (matches MDBOOK_VERSION pin in workflow)\n  - mdbook build docs: clean, no warnings\n  - cargo fmt --check: clean\n  - cargo clippy --all-targets: clean\n  - cargo test --lib: 339 passed\n  - YAML for docs.yml validated\n  - Visually previewed in browser \\xe2\\x80\\x94 nav, search, and edit-on-GitHub\n    links all work\n\n## One-time GitHub Pages admin step (post-merge)\n\nAfter this PR merges, set:\n  Settings \\xe2\\x86\\x92 Pages \\xe2\\x86\\x92 Build and deployment\n    \\xe2\\x86\\x92 Source: \"Deploy from a branch\"\n    \\xe2\\x86\\x92 Branch: gh-pages / (root)\n\nThen trigger the docs.yml workflow manually (workflow_dispatch) to do\nthe first deploy. Subsequent deploys are automatic on docs/ pushes.\n\nThe stub-file URLs will 404 until that admin click happens \\xe2\\x80\\x94 fine,\nsince the source content lives in the repo regardless.\n\n## Out of scope (intentionally)\n\n  - Rewriting any prose. Ports are content-preserving.\n  - Adding new chapters (e.g., contributor \"Getting started\").\n  - Translating to non-English.\n\nCo-authored-by: code-puppy-1d34f9 <code-puppy@users.noreply.github.com>\n\n* fix(ci): update file paths after #188 mdbook move\n\nTwo CI workflows + one CONTRIBUTING.md ref still pointed at the old\ndocs/<name>.md flat layout, surfaced by PR #190's first run:\n\n  - .github/workflows/ci.yml: 'Public API Surface' job hard-codes\n    docs/public-api.txt as its baseline path. Updated to\n    docs/src/reference/public-api.txt (where the file actually lives\n    now). Same for two job-summary echo lines that mention\n    docs/public-api.md.\n\n  - .github/workflows/benchmark.yml: docs comment pointing at\n    docs/benchmarks.md. Updated to docs/src/reference/benchmarks.md.\n\n  - CONTRIBUTING.md: ref to docs/compatibility.md. Updated.\n\nThese references all SHOULD have been caught when I rewrote the moved\nfiles' cross-refs, but I didn't grep CI/config/CONTRIBUTING.md for\nreferences back into docs/. Lesson logged.\n\nThe backwards-compat stubs at the OLD paths exist for *external* link\npreservation (issues, PRs, crates.io); for *internal* CI references,\nwe want to point at the canonical location to avoid the indirection.\n\n---------\n\nCo-authored-by: code-puppy-1d34f9 <code-puppy@users.noreply.github.com>",
          "timestamp": "2026-04-19T10:40:40-05:00",
          "tree_id": "4ba150455bd0dc66cf0a32b23e0e7104fdef344b",
          "url": "https://github.com/lijunzh/hunch/commit/97667d1b9d1c2bde8d87963715598b78b54c434d"
        },
        "date": 1776613325699,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 108770,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 243439,
            "range": "± 4491",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 113520,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 110093,
            "range": "± 1437",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 92976,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "minimal",
            "value": 22913,
            "range": "± 280",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "73b1ebb6f3cabb6410b5eaa58ce4e435c7bf441e",
          "message": "perf(bench): drop the `minimal` bench (closes #191) (#192)\n\nSurfaced by the first PR after the regression gate landed\n(#190 ran on a docs-only change yet the gate fired with ratio 1.37\non `minimal`).\n\n## Diagnosis\n\n`bench_minimal` parsed only \"movie.mkv\" \\u2014 the shortest possible input,\nrunning at 16-22 \\u00b5s on ubuntu-latest. Two consecutive #190 runs both\nreported ~22.7 \\u00b5s consistently, vs 16.6 \\u00b5s baseline from #189's merge.\n\nPattern: a roughly constant ~6 \\u00b5s delta hit *every* bench between the\ntwo runs. For larger benches (~100 \\u00b5s) that's a 5-8% ratio. For\n`minimal` (~17 \\u00b5s) it's 37%. This is a runner-hardware shift, not\nruntime jitter and not a parser regression.\n\n`minimal` was a noise generator: at 17 \\u00b5s total it primarily measured\nfunction-call overhead, not parser logic. The other 5 benches cover\nthe parse paths that actually matter (60-240 \\u00b5s on the runner).\n\n## Changes\n\n  - benches/parse.rs: removed `bench_minimal` and its\n    `criterion_group!` entry. Replaced with a comment explaining\n    why for future spelunkers.\n  - docs/src/reference/benchmarks.md: dropped the `minimal` row from\n    three tables (bench inventory, local baseline numbers, CI\n    estimates) and updated the \"run just one bench\" example to use\n    `movie_basic` instead.\n\n## Effect on the bench dashboard\n\nThe bench data file on gh-pages contains `minimal` in its history.\nAfter this merge, the next push-to-main bench run will record a new\nentry without `minimal`. The dashboard chart for `minimal` becomes a\nflat line that ends \\u2014 acceptable archaeology.\n\n## Verification\n\n  - cargo bench --bench parse --no-run: compiles cleanly\n  - mdbook build docs: clean, no warnings\n  - cargo test --lib: 339 passed (no behavior change)",
          "timestamp": "2026-04-19T10:52:21-05:00",
          "tree_id": "246ba547e1c8d2276f1bfe815c8a7a0a417f2ed9",
          "url": "https://github.com/lijunzh/hunch/commit/73b1ebb6f3cabb6410b5eaa58ce4e435c7bf441e"
        },
        "date": 1776614021874,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 73898,
            "range": "± 818",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 200843,
            "range": "± 1586",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 81701,
            "range": "± 1133",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 78719,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 63002,
            "range": "± 456",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83ced808b9903bad49071edf8ec8926ee6aa33ee",
          "message": "feat(ci): per-release benchmark snapshots (closes #179) (#194)\n\nCloses the #148 epic by adding the final piece: permanent\nper-release performance snapshots. Builds on #178 (gate),\n#188 (mdbook), and the gh-pages substrate from #186 + #190.\n\n## What this does\n\nOn every \\`v*\\` tag push, the Benchmarks workflow now ALSO:\n\n  1. Runs the same bench harness it runs on every PR/main push\n  2. Parses the bencher-format output into a structured JSON file\n  3. Pushes the file to gh-pages/release-snapshots/<tag>.json via\n     peaceiris/actions-gh-pages with destination_dir + keep_files\n\nThe snapshot format is intentionally minimal:\n\n    {\n      \"tag\": \"v1.1.9\",\n      \"sha\": \"<commit>\",\n      \"date\": \"2026-04-19T...Z\",\n      \"runner\": \"Linux-X64\",\n      \"benches\": [\n        { \"name\": \"movie_basic\", \"value\": 99013, \"unit\": \"ns/iter\",\n          \"variance\": 1877 },\n        ...\n      ]\n    }\n\nWhy a separate format from dev/bench/data.js (which already records\nper-commit history):\n\n  - data.js is the rolling per-commit history, mutable in scope —\n    benches may be added/removed (see #191's removal of \\`minimal\\`)\n  - release-snapshots/<tag>.json is the immutable per-release record;\n    consumers can diff v1.1.8.json vs v1.2.0.json without worrying\n    about history rewrites or schema changes\n\n## New mdbook page\n\ndocs/src/reference/release-trajectory.md — fetches the listed\nsnapshot JSONs in parallel, renders two tables:\n\n  1. Per-bench comparison (rows = benches, cols = release tags)\n  2. Snapshot metadata (tag, date, SHA, runner)\n\nThe list of release tags lives in a JS const at the top of the page.\nThis is a deliberate trade-off: GitHub Pages can't list directories,\nand a workflow-maintained manifest file would create race conditions\nwith the bench workflow's own pushes. Manual list update is one line\nduring release prep — already enumerated in the new CHANGELOG.md\nrelease-prep checklist comment.\n\n## CHANGELOG.md template\n\nAdded a release-prep checklist (HTML comment, invisible in rendered\nmarkdown) at the top so future maintainers know to:\n\n  - Bump version in Cargo.toml\n  - Move [Unreleased] entries\n  - Add the new tag to release-trajectory.md's RELEASE_TAGS array\n  - Optionally add a 'Performance' subsection linking to the trajectory page\n  - Tag + push\n\n## What this enables\n\n  - Maintainer narrative: 'we shipped v1.2.0 with a 15% parser speedup'\n    becomes verifiable from the trajectory page, not vibes\n  - Downstream consumer audit: 'did upgrading from v1.1.8 to v1.2.0\n    change perf?' answerable by fetching two JSON files\n  - Closes the #148 epic completely (the last DoD item)\n\n## Verification done locally\n\n  - jq snippet tested with sample bencher-format input — produces\n    correct JSON shape (no parsing edge cases hit)\n  - mdbook build docs: clean, no warnings, trajectory page renders\n  - cargo fmt + clippy: clean\n  - YAML validated for benchmark.yml — 8 steps total, last 2 gated\n    on \\`startsWith(github.ref, 'refs/tags/v')\\`\n\n## Verification deferred to first real release\n\nThe tag-push code path won't actually run until someone pushes a\n\\`v*\\` tag (the next release). Suggested first-release smoke test:\n\n  - [ ] Push a test tag like \\`v1.1.8.1-test\\` (or the next real release)\n  - [ ] Confirm bench workflow runs ~3 min after the tag push\n  - [ ] Confirm gh-pages/release-snapshots/<tag>.json appears\n  - [ ] Add the tag to release-trajectory.md's RELEASE_TAGS array\n  - [ ] Confirm the page renders the snapshot\n\nIf anything breaks, the snapshot generation is gated to tag push only\nso it can't affect normal PR/main bench runs.",
          "timestamp": "2026-04-19T12:21:44-05:00",
          "tree_id": "115cb5b57b5af4cb2a1d23249431e5fa2edad0fd",
          "url": "https://github.com/lijunzh/hunch/commit/83ced808b9903bad49071edf8ec8926ee6aa33ee"
        },
        "date": 1776619377817,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 91788,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 214526,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 99256,
            "range": "± 3089",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 95091,
            "range": "± 1994",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 79768,
            "range": "± 781",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "62015b9cd4d94f72d91c827252097bd9b9c0cba3",
          "message": "chore: pre-v2.0.0 polish (non_exhaustive coverage + doc accuracy) (#196)\n\nPR-2a of the v2.0.0 close-out plan. Three small cleanups that\nare 'free' under the v2.0.0 major bump but would each cost a\nseparate major bump if deferred. Plus two documentation drift\nfixes that real users would hit immediately.\n\n## API hygiene (free under v2.0.0; locks in future minor-safe extension)\n\n- src/hunch_result.rs: \\\\\\`Confidence\\\\\\` enum now \\\\\\`#[non_exhaustive]\\\\\\`.\n  This enum appears in user-facing examples (in user-manual.md and\n  the rustdoc) so its match-exhaustiveness contract is load-bearing.\n  Marking it now means future variants like a hypothetical\n  \\\\\\`Confidence::VeryHigh\\\\\\` (for context-resolved cross-file matches)\n  can land in a v2.x minor instead of forcing v3.0.0.\n\n- src/tokenizer.rs: \\\\\\`SegmentKind\\\\\\` enum now \\\\\\`#[non_exhaustive]\\\\\\`.\n  Less user-facing than \\\\\\`Confidence\\\\\\` but still in the public surface\n  per docs/src/reference/public-api.txt (line 459). Same logic:\n  future variants like \\\\\\`Volume\\\\\\` (disk-image roots) or \\\\\\`Archive\\\\\\`\n  (in-archive paths) can land in minor versions.\n\n- docs/src/reference/public-api.txt: refreshed to reflect the two\n  new \\\\\\`#[non_exhaustive]\\\\\\` annotations. Surgical 2-line diff at\n  lines 459 + 653 \\u2014 no other API changes snuck in.\n\nThis brings the project into full alignment on the documented API\nStability Policy: every \\\\\\`pub enum\\\\\\` in the public surface is now\n\\\\\\`#[non_exhaustive]\\\\\\`.\n\n## Documentation drift fixes\n\n- docs/src/user-guide/compatibility.md: removed the stale 'Intentional\n  divergences' entries for \\\\\\`audio_bit_rate\\\\\\`/\\\\\\`video_bit_rate\\\\\\` and\n  \\\\\\`mimetype\\\\\\`. PR #165 implemented all three (with the bit-rate split\n  classified by unit \\u2014 Kbps \\u2192 audio, Mbps \\u2192 video \\u2014 and mimetype as a\n  pure container-extension lookup), so claiming hunch doesn't emit them\n  is no longer true. Replaced with a brief 'no active divergences worth\n  listing' note. Also bumped the 'Last updated' stamp to 2026-04-19.\n\n- docs/src/user-guide/user-manual.md: extended the Library API section\n  with three new subsections covering the v2.0.0 API additions:\n  - 'Media-type checks' \\u2014 is_movie/is_episode/is_extra\n  - 'Bit rate and MIME type' \\u2014 audio_bit_rate/video_bit_rate/mimetype\n  Also added the wildcard arm to the Confidence \\\\\\`match\\\\\\` example so it\n  compiles against the now-non_exhaustive enum.\n\n## Path typo fix in the tripwire docs\n\n- docs/src/reference/public-api.md: the regenerate-the-baseline\n  instructions referenced \\\\\\`docs/public-api.txt\\\\\\` (legacy path from\n  before the mdbook port #190). Updated to the actual current\n  path \\\\\\`docs/src/reference/public-api.txt\\\\\\`. Also corrected the\n  invocation to \\\\\\`cargo +nightly public-api ... 2>/dev/null\\\\\\` which\n  is what actually works (the version without 2>/dev/null bleeds\n  cargo-build progress lines into the snapshot, as discovered while\n  regenerating for this PR).\n\n## Verification\n\n  - cargo test --lib: 339 passed (no internal exhaustive matches broke\n    \\u2014 our own internal handling already uses wildcards or is\n    non_exhaustive-aware)\n  - cargo clippy --all-targets: clean\n  - cargo fmt --check: clean\n  - mdbook build docs: clean\n  - public-api diff is exactly the 2 expected lines (Confidence +\n    SegmentKind \\\\\\`#[non_exhaustive]\\\\\\`)\n\n## What this PR explicitly does NOT do\n\n  - No version bump (saves that for PR-2c, the actual release commit)\n  - No #144 API audit work (that's PR-2b: a separate triage of the\n    853-line public surface to demote items that shouldn't be public).\n    The current refresh is a mechanical re-snapshot, not an audit.\n  - No CHANGELOG move (still in [Unreleased])\n\nRefs: #144, #165, #172. Pre-cursor to v2.0.0 release.",
          "timestamp": "2026-04-19T15:01:54-05:00",
          "tree_id": "742191be677ce5828e5186980a88734bc3c50134",
          "url": "https://github.com/lijunzh/hunch/commit/62015b9cd4d94f72d91c827252097bd9b9c0cba3"
        },
        "date": 1776628992965,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 105421,
            "range": "± 2590",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 246509,
            "range": "± 9260",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 111898,
            "range": "± 9538",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 109281,
            "range": "± 1793",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 92386,
            "range": "± 565",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3dfd88c7a1e3c7343a820c0be7c52ccd215fc565",
          "message": "feat!: shrink public API surface 853 → 202 lines via module demotion (closes #144) (#197)\n\n* feat!: shrink public API surface 853 → 202 lines via module demotion (closes #144)\n\nPR-2b of the v2.0.0 close-out plan. Aggressive scope per the\n'Polish + API audit' decision: demote all four leaked sub-modules\nto pub(crate), then clean up the dead-code findings the demotion\nexposed. 76% reduction in the SemVer-contracted public surface,\nzero behavior change.\n\n## The core change (src/lib.rs)\n\nFour module declarations changed:\n\n  -pub mod matcher;\n  -pub mod properties;\n  -pub mod tokenizer;\n  -pub mod zone_map;\n  +pub(crate) mod matcher;\n  +pub(crate) mod properties;\n  +pub(crate) mod tokenizer;\n  +pub(crate) mod zone_map;\n\nExisting pub use re-exports (Property, Confidence, MediaType,\nHunchResult, Pipeline) remain at the crate root and continue to\nwork for downstream callers using the documented import paths.\n\n## Public surface impact\n\n  Before:  853 lines (188 leaked internal items)\n  After:   202 lines (6 intended types + their impls/derives)\n  Reduction: 76%\n\nThe new surface is exactly the 6 user-facing types documented in\nthe User Manual:\n  - hunch() + hunch_with_context() entry points\n  - Pipeline (constructor + 3 run methods)\n  - HunchResult (struct + 41 typed accessors)\n  - Property (49 variants)\n  - Confidence (3 variants, non_exhaustive)\n  - MediaType (3 variants, non_exhaustive)\n\nItems removed from the public surface (no longer reachable):\n  - matcher::engine::resolve_conflicts\n  - matcher::regex_utils::{CharClass, BoundarySpec, BoundedRegex}\n  - matcher::rule_loader::{RuleSet, ZoneScope, PatternRule, ...}\n  - matcher::span::{MatchSpan, Source}\n  - tokenizer::{Token, Segment, BracketGroup, BracketKind, ...}\n  - zone_map::{ZoneMap, SegmentZone, YearInfo, TitleYear, ...}\n  - properties::* (all internal property matchers)\n\n## Internal cleanup the audit surfaced\n\nThe demotion let the dead-code lint reach previously-shielded items.\nTriage results:\n\nTruly dead, deleted:\n  - src/matcher/mod.rs: 2 unused re-exports (engine::resolve_conflicts,\n    span::{MatchSpan, Property}) \\u2014 lib.rs already re-exports Property\n    directly; the others were never reached\n  - src/matcher/span.rs: MatchSpan::is_empty (4 lines)\n  - src/tokenizer.rs: Token::{len, is_empty} + BracketGroup::{span,\n    content_span} (15 lines combined)\n\nTest-only, gated with #[cfg(test)]:\n  - src/matcher/rule_loader.rs: exact_count + pattern_count are only\n    called from #[cfg(test)] blocks (rule_registry.rs smoke tests +\n    rule_loader.rs unit tests)\n\nConstructor-set-but-never-read fields, marked #[allow(dead_code)]\nwith explanatory comment pointing to #144 and explaining the\nconstructor-cascade reason for not removing them outright:\n  - matcher::rule_loader::RuleSet::property\n  - tokenizer::PathSegment::depth\n  - tokenizer::TokenStream::input\n  - zone_map::ZoneMap::tech_zone\n  - zone_map::SegmentZone::tech_zone\n  - zone_map::YearInfo::end\n  - zone_map::TitleYear::value\n\nThese deserve a follow-up cleanup pass to actually remove them and\ntheir cascading construction sites, but that's a separate PR (this\none is scoped to 'audit + module demotion').\n\n## Builder-method rename (clippy::wrong_self_convention)\n\nThree consuming-builder methods on MatchSpan were renamed to follow\nRust convention (with_* for consuming builders that return Self,\nnot as_*):\n  - as_extension    → with_extension\n  - as_path_based   → with_path_based\n  - as_reclaimable  → with_reclaimable\n\nThese methods were public before this PR (because pub mod matcher\nre-exported them) but in practice never used by external callers \\u2014\nthe rename only affects 4 internal call-sites in src/. Brings them\nin line with the existing with_priority and with_source builders\non the same type.\n\nThe clippy lint was flagging these all along; we just couldn't act\non it without a major bump because the methods were technically\npublic. The audit unlocks the fix.\n\n## Test/doctest updates\n\nTests that used deep-path imports were updated to the re-exports:\n  - tests/integration.rs:    hunch::matcher::span::Property → hunch::Property\n  - tests/wrong_type.rs:     same\n  - tests/matching_constraints.rs: hunch::matcher::Property → hunch::Property\n\nDoctests in src/matcher/span.rs were updated:\n  - Property::from_name doctest: deep path → hunch::Property\n  - define_properties doctest: deep path → hunch::Property\n  - MatchSpan doctest: marked 'ignore' since MatchSpan is now\n    pub(crate) (still rendered in private rustdoc; just not run as\n    an external doctest, which would fail to resolve the import)\n\n## CHANGELOG.md updates\n\nTwo new BREAKING entries added to [Unreleased] → ### Changed:\n\n  1. The module surface reduction itself, with a Before/After\n     migration snippet showing how to switch from\n     'use hunch::matcher::span::Property;' to 'use hunch::Property;'.\n\n  2. The MatchSpan builder-method rename. Marked BREAKING for\n     paranoia (the methods were technically public before) but\n     migration is a no-op for any realistic downstream caller \\u2014\n     the methods aren't documented in the User Manual or rustdoc\n     examples.\n\nAlso corrected the existing non_exhaustive entry to list all 9\naffected enums (was previously 'Property, MediaType, Confidence,\nOutputFormat (and others)' \\u2014 wrong: there is no OutputFormat;\nthe actual set is Property, MediaType, Confidence, SegmentKind,\nSource, ZoneScope, Separator, BracketKind, CharClass).\n\n## Verification\n\n  - cargo build --all-targets: clean (zero warnings)\n  - cargo test: 12 doctests passed (2 ignored: priority module +\n    new MatchSpan-as-pub(crate) example), all 339+ integration tests\n    passed\n  - cargo clippy --all-targets: clean\n  - cargo fmt --check: clean\n  - cargo +nightly public-api --simplified: 202 lines\n    (matches new docs/src/reference/public-api.txt baseline exactly)\n\n## What this PR explicitly does NOT do\n\n  - No version bump (saves that for PR-2c)\n  - No removal of #[allow(dead_code)] fields \\u2014 deserves its own\n    follow-up because removing them cascades through every\n    constructor and would inflate this PR's diff considerably\n  - No deprecation warnings on the demoted items \\u2014 they're\n    pub(crate) now, so deprecation is moot. Downstream callers\n    using deep paths will get a hard E0603 'module is private'\n    error at compile time, which is the clearest possible signal.\n\nCloses #144.\n\n* fix(docs): drop intra-doc link to now-private MatchSpan\n\nThe crate-level docstring at src/lib.rs:95 referenced\n[MatchSpan](matcher::span::MatchSpan) as part of the architecture\noverview. Since PR-2b's module demotion made matcher pub(crate),\nthat path is no longer reachable from public docs and rustdoc fails\nthe build under -Dwarnings (rustdoc::private_intra_doc_links lint).\n\nReplaced the intra-doc link with the prose 'internal match spans' \\u2014\nthe architecture overview is for orientation, not API surface, and\nlinking to a private type from public docs would just leak the same\ninternal name we just removed from the SemVer contract.\n\nDiscovered by the Docs job on PR #197. Trivial follow-up to PR-2b's\ncore demotion work.",
          "timestamp": "2026-04-19T16:03:14-05:00",
          "tree_id": "8a300fd806a3e26ac97f2fec39f71e6f7c6dcf82",
          "url": "https://github.com/lijunzh/hunch/commit/3dfd88c7a1e3c7343a820c0be7c52ccd215fc565"
        },
        "date": 1776632668296,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 106081,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 245564,
            "range": "± 3340",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 112745,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 108994,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 92360,
            "range": "± 474",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "93032450275ea32eda790994ac030ba406db128b",
          "message": "feat!: remove deprecated Property::BitRate variant for v2.0.0 (#198)\n\nPR-2c of the v2.0.0 close-out plan. Honors the deprecation\ndeferred to the next major bump (#165) by actually removing the\nvariant now \\u2014 same logic that justified shipping #[non_exhaustive]\nand the module demotion under v2.0.0:\n\n  Deferring a known breaking change forces the next major bump\n  later. Bundling all known-breaking work into one major version\n  is strictly better than spreading it across multiple.\n\n## What changed\n\n- src/matcher/span.rs: removed the BitRate variant from the\n  define_properties! macro. The variant's docstring already said\n  'no parser produces it as of the bit-rate split (#158)'.\n\n- src/properties/bit_rate.rs: replaced the convoluted\n  match-then-match (normalize_unit -> property) with a single\n  match that returns both pieces. The unreachable defensive\n  fallback that previously emitted Property::BitRate is replaced\n  by unreachable!() with an explanatory message.\n\n  Why unreachable!() rather than another fallback variant:\n    - The regex character class is literally [KkMm], so reaching\n      the fallback requires a regex change that breaks the unit\n      contract.\n    - With unreachable!(), such a regex change fails loudly in\n      tests instead of producing silently-wrong output (an audio\n      bit rate labeled as video, or vice versa).\n    - The Zen: 'Errors should never pass silently. Unless\n      explicitly silenced.' We are not silencing.\n\n- docs/src/user-guide/user-manual.md: replaced the 'bit_rate' row\n  in the Audio properties table with separate 'audio_bit_rate'\n  and 'video_bit_rate' rows that match the actual emitted\n  properties.\n\n- CHANGELOG.md:\n  - Promoted the entry from ### Deprecated to ### Changed with the\n    BREAKING marker, including a migration snippet showing how to\n    convert exhaustive Property::BitRate match arms to the\n    AudioBitRate/VideoBitRate split.\n  - Removed the now-empty ### Deprecated subsection \\u2014 there are\n    no other deprecations in [Unreleased].\n\n- docs/src/reference/public-api.txt: refreshed. Surface goes from\n  202 lines to 201 (one variant removed).\n\n## Verification\n\n  - cargo build --all-targets: clean (zero warnings)\n  - cargo test: 339+ integration tests pass, 12 doctests pass\n  - cargo clippy --all-targets: clean\n  - cargo fmt --check: clean\n  - public-api diff: exactly one removal (Property::BitRate);\n    AudioBitRate and VideoBitRate still present\n  - grep BitRate in src/: only AudioBitRate and VideoBitRate remain\n\n## What this PR explicitly does NOT do\n\n  - No version bump (saves that for the actual release commit)\n nges to the bit_rate module name (the file is still\n    src/properties/bit_rate.rs because both AudioBitRate and\n    VideoBitRate are produced by it; renaming the module would be\n    pure churn)\n\nRefs: #144, #158, #165. Final pre-release cleanup before v2.0.0.",
          "timestamp": "2026-04-19T16:16:47-05:00",
          "tree_id": "b6ac92974469c32ca9d5d882ff583ca0cefe7c4c",
          "url": "https://github.com/lijunzh/hunch/commit/93032450275ea32eda790994ac030ba406db128b"
        },
        "date": 1776633488741,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 72660,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 201741,
            "range": "± 5236",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 81415,
            "range": "± 1278",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 78184,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 62878,
            "range": "± 181",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "lijunzh@users.noreply.github.com",
            "name": "Lijun Zhu",
            "username": "lijunzh"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "53e54ee8ca5e59b6872e16710fb9c0170df00605",
          "message": "fix(#212): parse CJK fansub patterns [Nth - NN] and [总第NN] (#213)\n\nTwo new regex patterns for Chinese fansub releases that were silently\ndropped before this commit, fixing three of the four bugs reported in\n#212:\n\n  Bug 1: `[4th - 01]` Latin-ordinal-in-CJK-bracket → no season/episode\n  Bug 2: `[总第67]` cumulative episode → no absolute_episode\n  Bug 4 (cascading): `type: movie` was a downstream effect of bugs 1+2;\n         once season+episode are detected the type classifier\n         correctly returns `episode`. (#212 also reports a separate\n         --context-vs--batch divergence; that's not addressed here\n         and remains open as the residual bug 4 for a follow-up PR.)\n\n## Patterns\n\n`NTH_DASH_EPISODE` — `\\[\\s*(\\d)(?:st|nd|rd|th)\\s*[-–—]\\s*(\\d{1,4})(?:[vV]\\d+)?\\s*\\]`\n  Matches `[1st - 03]` through `[9th - 12v2]`. Single-digit ordinals\n  only — `10th`/`20th` would too easily collide with scene tags or\n  group names (deliberate guardrail; revisit if we see real-world\n  filenames needing it).\n  Emits BOTH Season and Episode at STRUCTURAL priority — this is an\n  explicit marker, not a heuristic.\n  Accepts hyphen, en-dash (–), and em-dash (—) as separators since\n  fansubs vary.\n\n`CJK_CUMULATIVE_EPISODE` — `\\[\\s*总第\\s*(\\d{1,4})\\s*\\]`\n  Matches `[总第67]` and `[总第 100]`. Maps to existing\n  Property::AbsoluteEpisode (no new variant needed — semantics match\n  the existing absolute-episode concept used for anime).\n  Runs unconditionally in the dispatch chain, independent of regular\n  Episode detection, since the two coexist by design (S04E01 == ep 67\n  cumulative).\n\n## Wiring (find_matches dispatch order)\n\n  1. SxxExx family\n  2. `try_nth_dash_episode` (NEW) — only if SxxExx didn't match\n  3. NxN\n  4. Season patterns\n  5. Episode standalone\n  6. CJK bracket episode\n  7. CJK ordinal markers\n  8. `try_cjk_cumulative_episode` (NEW) — always runs, emits AbsoluteEpisode\n  9. Digit decomposition\n 10. detect_absolute_episodes (existing post-pass)\n\n## Verification on the exact bug-report filenames\n\nBefore:\n  {\"title\":\"...\",\"source\":\"Web\",\"type\":\"movie\"}   ← no season/ep, wrong type\n\nAfter:\n  {\"season\":4,\"episode\":1,\"absolute_episode\":67,\n   \"title\":\"...\",\"type\":\"episode\"}   ✅\n\n## Tests\n\n9 new tests in src/properties/episodes/tests.rs:\n  - test_nth_dash_episode_basic\n  - test_nth_dash_episode_all_ordinals (1st through 9th)\n  - test_nth_dash_episode_with_version (`[4th - 01v2]`)\n  - test_nth_dash_episode_em_dash (en-dash variant)\n  - test_nth_dash_episode_ignores_two_digit_ordinals (anti-FP guardrail)\n  - test_cjk_cumulative_episode_basic\n  - test_cjk_cumulative_episode_with_whitespace (`[总第 100]`)\n  - test_cjk_cumulative_episode_independent_of_episode\n  - test_212_full_filename_regression (both exact filenames from #212)\n\nAll quality gates green:\n  - cargo test (lib + integration + doctests): all pass\n  - cargo clippy --all-targets: clean\n  - cargo fmt --check: clean\n  - Compatibility corpus: 1080/1311 (82.38%) — no regression\n\n## Out of scope (filed as residual #212 work)\n\nBug 3 (path component `tv/` contaminating `source`) and the residual\nbug 4 (--context single-file mode classifying as `movie` when --batch\ncorrectly says `episode`) are not addressed here. They need different\nfixes (source matcher hardening + cli mode investigation) and will\nfollow in separate PRs to keep this one focused.",
          "timestamp": "2026-04-19T20:31:09-05:00",
          "tree_id": "b31ce3aeef5f7fea017209ae3a6f597edf2f0838",
          "url": "https://github.com/lijunzh/hunch/commit/53e54ee8ca5e59b6872e16710fb9c0170df00605"
        },
        "date": 1776648745791,
        "tool": "cargo",
        "benches": [
          {
            "name": "movie_basic",
            "value": 106755,
            "range": "± 1052",
            "unit": "ns/iter"
          },
          {
            "name": "movie_complex",
            "value": 245851,
            "range": "± 3492",
            "unit": "ns/iter"
          },
          {
            "name": "episode_sxxexx",
            "value": 112945,
            "range": "± 2358",
            "unit": "ns/iter"
          },
          {
            "name": "episode_with_path",
            "value": 110304,
            "range": "± 1946",
            "unit": "ns/iter"
          },
          {
            "name": "anime_bracket",
            "value": 92940,
            "range": "± 1137",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}