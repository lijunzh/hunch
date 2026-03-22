//! Part, disc, CD, film detection.
//!
//! Detects part/disc/cd/film numbers commonly used in media filenames.

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Boundary: not preceded by alpha, not followed by alphanumeric.
static ALPHA_ALPHADIGIT: BoundarySpec = BoundarySpec {
    left: Some(CharClass::Alpha),
    right: Some(CharClass::AlphaDigit),
};

/// Boundary: not preceded/followed by alphanumeric.
static ALPHADIGIT_BOTH: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),
    right: Some(CharClass::AlphaDigit),
};

/// Part number: Part 1, Part.2, pt1, pt.2, Part Three, Part Trois
static PART_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Part|Pt)[-. ]?(?P<num>[0-9]+|I{1,4}|IV|VI{0,3}|(?:one|two|three|four|five|six|seven|eight|nine|ten|un|deux|trois|quatre|cinq|six|sept|huit|neuf|dix))")
        .unwrap()
});

/// Apt (apartado) pattern: Apt.1, Apt 2
static APT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)Apt[-. ]?(?P<num>[0-9]+)").unwrap());

/// Disc number: Disc 1, Disk.2, D1, S01D01
static DISC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Disc?k?|S\d+D)[-. ]?(?P<nums>[0-9]+(?:[.&-][0-9]+)*)").unwrap()
});

/// CD count: 2 CD, 2CD
static CD_COUNT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?P<num>[0-9]+)\s*CD(?:s)?").unwrap());

/// CD number: CD1, CD 2
static CD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)CD[-. ]?(?P<num>[0-9]+)").unwrap());

/// Film number: f01, f21
static FILM_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)f(?P<num>[0-9]{1,2})").unwrap());

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

fn word_to_int(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "one" | "un" => Some(1),
        "two" | "deux" => Some(2),
        "three" | "trois" => Some(3),
        "four" | "quatre" => Some(4),
        "five" | "cinq" => Some(5),
        "six" => Some(6),
        "seven" | "sept" => Some(7),
        "eight" | "huit" => Some(8),
        "nine" | "neuf" => Some(9),
        "ten" | "dix" => Some(10),
        _ => None,
    }
}

/// Scan for part/CD/disc markers (e.g., `Part 2`, `CD1`, `Disc 3`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();

    if let Some(cap) = PART_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHA_ALPHADIGIT) {
            let value = if let Ok(n) = num.as_str().parse::<u32>() {
                n.to_string()
            } else if let Some(n) = roman_to_int(num.as_str()) {
                n.to_string()
            } else if let Some(n) = word_to_int(num.as_str()) {
                n.to_string()
            } else {
                String::new()
            };
            if !value.is_empty() {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Part, &value)
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
    }

    if let Some(cap) = DISC_PATTERN.captures(input)
        && let Some(nums) = cap.name("nums")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHA_ALPHADIGIT) {
            let disc_nums = parse_disc_nums(nums.as_str());
            for n in disc_nums {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Disc, n.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
    }

    if let Some(cap) = CD_COUNT_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHADIGIT_BOTH) {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::CdCount, num.as_str())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
    }

    if let Some(cap) = CD_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHA_ALPHADIGIT) {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Cd, num.as_str())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
    }

    if let Some(cap) = FILM_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHADIGIT_BOTH) {
            let n: u32 = num.as_str().parse().unwrap_or(0);
            if n > 0 {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Film, n.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
    }

    // Apt (apartado) pattern.
    if let Some(cap) = APT_PATTERN.captures(input)
        && let Some(num) = cap.name("num")
    {
        let full = cap.get(0).unwrap();
        if check_boundary(bytes, full.start(), full.end(), &ALPHA_ALPHADIGIT) {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Part, num.as_str())
                    .with_priority(crate::priority::VOCABULARY),
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
    for segment in s.split(['&', '.']) {
        if let Some((start, end)) = segment.split_once('-') {
            let s: u32 = start.parse().unwrap_or(0);
            let e: u32 = end.parse().unwrap_or(0);
            if s > 0 && e >= s {
                result.extend(s..=e);
            }
        } else if let Ok(n) = segment.parse::<u32>()
            && n > 0
        {
            result.push(n);
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
