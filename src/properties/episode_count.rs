//! Episode count and season count detection.
//!
//! Detects `X of Y` patterns for episode and season totals:
//! - `1of4` / `1 of 4` → episode=1, episode_count=4
//! - `Season.2of5` → season=2, season_count=5
//! - `14.of.21` → episode=14, episode_count=21

use regex::Regex;

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches `Season.Xof Y` or `Season.X of Y` → season_count.
static SEASON_COUNT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:season|saison)[._ ](\d+)\s*of\s*(\d+)").unwrap());

/// Matches `XofY` or `X of Y` or `X.of.Y` → episode + episode_count.
static EPISODE_COUNT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\d+)[. _]*of[. _]*(\d+)").unwrap());

static EPISODE_COUNT_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::Alpha),       // (?i)(?<![a-z])
    right: Some(CharClass::AlphaDigit), // (?i)(?![a-z0-9])
};

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    // Track full spans of season count matches to avoid double-counting.
    let mut season_count_spans: Vec<(usize, usize)> = Vec::new();

    // Season count: `Season.2of5`
    let mut pos = 0;
    while pos < input.len() {
        let Some(cap) = SEASON_COUNT_RE.captures_at(input, pos) else {
            break;
        };
        let full = cap.get(0).unwrap();
        pos = full.end();
        season_count_spans.push((full.start(), full.end()));

        if let Some(count_m) = cap.get(2) {
            let count_val = &input[count_m.start()..count_m.end()];
            matches.push(MatchSpan::new(
                full.start(),
                full.end(),
                Property::SeasonCount,
                count_val.to_string(),
            ));
        }
    }

    // Episode count: `14 of 21`, `1of4`
    let bytes = input.as_bytes();
    let mut pos = 0;
    while pos < input.len() {
        let Some(cap) = EPISODE_COUNT_RE.captures_at(input, pos) else {
            break;
        };
        let full = cap.get(0).unwrap();
        if !check_boundary(bytes, full.start(), full.end(), &EPISODE_COUNT_BOUNDARY) {
            pos = full.start() + 1;
            continue;
        }
        pos = full.end();

        // Skip if this overlaps with a season count match.
        if season_count_spans
            .iter()
            .any(|(s, e)| full.start() >= *s && full.end() <= *e)
        {
            continue;
        }

        if let Some(count_m) = cap.get(2) {
            let count_val = &input[count_m.start()..count_m.end()];
            // Sanity: count should be > 1
            if count_val.parse::<u32>().is_ok_and(|n| n <= 1) {
                continue;
            }
            matches.push(MatchSpan::new(
                full.start(),
                full.end(),
                Property::EpisodeCount,
                count_val.to_string(),
            ));
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_count_1of4() {
        let m = find_matches("Something.Season.2.1of4.Ep.Title.HDTV");
        let ec: Vec<_> = m
            .iter()
            .filter(|s| s.property == Property::EpisodeCount)
            .collect();
        assert_eq!(ec.len(), 1);
        assert_eq!(ec[0].value, "4");
    }

    #[test]
    fn season_count_2of5() {
        let m = find_matches("Something.Season.2of5.3of9.Ep.Title.HDTV.torrent");
        let sc: Vec<_> = m
            .iter()
            .filter(|s| s.property == Property::SeasonCount)
            .collect();
        assert_eq!(sc.len(), 1);
        assert_eq!(sc[0].value, "5");
    }

    #[test]
    fn episode_count_14_of_21() {
        let m = find_matches("FlexGet.14.of.21.Title.Here.720p.HDTV.AAC5.1.x264-NOGRP");
        let ec: Vec<_> = m
            .iter()
            .filter(|s| s.property == Property::EpisodeCount)
            .collect();
        assert_eq!(ec.len(), 1);
        assert_eq!(ec[0].value, "21");
    }

    #[test]
    fn episode_count_1_of_6_spaced() {
        let m = find_matches("BBC The Story of China 1 of 6 - Ancestors CC HDTV");
        let ec: Vec<_> = m
            .iter()
            .filter(|s| s.property == Property::EpisodeCount)
            .collect();
        assert_eq!(ec.len(), 1);
        assert_eq!(ec[0].value, "6");
    }
}
