window.BENCHMARK_DATA = {
  "lastUpdate": 1776611200147,
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
      }
    ]
  }
}