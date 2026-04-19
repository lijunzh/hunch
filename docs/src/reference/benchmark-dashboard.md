# Benchmark Dashboard

Live performance trends for hunch's parser, sourced from the
[Benchmarks workflow](https://github.com/lijunzh/hunch/actions/workflows/benchmark.yml)
that runs on every push to `main`.

> **About this data**: each chart shows wall-clock parse time per
> bench commit-by-commit. The data is committed to
> [`gh-pages/dev/bench/data.js`](https://github.com/lijunzh/hunch/blob/gh-pages/dev/bench/data.js)
> by the [`benchmark-action/github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark)
> step in `benchmark.yml`. See [Benchmarks](./benchmarks.md) for
> methodology, the regression-gate threshold, and triage protocol.

## Trend per benchmark

<div id="bench-charts">
  <p id="bench-status">Loading bench data…</p>
</div>

<!-- Chart.js: pinned to the latest stable major (4.x) via jsDelivr CDN.
     SRI hash omitted because the version selector breaks SRI; if we ever
     hit a CSP issue, switch to bundling chart.umd.min.js under theme/. -->
<script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
<script src="../dev/bench/data.js"></script>
<script>
(function () {
  const status = document.getElementById('bench-status');
  const container = document.getElementById('bench-charts');

  // The github-action-benchmark action populates window.BENCHMARK_DATA
  // with shape { lastUpdate, repoUrl, entries: { 'hunch criterion benches': [...] } }.
  if (typeof window.BENCHMARK_DATA === 'undefined') {
    status.textContent =
      'No benchmark data yet — the dashboard will populate after ' +
      'the first push to main runs the Benchmarks workflow.';
    return;
  }

  const suite = window.BENCHMARK_DATA.entries['hunch criterion benches'];
  if (!suite || suite.length === 0) {
    status.textContent = 'Bench data file present but empty. Check workflow logs.';
    return;
  }

  // Pivot from per-commit-list-of-benches to per-bench-list-of-commits.
  const benchSeries = new Map();
  for (const commit of suite) {
    const ts = new Date(commit.date).toISOString().slice(0, 10);
    const sha = commit.commit.id.slice(0, 7);
    for (const b of commit.benches) {
      if (!benchSeries.has(b.name)) benchSeries.set(b.name, []);
      benchSeries.get(b.name).push({
        x: `${ts}\n${sha}`,
        y: b.value,
        unit: b.unit,
      });
    }
  }

  status.remove();
  for (const [name, series] of benchSeries) {
    const wrapper = document.createElement('div');
    wrapper.style.cssText = 'height: 320px; margin: 1.5rem 0;';
    const canvas = document.createElement('canvas');
    wrapper.appendChild(canvas);
    container.appendChild(wrapper);

    new Chart(canvas, {
      type: 'line',
      data: {
        labels: series.map(p => p.x),
        datasets: [{
          label: `${name} (${series[0].unit})`,
          data: series.map(p => p.y),
          borderColor: '#0053e2',
          backgroundColor: 'rgba(0, 83, 226, 0.1)',
          tension: 0.2,
          fill: true,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          y: { title: { display: true, text: series[0].unit }, beginAtZero: false },
        },
        plugins: { legend: { position: 'top' } },
      },
    });
  }
})();
</script>

## What the gate does

The [Benchmarks workflow](https://github.com/lijunzh/hunch/actions/workflows/benchmark.yml)
also runs on every PR and **fails any bench that gets >20% slower**
(threshold: `120%` of latest main baseline, p<0.01 not yet enforced —
shared-runner noise still dominates). See [Benchmarks → Threshold rationale](./benchmarks.md#threshold-rationale).

## Drill-down

If you spot a step-change above:

1. Click "Benchmarks" in the GitHub Actions UI to find the runs around
   that commit.
2. Download the `bench-results-<sha>` artifact (90-day retention) for
   the full criterion log.
3. Reproduce locally: `cargo bench --bench parse -- --baseline <known-good-sha>`.

Triage protocol: see [Benchmarks → Triage](./benchmarks.md#triage-when-the-gate-fires).
