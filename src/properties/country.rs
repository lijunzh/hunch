//! Country detection.
//!
//! Detects country codes/names in media filenames.

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref COUNTRY_PATTERNS: Vec<ValuePattern> = vec![
        ValuePattern::new(r"(?<![a-zA-Z])US(?![a-zA-Z])", "US"),
        ValuePattern::new(r"(?<![a-zA-Z])UK(?![a-zA-Z])", "GB"),
        ValuePattern::new(r"(?<![a-zA-Z])GB(?![a-zA-Z])", "GB"),
        ValuePattern::new(r"(?<![a-zA-Z])CA(?![a-zA-Z])", "CA"),
        ValuePattern::new(r"(?<![a-zA-Z])AU(?![a-zA-Z])", "AU"),
        ValuePattern::new(r"(?<![a-zA-Z])NZ(?![a-zA-Z])", "NZ"),
    ];
}

pub struct CountryMatcher;

impl PropertyMatcher for CountryMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in COUNTRY_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(
                    MatchSpan::new(start, end, Property::Country, pattern.value)
                        .with_priority(-2),
                );
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us() {
        let m = CountryMatcher.find_matches("The Office (US) S01E01.mkv");
        assert!(m.iter().any(|x| x.value == "US"));
    }
}
