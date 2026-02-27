# Action Plan: 80.5% → 85%

> **Date**: 2026-02-27
> **Current**: 80.5% (1,054 / 1,309) — 86 single-property failures
> **Target**: 85% (1,113 / 1,309) — need +59 test cases

---

## Infrastructure Status

| Component | Built | Active | Impact |
|---|---|---|---|
| Two-pass pipeline | ✅ | ✅ release_group | Eliminated 130-token DRY violation |
| TokenStream in Pass 2 | ✅ | ✅ release_group | ⚠️ 4 extractors still unused |
| BracketGroup model | ✅ | ✅ release_group | +4 compound bracket cases |
| Per-dir SegmentZone | ✅ | ✅ TOML matching | +3 Other-in-dir cases |
| Suspicious Other detection | ✅ | ✅ episode_title | +3 ep_title cases |
| Post-RG zone rules | ✅ | ✅ HQ/FanSub pruning | +6 cases |

---

## Failure Categories (86 single-property)

### Tier 1: Release Group (14 failures, ~10 fixable)

| Failure | Root Cause | Fix | Effort | Uses |
|---|---|---|---|---|
| `sc` ×2 | VideoProfile:SVC claims position | SC only after codec (done) but still fails | Needs further investigation | - |
| Parent dir: `hkd`, `immerse`, `edhd`, `megusta` ×4 | Parent dir `-GROUP` not detected for abbreviated filenames | Improve parent dir scanning with TokenStream segment info | Medium | TokenStream |
| `[SGKK]` with `[720p/MKV]` | Start bracket conflicts with end bracket containing `/` | Skip bracket groups with `/` in content | Low | BracketGroup |
| `[HorribleSubs]` between dots | Mid-filename bracket group, not at start or end | Use BracketGroup model to find brackets in tech zone | Medium | BracketGroup |
| `trollhd` with leading `[ ` | Leading space in bracket, truncated input | Handle `[ text` (space after opening bracket) | Low | BracketGroup |
| `d z0n3` space-in-group | Space within group name | Heuristic: merge last 2 dot-tokens when both unclaimed | Hard | Resolved matches |
| `yts.lt` multi-dot group | Multi-segment group name | Detect `.GROUP.SUBGROUP` at end | Medium | - |
| `episode` → got `americanbh` | Group before `[website]`, website bracket interfering | Fix RELEASE_GROUP_BEFORE_BRACKET regex | Medium | - |

### Tier 2: Episode Title (10 failures, ~5 fixable)

| Failure | Root Cause | Fix | Effort | Uses |
|---|---|---|---|---|
| Dir-based: `my perspective`, `brain in the bot`, `2000 light years` ×3 | Episode in parent dir, filename abbreviated | Extract ep_title from parent dir segment using TokenStream | High | TokenStream, Per-dir ZoneMap |
| `ménage à trois` | Unicode chars in bracket `(14-01...)` stopping extraction | Don't truncate at parens containing dates | Low | - |
| `montreux jazz festival` → cut at `festival` | `720p` follows without separator, screen_size claims nearby | Check if ScreenSize is truly adjacent | Low | Resolved matches |
| `downward dogs and proper pigs` | Other:Proper stops extraction (original text check needed) | Already partially fixed; may need value="Proper" + original_text="Proper" to be suspicious | Low | Resolved matches |
| `0.8.4.` → `.0.8.4.` | Leading dot not stripped | Strip leading separators in clean_episode_title | Low | - |
| `october 8, 2014` | Date as episode title content | Don't stop at Date when it's the only content after ep marker | Medium | - |
| `dummy 45` | Ep range `S32-Dummy 45-Ep 6478` parsing | Episode range parser consuming too much | Hard | - |

### Tier 3: Title (6 failures, ~3 fixable)

| Failure | Root Cause | Fix | Effort | Uses |
|---|---|---|---|---|
| `how to be single` → `blow-how...` | Abbreviated filename, parent dir title not used | Improve `is_abbreviated` heuristic | Medium | TokenStream |
| `01 - Ep Name` → empty | Leading episode at position 0 | Special case: episode at start + ` - ` separator | Medium | - |
| `wavey` from obfuscated filename | `e01...x264-wavey-obfuscated.mkv` — title IS the group | Detect obfuscated filenames (hash-like dir name) | Hard | TokenStream |
| `flexget apt 1` → `flexget` | `1` claimed by Part | Don't match Part for bare numbers without Part/Disc/CD prefix | Medium | - |
| `t.i.t.l.e.` trailing dot ×2 | Dot-acronym trailing dot preserved | Strip trailing dots from acronyms before non-acronym content | Low | - |

### Tier 4: Language / Subtitle Language (12 failures, ~4 fixable)

| Failure | Root Cause | Fix | Effort | Uses |
|---|---|---|---|---|
| `fr` from `Love Gourou - FR` | Short language code after ` - ` not detected | Language matching in anchor-less filenames | Medium | ZoneMap |
| `en` from dir path with `English DVD` | Language in directory not matched (FilenameOnly) | Enable Language AllSegments with dir zone filtering | Medium | Per-dir ZoneMap |
| `ca` from `(Catalan)` dir | Parenthesized language in directory | Bracket group + dir language matching | Medium | BracketGroup |
| `pt` standalone | 2-letter standalone input | Allow 2-letter language codes as standalone input | Low | - |
| `en` → `[en, sv]` | SWE Sub → both matched, should only be EN | SubtitleLanguage "Sub" suffix consuming SWE | Medium | - |

### Tier 5: Alternative Title / Bonus Title (10 failures, ~3 fixable)

| Failure | Root Cause | Fix | Effort | Uses |
|---|---|---|---|---|
| alt_title separator detection ×5 | ` - ` and `()` parsing edge cases | Enhance `find_title_boundary` for colons, multiple separators | Medium | - |
| bonus_title range/content ×5 | Bonus content extraction boundaries | Improve bonus.rs boundary detection | Medium | - |

### Tier 6: Other Properties (19 failures, ~8 fixable)

| Property | Failures | Quick Fix? |
|---|---|---|
| episode (range/multi) | 5 | Medium — E01 02 03 space-separated |
| audio_bit_rate | 3 | Low — new property (video_bit_rate split) |
| video_bit_rate | 3 | Low — new property (video_bit_rate split) |
| season (ranges, `Temp`) | 3 | Medium — `Temp.1`, `Season.1&3` |
| audio_profile | 3 | Low — TOML pattern additions |
| type (episode detection) | 2 | Medium — `x02` bonus marker as episode |
| website | 2 | Low |
| country | 2 | Low — context-dependent matching |
| mimetype | 2 | N/A — not implemented |

---

## Execution Phases

### Phase 1: Low-Effort High-ROI Fixes (~+12, reach 81.5%)

**Target**: Fix failures that need < 20 lines of code each.

1. **Leading dot in episode title** — strip leading separators in `clean_episode_title`
2. **`[SGKK]` with `/` in end bracket** — skip bracket groups containing `/`
3. **Trailing acronym dots** — strip `.` from acronyms before space
4. **`ménage à trois`** — don't truncate at parens containing digits
5. **`pt` standalone** — allow 2-letter language codes as whole input
6. **audio_profile TOML patterns** — add missing patterns
7. **`DD+` → Dolby Digital Plus** — TOML fix for `DD5.1` vs `DD+`

### Phase 2: Use Infrastructure for Release Group (~+5, reach 82.5%)

**Target**: Put BracketGroup model and TokenStream to work.

1. **Parent dir group with TokenStream** — use `token_stream.segments` for cleaner parent dir scanning instead of raw string `rfind('/')`
2. **Mid-filename bracket group** — `[HorribleSubs]` detection using `bracket_groups`
3. **`[ text` with leading space** — trim bracket content
4. **`yts.lt` multi-dot group** — detect `.A.B` pattern at end

### Phase 3: Use Infrastructure for Episode Title (~+4, reach 83%)

**Target**: Put TokenStream and per-dir ZoneMap to work.

1. **Dir-based episode title** — when filename is abbreviated, extract episode title from parent dir using `token_stream.segments`
2. **`montreux jazz festival`** — use resolved match positions to check if ScreenSize is truly adjacent or has a gap
3. **`october 8, 2014` as ep_title** — date content after episode marker

### Phase 4: Language/Subtitle in Directories (~+4, reach 83.5%)

**Target**: Leverage per-dir ZoneMap for language detection.

1. **Language AllSegments** — enable with dir zone filtering
2. **Bracket language in dirs** — `(Catalan)` in directory path
3. **Short standalone language codes** — `FR`, `pt`

### Phase 5: Episode and Season Edge Cases (~+5, reach 84%)

1. **Space-separated episodes** — `E01 02 03`
2. **Season ranges** — `Season.1&3`
3. **`x02` as episode type** — Band of Brothers bonus detection
4. **`Temp.1`** — Spanish season word

### Phase 6: Bit Rate / Audio Profile / Remaining (~+8, reach 85%)

1. **Split bit_rate into audio/video** — 3+3 cases
2. **Audio profile patterns** — TOML additions
3. **Alternative title separator** — enhanced parsing

---

## Infrastructure Backlog (No Direct Test Impact)

These are architectural improvements that don't directly fix test cases
but improve code quality and unblock future fixes:

| Item | Current State | Target | Benefit |
|---|---|---|---|
| `_token_stream` in title extractors | Wired but unused | Use `token_stream.filename_start` | DRY: eliminate `input.rfind('/')` |
| `_token_stream` in episode_title | Wired but unused | Use segments for dir-based extraction | Enables Phase 3 |
| Legacy matcher signatures | `fn(&str)` | `fn(&str, &TokenStream, &ZoneMap)` | Path-safe year/language |
| Episode title zone in ZoneMap | Not built | `ep_title_zone: Option<Range>` | Cleaner ep_title extraction |
| Bracket model for subtitle_language | Not used | Parse `{Fr-Eng}` bracket codes | Subtitle language accuracy |

---

## Decision: What to Prioritize

**Recommended order**: Phase 1 → Phase 2 → Phase 4 → Phase 3 → Phase 5 → Phase 6

Rationale:
- Phase 1 is pure surgical fixes, high ROI, low risk
- Phase 2 proves the bracket/TokenStream infrastructure on release_group
- Phase 4 proves per-dir ZoneMap on language (high ROI)
- Phase 3 is the hardest (dir-based episode title)
- Phases 5-6 are diminishing returns

**Stopping point**: 83-84% is achievable without major refactors.
85% requires episode range parsing improvements and bit_rate split,
which are significant effort.
