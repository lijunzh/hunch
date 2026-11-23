# Hunch vs guessit — Compatibility

Hunch started as a Rust port inspired by Python's
[guessit](https://github.com/guessit-io/guessit), and we still run hunch
against guessit's test suite as a **secondary benchmark**.

But guessit compatibility is no longer the primary optimization target.
Hunch is tuned first for **real-world media-library accuracy**, with guessit
compatibility used as a reference point rather than a product goal.

> **Last updated:** 2026-11-23 (fixtures from guessit v4.5.5)

---

## Current snapshot

Latest compatibility rerun:

```bash
cargo test compatibility_report --release -- --ignored --nocapture
```

Results against guessit **v4.5.5**'s suites (1,631 cases, about 300 more
than the February 2026 copy):

- **1,276 / 1,631** cases passed
- **78.2%** overall compatibility (71.6% before the November 2026 fixes)
- **1 intentional divergence** (below)

The refresh added harder cases, so the rate is lower than the 81.8% the
old, smaller suite gave in April. On the old copy the new code passes
1,084 cases against 1,086 before; the six that flipped are all cases where
guessit itself changed its answer since (Castellano is Spanish, 3D layouts
are named, a `COMPLETE` run is a series).

A few examples of strong property areas:

- `container`: **94.8%**
- `type`: **93.8%**
- `season`: **92.9%**
- `episode`: **91.2%**
- `release_group`: **91.0%**
- `title`: **89.5%**

Areas with the most room left: `episode_title` (73.3%), `subtitle_language`
(76.9%), and the anime credits markers (`NCOP`, `NCED`, `OP2`) that
guessit 4.5 reads as `other` plus `credits_number`.

---

## Keeping up with guessit

guessit is maintained again and releases often (4.5.3 to 4.5.5 shipped in
one week of November 2026). The **guessit Drift** workflow checks on the
1st of each month whether a newer release exists. If it does, it scores
hunch against the new suites and opens a `guessit-drift` issue with the
pass rate on the new cases and the first failures. To refresh:

```bash
git clone --depth 1 --branch <tag> https://github.com/guessit-io/guessit.git /tmp/guessit
python3 .github/scripts/refresh-guessit-fixtures.py /tmp/guessit <tag>
HUNCH_RESULTS=/tmp/after.tsv cargo test compatibility_report --release -- --ignored --nocapture
```

`HUNCH_RESULTS` writes one line per case (`PASS`/`FAIL`, file, filename,
failures), so two runs can be diffed case by case.

---

## How to interpret this

guessit compatibility is useful for:

- spotting regressions against a large public fixture set
- finding parser blind spots we may have missed
- measuring broad behavior drift over time

It is **not** the final definition of correctness.

Some guessit fixtures encode parser-specific conventions rather than universal
truth. When compatibility and real-world behavior disagree, hunch prefers the
behavior that is more accurate and maintainable for actual media libraries.

---

## Intentional divergences

Hunch intentionally does not mirror guessit in a few places. Each one is
listed in `DIVERGENCES` in `tests/guessit_regression.rs`, which skips
that property for that case, and here with its reason.

| Case | Property | guessit | hunch | Why |
|---|---|---|---|---|
| `mpeg4-AVC` | `video_profile` | Advanced Video Codec High Definition | (none) | AVC is the H.264 codec name, not the AVCHD camcorder format (#237) |

If you find another, please file an issue: divergences should be
deliberate and documented, not accidental.

---

## Real-world accuracy matters more

The main quality signal for hunch is behavior on real media libraries, not
perfect reproduction of guessit's opinions.

As of the latest audit referenced in the README, hunch achieved **99.8%**
accuracy on a real-world library of 7,838 files, with the remaining edge cases
tracked as known limitations.

That is the benchmark we optimize for first.

---

## Reproducing the report

```bash
# Full compatibility snapshot
cargo test compatibility_report --release -- --ignored --nocapture

# Include sampled failure details
HUNCH_DUMP_FAILURES=50 cargo test compatibility_report --release -- --ignored --nocapture
```
