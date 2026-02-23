//! Language detection.
//!
//! Detects language tags commonly found in media filenames.
//! While we don't fully implement guessit's language system (which uses
//! babelfish), we detect the most common language tokens to help
//! title extraction stop at the right place.

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref LANGUAGE_PATTERNS: Vec<ValuePattern> = vec![
        // Full names (case-insensitive, word-bounded).
        ValuePattern::new(r"(?i)(?<![a-z])English(?![a-z])", "English"),
        ValuePattern::new(r"(?i)(?<![a-z])French(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])Spanish(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])German(?![a-z])", "German"),
        ValuePattern::new(r"(?i)(?<![a-z])Italian(?![a-z])", "Italian"),
        ValuePattern::new(r"(?i)(?<![a-z])Portuguese(?![a-z])", "Portuguese"),
        ValuePattern::new(r"(?i)(?<![a-z])Russian(?![a-z])", "Russian"),
        ValuePattern::new(r"(?i)(?<![a-z])Japanese(?![a-z])", "Japanese"),
        ValuePattern::new(r"(?i)(?<![a-z])Chinese(?![a-z])", "Chinese"),
        ValuePattern::new(r"(?i)(?<![a-z])Korean(?![a-z])", "Korean"),
        ValuePattern::new(r"(?i)(?<![a-z])Arabic(?![a-z])", "Arabic"),
        ValuePattern::new(r"(?i)(?<![a-z])Hindi(?![a-z])", "Hindi"),
        ValuePattern::new(r"(?i)(?<![a-z])Dutch(?![a-z])", "Dutch"),
        ValuePattern::new(r"(?i)(?<![a-z])Swedish(?![a-z])", "Swedish"),
        ValuePattern::new(r"(?i)(?<![a-z])Norwegian(?![a-z])", "Norwegian"),
        ValuePattern::new(r"(?i)(?<![a-z])Danish(?![a-z])", "Danish"),
        ValuePattern::new(r"(?i)(?<![a-z])Finnish(?![a-z])", "Finnish"),
        ValuePattern::new(r"(?i)(?<![a-z])Polish(?![a-z])", "Polish"),
        ValuePattern::new(r"(?i)(?<![a-z])Czech(?![a-z])", "Czech"),
        ValuePattern::new(r"(?i)(?<![a-z])Turkish(?![a-z])", "Turkish"),
        ValuePattern::new(r"(?i)(?<![a-z])Greek(?![a-z])", "Greek"),
        ValuePattern::new(r"(?i)(?<![a-z])Hungarian(?![a-z])", "Hungarian"),
        ValuePattern::new(r"(?i)(?<![a-z])Romanian(?![a-z])", "Romanian"),
        ValuePattern::new(r"(?i)(?<![a-z])Thai(?![a-z])", "Thai"),
        ValuePattern::new(r"(?i)(?<![a-z])Vietnamese(?![a-z])", "Vietnamese"),
        ValuePattern::new(r"(?i)(?<![a-z])Catalan(?![a-z])", "Catalan"),
        ValuePattern::new(r"(?i)(?<![a-z])Croatian(?![a-z])", "Croatian"),
        ValuePattern::new(r"(?i)(?<![a-z])Serbian(?![a-z])", "Serbian"),
        ValuePattern::new(r"(?i)(?<![a-z])Bulgarian(?![a-z])", "Bulgarian"),
        ValuePattern::new(r"(?i)(?<![a-z])Ukrainian(?![a-z])", "Ukrainian"),
        ValuePattern::new(r"(?i)(?<![a-z])Hebrew(?![a-z])", "Hebrew"),
        // Localized language names.
        ValuePattern::new(r"(?i)(?<![a-z])Fran[cç]ais(?:e)?(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])Espa[nñ]ol(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])Castellano(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])Deutsch(?![a-z])", "German"),
        ValuePattern::new(r"(?i)(?<![a-z])Italiano(?![a-z])", "Italian"),
        ValuePattern::new(r"(?i)(?<![a-z])Portugu[eê]s(?![a-z])", "Portuguese"),
        ValuePattern::new(r"(?i)(?<![a-z])Vostfr(?![a-z])", "French"),
        // Common abbreviation tags.
        ValuePattern::new(r"(?i)(?<![a-z])FRENCH(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])TRUEFRENCH(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])VFF(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])VFQ(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])VFI(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])VF2(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])VF(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])SPANISH(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])GERMAN(?![a-z])", "German"),
        ValuePattern::new(r"(?i)(?<![a-z])ITALIAN(?![a-z])", "Italian"),
        ValuePattern::new(r"(?i)(?<![a-z])LATINO(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])MULTI(?:LANG(?:UAGE)?)?(?![a-z])", "mul"),
        ValuePattern::new(r"(?i)(?<![a-z])ENG(?![a-z])", "English"),
        ValuePattern::new(r"(?i)(?<![a-z])ITA(?![a-z])", "Italian"),
        ValuePattern::new(r"(?i)(?<![a-z])SPA(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])GER(?![a-z])", "German"),
        ValuePattern::new(r"(?i)(?<![a-z])FRE(?![a-z])", "French"),
        ValuePattern::new(r"(?i)(?<![a-z])JPN(?![a-z])", "Japanese"),
        ValuePattern::new(r"(?i)(?<![a-z])RUS(?![a-z])", "Russian"),
        ValuePattern::new(r"(?i)(?<![a-z])KOR(?![a-z])", "Korean"),
    ];
}

pub struct LanguageMatcher;

impl PropertyMatcher for LanguageMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in LANGUAGE_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(
                    MatchSpan::new(start, end, Property::Language, pattern.value)
                        .with_priority(-1),
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
    fn test_french() {
        let m = LanguageMatcher.find_matches("Movie.FRENCH.DVDRip.mkv");
        assert!(m.iter().any(|x| x.value == "French"));
    }

    #[test]
    fn test_english() {
        let m = LanguageMatcher.find_matches("Movie.English.mkv");
        assert!(m.iter().any(|x| x.value == "English"));
    }

    #[test]
    fn test_spanish() {
        let m = LanguageMatcher.find_matches("Movie.Spanish.mkv");
        assert!(m.iter().any(|x| x.value == "Spanish"));
    }

    #[test]
    fn test_multi() {
        let m = LanguageMatcher.find_matches("Movie.MULTi.1080p.mkv");
        assert!(m.iter().any(|x| x.value == "mul"));
    }
}
