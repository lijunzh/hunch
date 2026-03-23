# 🔍 Hunch

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

| Document | Audience | Content |
|---|---|---|
| [**User Manual**](docs/user_manual.md) | Users | Install, CLI, library API, all 49 properties, FAQ |
| [**Design**](docs/design.md) | Contributors | Principles, architecture, key decisions |
| [**Compatibility**](docs/compatibility.md) | Everyone | guessit test suite pass rates by property |
| [**API Reference**](https://docs.rs/hunch) | Developers | Full Rust API docs |
| [**Changelog**](CHANGELOG.md) | Everyone | Version history |

## guessit Compatibility

All 49 guessit properties implemented. Validated against guessit's
1,309-case test suite.

| Metric | Value |
|---|---|
| Pass rate | **81.8%** (1,071 / 1,309) |
| Properties at 95%+ | 22 |
| Properties at 100% | 16 |

See [docs/compatibility.md](docs/compatibility.md) for per-property breakdowns.

## Known Limitations

Hunch achieves **99.8% accuracy** on a real-world media library of 7,838 files
(validated against a mixed Anime/English/Japanese/Kids collection). The
remaining edge cases are documented below as honest limitations of a
deterministic, offline filename parser.

### Bonus content without episode numbers

Files in bonus directories (`Bonus/`, `特典映像/`) that lack numeric episode
markers are typed as `episode` with no episode number. Hunch recognizes these
directory names for title cleanup but does not infer `type=extra` from
directory names alone.

```
tv/Anime/.../特典映像/[DBD-Raws][Natsume Yuujinchou Shichi][声優トークショー][1080P][BDRip][HEVC-10bit][FLAC].mkv
  → type=episode, episode=None  (expected: type=extra)

tv/English/Power Rangers/17 - Power Rangers RPM/Bonus/Power Rangers RPM - Stuntman Behind The Scenes (Japanese).mp4
  → type=episode, episode=None  (expected: type=extra)
```

**Why not fix it?** Extending directory-name detection to set `type` couples
title cleanup with type inference. The set of bonus directory names is
unbounded (`Extras/`, `Featurettes/`, `Behind the Scenes/`, `Making Of/`,
etc.) — each new rule risks regressions on other libraries.

### Sample/preview clips in movie directories

Verification clips like `Sample1.mkv` in `Samples/` subdirectories may have
digits parsed as episode numbers, causing `type=episode` in a movie context.

```
movie/.../Samples/Sample1.mkv
  → type=episode, episode=1  (expected: not real media content)
```

**Why not fix it?** Sample files are not meaningful media content. Filtering
them would require special-casing directory and filename patterns that vary
across release groups.

### Ambiguous special/episode cross-references

Filenames with both `SP` (special) and `EP` (episode) markers where the EP
number refers to a related TV episode rather than this file's own episode
number.

```
movie/.../[Detective Conan][Tokuten BD][SP02][TV Series EP1080][BDRIP][1080P][H264_FLAC].mkv
  → type=episode, episode=1080  (EP1080 is a cross-reference, not this file's episode)
```

**Why not fix it?** Distinguishing "this file is episode 1080" from "this file
relates to episode 1080" requires semantic understanding that a filename parser
cannot provide.

### Malformed filenames

Genuinely broken filenames like `1.The.mkv.mkv` produce nonsensical results.
This is garbage-in, garbage-out — no parser can extract structure from
structureless input.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The easiest contribution is
[reporting a failed parse](https://github.com/lijunzh/hunch/issues/new/choose).

```bash
cargo test              # 295 tests
cargo test -- --ignored # guessit compatibility report
cargo bench             # benchmarks
```

## License

[MIT](LICENSE)
