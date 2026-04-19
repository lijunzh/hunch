# Hunch vs guessit — Compatibility

Hunch started as a Rust port inspired by Python's
[guessit](https://github.com/guessit-io/guessit), and we still run hunch
against guessit's test suite as a **secondary benchmark**.

But guessit compatibility is no longer the primary optimization target.
Hunch is tuned first for **real-world media-library accuracy**, with guessit
compatibility used as a reference point rather than a product goal.

> **Last updated:** 2026-04-19 (pre-v2.0.0)

---

## Current snapshot

Latest compatibility rerun:

```bash
cargo test compatibility_report --release -- --ignored --nocapture
```

Results:

- **1,080 / 1,311** cases passed
- **81.8%** overall compatibility
- **50 / 50** properties implemented
- **3 intentional divergences**

A few examples of still-strong property areas:

- `source`: **96.1%**
- `type`: **93.7%**
- `title`: **91.8%**
- `episode`: **90.6%**
- `release_group`: **90.4%**

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

Hunch intentionally does not mirror guessit in a few places. The list is
smaller than it used to be — several earlier divergences (notably the
bit_rate split and mimetype derivation) were resolved in v2.0.0 (#165)
because real-world filenames turned out to provide enough signal after
all.

*Active divergences as of v2.0.0: none worth listing.* If you find one,
please file an issue — the goal is for divergences to be deliberate and
documented, not accidental.

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
