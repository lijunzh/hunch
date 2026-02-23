//! Website detection.
//!
//! Detects website names in brackets or prefixed in filenames.

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Website in brackets: [www.example.com] or [site.name]
    static ref WEBSITE_BRACKET: Regex = Regex::new(
        r"\[(?P<site>(?:www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\.[a-zA-Z]{2,})?)\]"
    ).unwrap();

    /// Website prefixed: "From [ site.com ] -"
    static ref WEBSITE_FROM: Regex = Regex::new(
        r"(?i)from\s*\[?\s*(?P<site>(?:www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\.[a-zA-Z]{2,})?)\s*\]?"
    ).unwrap();

    /// Website at end before extension: -site.com.mkv
    static ref WEBSITE_END: Regex = Regex::new(
        r"[-. ](?P<site>(?:www\.)?[a-zA-Z0-9-]+\.(?:com|org|net|info|tv|to|cc|me))(?:\.[a-z]{2,5})?$"
    ).unwrap();
}

pub struct WebsiteMatcher;

impl PropertyMatcher for WebsiteMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        for cap in WEBSITE_BRACKET.captures_iter(input) {
            if let Some(site) = cap.name("site") {
                matches.push(
                    MatchSpan::new(
                        site.start(),
                        site.end(),
                        Property::Website,
                        site.as_str(),
                    )
                    .with_priority(2),
                );
            }
        }

        for cap in WEBSITE_FROM.captures_iter(input) {
            if let Some(site) = cap.name("site") {
                if !matches.iter().any(|m| m.overlaps(&MatchSpan::new(site.start(), site.end(), Property::Website, ""))) {
                    matches.push(
                        MatchSpan::new(
                            site.start(),
                            site.end(),
                            Property::Website,
                            site.as_str(),
                        )
                        .with_priority(1),
                    );
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
    fn test_from_website() {
        let m = WebsiteMatcher.find_matches("From [ WWW.TORRENTING.COM ] - Movie.mkv");
        assert!(m.iter().any(|x| x.value == "WWW.TORRENTING.COM"));
    }
}
