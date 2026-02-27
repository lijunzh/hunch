//! Year detection (4-digit years in a reasonable range).

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

const MIN_YEAR: i32 = 1920;
const MAX_YEAR: i32 = 2029;

static YEAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:19|20)\d{2}").unwrap());

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        let Some(m) = YEAR_RE.find_at(input, pos) else {
            break;
        };
        pos = m.start() + 1;

        // Boundary: no digit before or after.
        if m.start() > 0 && bytes[m.start() - 1].is_ascii_digit() {
            continue;
        }
        if m.end() < bytes.len() && bytes[m.end()].is_ascii_digit() {
            continue;
        }

        let raw = m.as_str();
        let Ok(year) = raw.parse::<i32>() else {
            continue;
        };
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            continue;
        }

        // Skip years attached to tech terms: BT2020, x1920.
        if m.start() > 0 && bytes[m.start() - 1].is_ascii_alphabetic() {
            continue;
        }
        // Skip "1920x1080" resolution patterns.
        if m.end() < bytes.len() && matches!(bytes[m.end()], b'x' | b'X') {
            continue;
        }

        matches.push(MatchSpan::new(m.start(), m.end(), Property::Year, raw).with_priority(-1));
        pos = m.end();
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_found() {
        let m = find_matches("The Matrix 1999 1080p");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1999");
    }

    #[test]
    fn test_year_2024() {
        let m = find_matches("Movie.2024.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "2024");
    }

    #[test]
    fn test_no_year_in_codec() {
        let m = find_matches("Movie.x264.mkv");
        assert!(m.is_empty());
    }

    #[test]
    fn test_year_too_old() {
        let m = find_matches("Movie.1800.mkv");
        assert!(m.is_empty());
    }
}
