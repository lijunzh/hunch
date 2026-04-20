# 🔍 Hunch

[![Coverage](https://img.shields.io/badge/coverage-94.34%25-brightgreen)](https://lijunzh.github.io/hunch/contributor-guide/coverage.html)

**A fast, offline media filename parser for Rust — extract title, year, season,
episode, codec, language, and 49 properties from messy filenames.**

Hunch is a Rust rewrite of Python's [guessit](https://github.com/guessit-io/guessit).
Pure, deterministic, single-binary, linear-time regex only (ReDoS-immune).

## Quick Start

```bash
# Install
brew install lijunzh/hunch/hunch   # macOS/Linux
cargo install hunch                 # from source
cargo binstall hunch                # pre-built binary
```

### CLI

```bash
$ hunch "The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv"
{
  "container": "mkv",
  "episode": 3,
  "release_group": "DEMAND",
  "screen_size": "720p",
  "season": 5,
  "source": "Blu-ray",
  "title": "The Walking Dead",
  "type": "episode",
  "video_codec": "H.264"
}
```

### Library

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    assert_eq!(result.title(), Some("The Walking Dead"));
    assert_eq!(result.season(), Some(5));
    assert_eq!(result.episode(), Some(3));
}
```

### Cross-file context

For CJK, anime, or ambiguous filenames, hunch uses sibling files and
directory names as context for better title extraction and type detection.

```bash
# Single file with sibling context:
hunch --context ./Season1/ "(BD)十二国記 第13話(1440x1080 x264-10bpp flac).mkv"

# Batch-parse a single directory:
hunch --batch ./Season1/ --json

# Batch-parse an entire media library (recommended):
hunch --batch /path/to/tv/ -r -j
```

> **💡 Tip:** Always use `--batch -r` from your library root (e.g., `tv/`,
> `movies/`) rather than running `--batch` on each leaf directory individually.
> The `-r` flag preserves full relative paths like
> `tv/Anime/Show/Extra/Menu.mkv`, giving the parser critical context from
> directory names (`tv/`, `Anime/`, `Season 1/`) for accurate type detection.
> Without `-r`, files in deep subdirectories lose their path context and
> bonus content (SP, OVA, NCED) may be misclassified.

## Documentation

📖 **Full documentation site:** <https://lijunzh.github.io/hunch>

| Document | Audience | Content |
|---|---|---|
| [**User Manual**](https://lijunzh.github.io/hunch/user-guide/user-manual.html) | Users | Install, CLI, library API, all 49 properties, FAQ |
| [**Design**](DESIGN.md) | Contributors | Principles, architecture, key decisions |
| [**Compatibility**](https://lijunzh.github.io/hunch/user-guide/compatibility.html) | Everyone | guessit test suite pass rates by property |
| [**Benchmark Dashboard**](https://lijunzh.github.io/hunch/reference/benchmark-dashboard.html) | Maintainers | Live perf trends per commit |
| [**API Reference**](https://docs.rs/hunch) | Developers | Full Rust API docs |
| [**Changelog**](CHANGELOG.md) | Everyone | Version history |

## guessit Compatibility

All 49 guessit properties implemented. Validated against guessit's
upstream test suite — see [the compatibility
report](https://lijunzh.github.io/hunch/user-guide/compatibility.html)
for the live pass rate, per-property breakdowns, and the methodology
behind the numbers. (Single source of truth: that page is regenerated
from `cargo test -- --ignored guessit_compat` so it can't drift.)

## Known Limitations

In one real-world library audit of 7,838 files, hunch achieved **99.8%
accuracy** across a mixed Anime / English / Japanese / Kids collection. The
remaining failures fall into a small number of edge-case categories that are
difficult to solve reliably with a deterministic, offline filename parser.

These examples illustrate the main categories of remaining failures rather than
an exhaustive list of every individual filename.

### Bonus content without episode numbers

Files in bonus directories such as `Bonus/` or `特典映像/` that contain no
numeric episode marker may still be classified as `episode` with no episode
number. Hunch recognizes these directory names for title cleanup but does not
currently infer `type=extra` from directory names alone.

```
tv/Anime/.../特典映像/[DBD-Raws][Natsume Yuujinchou Shichi][声優トークショー][1080P][BDRip][HEVC-10bit][FLAC].mkv
  → type=episode, episode=None  (expected: type=extra)

tv/English/Power Rangers/17 - Power Rangers RPM/Bonus/Power Rangers RPM - Stuntman Behind The Scenes (Japanese).mp4
  → type=episode, episode=None  (expected: type=extra)
```

**Why this remains difficult:** directory names are useful context, but using
them alone to infer `type=extra` would require an open-ended set of
library-specific rules (`Extras/`, `Featurettes/`, `Behind the Scenes/`,
`Making Of/`, etc.), increasing regression risk across other collections.

### Sample / preview clips

Verification clips such as `Sample1.mkv` inside `Samples/` directories may have
their digits interpreted as episode numbers.

```
movie/.../Samples/Sample1.mkv
  → type=episode, episode=1  (expected: not real media content)
```

**Why this is low priority:** sample files are typically release artifacts
rather than meaningful library entries. Reliable detection would require
special-casing many filename and directory conventions that vary across release
groups.

### Ambiguous special / episode cross-references

Some filenames contain both special markers (`SP`) and episode markers (`EP`),
where the episode number refers to a related TV episode rather than the file
itself.

```
movie/.../[Detective Conan][Tokuten BD][SP02][TV Series EP1080][BDRIP][1080P][H264_FLAC].mkv
  → type=episode, episode=1080  (EP1080 is a cross-reference, not this file's episode)
```

**Why this remains difficult:** distinguishing "this file is episode 1080" from
"this file references episode 1080" requires semantic understanding beyond
hunch's current deterministic filename heuristics.

### Malformed filenames

Genuinely malformed inputs such as `1.The.mkv.mkv` can still produce poor
results.

**Why this is not prioritized:** hunch assumes filenames contain at least some
recoverable structure. Severely malformed input is treated as garbage-in,
garbage-out.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The easiest contribution is
[reporting a failed parse](https://github.com/lijunzh/hunch/issues/new/choose).

```bash
cargo test              # full suite
cargo test -- --ignored # guessit compatibility report
cargo bench             # benchmarks
```

## License

[MIT](LICENSE)
