//! Audio profile detection.
//!
//! Detects audio encoding profiles: HD, HD-MA, HE, HRA, etc.

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static AUDIO_PROFILE_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        // DTS variants (most specific first)
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DTS[-. ]?)?HD[-. ]?(?:Master[-. ]?Audio|MA)(?![a-z])",
            "Master Audio",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DTS[-. ]?)?HD[-. ]?HRA?(?![a-z])",
            "High Resolution Audio",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DTS[-. ]?)?ES(?:[-. ]?(?:Matrix|Discrete))?(?![a-z])",
            "Extended Surround",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])DTS[-. ]?X(?!264|265|[0-9])(?![a-z])",
            "DTS:X",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-. ]?EX(?![a-z])", "EX"),
        // Dolby variants
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DD|Dolby[-. ]?Digital)?[-. ]?(?:Atmos|ATMOS)(?![a-z])",
            "Atmos",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DD|Dolby[-. ]?Digital)?\+(?![a-z0-9])",
            "Dolby Digital Plus",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DDP|DD[Pp]lus|EAC3)(?![a-z])",
            "Dolby Digital Plus",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])TrueHD(?![a-z])", "TrueHD"),
        // AAC variants
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:AAC[-. ]?)?(?:HE|High[-. ]?Efficiency)(?![a-z])",
            "High Efficiency",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])(?:AAC[-. ]?)?LC(?![a-z])", "Low Complexity"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:AAC[-. ]?)?HQ(?![a-z])", "High Quality"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in AUDIO_PROFILE_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(
                MatchSpan::new(start, end, Property::AudioProfile, pattern.value).with_priority(1),
            );
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hd_ma() {
        let m = find_matches("Movie.DTS-HD.MA.mkv");
        assert!(m.iter().any(|x| x.value == "Master Audio"));
    }

    #[test]
    fn test_atmos() {
        let m = find_matches("Movie.Atmos.mkv");
        assert!(m.iter().any(|x| x.value == "Atmos"));
    }

    #[test]
    fn test_truehd() {
        let m = find_matches("Movie.TrueHD.mkv");
        assert!(m.iter().any(|x| x.value == "TrueHD"));
    }
}
