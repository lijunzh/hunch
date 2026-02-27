# Architecture Review: Hard Challenges

> **Date**: 2026-02-26  
> **Scope**: release_group, episode_title, title edge cases, and other high-ROI targets  
> **Current**: 79.0% (1,034 / 1,309) guessit compatibility

---

## Executive Summary

The low-hanging fruit is done. What remains are **structurally hard problems**
where the current architecture hits fundamental limits. This review categorizes
every remaining failure, identifies root causes, and proposes three architectural
options ranging from surgical fixes to a deeper refactor.

**Biggest ROI targets by single-property failures:**

| Property | Single-Prop Fails | Total Fails | Current Rate | Notes |
|---|---|---|---|---|
| release_group | 19 | 28 | 89.1% | Diverse failure modes |
| episode_title | 13 | 56 | 72.1% | Boundary detection |
| title | 11 | 115 | 90.1% | Edge cases, language in title |
| subtitle_language | 6 | 19 | 76.5% | Bracket code parsing |
| language | 5 | varies | 84.5% | Context-dependent |
| alternative_title | 5 | varies | 43.8% | Separator detection |

---

## Part 1: Release Group Failure Taxonomy

28 total failures, clustered into **9 distinct failure modes**:

### Mode 1: Compound Bracket Groups (4 failures)

**Pattern**: `(GroupA) [GroupB]` — two bracket sections that should merge.

```
Archer... (1080p AMZN Webrip x265 10bit EAC3 5.1 - JBENT)[TAoE]
  expected: "jbent taoe",  got: "taoe"

Dark Phoenix... (1080p BluRay x265 HEVC 10bit AAC 7.1 Tigole) [QxR]
  expected: "tigole qxr",  got: "qxr"

The Peripheral... (1080p AMZN WEB-DL x265 HEVC 10bit DDP5.1 D0ct0rLew) [SEV]
  expected: "d0ct0rlew sev",  got: "sev"

True Detective S02E04 720p HDTV x264-0SEC [GloDLS].mkv
  expected: "0sec [glodls]",  got: "0sec"
```

**Root cause**: Current regex captures `-GROUP` OR `[GROUP]` independently.
No logic merges adjacent bracket groups. The `(Tigole) [QxR]` pattern has
the first group inside parens (not after `-`), so only `[QxR]` matches.

**Fix complexity**: Medium. Need bracket-pair scanning + adjacency merging.

### Mode 2: `-by.Group[Suffix]` Pattern (2 failures)

```
Some.Title.XViD-by.Artik[SEDG].avi
  expected: "artik[sedg]",  got: "sedg"

El.dia.de.la.bestia.DVDrip.Spanish.DivX-by.Artik[SEDG].avi
  expected: "artik[sedg]",  got: "sedg"
```

**Root cause**: The `-by.` prefix is not handled. `RELEASE_GROUP_END` matches
`-by` as the group (which gets filtered by `is_known_token`?), then
`RELEASE_GROUP_END_BRACKET` captures just `[SEDG]`. The `by.Artik` compound
is never detected.

**Fix complexity**: Low. Add a regex pattern for `-by.GROUP[SUFFIX]`.

### Mode 3: Space-Separated Group Names (1 failure)

```
The.Kings.Speech.2010.1080p.BluRay.DTS.x264.D Z0N3.mkv
  expected: "d z0n3",  got: "z0n3"
```

**Root cause**: `RELEASE_GROUP_SPACE_END` captures only the last space-delimited
word. The group name contains a space (`D Z0N3`). No logic handles multi-word
space-separated groups.

**Fix complexity**: Medium. Need heuristic for "is this preceding word part of
the group?" — tricky because tech tokens also precede groups.

### Mode 4: Bracket Groups Not Detected (4 failures)

```
[SGKK] Bleach 312v1 [720p/MKV]
  expected: "sgkk",  got: ""

Show!.Name.2.-.10.(2016).[HorribleSubs][WEBRip]..[HD.720p]
  expected: "horriblesubs",  got: ""

[ Engineering Catastrophes S02E10 1080p AMZN WEB-DL DD+ 2.0 x264-TrollHD
  expected: "trollhd",  got: ""

Second.Chance.S01E02.One.More.Notch.1080p.WEB-DL.DD5.1.H264-SC[rartv]
  expected: "rartv",  got: "sc[rartv]"
```

**Root causes** (multiple):
- `[SGKK]`: start bracket logic works, but `[720p/MKV]` at end confuses
  the end-bracket regex (or `720p` is known, so end bracket is empty, and
  start bracket fallback doesn't fire because... the end has brackets too?)
- `[HorribleSubs]`: it's between dots and followed by `[WEBRip]` — the
  regex doesn't handle `.[GROUP][TECH]..` patterns.
- `[ Engineering...TrollHD`: leading `[ ` with a space, and the actual group
  is `-TrollHD` at the end, but the truncated input starts with `[` so
  start-bracket regex fires first (and fails because of the space).
- `SC[rartv]`: Regex matches `-SC[rartv]` as a single group with suffix.
  Expected behavior: `rartv` is the indexer tag, `SC` is the actual group.
  But guessit expects `rartv` as the group — this is an indexer-tag semantic.

**Fix complexity**: High. Multiple sub-patterns, each with different logic.

### Mode 5: Group From Parent Directory (3 failures)

```
Zoo.S02E05.1080p.WEB-DL.DD5.1.H.264.HKD/160725_02.mkv
  expected: "hkd",  got: ""

c:\Temp\...\2 Broke Girls-IMMERSE\file.mkv
  expected: "immerse",  got: ""

wwiis.most.daring.raids.s01e04.storming.mussolinis.island.1080p.web...
  expected: "edhd",  got: ""
```

**Root cause**: Step 8 (parent directory scan) works for simple cases but
fails when:
- Parent dir has tech tokens embedded (`.H.264.HKD` — H.264 is matched as
  a multi-token, leaving HKD orphaned)
- Windows paths with backslash separators
- Complex parent dir names

**Fix complexity**: Medium. Improve parent-dir `-GROUP` detection heuristic.

### Mode 6: Short Groups / Last-Token (3 failures)

```
Show.Name.S01E03.HDTV.Subtitulado.Esp.SC
  expected: "sc",  got: ""

Show.Name.S01E03.HDTV.Subtitulado.Espanol.SC
  expected: "sc",  got: ""

Vita.&.Virginia.2018.720p.H.264.YTS.LT.mp4
  expected: "yts.lt",  got: ""
```

**Root cause**:
- `SC` (2 chars): `RELEASE_GROUP_SPACE_END` has `min length >= 3` filter.
  There's no `-` before it, just a dot. `RELEASE_GROUP_LAST_DOT` matches
  but... `SC` is only 2 chars so `[A-Za-z][A-Za-z0-9]{2,15}` rejects it.
- `YTS.LT`: Multi-dot group name. No pattern handles `GROUP.SUBGROUP` as
  a single release group.

**Fix complexity**: Low-Medium. Relax length filters, add multi-dot pattern.

### Mode 7: Known-Token False Positive (2 failures)

```
American.The.Bill.Hicks.Story.2009.DVDRip.XviD-EPiSODE.[UsaBit...]
  expected: "episode",  got: "americanbh"

Movies/9 (2009)/9.2009.Blu-ray.DTS.720p.x264.HDBRiSe.[sharethefiles...]
  expected: "hdbrise",  got: ""
```

**Root cause**: `EPiSODE` is both a valid group name AND matches as an
episode-related keyword? No — actually `is_known_token` doesn't include
"episode". Let me check... the issue is likely that the regex pattern
for `RELEASE_GROUP_BEFORE_BRACKET` or `RELEASE_GROUP_END` isn't matching
because the `[UsaBit...]` website bracket is interfering.

For `HDBRiSe.[sharethefiles.com]`: The `-GROUP_BEFORE_BRACKET` regex should
match `-HDBRiSe.[ ` but the `.` between group and `[` may not be handled.
Actually looking at the regex: `-(?P<group>[A-Za-z0-9@µ!]+)\s*\.?\s*\[` —
this should match `HDBRiSe.[`. But the issue might be that there's no `-`
before `HDBRiSe` — it's `x264.HDBRiSe.[`, separated by dot not dash.

**Fix complexity**: Medium. Need dot-separated group-before-bracket pattern.

### Mode 8: Priority Conflicts (2 failures)

```
[XCT].Le.Prestige.(The.Prestige).DVDRip.[x264.HP.He-Aac.{Fr-Eng}...]-CHAPS
  expected: "chaps",  got: "xct"

[XCT] Persepolis [H264+Aac-128(Fr-Eng)+ST(Fr-Eng)]-IND
  expected: "ind",  got: "xct"
```

**Root cause**: When both `[GROUP]` at start AND `-GROUP` at end exist,
current logic picks start bracket first (Step 5 runs before Step 1 would
have a chance... actually Step 1 runs first). Looking at the code: Step 1
(`RELEASE_GROUP_END`) should match `-CHAPS` or `-IND`. If it does, Steps
2-5 are skipped. So why isn't `-CHAPS` matching?

The issue is the complex bracket content before `-CHAPS`. The full filename
ends with `...}-CHAPS` — the `}` is not handled by the end regex which
expects `[a-z0-9]{2,5}` file extension or end-of-string.

**Fix complexity**: Low. Adjust `RELEASE_GROUP_END` to handle `}` before `-`.

### Mode 9: Compound Group in Parent + Multi-Token (2 failures)

```
Show Name S01e10[Mux - 1080p - H264 - Ita Eng Ac3 - Sub Ita Eng]DLMux GiuseppeTnT LittleLinx
  expected: "giuseppetnt littlelinx",  got: "littlelinx"

La.Science.Des.Reves.FRENCH.DVDRip.x264.AC3.mp-acebot.mkv
  expected: "acebot",  got: "mp-acebot"
```

**Root cause**:
- Multi-space groups: `GiuseppeTnT LittleLinx` — two space-separated words
  that are both part of the group name.
- `mp-acebot`: The `-acebot` should be the group but `mp-` prefix is included.
  `expand_group_backwards` includes `mp` because it doesn't recognize it as
  a tech token.

**Fix complexity**: Medium-High.

---

## Part 2: Episode Title Failure Taxonomy

13 single-property failures, 56 total.

### Key Patterns

| Failure | Root Cause |
|---|---|
| Episode title includes `Part N` | Part property stops extraction but expected to be included |
| No episode title extracted | Last ep match too close to tech tokens (no gap) |
| Episode title cut short | `Other` or `Language` match in the middle of title text |
| Date-based episodes | `The Soup - 11x41 - October 8, 2014` — date IS the ep title |
| Dir-based episodes | Episode in dir path, title in filename — boundary confusion |
| `Proper` in title | `Proper Pigs` — `Proper` is claimed by Other, cutting title |

### Core Issue

Episode title extraction runs AFTER conflict resolution, so tokens claimed
by `Other` ("Proper") or `Language` are already consumed. The episode title
extractor sees a gap that's been fragmented by these matches.

This is the same structural problem as the title extractor ("match-then-prune"
loses information), but for the zone between episodes and tech properties.

---

## Part 3: Title Edge Cases

11 single-property failures.

| Input | Expected | Got | Issue |
|---|---|---|---|
| `h265 - HEVC Riddick...` | `riddick` | `` | Leading codec eats title zone |
| `Immersion.French.2011...` | `immersion french` | `immersion` | "French" → Language in title |
| `blow-how.to.be.single...` | `how to be single` | `blow-how to be...` | Parent dir `blow-` prefix |
| `Pacific.Rim.3D.2013...` | `pacific rim` | `pacific rim 3d` | 3D in title zone |
| `Pirates.de.langkasuka.2008...1920X1080...` | year=2008 | year missing | 1920x1080 screen_size misparse |
| `FlexGet.Apt.1` | `flexget apt 1` | `flexget` | `1` parsed as episode |
| `01 - Ep Name` | `ep name` | `` | Leading episode number, title after |

### Core Issue

Title extraction uses "first non-extension match" as its boundary. When the
first match is wrong (false positive Language, 3D as Other, etc.), the title
gets corrupted. ZoneMap helps for many cases but these edge cases slip through.

---

## Part 4: Architectural Options

### Option A: Surgical Fixes (Low Risk, ~+3-5%)

**Philosophy**: Fix each failure mode individually without architectural change.
Keep the existing pipeline, add regex patterns, tune heuristics.

**Release Group fixes:**
1. Add `-by.GROUP[SUFFIX]` regex pattern → fixes Mode 2 (2 cases)
2. Relax min-length filter from 3→2 for `RELEASE_GROUP_LAST_DOT` → fixes Mode 6 partly
3. Add dot-separated group-before-bracket (`GROUP.[website]`) → fixes Mode 7
4. Adjust `RELEASE_GROUP_END` to handle `}` before dash → fixes Mode 8 partly
5. Add `mp` to `is_known_token` → fixes `mp-acebot`
6. Improve `expand_group_backwards` to avoid `mp-` inclusion

**Episode title fixes:**
1. Don't stop at `Other:Proper` when followed by a non-tech word
2. Allow date content as episode title when it follows episode markers

**Title fixes:**
1. When title zone is empty and input starts with codec, skip to next gap
2. Keep "French" in title when no other language exists in tech zone

**Pros:**
- Low risk, each fix is a single commit
- No pipeline changes
- Testable independently

**Cons:**
- Doesn't fix compound bracket groups (4 failures)
- Doesn't fix multi-word space groups
- Each fix is ad-hoc, may introduce new regressions
- `is_known_token` list grows forever (DRY violation)

**Estimated improvement**: +30-40 test cases (3-4%)

---

### Option B: Release Group Restructure (Medium Risk, ~+5-7%)

**Philosophy**: Rewrite `release_group.rs` from regex soup into a structured
extractor with distinct phases. Keep the pipeline position (pre-resolution),
but use the ZoneMap and TokenStream for better context.

**New release_group architecture:**

```
release_group::find_matches(input, zone_map, token_stream) -> Vec<MatchSpan>
  │
  ├─ Phase 1: Candidate Detection (regex-based, produces candidates)
  │   ├─ Detect `-GROUP` at end of filename
  │   ├─ Detect `-GROUP[SUFFIX]` and `-by.GROUP[SUFFIX]`
  │   ├─ Detect `-GROUP.[website]` (before bracket)
  │   ├─ Detect `-[GROUP]` (dash-bracket at end)
  │   ├─ Detect `[GROUP]` at start (anime)
  │   ├─ Detect `[GROUP]` at end
  │   ├─ Detect `(GROUP) [GROUP2]` (compound paren+bracket)
  │   ├─ Detect space-separated last token
  │   └─ Detect from parent directory
  │
  ├─ Phase 2: Validation (structural, uses ZoneMap + tokens)
  │   ├─ Reject candidates that overlap with tech_zone anchors
  │   ├─ Reject candidates that are known tech vocabulary (Tier 2)
  │   ├─ Reject CRC32 hex values
  │   └─ Prefer dash-groups over bracket-groups (priority)
  │
  └─ Phase 3: Compound Merging
      ├─ Merge adjacent bracket groups: (A) [B] → "A B"
      ├─ Merge `-GROUP [SUFFIX]` → "GROUP [SUFFIX]"
      └─ Handle `D Z0N3` space-within-group cases
```

**Key changes:**
1. **Pass ZoneMap + TokenStream to release_group** — currently it only gets
   raw input. With zone awareness, it can use `tech_zone_start` instead of
   `is_known_token` for many cases.
2. **Replace `is_known_token`** partly with ZoneMap Tier 2 token check.
   Keep a small curated list for compound tokens only.
3. **Add compound bracket merging** — scan for adjacent `(...)` and `[...]`
   sections in the release zone.
4. **Make release_group a method on Pipeline** instead of a standalone
   function, so it can access the TOML rule sets for validation.

**Pros:**
- Fixes compound bracket groups (4 failures)
- Fixes most bracket detection issues
- Reduces `is_known_token` DRY violation
- Structured phases are easier to debug
- Uses ZoneMap investment

**Cons:**
- Medium refactor risk
- release_group signature change breaks the `LegacyMatcherFn` pattern
- Still pre-resolution, so can't use resolved match positions
- Need to update pipeline to pass extra args

**Estimated improvement**: +45-55 test cases (4-5%), mostly release_group + some title/ep_title cascade

---

### Option C: Post-Resolution Release Group + Two-Pass Pipeline (Higher Risk, ~+7-10%)

**Philosophy**: Split the pipeline into two passes. First pass resolves all
tech properties. Second pass extracts positional properties (title, episode_title,
release_group) using the resolved match positions.

**New pipeline:**

```
Input
  │
  ├─ 1. Tokenize
  ├─ 2. Build ZoneMap
  │
  ├─ === PASS 1: Tech Property Resolution ===
  ├─ 3a. TOML rules (zone-aware)
  ├─ 3b. Legacy matchers (year, date, episodes, language, etc.)
  ├─ 3c. Extension → Container
  ├─ 4. Conflict resolution
  ├─ 5. Zone disambiguation
  │   └─ Output: resolved_tech_matches: Vec<MatchSpan>
  │
  ├─ === PASS 2: Positional Property Extraction ===
  ├─ 6a. release_group(input, zone_map, resolved_tech_matches)
  │      └─ Uses resolved positions to know which tokens are claimed
  ├─ 6b. title(input, all_matches, zone_map)  [unchanged]
  ├─ 6c. episode_title(input, all_matches)  [unchanged]
  ├─ 6d. alternative_title(input, all_matches)  [unchanged]
  │
  └─ 7. Build HunchResult
```

**Release group in Pass 2:**

```rust
pub fn find_matches(
    input: &str,
    zone_map: &ZoneMap,
    resolved_matches: &[MatchSpan],
) -> Vec<MatchSpan> {
    // "Unclaimed" = not overlapping any resolved match.
    // Walk backwards from end of filename, find the last unclaimed
    // token after a `-` separator.
    // Also check bracket groups.
}
```

**Key insight**: The `is_known_token` list exists because release_group
can't see what other matchers have claimed. In Pass 2, it CAN see — a
token claimed as `VideoCodec:H.264` is definitively not a release group.
No exclusion list needed.

**Handling the edge cases from the reverted v0.2.1 attempt:**

The ARCHITECTURE.md documents why this failed before:

| Problem | v0.2.1 Failure | Option C Solution |
|---|---|---|
| Compound tokens (`XviD-GROUP`) | Position overlap ambiguous | Tokenizer splits at `-`; XviD and GROUP are separate tokens |
| Trailing metadata stripping | Position-based too aggressive | Keep word-level META_TOKENS list for trailing strip only |
| `expand_group_backwards` | Needs absolute positions | Token positions are absolute; overlap check is clean |

**Pros:**
- Eliminates `is_known_token` entirely (DRY win)
- Release group extraction is provably correct (uses ground truth)
- Fixes most failure modes naturally
- Episode title can also benefit from Pass 2 awareness
- Clean separation of concerns: tech → positional

**Cons:**
- Biggest refactor — pipeline restructure
- Risk of regression during transition
- Need to handle edge cases where resolved matches are wrong
  (e.g., false positive Source match consuming a group name token)
- Two passes means title extraction sees all matches including
  release group (ordering dependency)

**Estimated improvement**: +60-80 test cases (5-7%)

---

## Part 5: Other Hard Challenges (Non-Release-Group)

### Subtitle Language (76.5% → target 85%+)

**Core issues:**
- Bracket codes like `{Fr-Eng}`, `(Fr-Eng)` not parsed
- `+ST(Fr-Eng)` pattern (subtitle prefix + bracket languages)
- Conflict with Language matches (same tokens, different semantics)

**Approach**: Dedicated bracket-language parser that scans `{...}` and `(...)`
for ISO language codes, qualified by preceding `ST`/`Sub`/`Subs` keywords.
This is algorithmic, not TOML.

### Episode Title (72.1% → target 80%+)

**Core issues (beyond release_group cascade):**
- `Other:Proper` in "Proper Pigs" cuts the episode title
- Date-as-episode-title ("October 8, 2014")
- Episode title after Part marker
- Dir-based episode titles not extracted

**Approach**: Episode title needs "negative space" awareness — it's the gap
between episode markers and tech properties, but must skip over false-positive
matches within that gap. Option C (two-pass) helps here: in Pass 2, we can
check if an `Other` match in the episode title zone is suspiciously surrounded
by non-tech words.

### Alternative Title (43.8%)

**Core issues:**
- `Star Wars: Episode IV - A New Hope` — " - " separates title from alt title
- `Echec et Mort - Hard to Kill - Steven Seagal` — multiple " - " separators

**Approach**: Enhanced separator parsing in `find_title_boundary`. Currently
detects the first " - " or "(" — needs to also handle colons and multiple
separators.

### Country (69.2%)

**Core issues:**
- Short codes (`US`, `GB`) false-positive in title zone
- `NZ` case-sensitive match not triggering

**Approach**: Country already has `zone_scope` in TOML. Issues are mostly
ambiguous short tokens. ZoneMap filtering helps but some cases need
contextual awareness (country near year → likely metadata).

---

## Part 6: Recommendation

### Recommended: Option B first, then evolve toward Option C

**Phase 1 (Option B): Release Group Restructure** (1-2 weeks)

1. Restructure `release_group.rs` into phased extraction
2. Pass ZoneMap to release_group (break LegacyMatcherFn signature)
3. Add compound bracket merging
4. Fix short-group, `-by.GROUP`, and bracket-detection patterns
5. Reduce `is_known_token` using ZoneMap Tier 2 tokens

**Phase 2 (Surgical): Episode Title + Title fixes** (1 week)

1. Fix episode title boundary for `Other:Proper` in title words
2. Handle leading-codec title extraction
3. Keep language in title zone when it's a title word ("French" in
   "Immersion French")

**Phase 3 (Option C evolution): Two-Pass Pipeline** (2-3 weeks)

1. Extract Pass 1 (tech resolution) into its own method
2. Move release_group to Pass 2 with resolved match access
3. Move episode_title to benefit from Pass 2 awareness
4. Delete `is_known_token` entirely

**Why not Option C immediately?**

Option C is the right end-state but the refactor risk is high. Option B
gives us most of the release_group wins with lower risk, and the structured
phases in Option B naturally evolve into Option C. We don't throw away work.

**Why not Option A?**

Option A can't fix compound bracket groups (4 failures) or the DRY violation
of `is_known_token`. Those are structural problems requiring structural
solutions.

---

## Part 7: Specific Fix List (Ordered by ROI)

Regardless of which option we choose, these specific fixes have the highest
ROI and lowest risk:

### Tier 1: Quick Wins (each fixes 1-3 test cases)

| # | Fix | Failures Fixed | Risk |
|---|---|---|---|
| 1 | Add `-by.GROUP[SUFFIX]` pattern | 2 (Artik[SEDG]) | Low |
| 2 | Relax min-length to 2 for last-dot groups | 2 (SC) | Low |
| 3 | Add `mp` to is_known_token | 1 (mp-acebot) | Low |
| 4 | Handle `}` before `-GROUP` in RELEASE_GROUP_END | 2 ([XCT]..-CHAPS) | Low |
| 5 | Add dot-separated group-before-bracket | 2 (HDBRiSe.[site]) | Low |
| 6 | Handle `SC[rartv]` → prefer `[rartv]` as indexer | 1 | Low |
| 7 | YTS.LT multi-dot group pattern | 1 | Low |

### Tier 2: Medium Effort (each fixes 2-5 test cases)

| # | Fix | Failures Fixed | Risk |
|---|---|---|---|
| 8 | Compound bracket merging `(A) [B]` | 4 | Medium |
| 9 | Improve parent-dir group detection | 3 | Medium |
| 10 | Episode title: don't stop at word-embedded Other | 3 | Medium |
| 11 | Title: leading codec → skip to next gap | 1-2 | Medium |
| 12 | Title: keep language in title when title-like | 2-3 | Medium |
| 13 | Space-in-group: `D Z0N3` heuristic | 1 | Medium |

### Tier 3: Structural Changes (broad impact)

| # | Fix | Failures Fixed | Risk |
|---|---|---|---|
| 14 | Pass ZoneMap to release_group | 5-10 (indirect) | Medium |
| 15 | Two-pass pipeline | 10-20 (broad) | High |
| 16 | Bracket-language parser for subtitle_language | 5-8 | Medium |
| 17 | Alternative title separator enhancement | 3-5 | Medium |

---

## Appendix: Full Failure Inventory

See `cargo test compatibility_report -- --ignored --nocapture` for the
complete list. The 99 single-property failures break down as:

```
release_group      19  ████████████████████
episode_title      13  █████████████
title              11  ███████████
subtitle_language   6  ██████
language            5  █████
alternative_title   5  █████
episode             5  █████
bonus_title         5  █████
other               4  ████
audio_profile       3  ███
video_bit_rate      3  ███
type                3  ███
season              2  ██
screen_size         2  ██
remaining          13  (various, 1-2 each)
```
