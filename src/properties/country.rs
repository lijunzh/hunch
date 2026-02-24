//! Country detection.
//!
//! Detects country codes/names in media filenames.

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static COUNTRY_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        ValuePattern::new(r"(?<![a-zA-Z0-9])US(?![a-zA-Z0-9])", "US"),
        ValuePattern::new(r"(?<![a-zA-Z0-9])UK(?![a-zA-Z0-9])", "GB"),
        ValuePattern::new(r"(?<![a-zA-Z0-9])GB(?![a-zA-Z0-9])", "GB"),
        ValuePattern::new(r"(?<![a-zA-Z0-9])CA(?![a-zA-Z0-9])", "CA"),
        ValuePattern::new(r"(?<![a-zA-Z0-9])AU(?![a-zA-Z0-9])", "AU"),
        ValuePattern::new(r"(?<![a-zA-Z0-9])NZ(?![a-zA-Z0-9])", "NZ"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in COUNTRY_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(
                MatchSpan::new(start, end, Property::Country, pattern.value).with_priority(-2),
            );
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us() {
        let m = find_matches("The Office (US) S01E01.mkv");
        assert!(m.iter().any(|x| x.value == "US"));
    }
}
