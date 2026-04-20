//! Regression tests for #212 bug 3: ancestor-path source contamination.
//!
//! The Source rule set is registered as `AllSegments` so that legitimate
//! organisational dirs (`Blu-ray/`, `WEB-DL/`, `HDTV/`) still produce a
//! source when the filename itself carries no source marker. The risk:
//! when the filename DOES carry an explicit source (e.g. `WEB-DL`), the
//! ancestor-path match leaks alongside it and the result becomes a
//! `["TV", "Web"]` JSON array instead of just `"Web"`.
//!
//! The `ancestor_source_yields_to_filename` zone rule (added with this
//! test) drops ancestor-path Source matches when any filename Source
//! match exists. These tests pin that contract from both directions:
//!
//!   1. The reported bug is fixed (filename source wins, single value).
//!   2. The previous behaviour is preserved when the filename has no
//!      source signal (ancestor-only sources are still detected).

use hunch::hunch;
use serde_json::Value;

/// Helper: how many distinct source values does this result expose?
/// Returns 0 if no source, 1 for a single-value source, ≥2 for the
/// `["TV", "Web"]` style array that bug 3 produces.
fn source_count(r: &hunch::HunchResult) -> usize {
    match r.to_flat_map().get("source") {
        Some(Value::String(_)) => 1,
        Some(Value::Array(arr)) => arr.len(),
        Some(_) | None => 0,
    }
}

// ── Bug 3 reproduction: filename source must dominate ancestor source ──

/// The exact failing case from #212.
#[test]
fn issue_212_bug3_tv_ancestor_with_filename_webdl() {
    let r = hunch(
        "/Volumes/media/tv/Anime/[\u{665a}\u{8857}\u{4e0e}\u{706f}]\
         [Re Zero kara Hajimeru Isekai Seikatsu]\
         [4th - 01][\u{603b}\u{7b2c}67]\
         [WEB-DL Remux][1080P_AVC_AAC]\
         [\u{7b80}\u{7e41}\u{65e5}\u{5185}\u{5c01}PGS][V2].mkv",
    );
    assert_eq!(
        r.source(),
        Some("Web"),
        "filename WEB-DL should override ancestor /tv/"
    );
    assert_eq!(
        source_count(&r),
        1,
        "expected exactly one source value, got JSON: {:?}",
        r.to_flat_map().get("source")
    );
}

#[test]
fn filename_webdl_overrides_tv_ancestor() {
    let r = hunch("/media/tv/Show.S01E01.WEB-DL.mkv");
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(source_count(&r), 1);
}

#[test]
fn filename_bdrip_overrides_tv_ancestor() {
    let r = hunch("/media/tv/Show.S01E01.BDRip.mkv");
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(source_count(&r), 1);
}

#[test]
fn filename_webdl_overrides_hdtv_ancestor() {
    // Different ancestor source value (HDTV) — filename WEB-DL must still win.
    let r = hunch("/media/HDTV/Show.S01E01.WEB-DL.mkv");
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(source_count(&r), 1);
}

// ── Anti-regression: ancestor-only sources MUST still be detected ──

#[test]
fn ancestor_bluray_preserved_when_filename_has_no_source() {
    // No source marker in filename → ancestor /Blu-ray/ should be the source.
    let r = hunch("/Volumes/Blu-ray/Some.Movie.2020.mkv");
    assert_eq!(
        r.source(),
        Some("Blu-ray"),
        "ancestor-only source should still be detected when filename \
         carries no source marker"
    );
    assert_eq!(source_count(&r), 1);
}

#[test]
fn ancestor_tv_preserved_when_filename_has_no_source() {
    let r = hunch("/media/tv/Show.S01E01.mkv");
    assert_eq!(
        r.source(),
        Some("TV"),
        "ancestor /tv/ should still produce TV source when filename has \
         no source marker"
    );
    assert_eq!(source_count(&r), 1);
}

#[test]
fn ancestor_webdl_preserved_when_filename_has_no_source() {
    let r = hunch("/media/WEB-DL/Movie.2024.mkv");
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(source_count(&r), 1);
}

// ── Filename-internal source dedup is unaffected ──

#[test]
fn filename_only_no_ancestor_unaffected() {
    // Pure filename input — no path — should behave identically to before.
    let r = hunch("Show.S01E01.WEB-DL.1080p.mkv");
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(source_count(&r), 1);
}
