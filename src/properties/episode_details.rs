//! Episode details detection.
//!
//! Detects special episode markers: Special, Pilot, Unaired, OVA, etc.

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static EPISODE_DETAILS_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        ValuePattern::new(
            r"(?i)(?<![a-z])Special(?![-. ]*(?:Edition|Feature|Effect))(?![a-z])",
            "Special",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])Pilot(?![a-z])", "Pilot"),
        ValuePattern::new(r"(?i)(?<![a-z])Unaired(?![a-z])", "Unaired"),
        ValuePattern::new(r"(?i)(?<![a-z])Final(?![a-z])", "Final"),
        ValuePattern::new(r"(?i)(?<![a-z])Premiere(?![a-z])", "Premiere"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in EPISODE_DETAILS_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(
                MatchSpan::new(start, end, Property::EpisodeDetails, pattern.value)
                    .with_priority(-1),
            );
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special() {
        let m = find_matches("Show.S01.Special.mkv");
        assert!(m.iter().any(|x| x.value == "Special"));
    }

    #[test]
    fn test_pilot() {
        let m = find_matches("Show.Pilot.720p.mkv");
        assert!(m.iter().any(|x| x.value == "Pilot"));
    }
}
