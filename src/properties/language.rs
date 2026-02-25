//! Language detection.
//!
//! Detects language tags commonly found in media filenames.
//! While we don't fully implement guessit's language system (which uses
//! babelfish), we detect the most common language tokens to help
//! title extraction stop at the right place.

use regex::Regex;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static LANGUAGE_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
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
        ValuePattern::new(r"(?i)(?<![a-z])Espa[nñ]ol[. ]Castellano(?![a-z])", "Catalan"),
        ValuePattern::new(r"(?i)(?<![a-z])Espa[nñ]ol(?![a-z])", "Spanish"),
        ValuePattern::new(r"(?i)(?<![a-z])Castellano(?![a-z])", "Catalan"),
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
        ValuePattern::new(r"(?i)(?<![a-z])FLEMISH(?![a-z])", "nl"),
        ValuePattern::new(r"(?i)(?<![a-z])Ukr(?![a-z])", "Ukrainian"),
        ValuePattern::new(r"(?i)(?<![a-z])DUBLADO(?![a-z])", "und"),
        ValuePattern::new(r"(?i)(?<![a-z])Dual[. ]?Audio(?![a-z])", "und"),
        // DL = Dual Language / multilingual (but NOT inside WEB-DL).
        ValuePattern::new(r"(?i)(?<!WEB[-. ])(?<![a-z])DL(?![a-z])", "mul"),
    ]
});

/// Matches bracketed multi-language codes: [ENG+RU+PT], [ENG+DE+IT].
static BRACKET_LANGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([A-Za-z]{2,4}(?:[+][A-Za-z]{2,4})+)\]").unwrap());

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in LANGUAGE_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(
                MatchSpan::new(start, end, Property::Language, pattern.value).with_priority(-1),
            );
        }
    }

    // Parse bracketed multi-language codes: [ENG+RU+PT].
    for cap in BRACKET_LANGS.find_iter(input) {
        let inner = &input[cap.start() + 1..cap.end() - 1];
        for code in inner.split('+') {
            if let Some(lang) = lang_code_to_name(code) {
                matches.push(
                    MatchSpan::new(cap.start(), cap.end(), Property::Language, lang)
                        .with_priority(0),
                );
            }
        }
    }

    matches
}

/// Map common 2-3 letter language codes to language names.
fn lang_code_to_name(code: &str) -> Option<&'static str> {
    match code.to_uppercase().as_str() {
        "EN" | "ENG" => Some("English"),
        "FR" | "FRE" => Some("French"),
        "ES" | "SPA" => Some("Spanish"),
        "DE" | "GER" => Some("German"),
        "IT" | "ITA" => Some("Italian"),
        "PT" | "POR" => Some("Portuguese"),
        "RU" | "RUS" => Some("Russian"),
        "JA" | "JP" | "JPN" => Some("Japanese"),
        "ZH" | "CHI" => Some("Chinese"),
        "KO" | "KOR" => Some("Korean"),
        "AR" | "ARA" => Some("Arabic"),
        "HI" | "HIN" => Some("Hindi"),
        "NL" | "DUT" => Some("Dutch"),
        "SV" | "SWE" => Some("Swedish"),
        "NO" | "NOR" => Some("Norwegian"),
        "DA" | "DAN" => Some("Danish"),
        "FI" | "FIN" => Some("Finnish"),
        "PL" | "POL" => Some("Polish"),
        "CS" | "CZE" => Some("Czech"),
        "TR" | "TUR" => Some("Turkish"),
        "EL" | "GRE" => Some("Greek"),
        "HU" | "HUN" => Some("Hungarian"),
        "RO" | "ROM" => Some("Romanian"),
        "TH" | "THA" => Some("Thai"),
        "VI" | "VIE" => Some("Vietnamese"),
        "UK" | "UKR" => Some("Ukrainian"),
        "HE" | "HEB" => Some("Hebrew"),
        "HR" | "HRV" => Some("Croatian"),
        "SR" | "SRP" => Some("Serbian"),
        "BG" | "BUL" => Some("Bulgarian"),
        "CA" | "CAT" => Some("Catalan"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french() {
        let m = find_matches("Movie.FRENCH.DVDRip.mkv");
        assert!(m.iter().any(|x| x.value == "French"));
    }

    #[test]
    fn test_english() {
        let m = find_matches("Movie.English.mkv");
        assert!(m.iter().any(|x| x.value == "English"));
    }

    #[test]
    fn test_spanish() {
        let m = find_matches("Movie.Spanish.mkv");
        assert!(m.iter().any(|x| x.value == "Spanish"));
    }

    #[test]
    fn test_multi() {
        let m = find_matches("Movie.MULTi.1080p.mkv");
        assert!(m.iter().any(|x| x.value == "mul"));
    }
}
