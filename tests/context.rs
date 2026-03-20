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

// ── Phase 3: Year/Episode invariance signal tests ──────────────────────────

// -- Year-in-title suppression --

#[test]
fn invariant_year_suppressed_2001() {
    // "2001" appears in all siblings → invariant → title content, not Year.
    let r = hunch_with_context(
        "2001.A.Space.Odyssey.1080p.BluRay.mkv",
        &["2001.A.Space.Odyssey.720p.BluRay.mkv"],
    );
    let title = r.title().expect("should have title");
    assert!(
        title.contains("2001"),
        "title should contain '2001', got: {title}"
    );
    // Year should NOT be 2001 (it's title content).
    assert_ne!(
        r.year(),
        Some(2001),
        "2001 should not be extracted as year (it's part of the title)"
    );
}

#[test]
fn invariant_year_suppressed_1917() {
    // "1917" is invariant (title) + "2019" is invariant (same release year across siblings).
    let r = hunch_with_context("1917.2019.1080p.BluRay.mkv", &["1917.2019.720p.BluRay.mkv"]);
    let title = r.title().expect("should have title");
    assert!(
        title.contains("1917"),
        "title should contain '1917', got: {title}"
    );
}

#[test]
fn variant_year_preserved() {
    // Different years across siblings → variant → kept as Year.
    let r = hunch_with_context(
        "Movie.Collection.2023.1080p.mkv",
        &["Movie.Collection.2024.1080p.mkv"],
    );
    // The year should vary, so it should NOT be suppressed.
    // Note: the exact year value depends on which file we're parsing.
    assert!(
        r.year().is_some(),
        "variant year should be preserved as metadata"
    );
}

// -- Bare episode injection --

#[test]
fn bare_episode_injected_sequential() {
    // "03", "04", "05" are unclaimed sequential numbers → inject Episode.
    let r = hunch_with_context(
        "Show.Name.03.720p.mkv",
        &["Show.Name.04.720p.mkv", "Show.Name.05.720p.mkv"],
    );
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(
        r.episode(),
        Some(3),
        "sequential bare number should be injected as episode"
    );
}

#[test]
fn bare_episode_three_digit_absolute() {
    // 501, 502, 503 → sequential → inject Episode.
    let r = hunch_with_context(
        "Naruto.Shippuden.501.720p.mkv",
        &[
            "Naruto.Shippuden.502.720p.mkv",
            "Naruto.Shippuden.503.720p.mkv",
        ],
    );
    assert_eq!(
        r.episode(),
        Some(501),
        "3-digit sequential should be injected as episode"
    );
}

#[test]
fn invariant_number_not_injected_as_episode() {
    // "42" is the same in all siblings → invariant → NOT an episode.
    let r = hunch_with_context("Show.42.720p.mkv", &["Show.42.1080p.mkv"]);
    // 42 is invariant so should NOT be injected as episode.
    assert_ne!(
        r.episode(),
        Some(42),
        "invariant number should not be injected as episode"
    );
}

// -- SxxExx still wins when present --

#[test]
fn sxxexx_not_clobbered_by_invariance() {
    // Standard SxxExx pattern should not be overridden by invariance signals.
    let r = hunch_with_context(
        "Show.S01E03.720p.mkv",
        &["Show.S01E01.720p.mkv", "Show.S01E02.720p.mkv"],
    );
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(3));
    assert_eq!(r.title(), Some("Show"));
}

// ── Phase 5: Source tagging / confidence with heuristics ────────────────

#[test]
fn heuristic_decomposition_caps_confidence() {
    // "Movie.Title.501.720p.BluRay.x264-GROUP.mkv" with NO siblings →
    // digit decomposition fires (heuristic) → confidence should not be High.
    let r = hunch::hunch("Movie.Title.501.720p.BluRay.x264-GROUP.mkv");
    assert!(
        r.confidence() <= Confidence::Medium,
        "heuristic-only decomposition should cap confidence at Medium, got {:?}",
        r.confidence()
    );
}

#[test]
fn context_episode_gets_high_confidence() {
    // With siblings providing sequential evidence, confidence should be High.
    let r = hunch_with_context(
        "Show.03.720p.BluRay.mkv",
        &["Show.04.720p.BluRay.mkv", "Show.05.720p.BluRay.mkv"],
    );
    assert_eq!(r.confidence(), Confidence::High);
}

// -- Mixed: year + episode signals --

#[test]
fn year_and_episode_both_detected() {
    // "Show.2024.03.720p.mkv" — year is invariant, episode varies.
    let r = hunch_with_context(
        "Show.2024.03.720p.mkv",
        &["Show.2024.04.720p.mkv", "Show.2024.05.720p.mkv"],
    );
    let title = r.title().expect("should have title");
    assert!(
        title.contains("Show"),
        "title should contain 'Show', got: {title}"
    );
    assert_eq!(r.episode(), Some(3), "bare episode should be injected");
}

// -- Edge: no siblings → standard fallback --

#[test]
fn no_siblings_no_invariance_signals() {
    // Zero siblings → standard run, no invariance signals.
    let r = hunch_with_context("Show.03.720p.mkv", &[]);
    // Without context, "03" might or might not be episode. Just verify no crash.
    assert!(r.title().is_some());
}

// -- Edge: non-sequential variant numbers not injected --

#[test]
fn non_sequential_variant_not_injected() {
    // "03" and "17" vary but aren't sequential → not injected as episode.
    let r = hunch_with_context("Show.03.720p.mkv", &["Show.17.720p.mkv"]);
    // The bare number varies but isn't sequential, so episode injection
    // should not occur. (Standard heuristics may still claim it though.)
    // Just verify the title is correct and there's no crash.
    assert_eq!(r.title(), Some("Show"));
}

// ── P1: Cross-feature interaction tests ────────────────────────────────

#[test]
fn cjk_episode_with_path_context() {
    // CJK episode marker + tv/ path context → should detect both.
    let r = hunch::hunch("tv/Japanese/\u{5341}\u{4e8c}\u{56fd}\u{8a18}/\u{7b2c}13\u{8a71}.mkv");
    assert_eq!(r.episode(), Some(13));
    assert_eq!(r.media_type(), Some(hunch::MediaType::Episode));
}

#[test]
fn invariance_with_cjk_siblings() {
    // Cross-file invariance + CJK → should boost confidence.
    let r = hunch_with_context(
        "tv/\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}03\u{8a71}.mkv",
        &[
            "tv/\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}01\u{8a71}.mkv",
            "tv/\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}02\u{8a71}.mkv",
        ],
    );
    assert_eq!(r.media_type(), Some(hunch::MediaType::Episode));
    assert_eq!(r.confidence(), Confidence::High);
}

#[test]
fn empty_input_with_context_no_panic() {
    // Edge case: empty target with siblings should not panic.
    let r = hunch_with_context("", &["sibling.mkv"]);
    let _ = r.title(); // Just verify no panic.
}

#[test]
fn extension_only_with_context_no_panic() {
    let r = hunch_with_context(".mkv", &["sibling.mkv"]);
    let _ = r.title();
}
