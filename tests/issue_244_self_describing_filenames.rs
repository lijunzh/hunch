//! Regression tests for the #246 follow-up identified by post-merge
//! review: in `--batch -r` against a multi-show directory, files in
//! sub-directories with their own bracket-style titles were getting
//! clobbered by sibling-consensus / path-prefix invariance from the
//! parent directory.
//!
//! Pre-fix shape:
//!   ShowS2/[...][Re Zero ... S2][01].mkv      → title "Re Zero ..."  (correct)
//!   ShowS2/mini-anims/[...][Otto's Diary][01].mkv
//!     → title "Re Zero ..."  ❌ clobbered by parent's cached title
//!
//! Two distinct mechanisms could clobber the title:
//!   1. Ancestor fallback override — parent's cached title was passed
//!      as `fallback_title` and used as an authoritative override even
//!      when the file's own bracket extraction would succeed.
//!   2. Path-prefix invariance — invariance found a normalized form
//!      of the parent dir name (`hunch_show2` → "hunch show") as the
//!      common-text title across siblings; `is_path_dir_name` missed
//!      the normalization so it was treated as a legitimate invariance
//!      title.
//!
//! Fix: when the FILENAME itself has bracket structure (the dominant
//! fansub convention for explicit titles), demote both invariance and
//! ancestor fallback from override to last-resort. The file's own
//! extraction speaks first; cross-file signals only fire if extraction
//! returns nothing.
//!
//! Files WITHOUT brackets in their filename (e.g. `Special.720p.mkv`,
//! `Making.Of.720p.mkv`) keep the legacy behavior: ancestor fallback
//! overrides extraction (covered by `tests/parent_context.rs`).

use hunch::Pipeline;

// ── 1. The exact regression from the PR review ────────────────────────

#[test]
fn mini_anim_subdir_keeps_distinct_title() {
    // Setup mirrors the user-reported failing scenario:
    //   batch root has `[...][Main Show S2][NN]` files (consensus title)
    //   subdir `迷你动画/` has `[...][Mini Anim X][NN]` files (distinct titles)
    //
    // The mini-anim file should keep its OWN bracket title, not inherit
    // the main show's title via parent-context fallback.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "迷你动画/[DBD-Raws][Re Zero Kara Hajimeru Break Time Maid's Days][01][1080P][BDRip][HEVC-10bit][FLAC].mkv",
        "迷你动画/[DBD-Raws][Re Zero Kara Hajimeru Break Time Otto's Diary][02][1080P][BDRip][HEVC-10bit][FLAC].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "迷你动画/[DBD-Raws][Re Zero Kara Hajimeru Break Time Otto's Diary][01][1080P][BDRip][HEVC-10bit][FLAC].mkv",
        &siblings,
        Some("Re Zero kara Hajimeru Isekai Seikatsu"), // parent's cached title
    );
    assert_eq!(
        result.title(),
        Some("Re Zero Kara Hajimeru Break Time Otto's Diary"),
        "file's own bracket title must win over parent fallback when filename is self-describing"
    );
}

#[test]
fn distinct_bracket_titles_in_one_dir_each_keep_own_title() {
    // Multiple files in the same dir, each with a DIFFERENT bracket title.
    // Sibling-consensus must NOT collapse them to a single common title.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "shorts/[Group][Show A][01][1080p].mkv",
        "shorts/[Group][Show B][01][1080p].mkv",
        "shorts/[Group][Show C][01][1080p].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "shorts/[Group][Show D][01][1080p].mkv",
        &siblings,
        Some("Inherited Wrong Title"),
    );
    assert_eq!(result.title(), Some("Show D"));
}

// ── 2. Path-prefix invariance no longer clobbers self-describing files ─

#[test]
fn path_prefix_normalization_does_not_become_title() {
    // When the batch-root dir name normalizes to something invariance
    // treats as common (`hunch_show2` → "hunch show"), and that
    // normalized form doesn't match `is_path_dir_name`'s exact check,
    // the file's bracket title must STILL win because the filename is
    // self-describing.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "hunch_show2/[Group][Real Show Name S2][02][1080p].mkv",
        "hunch_show2/[Group][Real Show Name S2][03][1080p].mkv",
        "hunch_show2/[Group][Real Show Name S2][04][1080p].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "hunch_show2/[Group][Real Show Name S2][01][1080p].mkv",
        &siblings,
        None,
    );
    let title = result.title().unwrap_or("");
    assert_ne!(
        title, "hunch show",
        "normalized path prefix must not become title when filename is self-describing"
    );
    assert_eq!(title, "Real Show Name");
}

// ── 3. Anti-regression: bracketless files still inherit fallback ──────

#[test]
fn bracketless_file_still_inherits_fallback() {
    // Files WITHOUT bracket structure (the parent_context.rs scenario)
    // must continue to use the ancestor fallback as override. The fix
    // is scoped to filenames with bracket structure ONLY.
    let pipeline = Pipeline::new();
    let result = pipeline.run_with_context_and_fallback(
        "Paw Patrol/SP/Special.720p.mkv",
        &Vec::<&str>::new(),
        Some("Paw Patrol"),
    );
    assert_eq!(
        result.title(),
        Some("Paw Patrol"),
        "bracketless file should still inherit parent fallback (legacy behavior)"
    );
}

// ── 4. Anti-regression: real invariance still wins for non-bracket files

#[test]
fn real_invariance_still_overrides_for_dotted_filenames() {
    // The standard `Show.S01EXX` invariance pattern — no brackets, real
    // invariance signal across siblings — must continue to override.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "Show/Season 2/Show.S02E01.720p.mkv",
        "Show/Season 2/Show.S02E02.720p.mkv",
        "Show/Season 2/Show.S02E03.720p.mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "Show/Season 2/Show.S02E04.720p.mkv",
        &siblings,
        Some("Wrong Show Name"),
    );
    assert_ne!(
        result.title(),
        Some("Wrong Show Name"),
        "real invariance from dotted filenames must still override fallback"
    );
}
