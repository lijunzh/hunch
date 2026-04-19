window.BENCHMARK_DATA = {
  "lastUpdate": 1776614022385,
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
      }
    ]
  }
}