//! Version detection.
//!
//! Detects release versions like `v2`, `V3`, or `07v4` commonly
//! found in anime fansub releases (e.g., `Episode.366v2`, `[Group] Show 07v4`).

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches `v2`, `v3`, etc. (case-insensitive).
static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)v(\d+)").expect("VERSION_REGEX regex is valid"));

static VERSION_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::Alpha),       // (?i)(?<![a-z]) → Alpha
    right: Some(CharClass::AlphaDigit), // (?i)(?![a-z0-9]) → AlphaDigit
};

/// Scan for version markers (e.g., `v2`, `V4`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        let Some(cap) = VERSION_REGEX.captures_at(input, pos) else {
            break;
        };
        let full = cap.get(0).expect("group 0 always present in a regex match");
        if check_boundary(bytes, full.start(), full.end(), &VERSION_BOUNDARY) {
            if let Some(m) = cap.get(1) {
                matches.push(MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Version,
                    &input[m.start()..m.end()],
                ));
            }
            pos = full.end().max(pos + 1);
        } else {
            pos = full.start() + 1;
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_v2() {
        let m = find_matches("Episode.366v2.VOSTFR.avi");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2");
    }

    #[test]
    fn version_v4() {
        let m = find_matches("FooBar.07v4.PDTV-FlexGet");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "4");
    }

    #[test]
    fn version_uppercase() {
        let m = find_matches("[Group] Show V2.mp4");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2");
    }

    #[test]
    fn no_false_positive_xvid() {
        let m = find_matches("Movie.XviD.mkv");
        assert!(m.is_empty());
    }
}
