# User Manual — Hunch

> Installation, CLI usage, library API, and all 49 properties.

---

## Installation

### Homebrew (macOS / Linux)

```bash
brew install lijunzh/hunch/hunch
```

### Cargo (from source)

```bash
cargo install hunch
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/lijunzh/hunch/releases).
Also supports [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall hunch
```

### As a library

```bash
cargo add hunch
```

---

## CLI Usage

### Basic

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

Multiple files:

```bash
hunch "Movie.2024.1080p.mkv" "Show.S01E01.mkv"
```

### Cross-file context

For CJK, anime, or ambiguous filenames, sibling files improve accuracy:

```bash
# Single file with context from its directory
hunch --context ./Season1/ "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv"

# Batch mode: parse all files in a directory (mutual context)
hunch --batch ./Season1/ --json

# Recursive batch: parse an entire media library (RECOMMENDED)
hunch --batch /path/to/tv/ -r -j
hunch --batch /path/to/movies/ -r -j
```

> **💡 Important:** For media libraries, always use `--batch -r` from the
> library root (e.g., `tv/`, `movies/`) rather than running `--batch` on
> each subdirectory individually. The `-r` flag preserves full relative
> paths like `tv/Anime/Show/Extra/Menu.mkv`, which gives the parser
> critical context from directory names (`tv/`, `Anime/`, `Season 1/`)
> for accurate type detection and title extraction.
>
> Without `-r`, files in deep subdirectories lose their path context.
> For example, `Extra/Menu 1-1.mkv` would be classified as a movie,
> but `tv/Anime/Show/Extra/Menu 1-1.mkv` is correctly classified as
> an episode because the parser sees the `tv/` and `Anime/` components.

### Options

| Flag | Description |
|---|---|
| `--context <DIR>` | Use sibling files for better title detection |
| `--batch <DIR>` | Parse all media files in a directory |
| `-r`, `--recursive` | Recurse into subdirectories (with `--batch`). Symlinks are skipped (loop-safe, sandbox-safe), and traversal stops at 32 levels deep. |
| `-j`, `--json` | Compact JSON output (default is pretty-printed) |
| `-v`, `--verbose` | Enable debug logging |

### Logging

Hunch uses the [`log`](https://docs.rs/log) crate for diagnostic output.
This is invaluable for debugging misparses.

```bash
# Debug level via --verbose
hunch -v "Movie.2024.1080p.BluRay.x264-GROUP.mkv"

# Fine-grained control via RUST_LOG
RUST_LOG=hunch=trace hunch "Movie.2024.1080p.mkv"
```

| Level | What it shows |
|---|---|
| `debug` | Pipeline stage transitions, match counts, title decisions |
| `trace` | Every match span, conflict evictions, zone rule filtering |

---

## Library API

### Basic usage

```rust
use hunch::hunch;

fn main() {
    let result = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    assert_eq!(result.title(), Some("The Walking Dead"));
    assert_eq!(result.season(), Some(5));
    assert_eq!(result.episode(), Some(3));
    assert_eq!(result.source(), Some("Blu-ray"));
    assert_eq!(result.video_codec(), Some("H.264"));
    assert_eq!(result.release_group(), Some("DEMAND"));
    assert_eq!(result.container(), Some("mkv"));
}
```

### Cross-file context

```rust
use hunch::hunch_with_context;

fn main() {
    let result = hunch_with_context(
        "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv",
        &[
            "(BD)十二国記 第01話「月の影 影の海　一章」(1440x1080 x264-10bpp flac).mkv",
            "(BD)十二国記 第02話「月の影 影の海　二章」(1440x1080 x264-10bpp flac).mkv",
        ],
    );
    assert_eq!(result.title(), Some("十二国記"));
}
```

### Pipeline reuse

For batch processing, reuse the `Pipeline` to avoid re-compiling
TOML rules on each call:

```rust
use hunch::Pipeline;

fn main() {
    let pipeline = Pipeline::new();
    let filenames = vec!["Movie.2024.mkv", "Show.S01E01.mkv"];

    for name in filenames {
        let result = pipeline.run(name);
        println!("{}: {}", name, result.to_json());
    }
}
```

### Confidence

```rust
use hunch::{hunch, Confidence};

fn main() {
    let result = hunch("ambiguous_file.mkv");
    match result.confidence() {
        Confidence::High   => println!("Confident parse"),
        Confidence::Medium => println!("Reasonable parse"),
        Confidence::Low    => println!("Consider using --context"),
        // `Confidence` is `#[non_exhaustive]` so future variants land
        // without forcing a major-version bump. Add a wildcard arm to
        // your `match`es:
        _                  => println!("Unknown confidence level"),
    }
}
```

### Media-type checks (added in v2.0.0)

Three convenience helpers route a result to the right downstream lookup
(e.g., TMDb for movies vs. TVDb for episodes) without an explicit
`MediaType` import:

```rust
use hunch::hunch;

fn main() {
    let r = hunch("Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv");
    if r.is_episode() {
        // route to TVDb
    }
    if r.is_movie() {
        // route to TMDb
    }
    if r.is_extra() {
        // bonus content / specials / NCOP / NCED — may not have a DB entry
    }
}
```

All three return `false` when the media type is unknown (rather than
defaulting to a guess). Callers that need to distinguish "definitely
not X" from "unknown" should use
[`media_type()`](https://docs.rs/hunch/latest/hunch/struct.HunchResult.html#method.media_type)
directly.

### Bit rate and MIME type (added in v2.0.0)

The `bit_rate` property is split by unit (`Kbps` → audio, `Mbps` →
video); MIME type is derived from the container extension:

```rust
use hunch::hunch;

fn main() {
    let r = hunch("Movie.2024.DD5.1.448Kbps.x264.5500Kbps.mp4");
    assert_eq!(r.audio_bit_rate(), Some("448Kbps"));
    assert_eq!(r.video_bit_rate(), Some("5500Kbps"));
    assert_eq!(r.mimetype(),       Some("video/mp4"));
}
```

MIME type returns `None` when the container is unknown rather than
fabricating a value — callers that need a fallback should provide it
at the call site.

### Full API reference

See **[docs.rs/hunch](https://docs.rs/hunch)** for all 49
[`Property`](https://docs.rs/hunch/latest/hunch/matcher/span/enum.Property.html)
variants and
[`HunchResult`](https://docs.rs/hunch/latest/hunch/struct.HunchResult.html)
accessors.

---

## All 49 Properties

### Structural (always unambiguous)

| Property | Example value | Example input |
|---|---|---|
| `title` | The Walking Dead | `The.Walking.Dead.S05E03` |
| `season` | 5 | `S05E03` |
| `episode` | 3 | `S05E03` |
| `year` | 2024 | `Movie.2024.1080p` |
| `date` | 2024-03-15 | `Show.2024.03.15` |
| `container` | mkv | `movie.mkv` |
| `type` | episode / movie | (inferred) |

### Video

| Property | Example value | Example input |
|---|---|---|
| `video_codec` | H.264 | `x264` |
| `screen_size` | 1080p | `1080p` |
| `frame_rate` | 23.976fps | `23.976fps` |
| `color_depth` | 10-bit | `10bit` |
| `video_profile` | High 10 | `Hi10P` |
| `video_api` | DXVA | `DXVA` |
| `aspect_ratio` | 16:9 | `16x9` |

### Audio

| Property | Example value | Example input |
|---|---|---|
| `audio_codec` | AAC | `AAC` |
| `audio_channels` | 5.1 | `5.1ch` |
| `audio_profile` | HD MA | `DTS-HD.MA` |
| `bit_rate` | 320kbps | `320kbps` |

### Source & Edition

| Property | Example value | Example input |
|---|---|---|
| `source` | Blu-ray | `BluRay` |
| `streaming_service` | Netflix | `NF` |
| `edition` | Director's Cut | `Directors.Cut` |
| `other` | Proper, Repack, 3D, ... | `PROPER` |

### Release metadata

| Property | Example value | Example input |
|---|---|---|
| `release_group` | DEMAND | `-DEMAND` |
| `website` | rarbg.to | `[rarbg.to]` |
| `crc32` | ABCD1234 | `[ABCD1234]` |
| `uuid` | ... | `{uuid}` |
| `size` | 1.4 GB | `1.4GB` |
| `proper_count` | 1 | `PROPER` |
| `version` | 2 | `v2` |

### Episode details

| Property | Example value | Example input |
|---|---|---|
| `episode_title` | The Brain In The Bot | (text after episode marker) |
| `film_title` | ... | (multi-film sets) |
| `alternative_title` | ... | (AKA titles) |
| `bonus` | 1 | `x01` |
| `bonus_title` | ... | (bonus feature title) |
| `episode_details` | Special | `Special` |
| `episode_format` | Miniseries | `Miniseries` |
| `episode_count` | 24 | `24eps` |
| `season_count` | 5 | `5seasons` |
| `absolute_episode` | 45 | (anime absolute numbering) |
| `week` | 12 | `Week.12` |
| `film` | 2 | `Film.2` |
| `disc` | 1 | `Disc.1` |
| `cd` | 2 | `CD2` |
| `cd_count` | 3 | `3CDs` |
| `part` | 1 | `Part.1` |

### Language

| Property | Example value | Example input |
|---|---|---|
| `language` | English | `English` |
| `subtitle_language` | French | `sub.French` |
| `country` | US | `US` |

---

## FAQ

### Why is the title wrong?

Title extraction is the hardest problem. The engine finds the gap before
the first tech anchor — if it can't find anchors, the title boundary is
a guess. Use `--context` to provide sibling files for structural evidence.

For batch processing, use `--batch -r` from the library root to give
the parser full path context. See [Cross-file context](#cross-file-context).

### Why is the year detected as title content?

Year-like numbers (e.g., "2001" in "2001.A.Space.Odyssey.1968") are
ambiguous. With `--context`, siblings reveal which numbers are invariant
(title) vs variant (metadata).

### How fast is it?

Single-file parsing: ~50–150µs. Batch mode with 100 files: ~5–15ms.
All regex is linear-time (Thompson NFA). No backtracking, ever.

### Does it work with non-Latin scripts?

Yes. CJK, Cyrillic, Arabic filenames all work. Cross-file context
(`--context` / `--batch`) significantly improves CJK title extraction.

### How do I debug a misparse?

```bash
hunch -v "problematic.filename.mkv"
# or for maximum detail:
RUST_LOG=hunch=trace hunch "problematic.filename.mkv"
```

The trace output shows every match, eviction, and decision.
