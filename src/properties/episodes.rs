//! Season / episode detection (S01E02, 1x03, etc.).

use lazy_static::lazy_static;
use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// S01E02, S01E02E03, S01E02-E05.
    static ref SXXEXX: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})\s*E(?P<episode>\d{1,4})(?:[-E]+(?P<episode_end>\d{1,4}))?(?![a-z0-9])"
    ).unwrap();

    /// 1x03 format.
    static ref NXN: Regex = Regex::new(
        r"(?<![a-z0-9])(?P<season>\d{1,2})x(?P<episode>\d{2,3})(?![a-z0-9])"
    ).unwrap();

    /// Standalone episode: E01, Ep01, Ep.01.
    static ref EP_ONLY: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();

    /// Season-only: Season 1, Season.01.
    static ref SEASON_ONLY: Regex = Regex::new(
        r"(?i)(?<![a-z])Season\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])"
    ).unwrap();

    /// Episode-only: Episode 1, Episode.01.
    static ref EPISODE_WORD: Regex = Regex::new(
        r"(?i)(?<![a-z])Episode\s*\.?\s*(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();
}

/// Helper: iterate fancy_regex captures (which return Result).
fn captures_iter<'a>(re: &'a Regex, input: &'a str) -> Vec<fancy_regex::Captures<'a>> {
    let mut results = Vec::new();
    let mut start = 0;
    while start < input.len() {
        match re.captures_from_pos(input, start) {
            Ok(Some(cap)) => {
                if let Some(full) = cap.get(0) {
                    results.push(cap);
                    start = full.end().max(start + 1);
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    results
}

pub struct EpisodeMatcher;

impl PropertyMatcher for EpisodeMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // S01E02 (highest priority).
        for cap in captures_iter(&SXXEXX, input) {
            let full = cap.get(0).unwrap();
            let season = cap.name("season").unwrap().as_str();
            let episode = cap.name("episode").unwrap().as_str();

            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, season)
                    .with_tag("SxxExx")
                    .with_priority(5),
            );
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                    .with_tag("SxxExx")
                    .with_priority(5),
            );
        }

        // 1x03 format.
        if matches.is_empty() {
            for cap in captures_iter(&NXN, input) {
                let full = cap.get(0).unwrap();
                let season = cap.name("season").unwrap().as_str();
                let episode = cap.name("episode").unwrap().as_str();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(3),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(3),
                );
            }
        }

        // Standalone season/episode words.
        if !matches.iter().any(|m| m.property == Property::Season) {
            for cap in captures_iter(&SEASON_ONLY, input) {
                let full = cap.get(0).unwrap();
                let season = cap.name("season").unwrap().as_str();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(2),
                );
            }
        }

        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&EP_ONLY, input) {
                let full = cap.get(0).unwrap();
                let episode = cap.name("episode").unwrap().as_str();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(2),
                );
            }
        }

        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&EPISODE_WORD, input) {
                let full = cap.get(0).unwrap();
                let episode = cap.name("episode").unwrap().as_str();
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(2),
                );
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s01e02() {
        let m = EpisodeMatcher.find_matches("Show.S01E02.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "01"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "02"));
    }

    #[test]
    fn test_1x03() {
        let m = EpisodeMatcher.find_matches("Show.1x03.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "03"));
    }

    #[test]
    fn test_episode_word() {
        let m = EpisodeMatcher.find_matches("Show Season 2 Episode 5");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "2"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "5"));
    }

    #[test]
    fn test_standalone_ep() {
        let m = EpisodeMatcher.find_matches("Show.E05.mkv");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "05"));
    }
}
