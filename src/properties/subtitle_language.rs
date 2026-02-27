//! Subtitle language detection — algorithmic patterns only.
//!
//! Simple vocabulary-based markers are handled by `rules/subtitle_language.toml`.
//! This module handles patterns that require algorithmic/positional logic:
//! - File extensions: movie.eng.srt
//! - SUBFORCED with optional language prefix
//! - LANG SUBS: "English Subs", "German.Subbed" (full language names)
//! - Sub.French / ST(Fr-Eng) (multi-language bracket parsing)
//! - Legendado/Subtitulado (regional conventions)

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static ALPHA_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::Alpha),
    right: Some(CharClass::Alpha),
};

/// Subtitle file with language code: movie.eng.srt, movie.fr.sub
static SUB_LANG_EXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\.(?P<lang>[a-z]{2,3})\.(srt|sub|ass|ssa|idx|smi|vtt|sup)$").unwrap()
});

/// SubForced / SUBFORCED
static SUB_FORCED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:(?P<lang>FRENCH|ENGLISH|SPANISH|GERMAN|ITALIAN)\s+)?SUBFORCED").unwrap()
});

/// LANG SubForced pattern (reversed order)
static LANG_SUBFORCED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)SUBFORCED\s+(?P<lang>FRENCH|ENGLISH|SPANISH|GERMAN|ITALIAN)").unwrap()
});

/// Explicit: Sub.French, sub FR, ST(Fr-Eng), Sub_ITA
static SUB_LANG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?:(?:sub(?:s|titled|titles)?|Soft[-. ]?sub)[-. _({\[]?|ST[-. _({\[])(?P<langs>[a-z]{2,}(?:[-. _+,)}&\]]+[a-z]{2,})*)"
    ).unwrap()
});

/// LANG SUBS: ENG SUBS, SPANISH SUBBED, German.Subbed
static LANG_SUBS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?P<lang>English|French|Spanish|German|Italian|Portuguese|Dutch|Swedish|Norwegian|Danish|Finnish|Greek|Turkish|Arabic|Russian|Hindi|Chinese|Japanese|Korean|Hebrew|Romanian|Polish|Czech|Hungarian|Croatian|Serbian|Slovak|Slovenian|Estonian|Latvian|Lithuanian|Catalan|Eng|Fre|Spa|Ger|Ita|Por|Dut|Swe|Nor|Dan|Fin|Gre|Tur|Ara|Rus|Hin|Chi|Jpn|Kor|Heb|Ron|Pol|Cze|Hun|Hrv|Srp|Slk|Slv|Est|Lav|Lit|Cat)[-. ]+(?:(?:Soft|Custom|Hard|Forced)[-. ])*(?:sub(?:s|bed|titled|titles)?)"
    ).unwrap()
});

/// Legendado/Legendas
static LEGENDADO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Legenda(?:do|s|))(?:\.(?P<lang>PT(?:-BR)?|EN|ES|FR))?").unwrap()
});

/// Subtitulado
static SUBTITULADO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Subtitulado(?:\s+(?P<lang>Espa[\u{f1}n]ol|Spanish|PT|EN|FR))?").unwrap()
});

/// Maps ISO 639 codes & full names to guessit-style language output.
fn normalize_language(code: &str) -> Option<&'static str> {
    match code.to_lowercase().as_str() {
        "en" | "eng" | "english" => Some("en"),
        "fr" | "fre" | "fra" | "french" | "vff" | "vf" | "vfq" | "vostfr" | "truefrench" => {
            Some("fr")
        }
        "es" | "spa" | "spanish" | "espanol" | "esp" | "espa\u{f1}ol" => Some("es"),
        "de" | "ger" | "deu" | "german" | "deutsch" => Some("deu"),
        "it" | "ita" | "italian" | "italiano" => Some("it"),
        "pt" | "por" | "portuguese" | "portugues" => Some("pt"),
        "pt-br" | "br" => Some("pt-BR"),
        "ja" | "jpn" | "japanese" => Some("ja"),
        "ko" | "kor" | "korean" => Some("ko"),
        "zh" | "chi" | "zho" | "chinese" => Some("zh"),
        "ru" | "rus" | "russian" => Some("ru"),
        "ar" | "ara" | "arabic" => Some("ar"),
        "hi" | "hin" | "hindi" => Some("hi"),
        "nl" | "dut" | "nld" | "dutch" => Some("nl"),
        "pl" | "pol" | "polish" => Some("pl"),
        "sv" | "swe" | "swedish" => Some("sv"),
        "no" | "nor" | "norwegian" => Some("no"),
        "da" | "dan" | "danish" => Some("da"),
        "fi" | "fin" | "finnish" => Some("fi"),
        "hu" | "hun" | "hungarian" => Some("hu"),
        "cs" | "cze" | "ces" | "czech" => Some("cs"),
        "ro" | "rum" | "ron" | "romanian" => Some("ro"),
        "el" | "gre" | "ell" | "greek" => Some("el"),
        "tr" | "tur" | "turkish" => Some("tr"),
        "th" | "tha" | "thai" => Some("th"),
        "vi" | "vie" | "vietnamese" => Some("vi"),
        "he" | "heb" | "hebrew" => Some("he"),
        "id" | "ind" | "indonesian" => Some("id"),
        "ms" | "may" | "msa" | "malay" => Some("ms"),
        "uk" | "ukr" | "ukrainian" => Some("uk"),
        "bg" | "bul" | "bulgarian" => Some("bg"),
        "hr" | "hrv" | "croatian" => Some("hr"),
        "sr" | "srp" | "serbian" => Some("sr"),
        "sk" | "slo" | "slk" | "slovak" => Some("sk"),
        "sl" | "slv" | "slovenian" => Some("sl"),
        "et" | "est" | "estonian" => Some("et"),
        "lv" | "lav" | "latvian" => Some("lv"),
        "lt" | "lit" | "lithuanian" => Some("lt"),
        "ca" | "cat" | "catalan" => Some("ca"),
        "eu" | "baq" | "eus" | "basque" => Some("eu"),
        "gl" | "glg" | "galician" => Some("gl"),
        "multi" | "mul" | "multiple" => Some("mul"),
        "und" | "undetermined" => Some("und"),
        _ => None,
    }
}

/// Split multi-language strings like "Fr-Eng" or "ita.eng" into parts.
fn split_languages(s: &str) -> Vec<&str> {
    s.split(|c: char| "-.,+_)(&] ".contains(c))
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let b = &ALPHA_BOUNDARY;
    let mut matches = Vec::new();

    // 1. Subtitle file extension: movie.eng.srt
    if let Some(cap) = SUB_LANG_EXT.captures(input)
        && let Some(lang) = cap.name("lang")
        && let Some(normalized) = normalize_language(lang.as_str())
    {
        matches.push(
            MatchSpan::new(
                lang.start(),
                lang.end(),
                Property::SubtitleLanguage,
                normalized,
            )
            .with_priority(2),
        );
    }

    // 2. SUBFORCED with language
    if let Some(cap) = SUB_FORCED.captures(input) {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b)
            && let Some(lang) = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
        {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(2),
            );
        }
    }
    if let Some(cap) = LANG_SUBFORCED.captures(input) {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b)
            && let Some(lang) = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
            && !matches.iter().any(|m| {
                m.overlaps(&MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::SubtitleLanguage,
                    "",
                ))
            })
        {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(2),
            );
        }
    }

    // 3. LANG SUBS: English Subs, German.Subbed, SPANISH SUBBED
    if let Some(cap) = LANG_SUBS.captures(input)
        && let Some(lang) = cap
            .name("lang")
            .and_then(|l| normalize_language(l.as_str()))
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b) {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(2),
            );
        }
    }

    // 4. Sub.French, Sub_ITA, ST(Fr-Eng)
    if let Some(cap) = SUB_LANG.captures(input)
        && let Some(langs) = cap.name("langs")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b)
            && !matches.iter().any(|m| {
                m.overlaps(&MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::SubtitleLanguage,
                    "",
                ))
            })
        {
            let parts = split_languages(langs.as_str());
            for part in parts {
                if let Some(normalized) = normalize_language(part) {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::SubtitleLanguage,
                            normalized,
                        )
                        .with_priority(1),
                    );
                }
            }
        }
    }

    // 5. Legendado/Legendas with optional language
    if matches.is_empty()
        && let Some(cap) = LEGENDADO.captures(input)
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b) {
            let lang = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
                .unwrap_or("und");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(0),
            );
        }
    }

    // 6. Subtitulado with optional language
    if matches.is_empty()
        && let Some(cap) = SUBTITULADO.captures(input)
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), b) {
            let lang = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
                .unwrap_or("und");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(0),
            );
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_extension() {
        let m = find_matches("movie.eng.srt");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "en");
    }

    #[test]
    fn test_vostfr() {
        // VOSTFR handled by TOML now; test through pipeline
        use crate::hunch;
        let r = hunch("One.Piece.E576.VOSTFR.720p.mkv");
        assert!(r.all(Property::SubtitleLanguage).iter().any(|v| v == &"fr"));
    }

    #[test]
    fn test_eng_subs() {
        let m = find_matches("Movie [ENG SUBS].mkv");
        assert!(m.iter().any(|x| x.value == "en"));
    }

    #[test]
    fn test_hebsubs() {
        // HebSubs handled by TOML now; test through pipeline
        use crate::hunch;
        let r = hunch("Show.S01E01.HDTV.HebSubs.mkv");
        assert!(
            r.all(Property::SubtitleLanguage)
                .iter()
                .any(|v| v.to_lowercase().contains("he"))
        );
    }

    #[test]
    fn test_swesub() {
        // SWESUB handled by TOML now; test through pipeline
        use crate::hunch;
        let r = hunch("Show.S06E16.HC.SWESUB.HDTV.x264");
        assert!(
            r.all(Property::SubtitleLanguage)
                .iter()
                .any(|v| v.to_lowercase().contains("sw"))
        );
    }
}
