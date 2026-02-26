//! Bit rate detection.
//!
//! Detects audio/video bit rates: 320Kbps, 448Kbps, 19.1Mbps, etc.
//! Hunch emits a single `bit_rate` property (not split into audio/video).
//! See COMPATIBILITY.md for rationale.

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches: 320Kbps, 448 Kbps, 19.1Mbps, 1.5 Mbps, etc.
static BIT_RATE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?P<num>\d+(?:\.\d+)?)\s*(?P<unit>[KkMm]bps)").unwrap()
});

static BIT_RATE_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),
    right: Some(CharClass::Alpha),
};

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
        let full = cap.get(0).unwrap();
        let abs_start = search_start + full.start();
        let abs_end = search_start + full.end();

        if !check_boundary(bytes, abs_start, abs_end, &BIT_RATE_BOUNDARY) {
            // Rejected — advance one byte past match start and retry.
            search_start = abs_start + 1;
            continue;
        }

        let num = cap.name("num").unwrap().as_str();
        let unit = cap.name("unit").unwrap().as_str();

        // Normalize unit casing: "kbps" → "Kbps", "mbps" → "Mbps".
        let normalized_unit = match unit.to_ascii_lowercase().as_str() {
            "kbps" => "Kbps",
            "mbps" => "Mbps",
            _ => unit,
        };

        // Output without spaces: "320Kbps", "19.1Mbps".
        let value = format!("{num}{normalized_unit}");

        matches.push(
            MatchSpan::new(abs_start, abs_end, Property::BitRate, &value).with_priority(1),
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
    }

    #[test]
    fn test_kbps_with_space() {
        let m = find_matches("Music [320 Kbps].mp3");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "320Kbps");
    }

    #[test]
    fn test_mbps() {
        let m = find_matches("Show.Name.19.1Mbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "19.1Mbps");
    }

    #[test]
    fn test_mbps_integer() {
        let m = find_matches("Show.Name.20Mbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "20Mbps");
    }

    #[test]
    fn test_bracketed_mbps() {
        let m = find_matches("Title Name [480p][1.5Mbps][.mp4]");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1.5Mbps");
    }

    #[test]
    fn test_after_codec() {
        // "H264.384Kbps" — must match 384Kbps, not merge with 264.
        let m = find_matches("H264.384Kbps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "384Kbps");
    }

    #[test]
    fn test_no_false_positive() {
        let m = find_matches("Movie.2024.1080p.BluRay.mkv");
        assert!(m.is_empty());
    }
}
