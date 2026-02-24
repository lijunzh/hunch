//! Year detection (4-digit years in a reasonable range).

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

const MIN_YEAR: i32 = 1920;
const MAX_YEAR: i32 = 2030;

lazy_static! {
    static ref YEAR_RE: ValuePattern = ValuePattern::new(
        r"(?<![0-9])(?:19|20)\d{2}(?![0-9])",
        "",  // value computed dynamically
    );
}

pub struct YearMatcher;

impl PropertyMatcher for YearMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for (start, end) in YEAR_RE.find_iter(input) {
            let raw = &input[start..end];
            if let Ok(year) = raw.parse::<i32>()
                && (MIN_YEAR..=MAX_YEAR).contains(&year)
            {
                // Skip years that are part of technical terms (BT2020, x1920, etc.).
                if start > 0 {
                    let prev = input.as_bytes()[start - 1];
                    if prev.is_ascii_alphabetic() {
                        continue;
                    }
                }
                // Skip "1920x1080" and similar resolution patterns.
                if end < input.len() {
                    let next = input.as_bytes()[end];
                    if next == b'x' || next == b'X' {
                        continue;
                    }
                }
                matches.push(MatchSpan::new(start, end, Property::Year, raw).with_priority(-1));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_found() {
        let m = YearMatcher.find_matches("The Matrix 1999 1080p");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1999");
    }

    #[test]
    fn test_year_2024() {
        let m = YearMatcher.find_matches("Movie.2024.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2024");
    }

    #[test]
    fn test_no_year_in_codec() {
        let m = YearMatcher.find_matches("Movie.x264.mkv");
        assert!(m.is_empty());
    }

    #[test]
    fn test_year_too_old() {
        let m = YearMatcher.find_matches("Movie.1800.mkv");
        assert!(m.is_empty());
    }
}
