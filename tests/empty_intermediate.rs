//! Issue #97 regression: parent context propagation through empty intermediates.
//!
//! When there are empty intermediate directories (no media files, only subdirs)
//! between the batch root and the actual show directory, the parent context
//! chain must not break. The dir-name invariance detection must discard
//! titles derived from directory structure when no fallback is available,
//! allowing pass2's normal title extractor to find the real title.

use hunch::Pipeline;

#[test]
fn issue_97_dir_name_discarded_without_fallback() {
    // When invariance finds a directory name as the title and there's
    // no parent fallback, the title should be discarded (None) so the
    // normal extractor can run.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "tv/Anime/Show/[Group][Title][02][1080P].mkv",
        "tv/Anime/Show/[Group][Title][03][1080P].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "tv/Anime/Show/[Group][Title][01][1080P].mkv",
        &siblings,
        None, // no fallback — empty intermediate broke the chain
    );
    let title = result.title().unwrap_or("");
    // The title should NOT be "Anime" or "tv" — those are directory names.
    assert_ne!(title, "Anime", "dir name should not become the title");
    assert_ne!(title, "tv", "dir name should not become the title");
    assert_ne!(
        title, "tv Anime",
        "compound dir names should not be the title"
    );
}

#[test]
fn issue_97_non_consecutive_dir_names_detected() {
    // "Anime  特典映像" is two non-consecutive directory names —
    // is_path_dir_name should catch this via the all-parts check.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "tv/Anime/ShowDir/特典映像/[Group][Title][NC.Ver][1080P].mkv",
        "tv/Anime/ShowDir/特典映像/[Group][Title][TalkShow][1080P].mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "tv/Anime/ShowDir/特典映像/[Group][Title][Concert][1080P].mkv",
        &siblings,
        None,
    );
    let title = result.title().unwrap_or("");
    assert_ne!(title, "Anime");
    assert_ne!(title, "特典映像");
    assert!(
        !title.contains("Anime") || title.len() > 10,
        "title should not be purely directory names, got: {title:?}"
    );
}

#[test]
fn issue_97_fallback_still_works_through_chain() {
    // When a parent directory has a cached title and child files
    // have dir-name invariance, the fallback should win.
    let pipeline = Pipeline::new();
    let siblings = vec![
        "Show/Extras/Making.Of.720p.mkv",
        "Show/Extras/Gag.Reel.720p.mkv",
    ];
    let result = pipeline.run_with_context_and_fallback(
        "Show/Extras/Interview.720p.mkv",
        &siblings,
        Some("Natsume Yuujinchou Shichi"), // parent cached this
    );
    let title = result.title().unwrap_or("");
    assert_eq!(
        title, "Natsume Yuujinchou Shichi",
        "parent fallback should propagate through to child"
    );
}
