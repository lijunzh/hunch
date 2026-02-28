//! UUID detection.
//!
//! Detects UUIDs in filenames (commonly used in obfuscated releases).

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Standard UUID: 8-4-4-4-12 hex chars
static UUID_STANDARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?P<uuid>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})")
        .expect("UUID regex is valid")
});

/// Non-standard UUID: 32 hex chars without dashes (common in obfuscated releases)
static UUID_NODASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|/)(?P<uuid>[0-9a-f]{32})(?:[/.]|$)").expect("UUID_BARE regex is valid")
});

/// Scan for UUID patterns (hyphenated and bare 32-hex-char forms) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        let m = find_matches("Movie.a1b2c3d4-e5f6-7890-abcd-ef1234567890.mkv");
        assert_eq!(m.len(), 1);
    }
}
