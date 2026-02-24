//! Part, disc, CD, film detection.
//!
//! Detects part/disc/cd/film numbers commonly used in media filenames.

use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Part number: Part 1, Part.2, pt1, pt.2
static PART_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z])(?:Part|Pt)[-. ]?(?P<num>[0-9]+|I{1,4}|IV|VI{0,3})(?![a-z0-9])")
        .unwrap()
});

/// Disc number: Disc 1, Disk.2, D1, S01D01, S01D02.3-5, S01D02&4-6&8
static DISC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Disc?k?|S\d+D)[-. ]?(?P<nums>[0-9]+(?:[.&-][0-9]+)*)(?![a-z0-9])",
    )
    .unwrap()
});

/// CD count: 2 CD, 2CD, X cd
static CD_COUNT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])(?P<num>[0-9]+)\s*CD(?:s)?(?![a-z0-9])").unwrap()
});

/// CD number: CD1, CD 2
static CD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?<![a-z])CD[-. ]?(?P<num>[0-9]+)(?![a-z0-9])").unwrap());

/// Film number: f01, f21 (used in collections like James Bond)
/// Only match when preceded by separator and followed by separator.
static FILM_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?<![a-z0-9])f(?P<num>[0-9]{1,2})(?![a-z0-9])").unwrap());

fn roman_to_int(s: &str) -> Option<u32> {
    match s.to_uppercase().as_str() {
        "I" => Some(1),
        "II" => Some(2),
        "III" => Some(3),
        "IV" => Some(4),
        "V" => Some(5),
        "VI" => Some(6),
        "VII" => Some(7),
        "VIII" => Some(8),
        "IX" => Some(9),
        "X" => Some(10),
        _ => None,
    }
}

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    if let Ok(Some(cap)) = PART_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let value = if let Ok(n) = num.as_str().parse::<u32>() {
            n.to_string()
        } else if let Some(n) = roman_to_int(num.as_str()) {
            n.to_string()
        } else {
            String::new()
        };
        if !value.is_empty() {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Part, &value).with_priority(1),
            );
        }
    }

    if let Ok(Some(cap)) = DISC_PATTERN.captures(input)
        && let Some(nums) = cap.name("nums")
    {
        let full = cap.get(0).unwrap();
        // Parse multi-disc: "2.3-5" → [2, 3, 4, 5], "2&4-6&8" → [2, 4, 5, 6, 8]
        let disc_nums = parse_disc_nums(nums.as_str());
        for n in disc_nums {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Disc, n.to_string())
                    .with_priority(1),
            );
        }
    }

    if let Ok(Some(cap)) = CD_COUNT_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::CdCount, num.as_str())
                .with_priority(1),
        );
    }

    if let Ok(Some(cap)) = CD_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Cd, num.as_str()).with_priority(1),
        );
    }

    if let Ok(Some(cap)) = FILM_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let n: u32 = num.as_str().parse().unwrap_or(0);
        if n > 0 {
            let full = cap.get(0).unwrap();
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Film, n.to_string())
                    .with_priority(1),
            );
        }
    }

    matches
}

/// Parse disc number string with ranges and separators.
/// "2" → [2], "2.3-5" → [2, 3, 4, 5], "2&4-6&8" → [2, 4, 5, 6, 8]
fn parse_disc_nums(s: &str) -> Vec<u32> {
    let mut result = Vec::new();
    // Split by & or . to get segments, each segment can be a range "3-5" or single "2".
    for segment in s.split(|c| c == '&' || c == '.') {
        if let Some((start, end)) = segment.split_once('-') {
            let s: u32 = start.parse().unwrap_or(0);
            let e: u32 = end.parse().unwrap_or(0);
            if s > 0 && e >= s {
                result.extend(s..=e);
            }
        } else if let Ok(n) = segment.parse::<u32>() {
            if n > 0 {
                result.push(n);
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part() {
        let m = find_matches("Movie.Part.2.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Part && x.value == "2")
        );
    }

    #[test]
    fn test_disc() {
        let m = find_matches("Movie.Disc1.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Disc && x.value == "1")
        );
    }

    #[test]
    fn test_cd() {
        let m = find_matches("Movie.CD2.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Cd && x.value == "2")
        );
    }

    #[test]
    fn test_film() {
        let m = find_matches("James_Bond-f21-Casino.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Film && x.value == "21")
        );
    }
}
