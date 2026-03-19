# Hunch v1.1.3 — Structure-Aware Disambiguation

## Highlights

- **82.2% guessit compatibility** (1,076 / 1,309) — up from 81.7%.
- **Neighbor-context disambiguation** replaces fragile positional heuristics
  for language and source classification. Tokens are now classified based on
  what actually surrounds them, not where they sit in the filename.
- **Structure-aware episode title extraction** from parent directories.
- **TOML-driven disambiguation** with `requires_nearby` and `reclaimable`.

## What Changed

### Structure-aware token context (new)

Language and source disambiguation now uses three principled signals instead
of fragile positional heuristics:

1. **Neighbor roles** — Are adjacent tokens title words or tech tokens?
2. **Peer reinforcement** — Adjacent language tokens (FRENCH.ENGLISH) signal
   a metadata cluster, not title content.
3. **Structural separators** — Tokens after " - " or in brackets are metadata.

### Episode title from parent directories

```
Bones.S12E02.The.Brain.In.The.Bot.1080p.WEB-DL/161219_06.mkv
→ episode_title: "The Brain In The Bot" (extracted from parent dir)
```

### Removed dead code

`Options` struct, `hunch_with()`, `--type`/`--name-only` CLI flags were dead
code from v1.0.0 (never wired into the pipeline). Removed.

## Migration

If you used `hunch_with()` or `Options`, replace with plain `hunch()` —
the behavior is identical (Options was always ignored).

## Install / Upgrade

```bash
brew upgrade hunch
cargo install hunch
cargo add hunch@1.1.3
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete history.
