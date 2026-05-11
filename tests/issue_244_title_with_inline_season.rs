//! Regression tests for issue #244: anime filenames in bracket format
//! fail to extract title field when the title bracket contains an
//! inline season marker like `S2`.
//!
//! Symptom (v2.0.1):
//!   `[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][01]…`
//!   → title MISSING, season=2, episode=1
//!
//! Root cause: the `S2` token inside the title bracket created a
//! `Property::Season` match that overlapped the bracket. Bracket-based
//! title strategies (`CjkBracket`, `UnclaimedBracket`) treated that
//! overlap as a competing "claim" on the bracket and skipped it as a
//! title candidate.
//!
//! Fix:
//!   1. Exclude `Property::Season` and `Property::Episode` from the
//!      `is_claimed` check in both bracket strategies — they're inline
//!      metadata, not competing claims.
//!   2. Extend `RE_TRAILING_SEASON` (used by `clean_title`) to match the
//!      `S\d{1,3}` short form so the trailing `S2` is stripped.
//!   3. Have `CjkBracket` actually run its output through `clean_title`
//!      (it previously bypassed the cleaning pipeline).

use hunch::hunch;

// ── 1. The exact case from the issue ──────────────────────────────────

#[test]
fn issue_244_headline_case() {
    let r = hunch(
        "[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu S2][01][1080P][BDRip][HEVC-10bit][FLACx2].mkv",
    );
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
}

// ── 2. CjkBracket strategy (has Episode match → fires) ────────────────

#[test]
fn cjk_bracket_strips_short_season() {
    let r = hunch("[Group][Show Name S3][05][1080p].mkv");
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(3));
}

#[test]
fn cjk_bracket_strips_long_season() {
    let r = hunch("[Group][Show Name Season 2][05][1080p].mkv");
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(2));
}

#[test]
fn cjk_bracket_strips_double_digit_season() {
    let r = hunch("[Group][Show Name S10][05][1080p].mkv");
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(10));
}

#[test]
fn cjk_bracket_lowercase_s2() {
    let r = hunch("[Group][Show Name s2][05][1080p].mkv");
    assert_eq!(r.title(), Some("Show Name"));
    assert_eq!(r.season(), Some(2));
}

// ── 3. UnclaimedBracket strategy (no Episode → falls through to here) ─

#[test]
fn unclaimed_bracket_strips_trailing_season() {
    // No `[01]` episode bracket → CjkBracket bails, UnclaimedBracket fires.
    // Pre-fix this returned title=null; post-fix it returns the cleaned title.
    let r = hunch("[Group][My Show S2][1080p][BDRip][HEVC-10bit][FLAC].mkv");
    assert_eq!(r.title(), Some("My Show"));
    assert_eq!(r.season(), Some(2));
}

// ── 4. Anti-regression: titles WITHOUT inline season markers ──────────

#[test]
fn no_season_marker_unchanged() {
    let r = hunch("[DBD-Raws][Re Zero kara Hajimeru Isekai Seikatsu][01][1080P][BDRip].mkv");
    assert_eq!(r.title(), Some("Re Zero kara Hajimeru Isekai Seikatsu"));
    assert_eq!(r.season(), None);
}

#[test]
fn mini_animation_without_season_unchanged() {
    let r = hunch(
        "[DBD-Raws][Re Zero Kara Hajimeru Break Time Otto's Diary][01][1080P][BDRip][HEVC-10bit][FLAC].mkv",
    );
    assert_eq!(
        r.title(),
        Some("Re Zero Kara Hajimeru Break Time Otto's Diary")
    );
}

// ── 5. Anti-regression: titles where the trailing `S` is genuinely     ─
//    part of the title (e.g. `S` as a name suffix, no digits after).   ─

#[test]
fn bare_s_not_stripped() {
    // `S` alone (no digits) is NOT a season marker; must not be stripped.
    let r = hunch("[Group][Battlestar Galactica S][01].mkv");
    assert_eq!(r.title(), Some("Battlestar Galactica S"));
}
