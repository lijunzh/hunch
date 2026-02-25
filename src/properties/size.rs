//! Size detection.
//!
//! Detects file sizes: 700MB, 1.4GB, 4.7GB, etc.

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static SIZE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?P<size>[0-9]+(?:\.[0-9]+)?\s*(?:GB|MB|TB|GiB|MiB|TiB))").unwrap()
});

static SIZE_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit), // (?i)(?<![a-z0-9])
    right: Some(CharClass::Alpha),     // (?i)(?![a-z])
};

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();
    if let Some(cap) = SIZE_PATTERN.captures(input)
        && let Some(size) = cap.name("size")
        && check_boundary(bytes, size.start(), size.end(), &SIZE_BOUNDARY)
    {
        matches.push(
            MatchSpan::new(size.start(), size.end(), Property::Size, size.as_str())
                .with_priority(1),
        );
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_mb() {
        let m = find_matches("Movie.700MB.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "700MB");
    }

    #[test]
    fn test_size_gb() {
        let m = find_matches("Movie.1.4GB.mkv");
        assert_eq!(m.len(), 1);
    }
}
