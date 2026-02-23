//! Edition detection (Director's Cut, Extended, Unrated, etc.).

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref EDITION_PATTERNS: Vec<ValuePattern> = vec![
        ValuePattern::new(r"(?i)(?<![a-z])Director'?s?[-.]?Cut(?![a-z])", "Director's Cut"),
        ValuePattern::new(r"(?i)(?<![a-z])Extended(?:[-.]?(?:Cut|Edition))?(?![a-z])", "Extended"),
        ValuePattern::new(r"(?i)(?<![a-z])Unrated(?:[-.]?(?:Cut|Edition))?(?![a-z])", "Unrated"),
        ValuePattern::new(r"(?i)(?<![a-z])Theatrical(?:[-.]?(?:Cut|Edition))?(?![a-z])", "Theatrical"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Special|Collector'?s?)[-.]?Edition(?![a-z])", "Special Edition"),
        ValuePattern::new(r"(?i)(?<![a-z])Ultimate[-.]?Edition(?![a-z])", "Ultimate Edition"),
        ValuePattern::new(r"(?i)(?<![a-z])Anniversary[-.]?Edition(?![a-z])", "Anniversary Edition"),
        ValuePattern::new(r"(?i)(?<![a-z])Criterion[-.]?(?:Collection|Edition)?(?![a-z])", "Criterion"),
        ValuePattern::new(r"(?i)(?<![a-z])IMAX(?:[-.]?Edition)?(?![a-z])", "IMAX"),
        ValuePattern::new(r"(?i)(?<![a-z])Fan[-.]?Edit(?![a-z])", "Fan Edit"),
        ValuePattern::new(r"(?i)(?<![a-z])Remastered(?![a-z])", "Remastered"),
        ValuePattern::new(r"(?i)(?<![a-z])Restored(?![a-z])", "Restored"),
        ValuePattern::new(r"(?i)(?<![a-z])Uncensored(?![a-z])", "Uncensored"),
    ];
}

pub struct EditionMatcher;

impl PropertyMatcher for EditionMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in EDITION_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(start, end, Property::Edition, pattern.value));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directors_cut() {
        let m = EditionMatcher.find_matches("Movie.Directors.Cut.mkv");
        assert!(m.iter().any(|x| x.value == "Director's Cut"));
    }

    #[test]
    fn test_extended() {
        let m = EditionMatcher.find_matches("Movie.Extended.Edition.mkv");
        assert!(m.iter().any(|x| x.value == "Extended"));
    }

    #[test]
    fn test_imax() {
        let m = EditionMatcher.find_matches("Movie.IMAX.mkv");
        assert!(m.iter().any(|x| x.value == "IMAX"));
    }

    #[test]
    fn test_remastered() {
        let m = EditionMatcher.find_matches("Movie.Remastered.mkv");
        assert!(m.iter().any(|x| x.value == "Remastered"));
    }
}
