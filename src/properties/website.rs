//! Website detection.
//!
//! Detects website names in brackets or prefixed in filenames.

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// TLDs commonly found in media piracy/sharing website names.
/// Used to filter bracket-enclosed matches and avoid false positives
/// like `[ready.player.one]` where `.one` is a valid gTLD but clearly
/// not a website in context.
const KNOWN_TLDS: &[&str] = &[
    "com", "org", "net", "info", "tv", "io", "ru", "cc", "me", "to", "be", "de", "fr", "es", "it",
    "nl", "se", "pl", "cz", "at", "ch", "co", "uk", "us", "ca", "au", "nz", "jp", "kr", "cn", "tw",
    "br", "mx", "in", "za", "ua", "hu", "ro", "bg", "hr", "si", "sk", "lt", "lv", "ee", "fi", "dk",
    "no", "pt", "gr", "tr", "na",
];

/// Website in brackets with multi-part TLD: [tvu.org.ru], [www.site.com]
/// Also handles [.www.site.com.] with optional leading/trailing dots.
static WEBSITE_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"\[[. ]*(?P<site>(?:www\.)?[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?)+\.[a-zA-Z]{2,})[. ]*\]"
    ).unwrap()
});

/// Website prefixed: "From [ site.com ] -"
static WEBSITE_FROM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)from\s*\[?\s*(?P<site>(?:www\.)?[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*\.[a-zA-Z]{2,})\s*\]?"
    ).unwrap()
});

/// Unbracketed website in filename: MkvCage.com, www.divx-overnet.com
/// Excludes common file extensions and requires a boundary before the domain.
///
/// The trailing `\b` is critical: without it, `.ru` would false-positive inside
/// language abbreviations like `.rus` (Russian), producing nonsense like
/// `website: "s02e20.ru"` for `Community.s02e20.rus.eng.720p...` (issue #163).
static WEBSITE_INLINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?P<site>(?:www\.)?[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?\.(?:com|org|net|info|tv|io|ru|cc))\b"
    ).unwrap()
});

/// Scan for embedded website URLs (e.g., `[www.example.com]`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // Priority 1: Bracket-enclosed websites
    for cap in WEBSITE_BRACKET.captures_iter(input) {
        if let Some(site) = cap.name("site") {
            let val = site.as_str();
            // Validate TLD against known list to avoid false positives
            // like [ready.player.one] where .one is a gTLD, not a website.
            if has_known_tld(val) {
                matches.push(
                    MatchSpan::new(site.start(), site.end(), Property::Website, val)
                        .with_priority(crate::priority::KEYWORD),
                );
            }
        }
    }

    // Priority 2: "From" prefix
    for cap in WEBSITE_FROM.captures_iter(input) {
        if let Some(site) = cap.name("site")
            && !matches.iter().any(|m| {
                m.overlaps(&MatchSpan::new(
                    site.start(),
                    site.end(),
                    Property::Website,
                    "",
                ))
            })
        {
            matches.push(
                MatchSpan::new(site.start(), site.end(), Property::Website, site.as_str())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
    }

    // Priority 3: Inline websites (not in brackets)
    if matches.is_empty() {
        for cap in WEBSITE_INLINE.captures_iter(input) {
            if let Some(site) = cap.name("site") {
                let val = site.as_str();
                // Avoid matching things that look like domains but aren't
                // (e.g., AC3.5 or DD5.1)
                if val.len() > 7 {
                    matches.push(
                        MatchSpan::new(site.start(), site.end(), Property::Website, val)
                            .with_priority(crate::priority::DEFAULT),
                    );
                }
            }
        }
    }

    matches
}

/// Check whether a domain string ends with a known TLD.
///
/// Handles multi-part TLDs like `co.uk` by checking the final segment.
fn has_known_tld(domain: &str) -> bool {
    domain
        .rsplit('.')
        .next()
        .is_some_and(|tld| KNOWN_TLDS.iter().any(|k| k.eq_ignore_ascii_case(tld)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_website() {
        let m = find_matches("Movie.720p-GROUP.[sharethefiles.com].mkv");
        assert!(m.iter().any(|x| x.value == "sharethefiles.com"));
    }

    #[test]
    fn test_bracket_multipart_tld() {
        let m = find_matches("Movie.[tvu.org.ru].avi");
        assert!(m.iter().any(|x| x.value == "tvu.org.ru"));
    }

    #[test]
    fn test_inline_website() {
        let m = find_matches("Movie.720p.MkvCage.com");
        assert!(m.iter().any(|x| x.value == "MkvCage.com"));
    }

    #[test]
    fn test_dotted_brackets() {
        let m = find_matches("[.www.site.com.].-.Movie.mkv");
        assert!(m.iter().any(|x| x.value == "www.site.com"));
    }

    #[test]
    fn test_ready_player_one_not_website() {
        let m = find_matches(
            "[DBD-Raws][4K_HDR][ready.player.one][2160P][BDRip][HEVC-10bit][FLAC].mkv",
        );
        assert!(
            !m.iter().any(|x| x.value == "ready.player.one"),
            "ready.player.one should NOT be detected as a website"
        );
    }

    #[test]
    fn test_has_known_tld() {
        assert!(has_known_tld("sharethefiles.com"));
        assert!(has_known_tld("tvu.org.ru"));
        assert!(has_known_tld("www.nimp.na"));
        assert!(has_known_tld("wawa.co.uk"));
        assert!(!has_known_tld("ready.player.one"));
        assert!(!has_known_tld("some.thing.movie"));
    }

    /// Regression for #163: inline website matcher must not match `.ru` inside
    /// the Russian language abbreviation `.rus`. Bug surfaced from the
    /// parse-torrent-name corpus triage in PR #157.
    #[test]
    fn test_no_false_positive_on_language_abbrev_rus() {
        let m = find_matches("Community.s02e20.rus.eng.720p.Kybik.v.Kybe");
        assert!(
            m.is_empty(),
            "`.ru` inside `.rus` (Russian abbrev) must not match as a website, got: {:?}",
            m.iter().map(|x| &x.value).collect::<Vec<_>>()
        );
    }

    /// Genuine `.ru` domains must still be detected — fix shouldn't regress
    /// real ccTLD matches.
    #[test]
    fn test_genuine_ru_domain_still_matches() {
        let m = find_matches("Movie.tracker.ru.x264.mkv");
        assert!(
            m.iter().any(|x| x.value == "tracker.ru"),
            "genuine `.ru` domain should still match, got: {:?}",
            m.iter().map(|x| &x.value).collect::<Vec<_>>()
        );
    }

    /// Word-boundary guard should also reject other letter-extended TLDs:
    /// `.com` inside `.community`, `.net` inside `.network`, etc.
    #[test]
    fn test_no_false_positive_on_extended_tld_suffixes() {
        for sample in [
            "Show.s01e01.community.center.720p",
            "Show.s01e01.networking.event.720p",
            "Show.s01e01.organic.farm.720p",
        ] {
            let m = find_matches(sample);
            assert!(
                m.is_empty(),
                "TLD-as-prefix-of-word must not match for {sample:?}, got: {:?}",
                m.iter().map(|x| &x.value).collect::<Vec<_>>()
            );
        }
    }
}
