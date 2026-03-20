# Implementation Plan: Cross-File Context (#47) & Documentation (#48)

> Created: 2026-03-20
> Issues: #47 (feat: cross-file context), #48 (docs: design decisions)

---

## TL;DR

Add `run_with_context()` to `Pipeline` so hunch can use sibling filenames to
identify the **invariant** text (= title) across files. Start simple, YAGNI
the builder pattern and batch caching until needed.

---

## Phase 0: Prep & Refactoring (1–2 hours)

### 0.1 Extract Pass 1 into a reusable method

**File**: `src/pipeline/mod.rs`

Currently `run()` is a monolithic method (~200 lines). The first half (steps
1–4: tokenize → zone map → match → resolve → zone disambiguate) is "Pass 1"
and produces `Vec<MatchSpan>`. Extract this into:

```rust
/// Run Pass 1: tokenize + match + conflict resolve + zone disambiguate.
/// Returns (resolved_matches, token_stream, zone_map).
fn pass1(&self, input: &str) -> (Vec<MatchSpan>, TokenStream, ZoneMap)
```

Then `run()` calls `pass1()` followed by the existing Pass 2 logic. This is
a pure refactor — no behavior change, all existing tests must still pass.

**Why**: `run_with_context()` needs to run Pass 1 on the target **and** each
sibling. Without this extraction we'd duplicate ~100 lines.

### 0.2 Extract Pass 2 into a reusable method

**File**: `src/pipeline/mod.rs`

Extract steps 5a–6 (release group → title → episode title → alt titles →
media type → result) into:

```rust
/// Run Pass 2: positional extraction (title, release_group, episode_title, etc.)
/// `title_override` is an optional pre-resolved title from cross-file invariance.
fn pass2(
    &self,
    input: &str,
    matches: &mut Vec<MatchSpan>,
    zone_map: &ZoneMap,
    token_stream: &TokenStream,
    title_override: Option<&str>,
) -> HunchResult
```

When `title_override` is `Some(...)`, skip `extract_title()` and inject the
overridden title directly as a `MatchSpan` with `Property::Title`.

### 0.3 Verify refactor

```bash
cargo test
cargo test --test integration
cargo test --test guessit_regression
```

All green = commit: `refactor: extract pass1/pass2 from Pipeline::run`

---

## Phase 1: Core Algorithm — Invariance Detection (3–4 hours)

### 1.1 New module: `src/pipeline/context.rs`

**Responsibilities**:
- Accept target + siblings (all as `&str` filenames)
- Run Pass 1 on each
- Compute "unclaimed text regions" (gaps between resolved matches)
- Find longest common substring across unclaimed regions → that's the title

**Key types**:

```rust
/// A gap (unclaimed region) in a parsed filename.
#[derive(Debug, Clone)]
struct UnclaimedGap {
    /// Byte range in the original input.
    start: usize,
    end: usize,
    /// The text content of the gap (trimmed of separators).
    text: String,
}

/// Find unclaimed text gaps between resolved matches.
fn find_unclaimed_gaps(input: &str, matches: &[MatchSpan]) -> Vec<UnclaimedGap>

/// Find the longest common substring across multiple sets of unclaimed gaps.
/// Returns the common text (the invariant = title candidate).
fn find_invariant_text(all_gaps: &[Vec<UnclaimedGap>]) -> Option<String>
```

**Algorithm detail for `find_invariant_text`**:
1. Collect all gap texts from the target file
2. For each gap text in the target, check if it appears (as substring) in
   the unclaimed regions of every sibling
3. Keep the longest gap text that's present in all files
4. Normalize separators before comparison (`.` `_` `-` → space)
5. Handle CJK: no separator normalization needed (CJK has no separators
   between characters), but trim surrounding brackets/parens

**Edge cases**:
- 0 siblings → return `None` (fall back to standard title extraction)
- 1 sibling → still works (pairwise comparison)
- Mixed content (movie + extras) → invariance may be short/wrong; length
  threshold (≥2 chars) filters noise
- All files identical → entire unclaimed region is "invariant"; this is
  correct (same title)

### 1.2 `Pipeline::run_with_context()`

**File**: `src/pipeline/mod.rs`

```rust
/// Parse a filename using sibling filenames for cross-file title detection.
///
/// Siblings should be raw filenames (no directory paths). Even 1-2 siblings
/// can dramatically improve title extraction for CJK and non-standard formats.
///
/// Falls back to standard `run()` behavior when invariance detection
/// produces no result (e.g., 0 siblings or mixed content).
pub fn run_with_context(&self, input: &str, siblings: &[&str]) -> HunchResult {
    if siblings.is_empty() {
        return self.run(input);
    }

    // 1. Run Pass 1 on target + all siblings
    let (target_matches, target_ts, target_zm) = self.pass1(input);
    let sibling_results: Vec<_> = siblings
        .iter()
        .map(|s| self.pass1(s))
        .collect();

    // 2. Find unclaimed gaps in each
    let target_gaps = context::find_unclaimed_gaps(input, &target_matches);
    let sibling_gaps: Vec<_> = siblings.iter().zip(&sibling_results)
        .map(|(s, (matches, _, _))| context::find_unclaimed_gaps(s, matches))
        .collect();

    // 3. Find invariant text
    let mut all_gaps = vec![target_gaps];
    all_gaps.extend(sibling_gaps);
    let title_override = context::find_invariant_text(&all_gaps);

    // 4. Run Pass 2 with title override
    let mut matches = target_matches;
    self.pass2(input, &mut matches, &target_zm, &target_ts, title_override.as_deref())
}
```

### 1.3 Public convenience function

**File**: `src/lib.rs`

```rust
/// Parse a media filename using sibling filenames for improved title detection.
pub fn hunch_with_context(input: &str, siblings: &[&str]) -> HunchResult {
    Pipeline::default().run_with_context(input, siblings)
}
```

### 1.4 Confidence scoring

**File**: `src/hunch_result.rs`

Add a `Confidence` enum and a `confidence()` method:

```rust
/// How confident hunch is in the extracted result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Few or no properties extracted; title may be wrong.
    Low,
    /// Reasonable extraction but some ambiguity remains.
    Medium,
    /// Strong anchors found; high certainty in title and properties.
    High,
}
```

Stored as a field on `HunchResult`, set by the pipeline at the end of
`pass2()`. Heuristics:

| Signal | Effect |
|--------|--------|
| No title extracted | → Low |
| Title = entire filename (minus extension) | → Low |
| Title ≤ 2 chars | → Low |
| CJK detected + no cross-file context used | → Medium (downgrade from High) |
| Gap-based extraction used (fallback path) | → Medium |
| ≥3 tech anchors found | → High |
| Cross-file invariance used successfully | → High |

### 1.5 Tests

**File**: `tests/context.rs` (new)

Test cases from #47:

```rust
#[test]
fn cjk_fansub_cross_file_title() {
    let target = "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv";
    let siblings = &[
        "(BD)十二国記 第01話「月の影 影の海　一章」(1440x1080 x264-10bpp flac).mkv",
        "(BD)十二国記 第02話「月の影 影の海　二章」(1440x1080 x264-10bpp flac).mkv",
    ];
    let result = hunch::hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("十二国記"));
}

#[test]
fn leading_episode_number_cross_file() {
    let target = "01 - 皇太子秘史 第1集（大結局）.mkv";
    let siblings = &[
        "02 - 皇太子秘史 第2集.mkv",
        "03 - 皇太子秘史 第3集.mkv",
    ];
    let result = hunch::hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("皇太子秘史"));
}

#[test]
fn zero_siblings_falls_back() {
    let result = hunch::hunch_with_context(
        "The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv",
        &[],
    );
    assert_eq!(result.title(), Some("The Matrix"));
}

#[test]
fn western_filenames_cross_file() {
    let target = "Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv";
    let siblings = &[
        "Breaking.Bad.S05E14.720p.BluRay.x264-DEMAND.mkv",
        "Breaking.Bad.S05E15.720p.BluRay.x264-DEMAND.mkv",
    ];
    let result = hunch::hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("Breaking Bad"));
}

#[test]
fn confidence_high_with_anchors() {
    let result = hunch::hunch("Movie.2024.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(result.confidence(), Confidence::High);
}

#[test]
fn confidence_low_no_title() {
    let result = hunch::hunch(".mkv");
    assert!(result.confidence() <= Confidence::Medium);
}
```

### 1.6 Commit

`feat: cross-file context for title extraction (run_with_context) (#47)`

---

## Phase 2: CLI Integration (1–2 hours)

### 2.1 Add `--context` and `--batch` flags

**File**: `src/main.rs`

```rust
#[derive(Parser)]
struct Cli {
    /// Filename or release name to parse.
    filename: Vec<String>,

    /// Directory of sibling files to use as context for title detection.
    #[arg(long = "context", value_name = "DIR")]
    context_dir: Option<PathBuf>,

    /// Parse all media files in a directory (siblings used as mutual context).
    #[arg(long = "batch", value_name = "DIR")]
    batch_dir: Option<PathBuf>,

    // ... existing flags ...
}
```

**Behavior**:

- `hunch --context ./Season1/ "file.mkv"` → read `*.{mkv,mp4,avi,...}` from
  `./Season1/`, strip paths, pass as siblings to `run_with_context()`
- `hunch --batch ./Season1/` → for each media file in dir, use the other
  files as siblings. Output NDJSON (one JSON object per line) or pretty table.
- `--context` and `--batch` are mutually exclusive
- `--batch` with `--json` → NDJSON, without → pretty table

**Media file extensions filter**: `mkv`, `mp4`, `avi`, `wmv`, `flv`, `ts`,
`m4v`, `webm`, `ogv`, `mov`, `mpg`, `mpeg`, `m2ts`, `iso`, `img`

### 2.2 Low-confidence warning

When confidence is `Low` and neither `--context` nor `--batch` was used:

```
⚠ Low confidence result. Try: hunch --context . "file.mkv"
  (sibling files can improve title detection)
```

Suppressed by `--json` (machine output should be clean) and `--quiet` (TBD).

### 2.3 Tests

Manual CLI testing + a few integration tests that invoke the binary:

```rust
#[test]
fn cli_batch_mode() {
    // Create temp dir with test files, run `hunch --batch <dir> --json`
    // Assert NDJSON output with correct titles
}
```

### 2.4 Commit

`feat: CLI --context and --batch flags for cross-file parsing (#47)`

---

## Phase 3: Documentation (#48) (1 hour)

### 3.1 Update `ARCHITECTURE.md`

Add new section after "Layered Architecture":

```markdown
## Cross-File Context (v0.3.x)

### Design Principle

The title is the **invariant text** across sibling files. Episode numbers,
episode titles, and per-file metadata are the **variant** text.

### API Surface

- **Library**: `Pipeline::run_with_context(input, siblings)` — caller
  provides sibling filenames. The library NEVER does filesystem I/O.
- **CLI**: `--context <dir>` and `--batch <dir>` — CLI reads the filesystem.
- **Confidence**: `HunchResult::confidence()` → `High | Medium | Low`

### Hard Boundary (unchanged)

The library remains a **pure, offline, deterministic** function. Cross-file
context is caller-provided data, not filesystem access.
```

### 3.2 Update `README.md`

Add cross-file context example to Quick Start:

```rust
// When you have sibling files, use them for better title detection:
let result = hunch::hunch_with_context(
    "(BD)十二国記 第13話「月の影 影の海　終章」(1440x1080 x264-10bpp flac).mkv",
    &[
        "(BD)十二国記 第01話「月の影 影の海　一章」(1440x1080 x264-10bpp flac).mkv",
        "(BD)十二国記 第02話「月の影 影の海　二章」(1440x1080 x264-10bpp flac).mkv",
    ],
);
assert_eq!(result.title(), Some("十二国記"));
```

Add CLI examples:

```bash
# Single file with sibling context
hunch --context ./Season1/ "(BD)十二国記 第13話....mkv"

# Batch mode: parse all files in a directory
hunch --batch ./Season1/ --json
```

### 3.3 Update `CHANGELOG.md`

Add entry under new version.

### 3.4 Commit

`docs: document cross-file context API design decisions (#48)`

---

## Phase 4: Auto-Context & Performance (Future — NOT in scope now)

Deferred per YAGNI. Tracked here for reference:

- [ ] `--auto-context` CLI flag (re-run with context when confidence is low)
- [ ] `HunchContext` builder pattern (for advanced configuration)
- [ ] `Pipeline::batch()` with cached Pass 1 results
- [ ] Sibling sampling (use N random siblings, not all 500)
- [ ] Mixed-content clustering (group by common prefix before invariance)

---

## File Impact Summary

| File | Change | Lines (est) |
|------|--------|-------------|
| `src/pipeline/mod.rs` | Extract `pass1()`/`pass2()`, add `run_with_context()` | +60, -0 (refactor) |
| `src/pipeline/context.rs` | **NEW** — invariance detection | ~150 |
| `src/hunch_result.rs` | Add `Confidence` enum + field + accessor | ~40 |
| `src/lib.rs` | Add `hunch_with_context()` + re-export `Confidence` | ~10 |
| `src/main.rs` | Add `--context`, `--batch` flags + warning | ~80 |
| `tests/context.rs` | **NEW** — cross-file context tests | ~100 |
| `ARCHITECTURE.md` | Cross-file context section | ~40 |
| `README.md` | Examples | ~30 |
| `CHANGELOG.md` | New entry | ~10 |

**Total new code**: ~520 lines across 9 files. No file exceeds 600 lines.

---

## Execution Order & Dependencies

```
Phase 0 (refactor)  ──→  Phase 1 (core algo)  ──→  Phase 2 (CLI)
                                                       │
                                                       ├──→  Phase 3 (docs)
                                                       │
                                                       └──→  Phase 4 (future, deferred)
```

Phases 0–3 are sequential. Total estimated effort: **6–9 hours**.

---

## Open Decisions (to resolve during implementation)

1. **Confidence enum vs float**: Plan uses `High/Medium/Low` enum (simpler,
   more actionable). Revisit if we need finer granularity.
2. **`--batch` output format**: NDJSON for `--json`, pretty table otherwise.
   Could add `--format {ndjson,table,csv}` later.
3. **Minimum invariant length**: Propose ≥2 Unicode grapheme clusters to
   filter noise. May need tuning with real-world CJK data.
4. **Separator normalization**: Normalize `. _ -` → space before comparison?
   Or compare raw? Leaning toward normalized for robustness.
5. **Title override vs title hint**: Should `title_override` completely
   replace `extract_title()`, or should it be a "hint" that
   `extract_title()` can refine? Start with full override, iterate.
