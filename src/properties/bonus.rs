//! Bonus content detection.
//!
//! Detects bonus/extras markers: x01, x02 (used for bonus features).

use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Bonus number: x01, x02, x09.
static BONUS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])[xX](?P<num>0[0-9]|[1-9][0-9]?)(?![a-z0-9])").unwrap()
});

/// Bonus title after the bonus number: x01-Title_Here.
static BONUS_TITLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)[xX](?:0[0-9]|[1-9][0-9]?)[-. ](?P<title>[A-Za-z][A-Za-z0-9_ .',-]+?)(?:\.(?:mkv|avi|mp4|srt|sub|ass|ssa|idx|m4v|wmv|flv|webm|ts|m2ts|vob|divx|ogm|rmvb)$|$|-[a-zA-Z0-9]+$)"
    ).unwrap()
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    if let Ok(Some(cap)) = BONUS_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let n: u32 = num.as_str().parse().unwrap_or(0);
        if n > 0 {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Bonus, n.to_string())
                    .with_priority(0),
            );
        }
    }

    // Extract bonus title if present.
    if let Ok(Some(cap)) = BONUS_TITLE_PATTERN.captures(input)
        && let Some(title) = cap.name("title")
    {
        let cleaned = title.as_str().replace('_', " ").trim().to_string();
        if !cleaned.is_empty() {
            matches.push(
                MatchSpan::new(title.start(), title.end(), Property::BonusTitle, cleaned)
                    .with_priority(0),
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
