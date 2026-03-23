//! Issue #94 regression: child directories inherit parent context in batch -r.
//!
//! When a child directory (Extras/, SP/, 特典映像/) has too few files for
//! independent invariance detection, it should inherit the title from
//! the parent directory via the fallback mechanism.

use hunch::Pipeline;

// ── Fallback title from parent context ─────────────────────────────

#[test]
fn issue_94_fallback_title_single_file() {
    // A single file in a subdirectory has no siblings for invariance.
    // The fallback title from the parent should be used.
    let pipeline = Pipeline::new();
    let result = pipeline.run_with_context_and_fallback(
        "Paw Patrol/SP/Special.720p.mkv",
        &Vec::<&str>::new(),
        Some("Paw Patrol"),
    );
    assert_eq!(
        result.title(),
        Some("Paw Patrol"),
        "single file should inherit parent title via fallback"
    );
}

#[test]
fn issue_94_fallback_title_few_dissimilar_files() {
    // Files with different names — invariance can't find a common title.
    // Fallback title should be used.
    let pipeline = Pipeline::new();
    let result = pipeline.run_with_context_and_fallback(
        "ShowName/Extras/Making.Of.720p.mkv",
        &["ShowName/Extras/Gag.Reel.720p.mkv"],
        Some("ShowName"),
    );
    // The invariance might find "ShowName" from path, but fallback should
    // also be available. Either way, the title should include "ShowName".
    let title = result.title().unwrap_or("");
    assert!(
        title.contains("ShowName"),
        "title should include the parent show name, got: {title:?}"
    );
}

#[test]
fn issue_94_invariance_wins_over_fallback() {
    // When invariance finds a strong title from siblings, it should win
    // over the fallback ("inform but not force").
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
    // Invariance finds "Show" from siblings — should NOT be "Wrong Show Name".
    assert_ne!(
        result.title(),
        Some("Wrong Show Name"),
        "invariance should win over fallback when siblings produce a title"
    );
}

#[test]
fn issue_94_no_fallback_no_crash() {
    // Without fallback or siblings, should still produce a valid result.
    let pipeline = Pipeline::new();
    let result = pipeline.run_with_context_and_fallback(
        "Some.Movie.2024.720p.mkv",
        &Vec::<&str>::new(),
        None,
    );
    assert_eq!(result.title(), Some("Some Movie"));
}

// ── Invariance title = directory name → prefer fallback ─────────────

#[test]
fn issue_94_dir_name_invariance_yields_to_fallback() {
    // When invariance finds a title that's just a directory name from
    // the path, the parent fallback should win.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "夏目友人帐/特典映像/[DBD-Raws][Natsume.Yuujinchou.Shichi][NC.Ver][1080P].mkv",
        "夏目友人帐/特典映像/[DBD-Raws][Natsume.Yuujinchou.Shichi][TalkShow][1080P].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "夏目友人帐/特典映像/[DBD-Raws][Natsume.Yuujinchou.Shichi][劇伴コンサート][1080P].mkv",
        &siblings,
        Some("夏目友人帐"),
    );
    let title = result.title().unwrap_or("");
    // The invariance would find "特典映像" or "夏目友人帐" as invariant dir names.
    // With the fix, the parent fallback "夏目友人帐" should be preferred over
    // a dir-name-only invariance title like "特典映像".
    assert_ne!(
        title, "特典映像",
        "invariance should not use the sub-directory name as the title"
    );
}

#[test]
fn issue_94_dir_name_invariance_extras() {
    // "Extras" is a directory name. Even though invariance might find it
    // as common text, the fallback should be preferred.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "ShowName/Extras/Behind.The.Scenes.720p.mkv",
        "ShowName/Extras/Visual.Effects.720p.mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "ShowName/Extras/Interview.720p.mkv",
        &siblings,
        Some("ShowName"),
    );
    let title = result.title().unwrap_or("");
    assert_ne!(
        title, "Extras",
        "directory name 'Extras' should not be the title when fallback is available"
    );
    assert!(
        title.contains("ShowName"),
        "title should include the parent show name, got: {title:?}"
    );
}
