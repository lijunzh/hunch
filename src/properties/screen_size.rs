//! Screen size / resolution detection (720p, 1080p, 2160p, 4K, etc.).

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use regex::Regex;
use std::sync::LazyLock;

static RES_SCAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\d+)([ip]|hd)").unwrap());
static DIGITS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)").unwrap());

static STANDARD_RES: LazyLock<ValuePattern> = LazyLock::new(|| {
    ValuePattern::new(
        r"(?i)(?<![a-z0-9])(?:(?:\d{3,4})[x*])?(?:240|360|368|480|540|576|720|900|1080|1440|2160|4320)(?:[ip](?:x|HD|\d{2,3})?|hd)(?![a-z0-9])",
        "", // value computed dynamically
    )
});

static EXPLICIT_RES: LazyLock<ValuePattern> = LazyLock::new(|| {
    ValuePattern::new(
        r"(?i)(?<![a-z0-9])(\d{3,4})\s*[x*]\s*(\d{3,4})(?![a-z0-9])",
        "",
    )
});

/// Bare resolution number followed by Hi10p or similar profile marker.
/// e.g. `[720.Hi10p]`, `[1080.Hi10p]`
static BARE_RES_BEFORE_PROFILE: LazyLock<ValuePattern> = LazyLock::new(|| {
    ValuePattern::new(
        r"(?i)(?<![a-z0-9])(?:720|1080|480|2160)[. ]Hi(?:10|8)?p(?![a-z])",
        "",
    )
});

static FOUR_K: LazyLock<ValuePattern> =
    LazyLock::new(|| ValuePattern::new(r"(?i)(?<![a-z0-9])4K(?![a-z0-9])", "2160p"));

static EIGHT_K: LazyLock<ValuePattern> =
    LazyLock::new(|| ValuePattern::new(r"(?i)(?<![a-z0-9])8K(?![a-z0-9])", "4320p"));

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // Standard: 720p, 1080p, 1080i, 720hd, 720p60, etc.
    for (start, end) in STANDARD_RES.find_iter(input) {
        let raw = &input[start..end];
        let lower = raw.to_lowercase();

        // Strip optional WxH prefix.
        let height_part = if let Some(idx) = lower.rfind(['x', '*']) {
            &lower[idx + 1..]
        } else {
            &lower
        };

        // Extract the resolution number and scan type.
        let value = if let Some(caps) = RES_SCAN_RE.captures(height_part) {
            let num = caps.get(1).unwrap().as_str();
            let scan = caps.get(2).unwrap().as_str().to_lowercase();
            let scan_char = if scan == "hd" { "p" } else { &scan };
            format!("{num}{scan_char}")
        } else {
            height_part.to_string()
        };
        matches.push(MatchSpan::new(start, end, Property::ScreenSize, value));
    }

    // Explicit WxH: 1920x1080 -> 1080p.
    for (start, end) in EXPLICIT_RES.find_iter(input) {
        if matches.iter().any(|m| m.start == start && m.end == end) {
            continue;
        }
        if matches.iter().any(|m| !(m.end <= start || m.start >= end)) {
            continue; // Already matched by STANDARD_RES.
        }
        let raw = &input[start..end];
        // Extract height from WxH.
        if let Some(sep) = raw.find(['x', '*', 'X']) {
            let height_str = raw[sep + 1..].trim();
            let value = format!("{height_str}p");
            matches.push(MatchSpan::new(start, end, Property::ScreenSize, value));
        }
    }

    // 4K / 8K shorthands — skip when part of edition ("4K Restored", "4K Remastered").
    for (start, end) in FOUR_K.find_iter(input) {
        let after = &input[end..];
        let is_edition_qualifier = after
            .trim_start_matches(['.', ' ', '-', '_'])
            .to_lowercase()
            .starts_with("restor")
            || after
                .trim_start_matches(['.', ' ', '-', '_'])
                .to_lowercase()
                .starts_with("remaster");
        if !is_edition_qualifier {
            matches.push(MatchSpan::new(start, end, Property::ScreenSize, "2160p"));
        }
    }
    for (start, end) in EIGHT_K.find_iter(input) {
        matches.push(MatchSpan::new(start, end, Property::ScreenSize, "4320p"));
    }

    // Bare resolution before Hi10p profile: `[720.Hi10p]` → 720p.
    if matches.is_empty() {
        for (start, end) in BARE_RES_BEFORE_PROFILE.find_iter(input) {
            let raw = &input[start..end];
            if let Some(caps) = DIGITS_RE.captures(raw)
                && let Some(num) = caps.get(1)
            {
                let value = format!("{}p", num.as_str());
                matches.push(MatchSpan::new(
                    start,
                    start + num.end(),
                    Property::ScreenSize,
                    value,
                ));
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1080p() {
        let m = find_matches("Movie.1080p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080p");
    }

    #[test]
    fn test_720p() {
        let m = find_matches("Movie.720p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "720p");
    }

    #[test]
    fn test_4k() {
        let m = find_matches("Movie.4K.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2160p");
    }

    #[test]
    fn test_2160p() {
        let m = find_matches("Movie.2160p.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2160p");
    }

    #[test]
    fn test_1080i() {
        let m = find_matches("Movie.1080i.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080i");
    }

    #[test]
    fn test_explicit_1920x1080() {
        let m = find_matches("Movie.1920x1080.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1080p");
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_480p_in_brackets() {
        let input = "[Kaylith] Zankyou no Terror - 04 [480p][B4D4514E].mp4";
        let m = find_matches(input);
        assert!(
            m.iter().any(|x| x.value == "480p"),
            "Should detect 480p inside brackets"
        );
    }
}
