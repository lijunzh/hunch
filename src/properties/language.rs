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
        ValuePattern::new(r"(?i)(?<![a-z])English(?![a-z])", "en"),
        ValuePattern::new(r"(?i)(?<![a-z])French(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])Spanish(?![a-z])", "es"),
        ValuePattern::new(r"(?i)(?<![a-z])German(?![a-z])", "de"),
        ValuePattern::new(r"(?i)(?<![a-z])Italian(?![a-z])", "it"),
        ValuePattern::new(r"(?i)(?<![a-z])Portuguese(?![a-z])", "pt"),
        ValuePattern::new(r"(?i)(?<![a-z])Russian(?![a-z])", "ru"),
        ValuePattern::new(r"(?i)(?<![a-z])Japanese(?![a-z])", "ja"),
        ValuePattern::new(r"(?i)(?<![a-z])Chinese(?![a-z])", "zh"),
        ValuePattern::new(r"(?i)(?<![a-z])Korean(?![a-z])", "ko"),
        ValuePattern::new(r"(?i)(?<![a-z])Arabic(?![a-z])", "ar"),
        ValuePattern::new(r"(?i)(?<![a-z])Hindi(?![a-z])", "hi"),
        ValuePattern::new(r"(?i)(?<![a-z])Dutch(?![a-z])", "nl"),
        ValuePattern::new(r"(?i)(?<![a-z])Swedish(?![a-z])", "sv"),
        ValuePattern::new(r"(?i)(?<![a-z])Norwegian(?![a-z])", "no"),
        ValuePattern::new(r"(?i)(?<![a-z])Danish(?![a-z])", "da"),
        ValuePattern::new(r"(?i)(?<![a-z])Finnish(?![a-z])", "fi"),
        ValuePattern::new(r"(?i)(?<![a-z])Polish(?![a-z])", "pl"),
        ValuePattern::new(r"(?i)(?<![a-z])Czech(?![a-z])", "cs"),
        ValuePattern::new(r"(?i)(?<![a-z])Turkish(?![a-z])", "tr"),
        ValuePattern::new(r"(?i)(?<![a-z])Greek(?![a-z])", "el"),
        ValuePattern::new(r"(?i)(?<![a-z])Hungarian(?![a-z])", "hu"),
        ValuePattern::new(r"(?i)(?<![a-z])Romanian(?![a-z])", "ro"),
        ValuePattern::new(r"(?i)(?<![a-z])Thai(?![a-z])", "th"),
        ValuePattern::new(r"(?i)(?<![a-z])Vietnamese(?![a-z])", "vi"),
        ValuePattern::new(r"(?i)(?<![a-z])Catalan(?![a-z])", "ca"),
        ValuePattern::new(r"(?i)(?<![a-z])Croatian(?![a-z])", "hr"),
        ValuePattern::new(r"(?i)(?<![a-z])Serbian(?![a-z])", "sr"),
        ValuePattern::new(r"(?i)(?<![a-z])Bulgarian(?![a-z])", "bg"),
        ValuePattern::new(r"(?i)(?<![a-z])Ukrainian(?![a-z])", "uk"),
        ValuePattern::new(r"(?i)(?<![a-z])Hebrew(?![a-z])", "he"),
        ValuePattern::new(r"(?i)(?<![a-z])Dubbed(?![a-z])", "dubbed"),
        // Localized language names.
        ValuePattern::new(r"(?i)(?<![a-z])Fran[cç]ais(?:e)?(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])Espa[nñ]ol(?![a-z])", "es"),
        ValuePattern::new(r"(?i)(?<![a-z])Castellano(?![a-z])", "es"),
        ValuePattern::new(r"(?i)(?<![a-z])Deutsch(?![a-z])", "de"),
        ValuePattern::new(r"(?i)(?<![a-z])Italiano(?![a-z])", "it"),
        ValuePattern::new(r"(?i)(?<![a-z])Portugu[eê]s(?![a-z])", "pt"),
        ValuePattern::new(r"(?i)(?<![a-z])Vostfr(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VOST(?![a-z])", "und"),
        ValuePattern::new(r"(?i)(?<![a-z])VO(?:ST)?(?![a-z])", "und"),
        // Common abbreviation tags (only in context of media filenames).
        ValuePattern::new(r"(?i)(?<![a-z])FRENCH(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])TRUEFRENCH(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VFF(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VFQ(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VFI(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VF2(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])VF(?![a-z])", "fr"),
        ValuePattern::new(r"(?i)(?<![a-z])SPANISH(?![a-z])", "es"),
        ValuePattern::new(r"(?i)(?<![a-z])GERMAN(?![a-z])", "de"),
        ValuePattern::new(r"(?i)(?<![a-z])ITALIAN(?![a-z])", "it"),
        ValuePattern::new(r"(?i)(?<![a-z])LATINO(?![a-z])", "es"),
        ValuePattern::new(r"(?i)(?<![a-z])MULTI(?:LANG(?:UAGE)?)?(?![a-z])", "mul"),
        // Two-letter ISO codes (only uppercase, to avoid false positives).
        ValuePattern::new(r"(?<![a-zA-Z])(?:Fr|FR)(?![a-zA-Z])", "fr"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:En|EN)(?![a-zA-Z])", "en"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:De|DE)(?![a-zA-Z])", "de"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Es|ES)(?![a-zA-Z])", "es"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:It|IT)(?![a-zA-Z])", "it"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Pt|PT)(?![a-zA-Z])", "pt"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Ja|JA)(?![a-zA-Z])", "ja"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Ru|RU)(?![a-zA-Z])", "ru"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Nl|NL)(?![a-zA-Z])", "nl"),
        ValuePattern::new(r"(?<![a-zA-Z])(?:Ko|KO)(?![a-zA-Z])", "ko"),
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
        assert!(m.iter().any(|x| x.value == "fr"));
    }

    #[test]
    fn test_english() {
        let m = LanguageMatcher.find_matches("Movie.English.mkv");
        assert!(m.iter().any(|x| x.value == "en"));
    }

    #[test]
    fn test_spanish() {
        let m = LanguageMatcher.find_matches("Movie.Spanish.mkv");
        assert!(m.iter().any(|x| x.value == "es"));
    }

    #[test]
    fn test_multi() {
        let m = LanguageMatcher.find_matches("Movie.MULTi.1080p.mkv");
        assert!(m.iter().any(|x| x.value == "mul"));
    }
}
