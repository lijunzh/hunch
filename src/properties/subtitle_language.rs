//! Subtitle language detection.
//!
//! Detects subtitle language tags from a wide variety of patterns:
//! - File extensions: movie.eng.srt
//! - VOSTFR/FASTSUB conventions (French anime/TV)
//! - Explicit markers: Sub.French, ENG SUBS, etc.
//! - Compound markers: HebSubs, SWESUB, NLsubs, etc.
//! - Generic sub markers: Subbed, Legendado, Subtitles

use fancy_regex::Regex;
use lazy_static::lazy_static;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Subtitle file with language code: movie.eng.srt, movie.fr.sub
    static ref SUB_LANG_EXT: Regex = Regex::new(
        r"(?i)\.(?P<lang>[a-z]{2,3})\.(srt|sub|ass|ssa|idx|smi|vtt|sup)$"
    ).unwrap();

    /// VOSTFR / VOST.FR / vostfr → French subtitles
    static ref VOSTFR: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:VOSTFR|FASTSUB(?:\.VOSTFR)?)(?![a-z])"
    ).unwrap();

    /// SubForced / SUBFORCED (language from nearby context)
    static ref SUB_FORCED: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:(?P<lang>FRENCH|ENGLISH|SPANISH|GERMAN|ITALIAN)\s+)?SUBFORCED(?![a-z])"
    ).unwrap();

    /// LANG SubForced pattern (reversed order)
    static ref LANG_SUBFORCED: Regex = Regex::new(
        r"(?i)(?<![a-z])SUBFORCED\s+(?P<lang>FRENCH|ENGLISH|SPANISH|GERMAN|ITALIAN)(?![a-z])"
    ).unwrap();

    /// Explicit: Sub.French, sub FR, ST(Fr-Eng), Sub_ITA, sub.ita.eng
    static ref SUB_LANG: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:sub(?:s|titled|titles)?|ST|Soft[-. ]?sub)[-. _({\[]?(?P<langs>[a-z]{2,}(?:[-. _+,)}&\]]+[a-z]{2,})*)(?![a-z])"
    ).unwrap();

    /// Compound sub markers: HebSubs, SWESUB, NLsubs, Nlsubs, PLsub
    static ref COMPOUND_SUB: Regex = Regex::new(
        r"(?i)(?<![a-z])(?P<lang>Heb|Swe|Nl|Pl|Ro|De|Kor|Eng|Fre|Ita|Spa|Dan|Nor|Fin|Gre|Tur|Ara|Rus|Hin|Chi|Jpn|Ukr|Bul|Hun|Cze|Hrv|Slk|Slv|Est|Lav|Lit|Cat|Pt|Br)[-. ]?(?:sub(?:s|bed|titles?)?)(?![a-z])"
    ).unwrap();

    /// LANG SUBS pattern: ENG SUBS, SPANISH SUBBED, German.Subbed, German.Custom.Subbed
    static ref LANG_SUBS: Regex = Regex::new(
        r"(?i)(?<![a-z])(?P<lang>English|French|Spanish|German|Italian|Portuguese|Dutch|Swedish|Norwegian|Danish|Finnish|Greek|Turkish|Arabic|Russian|Hindi|Chinese|Japanese|Korean|Hebrew|Romanian|Polish|Czech|Hungarian|Croatian|Serbian|Slovak|Slovenian|Estonian|Latvian|Lithuanian|Catalan|Eng|Fre|Spa|Ger|Ita|Por|Dut|Swe|Nor|Dan|Fin|Gre|Tur|Ara|Rus|Hin|Chi|Jpn|Kor|Heb|Ron|Pol|Cze|Hun|Hrv|Srp|Slk|Slv|Est|Lav|Lit|Cat)[-. ]+(?:(?:Soft|Custom|Hard|Forced)[-. ])*(?:sub(?:s|bed|titled|titles)?)(?![a-z])"
    ).unwrap();

    /// EN-SUB, EN.SUB pattern
    static ref LANG_DASH_SUB: Regex = Regex::new(
        r"(?i)(?<![a-z])(?P<lang>[a-z]{2,3})[-.]SUB(?:S)?(?![a-z])"
    ).unwrap();

    /// Legendado/Legendas → undetermined, unless followed by PT/PT-BR
    static ref LEGENDADO: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:Legenda(?:do|s|))(?:\.(?P<lang>PT(?:-BR)?|EN|ES|FR))?(?![a-z])"
    ).unwrap();

    /// Subtitulado → Spanish subtitle convention, may have language after it
    static ref SUBTITULADO: Regex = Regex::new(
        r"(?i)(?<![a-z])Subtitulado(?:\s+(?P<lang>Espa[ñn]ol|Spanish|PT|EN|FR))?(?![a-z])"
    ).unwrap();

    /// Generic sub markers without language context: ESub, subs, subbed, Subtitles
    static ref GENERIC_SUB: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:E[-. ]?Sub(?:s|bed|titles?)?|Sub(?:s|bed|titles)?|HC)(?![a-z])"
    ).unwrap();

    /// Multiple Subtitle marker
    static ref MULTIPLE_SUB: Regex = Regex::new(
        r"(?i)(?:Multiple|Multi)\s+Sub(?:s|title|titles)?(?![a-z])"
    ).unwrap();
}

/// Maps ISO 639 codes & full names to guessit-style language output.
fn normalize_language(code: &str) -> Option<&'static str> {
    match code.to_lowercase().as_str() {
        "en" | "eng" | "english" => Some("en"),
        "fr" | "fre" | "fra" | "french" | "vff" | "vf" | "vfq" | "vostfr" | "truefrench" => {
            Some("fr")
        }
        "es" | "spa" | "spanish" | "espanol" | "esp" | "español" => Some("es"),
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
    // Split on -,.+_)(&] separators.
    s.split(|c: char| "-.,+_)(&] ".contains(c))
        .filter(|s| !s.is_empty())
        .collect()
}

pub struct SubtitleLanguageMatcher;

impl PropertyMatcher for SubtitleLanguageMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // 1. Subtitle file extension: movie.eng.srt
        if let Ok(Some(cap)) = SUB_LANG_EXT.captures(input)
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

        // 2. VOSTFR/FASTSUB → French
        if let Ok(Some(cap)) = VOSTFR.captures(input) {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, "fr")
                    .with_priority(2),
            );
        }

        // 3. SUBFORCED with language
        if let Ok(Some(cap)) = SUB_FORCED.captures(input) {
            let full = cap.get(0).unwrap();
            let lang = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()));
            if let Some(lang) = lang {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                        .with_priority(2),
                );
            }
        }
        if let Ok(Some(cap)) = LANG_SUBFORCED.captures(input) {
            let full = cap.get(0).unwrap();
            if let Some(lang) = cap
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

        // 4. LANG SUBS: English Subs, German.Subbed, SPANISH SUBBED
        if let Ok(Some(cap)) = LANG_SUBS.captures(input)
            && let Some(lang) = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
        {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(2),
            );
        }

        // 5. Compound: HebSubs, SWESUB, NLsubs, PLsub
        if let Ok(Some(cap)) = COMPOUND_SUB.captures(input)
            && let Some(lang) = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
        {
            let full = cap.get(0).unwrap();
            if !matches.iter().any(|m| {
                m.overlaps(&MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::SubtitleLanguage,
                    "",
                ))
            }) {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                        .with_priority(1),
                );
            }
        }

        // 6. Sub.French, Sub_ITA, ST(Fr-Eng)
        if let Ok(Some(cap)) = SUB_LANG.captures(input)
            && let Some(langs) = cap.name("langs")
        {
            let full = cap.get(0).unwrap();
            if !matches.iter().any(|m| {
                m.overlaps(&MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::SubtitleLanguage,
                    "",
                ))
            }) {
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

        // 7. LANG-SUB / LANG.SUB: EN-SUB
        if matches.is_empty()
            && let Ok(Some(cap)) = LANG_DASH_SUB.captures(input)
            && let Some(lang) = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
        {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(1),
            );
        }

        // 8. Legendado/Legendas with optional language
        if matches.is_empty()
            && let Ok(Some(cap)) = LEGENDADO.captures(input)
        {
            let full = cap.get(0).unwrap();
            let lang = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
                .unwrap_or("und");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(0),
            );
        }

        // 9. Subtitulado with optional language
        if matches.is_empty()
            && let Ok(Some(cap)) = SUBTITULADO.captures(input)
        {
            let full = cap.get(0).unwrap();
            let lang = cap
                .name("lang")
                .and_then(|l| normalize_language(l.as_str()))
                .unwrap_or("und");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, lang)
                    .with_priority(0),
            );
        }

        // 10. Multiple Subtitle
        if let Ok(Some(cap)) = MULTIPLE_SUB.captures(input) {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, "mul")
                    .with_priority(1),
            );
        }

        // 11. Generic sub markers (only if no specific language found): subs, subbed
        if matches.is_empty()
            && let Ok(Some(cap)) = GENERIC_SUB.captures(input)
        {
            let full = cap.get(0).unwrap();
            // Check it's not part of another word we already matched
            let text = full.as_str().to_lowercase();
            if ["subs", "sub", "subbed", "subtitles", "hc", "esub", "esubs"]
                .contains(&text.as_str())
            {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::SubtitleLanguage, "und")
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
    fn test_srt_extension() {
        let m = SubtitleLanguageMatcher.find_matches("movie.eng.srt");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "en");
    }

    #[test]
    fn test_vostfr() {
        let m = SubtitleLanguageMatcher.find_matches("One.Piece.E576.VOSTFR.720p.mkv");
        assert!(m.iter().any(|x| x.value == "fr"));
    }

    #[test]
    fn test_eng_subs() {
        let m = SubtitleLanguageMatcher.find_matches("Movie [ENG SUBS].mkv");
        assert!(m.iter().any(|x| x.value == "en"));
    }

    #[test]
    fn test_hebsubs() {
        let m = SubtitleLanguageMatcher.find_matches("Show.S01E01.HDTV.HebSubs.mkv");
        assert!(m.iter().any(|x| x.value == "he"));
    }

    #[test]
    fn test_swesub() {
        let m = SubtitleLanguageMatcher.find_matches("Show.S06E16.HC.SWESUB.HDTV.x264");
        assert!(m.iter().any(|x| x.value == "sv"));
    }
}
