//! CRC32 detection.
//!
//! Detects CRC32 checksums commonly found in anime filenames: `[ABCD1234]`.

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;
use std::sync::LazyLock;

/// Matches 8-char hex CRC32 in square brackets: [ABCD1234]
static CRC32_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(?P<crc>[0-9A-Fa-f]{8})\]").unwrap());

pub struct Crc32Matcher;

impl PropertyMatcher for Crc32Matcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for cap in CRC32_BRACKET.captures_iter(input) {
            if let Some(crc) = cap.name("crc") {
                matches.push(
                    MatchSpan::new(
                        crc.start(),
                        crc.end(),
                        Property::Crc,
                        crc.as_str().to_uppercase(),
                    )
                    .with_priority(2),
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
    fn test_crc32() {
        let m = Crc32Matcher.find_matches("[SubGroup] Anime - 01 [1080p] [ABCD1234].mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "ABCD1234");
    }

    #[test]
    fn test_no_false_positive() {
        // Non-hex chars shouldn't match
        let m = Crc32Matcher.find_matches("[SubGroup].mkv");
        assert!(m.is_empty());
    }
}
