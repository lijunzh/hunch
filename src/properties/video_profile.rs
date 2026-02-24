//! Video profile detection.
//!
//! Detects video encoding profiles and codec profile descriptions:
//! - H.264 profiles: High, Main, Baseline, High 10
//! - HEVC → High Efficiency Video Coding
//! - AVCHD → Advanced Video Codec High Definition
//! - SVC/SDH → Scalable Video Coding

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static VIDEO_PROFILE_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        // AVCHD → Advanced Video Codec High Definition
        ValuePattern::new(
            r"(?i)(?<![a-z])AVCHD(?![a-z])",
            "Advanced Video Codec High Definition",
        ),
        // Hi10P / Hi10 → High 10
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Hi|High)[-. ]?10(?:P)?(?![a-z0-9])",
            "High 10",
        ),
        // Hi422 → High 4:2:2
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Hi|High)[-. ]?422(?![a-z0-9])",
            "High 4:2:2",
        ),
        // Hi444PP → High 4:4:4 Predictive
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Hi|High)[-. ]?444(?:PP)?(?![a-z0-9])",
            "High 4:4:4 Predictive",
        ),
        // HP → High Profile
        ValuePattern::new(r"(?<![a-zA-Z])HP(?![a-zA-Z])", "High"),
        // HEVC → High Efficiency Video Coding (as profile)
        ValuePattern::new(
            r"(?i)(?<![a-z])HEVC(?![a-z])",
            "High Efficiency Video Coding",
        ),
        // SDH → Scalable Video Coding
        ValuePattern::new(r"(?<![a-zA-Z])S[CD]H(?![a-zA-Z])", "Scalable Video Coding"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in VIDEO_PROFILE_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(
                MatchSpan::new(start, end, Property::VideoProfile, pattern.value).with_priority(-2),
            );
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_10() {
        let m = find_matches("Movie.Hi10.mkv");
        assert!(m.iter().any(|x| x.value == "High 10"));
    }

    #[test]
    fn test_avchd() {
        let m = find_matches("Movie.AVCHD.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "Advanced Video Codec High Definition")
        );
    }

    #[test]
    fn test_hevc_profile() {
        let m = find_matches("Movie.HEVC.mkv");
        assert!(m.iter().any(|x| x.value == "High Efficiency Video Coding"));
    }
}
