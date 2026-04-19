# Release Performance Trajectory

How hunch's parser performance has evolved across released versions.

> **Source of truth:** `gh-pages/release-snapshots/<tag>.json` — one
> immutable file per version tag. Generated automatically by the
> `Benchmarks` workflow whenever a `v*` tag is pushed (per
> [#179](https://github.com/lijunzh/hunch/issues/179)). Each snapshot
> records the bench numbers, git SHA, runner identity, and timestamp.
>
> See [Benchmarks](./benchmarks.md) for methodology and the
> [Live Dashboard](./benchmark-dashboard.md) for per-commit history.

## Snapshots

<div id="trajectory-status">Loading release snapshots…</div>
<div id="trajectory-content"></div>

<!-- The list of release tags below is the ONLY thing that needs to be
     updated as part of release prep. Newest tag first. Tags older
     than the bench harness landing in #176 (= older than v1.1.8) won't
     have snapshots — they're not in this list.

     Release-prep checklist (per #179):
       1. Bump version in Cargo.toml
       2. Update CHANGELOG.md
       3. Add the new tag to the RELEASE_TAGS array below (top of list)
       4. Tag + push: `git tag vX.Y.Z && git push origin vX.Y.Z`
       5. Wait ~3 min for the bench workflow to publish the snapshot
       6. (optional) Verify at https://lijunzh.github.io/hunch/release-snapshots/<tag>.json -->
<script>
const RELEASE_TAGS = [
  // Newest first. Add new entries to the TOP per the checklist above.
  // (Empty until the first post-#179 release lands.)
];

(async function () {
  const status = document.getElementById('trajectory-status');
  const content = document.getElementById('trajectory-content');

  if (RELEASE_TAGS.length === 0) {
    status.innerHTML =
      '<p><em>No release snapshots yet. The first one will appear after ' +
      'the next <code>v*</code> tag is pushed (post-#179 merge). ' +
      'Until then, see the <a href="./benchmark-dashboard.html">' +
      'live per-commit dashboard</a> for current performance trends.</em></p>';
    return;
  }

  // Fetch all snapshots in parallel; tolerate 404s (a tag in the list
  // without a published snapshot just gets skipped, with a console warning).
  const fetched = await Promise.all(RELEASE_TAGS.map(async (tag) => {
    try {
      const r = await fetch(`../release-snapshots/${tag}.json`);
      if (!r.ok) {
        console.warn(`No snapshot for ${tag} (HTTP ${r.status})`);
        return null;
      }
      return await r.json();
    } catch (e) {
      console.warn(`Failed to fetch ${tag}:`, e);
      return null;
    }
  }));

  const snapshots = fetched.filter(Boolean);
  if (snapshots.length === 0) {
    status.textContent =
      'Listed releases have no published snapshots yet. ' +
      'Re-deploying the docs after the next bench workflow run should fix this.';
    return;
  }

  // Build a per-bench comparison table: rows = bench names,
  // columns = release tags (newest first), cells = ns/iter.
  // Bench names taken from the newest snapshot; older snapshots that
  // are missing a bench (e.g. `minimal` was dropped in #191) show "—".
  const newest = snapshots[0];
  const benchNames = newest.benches.map(b => b.name);

  let html = '<h2>Per-bench comparison</h2><table><thead><tr><th>Bench</th>';
  for (const s of snapshots) html += `<th>${s.tag}</th>`;
  html += '</tr></thead><tbody>';

  for (const name of benchNames) {
    html += `<tr><td><code>${name}</code></td>`;
    for (const s of snapshots) {
      const b = s.benches.find(x => x.name === name);
      html += b
        ? `<td>${(b.value / 1000).toFixed(1)} \u00b5s</td>`
        : '<td><em>—</em></td>';
    }
    html += '</tr>';
  }
  html += '</tbody></table>';

  // Snapshot metadata table.
  html += '<h2>Snapshot details</h2><table><thead><tr><th>Tag</th><th>Date</th><th>SHA</th><th>Runner</th></tr></thead><tbody>';
  for (const s of snapshots) {
    const sha = s.sha.slice(0, 7);
    const date = s.date.slice(0, 10);
    html += `<tr><td><strong>${s.tag}</strong></td><td>${date}</td><td><code>${sha}</code></td><td><code>${s.runner}</code></td></tr>`;
  }
  html += '</tbody></table>';

  status.remove();
  content.innerHTML = html;
})();
</script>

## Methodology caveat

Numbers are wall-clock parse time on `ubuntu-latest` GitHub-hosted
runners — useful for **trajectory** (is the parser getting slower or
faster across releases?) but **not** for absolute comparisons against
your hardware. Local M-class hardware typically runs ~2–3× faster.

Per-bench variance from runner-hardware shifts has been observed at
±5–10% on the larger benches and >30% on benches under ~25 µs. The
[#178](https://github.com/lijunzh/hunch/issues/178) regression gate
uses a 120% threshold for exactly this reason; small absolute
swings shouldn't be over-interpreted as real regressions.
