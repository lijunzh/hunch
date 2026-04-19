//! Bit rate detection.
//!
//! Detects audio/video bit rates: 320Kbps, 448Kbps, 19.1Mbps, etc.
//!
//! Bit-rate matches are disambiguated at parse time using the unit as the
//! signal:
//!
//! - `Kbps` (kilobits per second) → [`Property::AudioBitRate`].
//!   Audio data rates are universally in the kilobit range (typically
//!   96–512 Kbps for compressed formats; rarely above 1024 Kbps even for
//!   lossless).
//! - `Mbps` (megabits per second) → [`Property::VideoBitRate`].
//!   Video data rates are universally in the megabit range (typically
//!   1–50 Mbps for compressed formats).
//!
//! This unit-based heuristic matches guessit's behavior and reflects how the
//! values appear in real-world filenames.

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches: 320Kbps, 448 Kbps, 19.1Mbps, 1.5 Mbps, 384kbit, 5Mbit, etc.
///
/// Accepts the long suffix (`bps` = bits per second), the short suffix
/// (`bit`), or the plural short suffix (`bits` — seen in some anime
/// release notations like `19.1mbits`). All three normalize to `bps` at
/// output for a single canonical form.
///
/// The decimal portion is intentionally bounded to 1–2 digits. Real-world
/// bit-rate values are almost never specified to higher precision, and the
/// loose `\d+` form caused greedy collisions with adjacent decimals like
/// audio-channel notation (`DD5.1.448kbps` would match `1.448Kbps` instead
/// of `448Kbps`). The bounded form lets the regex backtrack cleanly to the
/// next integer match in such cases.
static BIT_RATE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?P<num>\d+(?:\.\d{1,2})?)\s*(?P<unit>[KkMm])(?:bps|bits?)")
        .expect("BIT_RATE regex is valid")
});

static BIT_RATE_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),
    right: Some(CharClass::Alpha),
};

/// Scan for bit rate patterns (e.g., `320Kbps`, `1.5Mbps`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();
    let mut search_start = 0;

    // Manual position scanning: `captures_iter` skips past rejected matches,
    // which can miss valid sub-matches (e.g., "H264.384Kbps" — the greedy
    // regex matches "264.384Kbps", gets rejected, and "384Kbps" is lost).
    // Instead, we advance one byte past rejected matches to retry.
    while search_start < input.len() {
        let Some(cap) = BIT_RATE_PATTERN.captures(&input[search_start..]) else {
            break;
        };
        let full = cap.get(0).expect("group 0 always present in a regex match");
        let abs_start = search_start + full.start();
        let abs_end = search_start + full.end();

        if !check_boundary(bytes, abs_start, abs_end, &BIT_RATE_BOUNDARY) {
            // Rejected — advance one byte past match start and retry.
            search_start = abs_start + 1;
            continue;
        }

        let num = cap
            .name("num")
            .expect("num group always present in BIT_RATE")
            .as_str();
        let unit = cap
            .name("unit")
            .expect("unit group always present in BIT_RATE")
            .as_str();

        // Normalize unit prefix casing: "k" → "K", "m" → "M".
        // The trailing suffix is always normalized to "bps" regardless of
        // whether the input said "bps" or "bit" — single canonical form.
        // Disambiguate by unit (#158): Kbps → audio, Mbps → video.
        // The unit alone is a near-perfect signal in real-world filenames.
        let (normalized_unit, property) = match unit.to_ascii_lowercase().as_str() {
            "k" => ("Kbps", Property::AudioBitRate),
            "m" => ("Mbps", Property::VideoBitRate),
            // The regex character class is `[KkMm]`, so this is genuinely
            // unreachable. Use `unreachable!` rather than a defensive
            // fallback so a future regex change that breaks this contract
            // fails loudly in tests instead of producing silently-wrong
            // output. (Pre-v2.0.0 used a `Property::BitRate` fallback;
            // that variant was removed in v2.0.0 — see CHANGELOG.)
            _ => unreachable!("BIT_RATE regex captures only [KkMm] for the unit group"),
        };

        // Output without spaces: "320Kbps", "19.1Mbps".
        let value = format!("{num}{normalized_unit}");

        matches.push(
            MatchSpan::new(abs_start, abs_end, property, &value)
                .with_priority(crate::priority::VOCABULARY),
        );

        // Advance past this match.
        search_start = abs_end;
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kbps() {
        let m = find_matches("Music.Track.320Kbps.mp3");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "320Kbps");
        // Kbps must always be classified as AudioBitRate (#158).
        assert_eq!(m[0].property, Property::AudioBitRate);
    }

    #[test]
    fn test_kbps_with_space() {
        let m = find_matches("Music [320 Kbps].mp3");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "320Kbps");
        assert_eq!(m[0].property, Property::AudioBitRate);
    }

    #[test]
    fn test_mbps() {
        let m = find_matches("Show.Name.19.1Mbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "19.1Mbps");
        // Mbps must always be classified as VideoBitRate (#158).
        assert_eq!(m[0].property, Property::VideoBitRate);
    }

    #[test]
    fn test_mbps_integer() {
        let m = find_matches("Show.Name.20Mbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "20Mbps");
        assert_eq!(m[0].property, Property::VideoBitRate);
    }

    #[test]
    fn test_bracketed_mbps() {
        let m = find_matches("Title Name [480p][1.5Mbps][.mp4]");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1.5Mbps");
        assert_eq!(m[0].property, Property::VideoBitRate);
    }

    #[test]
    fn test_after_codec() {
        // "H264.384Kbps" — must match 384Kbps, not merge with 264.
        let m = find_matches("H264.384Kbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "384Kbps");
        assert_eq!(m[0].property, Property::AudioBitRate);
    }

    #[test]
    fn test_no_false_positive() {
        let m = find_matches("Movie.2024.1080p.BluRay.mkv");
        assert!(m.is_empty());
    }
}
