//! Part, disc, CD, film detection.
//!
//! Detects part/disc/cd/film numbers commonly used in media filenames.

use lazy_static::lazy_static;
use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Part number: Part 1, Part.2, pt1, pt.2
    static ref PART_PATTERN: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:Part|Pt)[-. ]?(?P<num>[0-9]+|I{1,4}|IV|VI{0,3})(?![a-z0-9])"
    ).unwrap();

    /// Disc number: Disc 1, Disk.2, D1, S01D01
    static ref DISC_PATTERN: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:Disc?k?|S\d+D)[-. ]?(?P<num>[0-9]+)(?:[-](?P<to>[0-9]+))?(?![a-z0-9])"
    ).unwrap();

    /// CD count: 2 CD, 2CD, X cd
    static ref CD_COUNT_PATTERN: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])(?P<num>[0-9]+)\s*CD(?:s)?(?![a-z0-9])"
    ).unwrap();

    /// CD number: CD1, CD 2
    static ref CD_PATTERN: Regex = Regex::new(
        r"(?i)(?<![a-z])CD[-. ]?(?P<num>[0-9]+)(?![a-z0-9])"
    ).unwrap();

    /// Film number: f01, f21 (used in collections like James Bond)
    static ref FILM_PATTERN: Regex = Regex::new(
        r"(?i)(?<![a-z])f(?P<num>[0-9]{1,3})(?![a-z0-9])"
    ).unwrap();
}

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

pub struct PartMatcher;

impl PropertyMatcher for PartMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        if let Ok(Some(cap)) = PART_PATTERN.captures(input) {
            if let Some(num) = cap.name("num") {
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
                        MatchSpan::new(full.start(), full.end(), Property::Part, &value)
                            .with_priority(1),
                    );
                }
            }
        }

        if let Ok(Some(cap)) = DISC_PATTERN.captures(input) {
            if let Some(num) = cap.name("num") {
                let full = cap.get(0).unwrap();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Disc, num.as_str())
                        .with_priority(1),
                );
            }
        }

        if let Ok(Some(cap)) = CD_COUNT_PATTERN.captures(input) {
            if let Some(num) = cap.name("num") {
                let full = cap.get(0).unwrap();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::CdCount, num.as_str())
                        .with_priority(1),
                );
            }
        }

        if let Ok(Some(cap)) = CD_PATTERN.captures(input) {
            if let Some(num) = cap.name("num") {
                let full = cap.get(0).unwrap();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Cd, num.as_str())
                        .with_priority(1),
                );
            }
        }

        if let Ok(Some(cap)) = FILM_PATTERN.captures(input) {
            if let Some(num) = cap.name("num") {
                let n: u32 = num.as_str().parse().unwrap_or(0);
                if n > 0 {
                    let full = cap.get(0).unwrap();
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Film, n.to_string())
                            .with_priority(1),
                    );
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part() {
        let m = PartMatcher.find_matches("Movie.Part.2.mkv");
        assert!(m.iter().any(|x| x.property == Property::Part && x.value == "2"));
    }

    #[test]
    fn test_disc() {
        let m = PartMatcher.find_matches("Movie.Disc1.mkv");
        assert!(m.iter().any(|x| x.property == Property::Disc && x.value == "1"));
    }

    #[test]
    fn test_cd() {
        let m = PartMatcher.find_matches("Movie.CD2.mkv");
        assert!(m.iter().any(|x| x.property == Property::Cd && x.value == "2"));
    }

    #[test]
    fn test_film() {
        let m = PartMatcher.find_matches("James_Bond-f21-Casino.mkv");
        assert!(m.iter().any(|x| x.property == Property::Film && x.value == "21"));
    }
}
