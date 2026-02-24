//! Version detection.
//!
//! Detects release versions like `v2`, `V3`, or `07v4` commonly
//! found in anime fansub releases (e.g., `Episode.366v2`, `[Group] Show 07v4`).

use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches `v2`, `v3`, etc. (case-insensitive), not preceded by a letter
/// (to avoid matching inside `XviD`, `DivX`, etc.).
static VERSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?<![a-z])v(\d+)(?![a-z0-9])").unwrap());

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    let mut search_start = 0;
    while search_start < input.len() {
        let Some(cap) = VERSION_REGEX
            .captures_from_pos(input, search_start)
            .ok()
            .flatten()
        else {
            break;
        };
        let full = cap.get(0).unwrap();
        search_start = full.end();

        if let Some(m) = cap.get(1) {
            let version_num = &input[m.start()..m.end()];
            matches.push(MatchSpan::new(
                full.start(),
                full.end(),
                Property::Version,
                version_num,
            ));
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
