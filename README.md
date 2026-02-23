# 🔍 hunch

> A media filename parser for Rust — spiritual descendant of Python's [guessit](https://github.com/guessit-io/guessit).

**hunch** extracts structured metadata from media filenames and release names:
title, year, season, episode, video codec, audio codec, source, resolution,
release group, and more.

## Quick Start

```rust
use hunch::guess;

let result = guess("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
assert_eq!(result.title(), Some("The Matrix"));
assert_eq!(result.year(), Some(1999));
assert_eq!(result.screen_size(), Some("1080p"));
assert_eq!(result.source(), Some("Blu-ray"));
assert_eq!(result.video_codec(), Some("H.264"));
assert_eq!(result.release_group(), Some("GROUP"));
assert_eq!(result.container(), Some("mkv"));
```

## CLI

```bash
$ hunch "Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv"
{
  "container": "mkv",
  "episode": 16,
  "release_group": "DEMAND",
  "screen_size": "720p",
  "season": 5,
  "source": "Blu-ray",
  "title": "Breaking Bad",
  "type": "episode",
  "video_codec": "H.264"
}
```

## Properties Detected

| Property | Examples |
|---|---|
| title | The Matrix, Breaking Bad |
| year | 1999, 2024 |
| season / episode | S01E02, 1x03 |
| video_codec | H.264, H.265, AV1 |
| audio_codec | AAC, DTS, Dolby Atmos |
| audio_channels | 5.1, 7.1 |
| source | Blu-ray, WEB-DL, HDTV |
| screen_size | 720p, 1080p, 2160p |
| container | mkv, mp4, avi |
| release_group | SPARKS, YTS |
| edition | Director's Cut, Extended |
| other | HDR, Remux, Proper |

## License

MIT
