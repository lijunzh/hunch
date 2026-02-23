//! Screen size / resolution detection (720p, 1080p, 2160p, 4K, etc.).

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref STANDARD_RES: ValuePattern = ValuePattern::new(
        r"(?i)(?<![a-z0-9])(?:(?:\d{3,4})[x*])?(?:240|360|480|540|576|720|900|1080|1440|2160|4320)[ip](?![a-z0-9])",
        "",  // value computed dynamically
    );
    static ref EXPLICIT_RES: ValuePattern = ValuePattern::new(
        r"(?i)(?<![a-z0-9])(\d{3,4})\s*[x*]\s*(\d{3,4})(?![a-z0-9])",
        "",
    );
    static ref FOUR_K: ValuePattern = ValuePattern::new(
        r"(?i)(?<![a-z0-9])4K(?![a-z0-9])",
        "2160p",
    );
    static ref EIGHT_K: ValuePattern = ValuePattern::new(
        r"(?i)(?<![a-z0-9])8K(?![a-z0-9])",
        "4320p",
    );
}

pub struct ScreenSizeMatcher;

impl PropertyMatcher for ScreenSizeMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Standard: 720p, 1080p, 1080i, etc.
        for (start, end) in STANDARD_RES.find_iter(input) {
            let raw = &input[start..end];
            let normalized = raw.to_lowercase();
            // Extract just height+scan from the end (strip optional WxH prefix).
            let value = if let Some(idx) = normalized.rfind(|c: char| c == 'x' || c == '*') {
                &normalized[idx + 1..]
            } else {
                &normalized
            };
            matches.push(MatchSpan::new(start, end, Property::ScreenSize, value.to_string()));
        }

        // Explicit WxH: 1920x1080 -> 1080p.
        for (start, end) in EXPLICIT_RES.find_iter(input) {
            if matches.iter().any(|m| m.start == start && m.end == end) {
                continue;
            }
            let raw = &input[start..end];
            // Extract height from WxH.
            if let Some(sep) = raw.find(|c: char| c == 'x' || c == '*' || c == 'X') {
                let height_str = raw[sep + 1..].trim();
                let value = format!("{height_str}p");
                matches.push(MatchSpan::new(start, end, Property::ScreenSize, value));
            }
        }

        // 4K / 8K shorthands.
        for (start, end) in FOUR_K.find_iter(input) {
            matches.push(MatchSpan::new(start, end, Property::ScreenSize, "2160p"));
        }
        for (start, end) in EIGHT_K.find_iter(input) {
            matches.push(MatchSpan::new(start, end, Property::ScreenSize, "4320p"));
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1080p() {
        let m = ScreenSizeMatcher.find_matches("Movie.1080p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080p");
    }

    #[test]
    fn test_720p() {
        let m = ScreenSizeMatcher.find_matches("Movie.720p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "720p");
    }

    #[test]
    fn test_4k() {
        let m = ScreenSizeMatcher.find_matches("Movie.4K.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2160p");
    }

    #[test]
    fn test_2160p() {
        let m = ScreenSizeMatcher.find_matches("Movie.2160p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2160p");
    }

    #[test]
    fn test_1080i() {
        let m = ScreenSizeMatcher.find_matches("Movie.1080i.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080i");
    }

    #[test]
    fn test_explicit_1920x1080() {
        let m = ScreenSizeMatcher.find_matches("Movie.1920x1080.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080p");
    }
}
