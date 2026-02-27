# Hunch v0.3.1 — Language/Subtitle Fixes & Detection Improvements

A **patch release** focused on language and subtitle_language detection
accuracy, plus a batch of parser fixes accumulated since v0.3.0.

## Highlights

- **language.yml pass rate: 66.7% → 100%** — All language fixture tests
  now pass, up from 6/9 to 9/9.
- **Zone Rule 8** — New disambiguation rule: Language matches contained
  within SubtitleLanguage spans are automatically suppressed. This fixes
  false positives where tokens like `FR` in `FR Sub` were detected as
  both an audio language and a subtitle indicator.
- **Tighter bracket subtitle parsing** — `St{Fr-Eng}` patterns now
  correctly extract both languages instead of greedily matching past
  the closing bracket.

## Key Fixes

| Fix | Before | After |
|-----|--------|-------|
| `ENG.-.FR Sub` language | `[en, fr]` | `en` |
| `ENG.-.SWE Sub` language | `[en, sv]` | `en` |
| `St{Fr-Eng}` subtitle_language | `fr` | `[fr, en]` |
| language.yml pass rate | 66.7% | **100%** |

## Other Improvements Since v0.3.0

- Enable Language TOML rules in directory segments
- Add LC-AAC audio profile pattern
- Detect space-separated zero-padded episode numbers
- Recognize `Temp` as Spanish season keyword (Temporada)
- Bonus without film/year implies episode type
- Add `pt` ISO 639-1 code for Portuguese
- Merge multi-dot release group names (e.g., `YTS.LT`)
- Detect release groups in mid-filename bracket groups
- Don't truncate episode title at parentheses with digit content
- Per-directory Other rules with zone filtering
- Compound bracket group tokenizer improvements

## Install

```bash
cargo install hunch
# or
cargo add hunch
```

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete list of changes.
