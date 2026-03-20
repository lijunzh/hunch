//! Integration tests for cross-file context (run_with_context).
//!
//! Tests the invariance detection algorithm: the title is the text that
//! doesn't change across sibling files.

use hunch::{Confidence, Pipeline, hunch_with_context};

// ── Western filenames ───────────────────────────────────────────────────────

#[test]
fn western_episode_series_cross_file() {
    let target = "Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv";
    let siblings = &[
        "Breaking.Bad.S05E14.720p.BluRay.x264-DEMAND.mkv",
        "Breaking.Bad.S05E15.720p.BluRay.x264-DEMAND.mkv",
    ];
    let result = hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("Breaking Bad"));
}

#[test]
fn western_movie_no_siblings_fallback() {
    // Zero siblings → falls back to standard run().
    let result = hunch_with_context("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv", &[]);
    assert_eq!(result.title(), Some("The Matrix"));
    assert_eq!(result.year(), Some(1999));
}

#[test]
fn western_one_sibling() {
    // Even a single sibling should work.
    let target = "Show.S01E03.720p.mkv";
    let siblings = &["Show.S01E01.720p.mkv"];
    let result = hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("Show"));
}

// ── CJK filenames ───────────────────────────────────────────────────────────

#[test]
fn cjk_fansub_cross_file_title() {
    let target = "(BD)\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}13\u{8a71}\u{300c}\u{6708}\u{306e}\u{5f71} \u{5f71}\u{306e}\u{6d77}\u{3000}\u{7d42}\u{7ae0}\u{300d}(1440x1080 x264-10bpp flac).mkv";
    let siblings = &[
        "(BD)\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}01\u{8a71}\u{300c}\u{6708}\u{306e}\u{5f71} \u{5f71}\u{306e}\u{6d77}\u{3000}\u{4e00}\u{7ae0}\u{300d}(1440x1080 x264-10bpp flac).mkv",
        "(BD)\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}02\u{8a71}\u{300c}\u{6708}\u{306e}\u{5f71} \u{5f71}\u{306e}\u{6d77}\u{3000}\u{4e8c}\u{7ae0}\u{300d}(1440x1080 x264-10bpp flac).mkv",
    ];
    let result = hunch_with_context(target, siblings);
    let title = result.title().expect("title should be detected");
    assert!(
        title.contains('\u{5341}') && title.contains('\u{56fd}'),
        "title should contain 十二国記, got: {title}"
    );
}

// ── Pipeline reuse ──────────────────────────────────────────────────────────

#[test]
fn pipeline_run_with_context_reuse() {
    let pipeline = Pipeline::new();

    let r1 = pipeline.run_with_context(
        "Show.S01E01.720p.mkv",
        &["Show.S01E02.720p.mkv", "Show.S01E03.720p.mkv"],
    );
    assert_eq!(r1.title(), Some("Show"));

    let r2 = pipeline.run_with_context(
        "Other.Show.S02E05.1080p.mkv",
        &["Other.Show.S02E06.1080p.mkv"],
    );
    assert_eq!(r2.title(), Some("Other Show"));
}

// ── Confidence scoring ─────────────────────────────────────────────────────

#[test]
fn confidence_high_with_anchors() {
    let result = hunch::hunch("Movie.2024.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(result.confidence(), Confidence::High);
}

#[test]
fn confidence_high_with_cross_file() {
    let target = "Show.S01E03.720p.mkv";
    let siblings = &["Show.S01E01.720p.mkv", "Show.S01E02.720p.mkv"];
    let result = hunch_with_context(target, siblings);
    assert_eq!(result.confidence(), Confidence::High);
}

#[test]
fn confidence_low_minimal_input() {
    let result = hunch::hunch("x.mkv");
    assert!(result.confidence() <= Confidence::Medium);
}

#[test]
fn confidence_medium_some_anchors() {
    let result = hunch::hunch("SomeShow.720p.mkv");
    assert!(result.confidence() >= Confidence::Medium);
}

#[test]
fn cross_file_preserves_tech_properties() {
    let target = "Show.S01E03.720p.BluRay.x264-GROUP.mkv";
    let siblings = &[
        "Show.S01E01.720p.BluRay.x264-GROUP.mkv",
        "Show.S01E02.720p.BluRay.x264-GROUP.mkv",
    ];
    let result = hunch_with_context(target, siblings);
    assert_eq!(result.title(), Some("Show"));
    assert_eq!(result.season(), Some(1));
    assert_eq!(result.episode(), Some(3));
    assert_eq!(result.screen_size(), Some("720p"));
    assert_eq!(result.source(), Some("Blu-ray"));
    assert_eq!(result.video_codec(), Some("H.264"));
    assert_eq!(result.release_group(), Some("GROUP"));
    assert_eq!(result.container(), Some("mkv"));
}
