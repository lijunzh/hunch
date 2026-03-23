# Hunch vs guessit — Compatibility

Hunch started as a Rust port inspired by Python's
[guessit](https://github.com/guessit-io/guessit), and we still run hunch
against guessit's test suite as a **secondary benchmark**.

But guessit compatibility is no longer the primary optimization target.
Hunch is tuned first for **real-world media-library accuracy**, with guessit
compatibility used as a reference point rather than a product goal.

> **Last updated:** 2026-03-23 (`main` after #108)

---

## Current snapshot

Latest compatibility rerun:

```bash
cargo test compatibility_report -- --ignored --nocapture
```

Results:

- **1,071 / 1,309** cases passed
- **81.8%** overall compatibility
- **49 / 49** properties implemented
- **3 intentional divergences**

A few examples of still-strong property areas:

- `source`: **96.1%**
- `title`: **92.1%**
- `episode`: **90.6%**
- `release_group`: **90.4%**
- `type`: **93.7%**

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

Hunch intentionally does not mirror guessit in a few places:

- **`audio_bit_rate` / `video_bit_rate`**
  - guessit splits these into separate properties
  - hunch emits a single `bit_rate`, because filenames rarely contain enough
    reliable context to disambiguate audio vs video bit rate cleanly

- **`mimetype`**
  - guessit derives MIME type from the file extension
  - hunch does not emit it, because that is a trivial container lookup better
    handled by the consumer

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
cargo test compatibility_report -- --ignored --nocapture

# Include sampled failure details
HUNCH_DUMP_FAILURES=50 cargo test compatibility_report -- --ignored --nocapture
```
