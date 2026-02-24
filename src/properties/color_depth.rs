//! Color depth detection (bit depth).
//!
//! Detects color bit depth: 8bit, 10bit, 12bit, etc.

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static COLOR_DEPTH_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        ValuePattern::new(r"(?i)(?<![a-z0-9])12[- ]?bits?(?![a-z0-9])", "12-bit"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])10[- ]?bits?(?![a-z0-9])", "10-bit"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])8[- ]?bits?(?![a-z0-9])", "8-bit"),
        // Hi10P / Hi10 implies 10-bit
        ValuePattern::new(r"(?i)(?<![a-z])Hi10(?:P|p)?(?![a-z0-9])", "10-bit"),
        // HEVC10 / x265-10 implies 10-bit
        ValuePattern::new(
            r"(?i)(?:HEVC|[xh]265|[xh]\.?265)[-. ]?10(?![0-9])",
            "10-bit",
        ),
        // yuv420p10 pixel format → 10-bit
        ValuePattern::new(r"(?i)yuv\d+p10", "10-bit"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in COLOR_DEPTH_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(MatchSpan::new(
                start,
                end,
                Property::ColorDepth,
                pattern.value,
            ));
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10bit() {
        let m = find_matches("Movie.10bit.mkv");
        assert_eq!(m[0].value, "10-bit");
    }

    #[test]
    fn test_8bit() {
        let m = find_matches("Movie.8bit.mkv");
        assert_eq!(m[0].value, "8-bit");
    }
}
