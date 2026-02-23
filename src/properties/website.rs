//! Website detection.
//!
//! Detects website names in brackets or prefixed in filenames.

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Website in brackets with multi-part TLD: [tvu.org.ru], [www.site.com]
    /// Also handles [.www.site.com.] with optional leading/trailing dots.
    static ref WEBSITE_BRACKET: Regex = Regex::new(
        r"\[[. ]*(?P<site>(?:www\.)?[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?)+\.[a-zA-Z]{2,})[. ]*\]"
    ).unwrap();

    /// Website prefixed: "From [ site.com ] -"
    static ref WEBSITE_FROM: Regex = Regex::new(
        r"(?i)from\s*\[?\s*(?P<site>(?:www\.)?[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*\.[a-zA-Z]{2,})\s*\]?"
    ).unwrap();

    /// Unbracketed website in filename: MkvCage.com, www.divx-overnet.com
    /// Excludes common file extensions to prevent over-matching.
    static ref WEBSITE_INLINE: Regex = Regex::new(
        r"(?P<site>(?:www\.)?[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?\.(?:com|org|net|info|tv|io|ru|cc|me|to))"
    ).unwrap();
}

pub struct WebsiteMatcher;

impl PropertyMatcher for WebsiteMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Priority 1: Bracket-enclosed websites
        for cap in WEBSITE_BRACKET.captures_iter(input) {
            if let Some(site) = cap.name("site") {
                matches.push(
                    MatchSpan::new(site.start(), site.end(), Property::Website, site.as_str())
                        .with_priority(2),
                );
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
                        .with_priority(1),
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
                    if val.len() > 5 {
                        matches.push(
                            MatchSpan::new(site.start(), site.end(), Property::Website, val)
                                .with_priority(0),
                        );
                    }
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_website() {
        let m = WebsiteMatcher.find_matches("Movie.720p-GROUP.[sharethefiles.com].mkv");
        assert!(m.iter().any(|x| x.value == "sharethefiles.com"));
    }

    #[test]
    fn test_bracket_multipart_tld() {
        let m = WebsiteMatcher.find_matches("Movie.[tvu.org.ru].avi");
        assert!(m.iter().any(|x| x.value == "tvu.org.ru"));
    }

    #[test]
    fn test_inline_website() {
        let m = WebsiteMatcher.find_matches("Movie.720p.MkvCage.com");
        assert!(m.iter().any(|x| x.value == "MkvCage.com"));
    }

    #[test]
    fn test_dotted_brackets() {
        let m = WebsiteMatcher.find_matches("[.www.site.com.].-.Movie.mkv");
        assert!(m.iter().any(|x| x.value == "www.site.com"));
    }
}
