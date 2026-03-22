//! Bonus content detection.
//!
//! Detects bonus/extras markers: x01, x02 (used for bonus features).

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Bonus number: x01, x02, x09.
static BONUS_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)[xX](?P<num>0[0-9]|[1-9][0-9]?)").unwrap());

static BONUS_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),  // (?<![a-z0-9])
    right: Some(CharClass::AlphaDigit), // (?![a-z0-9])
};

/// Bonus title after the bonus number: x01-Title_Here.
static BONUS_TITLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)[xX](?:0[0-9]|[1-9][0-9]?)[-. ](?P<title>[A-Za-z][A-Za-z0-9_ .',-]+?)(?:\.(?:mkv|avi|mp4|srt|sub|ass|ssa|idx|m4v|wmv|flv|webm|ts|m2ts|vob|divx|ogm|rmvb)$|$|-[a-zA-Z0-9]+$)"
    ).unwrap()
});

/// Scan for bonus content markers (e.g., `-x02`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();

    if let Some(cap) = BONUS_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &BONUS_BOUNDARY) {
            let n: u32 = num.as_str().parse().unwrap_or(0);
            if n > 0 {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Bonus, n.to_string())
                        .with_priority(crate::priority::DEFAULT),
                );
            }
        }
    }

    // Extract bonus title if present.
    if let Some(cap) = BONUS_TITLE_PATTERN.captures(input)
        && let Some(title) = cap.name("title")
    {
        let cleaned = title.as_str().replace('_', " ").trim().to_string();
        if !cleaned.is_empty() {
            matches.push(
                MatchSpan::new(title.start(), title.end(), Property::BonusTitle, cleaned)
                    .with_priority(crate::priority::DEFAULT),
            );
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonus_number() {
        let m = find_matches("Movie-x01-Behind_the_Scenes.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Bonus && x.value == "1")
        );
    }

    #[test]
    fn test_bonus_title() {
        let m = find_matches("Movie-x01-Behind_the_Scenes.mkv");
        assert!(m.iter().any(|x| x.property == Property::BonusTitle));
    }
}
