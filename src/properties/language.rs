//! Language detection — bracket/brace multi-language codes only.
//!
//! Simple language tokens are handled by `rules/language.toml`.
//! This module handles ONLY the custom parsing cases that TOML cannot express:
//! - Bracketed multi-language: `[ENG+RU+PT]`, `[ENG+DE+IT]`
//! - Brace-delimited codes: `{Fr-Eng}`, `{Fr-Eng}`

use crate::matcher::span::{MatchSpan, Property};
use regex::Regex;
use std::sync::LazyLock;

/// Matches bracketed multi-language codes: [ENG+RU+PT], [ENG+DE+IT].
static BRACKET_LANGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\[{]([A-Za-z]{2,4}(?:[+-][A-Za-z]{2,4})+)[\]}]").unwrap());

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    for cap in BRACKET_LANGS.find_iter(input) {
        let inner = &input[cap.start() + 1..cap.end() - 1];
        for code in inner.split(['+', '-']) {
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
        "IND" => Some("id"),
        "ST" => None, // "St" is subtitle marker, not a language
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn lang(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("language").map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                let strs: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                strs.join(", ")
            }
            _ => format!("{v}"),
        })
    }

    #[test]
    fn test_french() {
        assert_eq!(lang("Movie.FRENCH.DVDRip.mkv"), Some("French".into()));
    }

    #[test]
    fn test_english() {
        assert_eq!(lang("Movie.English.mkv"), Some("English".into()));
    }

    #[test]
    fn test_spanish() {
        assert_eq!(lang("Movie.Spanish.mkv"), Some("Spanish".into()));
    }

    #[test]
    fn test_multi() {
        assert_eq!(lang("Movie.MULTi.1080p.mkv"), Some("mul".into()));
    }
}
