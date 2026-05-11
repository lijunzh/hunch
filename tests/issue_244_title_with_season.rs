//! Regression tests for Issue #244: Anime filenames in bracket format fail to extract title field.
//!
//! The bug was that when a title bracket contains a season marker like "S2" 
//! (e.g., `[Show Name S2]`), the `S_ONLY` pattern would match just "S2" within 
//! that bracket, creating a Season match inside the title content. The CJK bracket 
//! strategy then rejected the entire bracket as "claimed" because it found this 
//! Season match inside.
//!
//! The fix:
//! 1. Allow Season matches inside title brackets (they're part of the title)
//! 2. Strip trailing season markers like " S2", " Season 2" from the extracted title

use hunch::hunch;

// ── Main issue: S2 in title bracket should extract title correctly ────

#[test]
fn issue_244_s2_in_title_bracket() {
    // The exact failing case from issue #244
    let r = hunch("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][01][1080P][BDRip][HEVC-10bit][FLACx2].mkv");
    
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
}

#[test]
fn issue_244_s3_in_title_bracket() {
    // Test with S3 as well
    let r = hunch("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S3][02][1080P][BDRip].mkv");
    
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    assert_eq!(r.season(), Some(3));
    assert_eq!(r.episode(), Some(2));
}

#[test]
fn issue_244_season_word_in_title_bracket() {
    // Test with "Season 2" instead of "S2"
    let r = hunch("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu Season 2][01][1080P][BDRip].mkv");
    
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(1));
}

// ── Anti-regression: titles without season markers should still work ────

#[test]
fn issue_244_no_season_marker_still_works() {
    // Title without season marker should extract correctly
    let r = hunch("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu][01][1080P][BDRip].mkv");
    
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    // Season should be absent since there's no season marker
    assert_eq!(r.season(), None);
}

// ── Anti-regression: titles with additional content after S2 should not strip ────

#[test]
fn issue_244_s2_with_additional_content() {
    // When there's additional content after the season marker, don't strip it
    let r = hunch("[DBD-Raws][Show Name S2 Special][01][1080P].mkv");
    
    // "S2 Special" is not just a season marker - it's part of the title
    assert_eq!(r.title(), Some("Show Name S2 Special"));
    assert_eq!(r.season(), Some(2));
}

// ── Anti-regression: mini-animations without season markers ────

#[test]
fn issue_244_mini_animation_without_season() {
    // Mini-animations like "Break Time Otto's Diary" should still work
    let r = hunch("[DBD-Raws][Re Zero Kara Hajimeru Break Time Otto's Diary][01][1080P][BDRip].mkv");
    
    assert_eq!(r.title(), Some("Re Zero Kara Hajimeru Break Time Otto's Diary"));
}

// ── Edge cases: S10, s2 (lowercase) ────

#[test]
fn issue_244_s10_in_title_bracket() {
    // Test with double-digit season number
    let r = hunch("[Group][Show Name S10][01].mkv");
    
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(10));
}

#[test]
fn issue_244_lowercase_s2_in_title_bracket() {
    // Test with lowercase 's'
    let r = hunch("[Group][Show Name s2][01].mkv");
    
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(2));
}
