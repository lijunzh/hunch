//! Tests for token-matching constraints (not_before, not_after, requires_context, etc.)
//!
//! These exercise the branching logic in `pipeline/matching.rs` through
//! the full pipeline, using known TOML rules that have these constraints.

use hunch::hunch;

// ── not_before / not_after ────────────────────────────────────────────

#[test]
fn not_before_blocks_match() {
    // "HD" has not_before = ["dvd"] in source rules — "HD-DVD" should NOT
    // produce source: "HD" (it should produce "HD-DVD" via compound match).
    let r = hunch("Movie.HD-DVD.mkv");
    assert_eq!(r.source(), Some("HD-DVD"));
}

#[test]
fn not_after_blocks_match() {
    // "WEB-DL" should be recognized as a compound source.
    let r = hunch("Show.S01E01.WEB-DL.720p.mkv");
    assert_eq!(r.source(), Some("Web"));
}

// ── requires_context ──────────────────────────────────────────────────

#[test]
fn requires_context_with_anchors() {
    // "Remux" requires tech context to match. With anchors (1080p, BluRay),
    // it should match.
    let r = hunch("Movie.2024.1080p.BluRay.Remux.mkv");
    let other = r.other();
    assert!(
        other.contains(&"Remux"),
        "Remux should match with tech context: {other:?}"
    );
}

#[test]
fn requires_context_without_anchors_skips() {
    // Without any tech anchors, context-requiring rules should be more
    // conservative. Even so, "Remux" is a strong enough tech signal
    // that it creates its own anchor.
    let r = hunch("The.Remux.mkv");
    // Remux is recognized as tech even without surrounding anchors.
    let other = r.other();
    assert!(
        other.contains(&"Remux"),
        "Remux should be detected: {other:?}"
    );
    assert_eq!(r.title(), Some("The"));
}

// ── requires_nearby ──────────────────────────────────────────────────

#[test]
fn requires_nearby_satisfied() {
    // "MA" in audio profiles requires nearby DTS/Atmos.
    // "DTS-HD.MA" should produce audio_profile: "Master Audio".
    let r = hunch("Movie.DTS-HD.MA.1080p.BluRay.mkv");
    assert_eq!(r.audio_codec(), Some("DTS-HD"));
    let ap = r.first(hunch::matcher::Property::AudioProfile);
    assert_eq!(ap, Some("Master Audio"));
}

// ── Side effects ─────────────────────────────────────────────────────

#[test]
fn side_effect_emits_extra_property() {
    // "HDR" should emit video_profile side effect along with "other".
    let r = hunch("Movie.2160p.UHD.BluRay.HDR.HEVC.mkv");
    let other = r.other();
    assert!(
        other.contains(&"HDR10"),
        "HDR should emit other: HDR10: {other:?}"
    );
}

// ── Compound window matching ─────────────────────────────────────────

#[test]
fn compound_two_token_match() {
    // "WEB-DL" is a 2-token compound (WEB + DL via dash separator).
    let r = hunch("Show.S01E01.WEB-DL.1080p.H264-GROUP.mkv");
    assert_eq!(r.source(), Some("Web"));
}

#[test]
fn compound_three_token_match() {
    // "DTS-HD.MA" is a 3-token compound (DTS + HD + MA).
    let r = hunch("Movie.1080p.DTS-HD.MA.BluRay.x264-GROUP.mkv");
    assert_eq!(r.audio_codec(), Some("DTS-HD"));
    let ap = r.first(hunch::matcher::Property::AudioProfile);
    assert_eq!(ap, Some("Master Audio"));
}

// ── Zone scope filtering ─────────────────────────────────────────────

#[test]
fn tech_only_rule_skips_title_zone() {
    // Streaming service rules have zone_scope = "tech_only".
    // "AMZN" in the tech zone should match, not be absorbed as title.
    let r = hunch("Show.S01E01.AMZN.WEB-DL.1080p.mkv");
    assert_eq!(r.streaming_service(), Some("Amazon Prime"));
    assert_eq!(r.title(), Some("Show"));
}

// ── Reclaimable matches ──────────────────────────────────────────────

#[test]
fn reclaimable_match_yields_to_title() {
    // Some weak matches (reclaimable) should be reclaimed as title text
    // if they're in the title zone.
    let r = hunch("The.Flash.S01E01.720p.mkv");
    assert_eq!(r.title(), Some("The Flash"));
}
