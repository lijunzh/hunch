# Known Limitations

In one real-world library audit of 7,838 files, hunch achieved **99.8%
accuracy** across a mixed Anime / English / Japanese / Kids collection.
The remaining failures fall into a small number of edge-case categories
that are difficult to solve reliably with a deterministic, offline
filename parser.

These examples illustrate the main categories of remaining failures
rather than an exhaustive list of every individual filename.

## Bonus content without episode numbers

Files in bonus directories such as `Bonus/` or `特典映像/` that contain
no numeric episode marker may still be classified as `episode` with no
episode number. Hunch recognizes these directory names for title
cleanup but does not currently infer `type=extra` from directory names
alone.

```text
tv/Anime/.../特典映像/[DBD-Raws][Natsume Yuujinchou Shichi][声優トークショー][1080P][BDRip][HEVC-10bit][FLAC].mkv
  → type=episode, episode=None  (expected: type=extra)

tv/English/Power Rangers/17 - Power Rangers RPM/Bonus/Power Rangers RPM - Stuntman Behind The Scenes (Japanese).mp4
  → type=episode, episode=None  (expected: type=extra)
```

**Why this remains difficult:** directory names are useful context, but
using them alone to infer `type=extra` would require an open-ended set
of library-specific rules (`Extras/`, `Featurettes/`, `Behind the Scenes/`,
`Making Of/`, etc.), increasing regression risk across other
collections.

## Sample / preview clips

Verification clips such as `Sample1.mkv` inside `Samples/` directories
may have their digits interpreted as episode numbers.

```text
movie/.../Samples/Sample1.mkv
  → type=episode, episode=1  (expected: not real media content)
```

**Why this is low priority:** sample files are typically release
artifacts rather than meaningful library entries. Reliable detection
would require special-casing many filename and directory conventions
that vary across release groups.

## Ambiguous special / episode cross-references

Some filenames contain both special markers (`SP`) and episode markers
(`EP`), where the episode number refers to a related TV episode rather
than the file itself.

```text
movie/.../[Detective Conan][Tokuten BD][SP02][TV Series EP1080][BDRIP][1080P][H264_FLAC].mkv
  → type=episode, episode=1080  (EP1080 is a cross-reference, not this file's episode)
```

**Why this remains difficult:** distinguishing "this file is episode
1080" from "this file references episode 1080" requires semantic
understanding beyond hunch's current deterministic filename heuristics.

## Malformed filenames

Genuinely malformed inputs such as `1.The.mkv.mkv` can still produce
poor results.

**Why this is not prioritized:** hunch assumes filenames contain at
least some recoverable structure. Severely malformed input is treated
as garbage-in, garbage-out.
