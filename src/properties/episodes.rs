//! Season / episode detection.
//!
//! Supports S01E02, 1x03, Season/Saison, Episode, 3/4-digit decomposition,
//! anime-style, path-based seasons, and Roman numeral seasons.
//!
//! Patterns and helpers live in [`episode_patterns`](super::episode_patterns).

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::episode_patterns::*;

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // 1. S01E02 (highest priority).
    for cap in captures_iter(&SXXEXX, input) {
        let full = cap.get(0).unwrap();
        let season: u32 = cap.name("season").unwrap().as_str().parse().unwrap_or(0);
        let ep_start: u32 = cap.name("ep_start").unwrap().as_str().parse().unwrap_or(0);

        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Season,
                season.to_string(),
            )
            .with_priority(-1),
        );

        // Check for multi-episode.
        let ep_end = cap.name("ep2").and_then(|m| m.as_str().parse::<u32>().ok());

        match ep_end {
            Some(end) if end > ep_start => {
                matches.extend(episode_range(ep_start, end, full.start(), full.end(), 5));
            }
            Some(end) => {
                // E01E02 style (not a range, individual episodes).
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(5),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, end.to_string())
                        .with_priority(5),
                );
            }
            None => {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(5),
                );
            }
        }
    }

    // 2. S03-E01 (dash separated).
    if matches.is_empty() {
        match_season_episode(&SXX_DASH_EXX, input, 4, &mut matches);
    }

    // 3. S06xE01.
    if matches.is_empty() {
        match_season_episode(&SXX_X_EXX, input, 4, &mut matches);
    }

    // 4. S03-X01 for bonus.
    if matches.is_empty() {
        match_season_episode(&SXX_DASH_XXX, input, 4, &mut matches);
    }

    // 5. NxN format: 1x03, 5x44x45.
    if matches.is_empty() {
        for cap in captures_iter(&NXN, input) {
            let full = cap.get(0).unwrap();
            let season: u32 = cap.name("season").unwrap().as_str().parse().unwrap_or(0);
            let ep_start: u32 = cap.name("ep_start").unwrap().as_str().parse().unwrap_or(0);

            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Season,
                    season.to_string(),
                )
                .with_priority(3),
            );

            let ep_end = cap.name("ep2").and_then(|m| m.as_str().parse::<u32>().ok());

            match ep_end {
                Some(end) if end > ep_start => {
                    matches.extend(episode_range(ep_start, end, full.start(), full.end(), 3));
                }
                Some(end) => {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Episode,
                            ep_start.to_string(),
                        )
                        .with_priority(3),
                    );
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Episode,
                            end.to_string(),
                        )
                        .with_priority(3),
                    );
                }
                None => {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Episode,
                            ep_start.to_string(),
                        )
                        .with_priority(3),
                    );
                }
            }
        }
    }

    // 6. Multi-season patterns (must come before single season).
    if !has_property(&matches, Property::Season) {
        // S01-S10 range.
        for cap in captures_iter(&S_RANGE, input) {
            let full = cap.get(0).unwrap();
            let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
            let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
            for s in s1..=s2 {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(1),
                );
            }
        }
    }

    if !has_property(&matches, Property::Season) {
        // Season 1-3, Season 1&3, Season 1.3.4.
        for cap in captures_iter(&SEASON_MULTI, input) {
            let full = cap.get(0).unwrap();
            let seasons_str = cap.name("seasons").unwrap().as_str();
            let nums: Vec<u32> = seasons_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            // Determine if it's a range (only two nums with dash) or a list.
            let is_range = seasons_str.contains('-') && nums.len() == 2;
            if is_range {
                for s in nums[0]..=nums[1] {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
            } else {
                for s in &nums {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
            }
        }
    }

    // 6b. Standalone season/episode words.
    if !has_property(&matches, Property::Season) {
        match_season(&SEASON_ONLY, input, 2, &mut matches);
    }

    // Roman numeral seasons.
    if !has_property(&matches, Property::Season) {
        for cap in captures_iter(&SEASON_ROMAN, input) {
            let full = cap.get(0).unwrap();
            let roman_str = cap.name("season").unwrap().as_str();
            if let Some(num) = roman_to_int(roman_str) {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, num.to_string())
                        .with_priority(2),
                );
            }
        }
    }

    // Season from path directory.
    if !has_property(&matches, Property::Season) {
        for cap in captures_iter(&SEASON_DIR, input) {
            let full = cap.get(0).unwrap();
            let season = parse_num(&cap, "season");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, season)
                    .as_path_based()
                    .with_priority(-2),
            );
        }
    }

    // S01-only (without episode, e.g., S01Extras).
    if !has_property(&matches, Property::Season) {
        match_season(&S_ONLY, input, 1, &mut matches);
    }

    // Episode standalone patterns.
    if !has_property(&matches, Property::Episode) {
        for cap in captures_iter(&EP_ONLY, input) {
            let full = cap.get(0).unwrap();
            let ep_start: u32 = parse_num(&cap, "ep_start").parse().unwrap_or(0);
            let ep2: Option<u32> = cap.name("ep2").and_then(|m| m.as_str().parse().ok());
            if let Some(ep_end) = ep2 {
                // Multi-episode: E02-03 or E02-E03.
                for ep in ep_start..=ep_end {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, ep.to_string())
                            .with_priority(2),
                    );
                }
            } else {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(2),
                );
            }
        }
    }

    if !has_property(&matches, Property::Episode) {
        match_episode(&EPISODE_WORD, input, 2, &mut matches);
    }

    // 7. Anime-style episode: `Show - 03` or `Show - 003`.
    if !has_property(&matches, Property::Episode) {
        match_episode(&ANIME_EPISODE, input, 1, &mut matches);
    }

    // 8. Bare episode after dots: `Show.05.Title`.
    if !has_property(&matches, Property::Episode)
        && let Some(cap) = captures_iter(&BARE_EPISODE, input).into_iter().next()
    {
        let full = cap.get(0).unwrap();
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode).with_priority(-1),
        );
    }

    // 9. Versioned episode: `Show.07v4` or `312v1`.
    if !has_property(&matches, Property::Episode)
        && let Some(cap) = captures_iter(&VERSIONED_EPISODE, input).into_iter().next()
    {
        let full = cap.get(0).unwrap();
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode).with_priority(1),
        );
    }

    // 9b. Leading episode: `01 - Ep Name`, `003. Show Name`.
    if !has_property(&matches, Property::Episode) {
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let filename = &input[fn_start..];
        for cap in captures_iter(&LEADING_EPISODE, filename) {
            let ep_match = cap.name("episode").unwrap();
            let ep_num: u32 = ep_match.as_str().parse().unwrap_or(0);
            if ep_num == 0 || ep_num > 999 {
                continue;
            }
            // Don't match if looks like a year.
            if (1900..=2039).contains(&ep_num) {
                continue;
            }
            let abs_start = fn_start + ep_match.start();
            let abs_end = fn_start + ep_match.end();
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::Episode, ep_num.to_string())
                    .with_priority(0),
            );
            break;
        }
    }

    // 10. 3/4-digit episode number decomposition: 101→S1E01, 2401→S24E01.
    // Only fires when no season/episode found yet.
    // Must appear after the title portion (not in first 5 chars of filename).
    if !has_property(&matches, Property::Season) && !has_property(&matches, Property::Episode) {
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        for cap in captures_iter(&THREE_DIGIT, input) {
            let full = cap.get(0).unwrap();
            // Skip if too close to the start of the filename (likely title).
            if full.start() < fn_start + 5 {
                continue;
            }
            let num_str = cap.name("num").unwrap().as_str();
            let num: u32 = num_str.parse().unwrap_or(0);
            if num == 0 {
                continue;
            }
            // Skip year-like and codec-related numbers.
            if num_str.len() == 4 && (1920..=2039).contains(&num) {
                continue;
            }
            if num == 264 || num == 265 || num == 128 {
                continue;
            }
            // Decompose: e.g., 501 → S5E01, 117 → S1E17, 2401 → S24E01.
            let (season, episode) = (num / 100, num % 100);
            if season == 0 || episode == 0 || season > 30 || episode > 99 {
                continue;
            }
            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Season,
                    season.to_string(),
                )
                .with_priority(0),
            );
            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Episode,
                    episode.to_string(),
                )
                .with_priority(0),
            );
            break; // Only decompose the first occurrence.
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s01e02() {
        let m = find_matches("Show.S01E02.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "2")
        );
    }

    #[test]
    fn test_multi_episode_e01e02() {
        let m = find_matches("Show.S01E01E02.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "2")
        );
    }

    #[test]
    fn test_multi_episode_e01_dash_02() {
        let m = find_matches("Show.S03E01-02.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "2")
        );
    }

    #[test]
    fn test_multi_episode_e01_dash_e02() {
        let m = find_matches("Show.S03E01-E02.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "2")
        );
    }

    #[test]
    fn test_1x03() {
        let m = find_matches("Show.1x03.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
    }

    #[test]
    fn test_s03_dash_e01() {
        let m = find_matches("Parks_and_Recreation-s03-e01.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "3")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
    }

    #[test]
    fn test_s06xe01() {
        let m = find_matches("The Office - S06xE01.avi");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "6")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
    }

    #[test]
    fn test_episode_word() {
        let m = find_matches("Show Season 2 Episode 5");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "2")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "5")
        );
    }

    #[test]
    fn test_standalone_ep() {
        let m = find_matches("Show.E05.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "5")
        );
    }

    #[test]
    fn test_season_dir() {
        let m = find_matches("TV/Show/Season 6/file.avi");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "6")
        );
    }

    #[test]
    fn test_s01_only() {
        let m = find_matches("Show.S01Extras.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "1")
        );
    }

    #[test]
    fn test_s03_dash_x01() {
        let m = find_matches("Parks_and_Recreation-s03-x01.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "3")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
    }

    #[test]
    fn test_three_digit_501() {
        let m = find_matches("the.mentalist.501.hdtv.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "5")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
    }

    #[test]
    fn test_three_digit_117() {
        let m = find_matches("new.girl.117.hdtv.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "1")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "17")
        );
    }

    #[test]
    fn test_four_digit_2401() {
        let m = find_matches("the.simpsons.2401.hdtv.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "24")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "1")
        );
    }

    #[test]
    fn test_anime_dash_episode() {
        let m = find_matches("Show Name - 03 Vostfr HD");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
    }

    #[test]
    fn test_bare_dot_episode() {
        let m = find_matches("Neverwhere.05.Down.Street.avi");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "5")
        );
    }

    #[test]
    fn test_s_range() {
        let m = find_matches("Friends.S01-S10.COMPLETE.720p.BluRay.x264-PtM");
        eprintln!(
            "matches: {:?}",
            m.iter()
                .map(|x| format!("{:?}={}", x.property, x.value))
                .collect::<Vec<_>>()
        );
        let seasons: Vec<&str> = m
            .iter()
            .filter(|x| x.property == Property::Season)
            .map(|x| x.value.as_str())
            .collect();
        assert!(
            seasons.len() >= 2,
            "Expected multi-season, got: {:?}",
            seasons
        );
    }
}
