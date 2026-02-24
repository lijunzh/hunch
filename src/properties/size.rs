//! Size detection.
//!
//! Detects file sizes: 700MB, 1.4GB, 4.7GB, etc.

use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;
use std::sync::LazyLock;

static SIZE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])(?P<size>[0-9]+(?:\.[0-9]+)?\s*(?:GB|MB|TB|GiB|MiB|TiB))(?![a-z])",
    )
    .unwrap()
});

pub struct SizeMatcher;

impl PropertyMatcher for SizeMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        if let Ok(Some(cap)) = SIZE_PATTERN.captures(input)
            && let Some(size) = cap.name("size")
        {
            matches.push(
                MatchSpan::new(size.start(), size.end(), Property::Size, size.as_str())
                    .with_priority(1),
            );
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_mb() {
        let m = SizeMatcher.find_matches("Movie.700MB.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "700MB");
    }

    #[test]
    fn test_size_gb() {
        let m = SizeMatcher.find_matches("Movie.1.4GB.mkv");
        assert_eq!(m.len(), 1);
    }
}
