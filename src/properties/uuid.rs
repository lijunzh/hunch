//! UUID detection.
//!
//! Detects UUIDs in filenames (commonly used in obfuscated releases).

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Standard UUID: 8-4-4-4-12 hex chars
    static ref UUID_STANDARD: Regex = Regex::new(
        r"(?i)(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})"
    ).unwrap();

    /// Non-standard UUID: 32 hex chars without dashes (common in obfuscated releases)
    static ref UUID_NODASH: Regex = Regex::new(
        r"(?i)(?:^|/)(?P<uuid>[0-9a-f]{32})(?:[/.]|$)"
    ).unwrap();
}

pub struct UuidMatcher;

impl PropertyMatcher for UuidMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Standard UUID with dashes
        for cap in UUID_STANDARD.captures_iter(input) {
            if let Some(uuid) = cap.name("uuid") {
                matches.push(
                    MatchSpan::new(uuid.start(), uuid.end(), Property::Uuid, uuid.as_str())
                        .with_priority(2),
                );
            }
        }

        // Non-standard UUID: 32 hex chars without dashes
        if matches.is_empty() {
            for cap in UUID_NODASH.captures_iter(input) {
                if let Some(uuid) = cap.name("uuid") {
                    matches.push(
                        MatchSpan::new(uuid.start(), uuid.end(), Property::Uuid, uuid.as_str())
                            .with_priority(2),
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
    fn test_uuid() {
        let m = UuidMatcher.find_matches("Movie.a1b2c3d4-e5f6-7890-abcd-ef1234567890.mkv");
        assert_eq!(m.len(), 1);
    }
}
