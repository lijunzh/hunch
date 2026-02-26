//! Season / episode detection.
//!
//! Supports S01E02, 1x03, Season/Saison, Episode, 3/4-digit decomposition,
//! anime-style, path-based seasons, and Roman numeral seasons.

mod patterns;
#[cfg(test)]
mod tests;

use crate::matcher::span::{MatchSpan, Property};
use patterns::*;
use std::sync::LazyLock;

/// Check that a THREE_DIGIT match is followed by a separator or end-of-string.
fn three_digit_trailing_ok(input: &str, end: usize) -> bool {
    end >= input.len() || matches!(input.as_bytes()[end], b'.' | b'-' | b'_' | b' ')
}

// ── Helpers ──

/// Match a simple season+episode pair from a regex with named groups `season` and `episode`.
fn match_season_episode(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in re.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let season = parse_num(&cap, "season");
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Season, season)
                .with_priority(priority),
        );
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(priority),
        );
    }
}

/// Match a season-only value from a regex with named group `season`.
fn match_season(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in re.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let season = parse_num(&cap, "season");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Season, season)
                .with_priority(priority),
        );
    }
}

/// Match an episode-only value from a regex with named group `episode`.
fn match_episode(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in re.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(priority),
        );
    }
}

/// Check if matches contain a given property.
fn has_property(matches: &[MatchSpan], property: Property) -> bool {
    matches.iter().any(|m| m.property == property)
}

/// Generate a range of episode numbers as MatchSpans.
/// Parse a multi-episode suffix like "E03" or "-05" or "-E07" into episode numbers.
/// Segments joined by just `E` (no separator) are discrete.
/// Segments joined by `-` or `+` indicate ranges.
fn parse_multi_episodes(first_ep: u32, rest: &str) -> Vec<u32> {
    let mut episodes = vec![first_ep];
    let mut prev = first_ep;

    // Split rest into segments: each starts with a separator and then digits.
    let segment_re = regex::Regex::new(r"(?i)([-+]E?|[. ]E|E)(\d{1,4})").unwrap();
    for cap in segment_re.captures_iter(rest) {
        let sep = cap.get(1).unwrap().as_str();
        let num: u32 = cap.get(2).unwrap().as_str().parse().unwrap_or(0);

        // Determine if this segment is a range or discrete.
        let is_range = sep.starts_with('-') || sep.starts_with('+');
        let sep_upper = sep.to_uppercase();
        let is_discrete = sep_upper == "E" || sep_upper.ends_with("E") && !is_range;

        if is_range && !is_discrete && num > prev {
            // Range: fill in episodes from prev+1 to num.
            for ep in (prev + 1)..=num {
                episodes.push(ep);
            }
        } else {
            // Discrete: just add the episode number.
            episodes.push(num);
        }
        prev = num;
    }

    episodes.sort();
    episodes.dedup();
    episodes
}

fn episode_range(
    start_ep: u32,
    end_ep: u32,
    span_start: usize,
    span_end: usize,
    priority: i32,
) -> Vec<MatchSpan> {
    (start_ep..=end_ep)
        .map(|ep| {
            MatchSpan::new(span_start, span_end, Property::Episode, ep.to_string())
                .with_priority(priority)
        })
        .collect()
}

/// Parse a named capture group as a u32 and return as String (strips leading zeros).
fn parse_num(cap: &regex::Captures, name: &str) -> String {
    cap.name(name)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap_or(0)
        .to_string()
}

/// Parse a Roman numeral string to an integer.
fn roman_to_int(s: &str) -> Option<u32> {
    let upper = s.to_uppercase();
    let mut result: i32 = 0;
    let mut prev = 0;
    for ch in upper.chars().rev() {
        let val = match ch {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if val < prev {
            result -= val;
        } else {
            result += val;
        }
        prev = val;
    }
    if result > 0 {
        Some(result as u32)
    } else {
        None
    }
}

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // 0. S01E01-S01E21 full range (must come before SxxExx to win).
    for cap in SXXEXX_TO_SXXEXX.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
        let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
        let e1: u32 = parse_num(&cap, "e1").parse().unwrap_or(0);
        let e2: u32 = parse_num(&cap, "e2").parse().unwrap_or(0);
        // Only expand if same season and valid range.
        if s1 == s2 && e2 >= e1 {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s1.to_string())
                    .with_priority(5),
            );
            matches.extend(episode_range(e1, e2, full.start(), full.end(), 5));
        }
    }

    // 1. S01E02 (highest priority).
    for cap in SXXEXX.captures_iter(input) {
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

        // Parse multi-episode suffix.
        let ep_rest = cap.name("ep_rest").map(|m| m.as_str()).unwrap_or("");

        if ep_rest.is_empty() {
            // Single episode.
            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Episode,
                    ep_start.to_string(),
                )
                .with_priority(5),
            );
        } else {
            // Parse each segment in the multi-episode suffix.
            let episodes = parse_multi_episodes(ep_start, ep_rest);
            for ep in &episodes {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, ep.to_string())
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
        for cap in NXN.captures_iter(input) {
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
        for cap in S_RANGE.captures_iter(input) {
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

    // s01.to.s04 range.
    if !has_property(&matches, Property::Season) {
        for cap in S_TO_S.captures_iter(input) {
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

    // S01S02S03 concatenated.
    static S_NUM_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)S(\d{1,3})").unwrap());
    if !has_property(&matches, Property::Season) {
        for cap in S_CONCAT.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let text = &input[full.start()..full.end()];
            // Extract all season numbers from SxxSxx pattern.
            for num_cap in S_NUM_RE.find_iter(text) {
                let num_str = &num_cap.as_str()[1..];
                if let Ok(s) = num_str.parse::<u32>() {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
            }
        }
    }

    // S01-02-03 (S prefix + dash/space separated numbers).
    if !has_property(&matches, Property::Season) {
        for cap in S_MULTI_NUM.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let seasons_str = cap.name("seasons").unwrap().as_str();
            let nums: Vec<u32> = seasons_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            for s in &nums {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(1),
                );
            }
        }
    }

    if !has_property(&matches, Property::Season) {
        // Season 1 to 3, Season 1~3, Saison 1 a 3 (word-based range).
        for cap in SEASON_RANGE_WORD.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
            let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
            if s2 > s1 {
                for s in s1..=s2 {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
            }
        }
    }

    if !has_property(&matches, Property::Season) {
        // Season 1.3 and 5, Season 1.3&5 (word-based list with "and"/"&").
        for cap in SEASON_LIST_AND.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let nums_str = cap.name("nums").unwrap().as_str();
            let last: u32 = parse_num(&cap, "last").parse().unwrap_or(0);
            let mut nums: Vec<u32> = nums_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            nums.push(last);
            for s in &nums {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(1),
                );
            }
        }
    }

    if !has_property(&matches, Property::Season) {
        // Season 1.2.3~5, Season 1.2.3 to 5 (discrete prefix + range end).
        for cap in SEASON_MULTI_RANGE.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let prefix_str = cap.name("prefix").unwrap().as_str();
            let end: u32 = parse_num(&cap, "end").parse().unwrap_or(0);
            let prefix_nums: Vec<u32> = prefix_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            // Emit all prefix numbers.
            for s in &prefix_nums {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(1),
                );
            }
            // Expand range from last prefix number to end.
            if let Some(&last) = prefix_nums.last() {
                for s in (last + 1)..=end {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
            }
        }
    }

    if !has_property(&matches, Property::Season) {
        // Season 1-3, Season 1&3, Season 1.3.4.
        for cap in SEASON_MULTI.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let seasons_str = cap.name("seasons").unwrap().as_str();
            let nums: Vec<u32> = seasons_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            // Check if the last separator indicates a range.
            let last_sep_is_range = seasons_str
                .rfind(|c: char| !c.is_ascii_digit())
                .map(|i| {
                    let sep = seasons_str[i..].chars().next().unwrap_or(' ');
                    sep == '-' || sep == '~'
                })
                .unwrap_or(false);

            if last_sep_is_range && nums.len() >= 2 {
                // All nums except the last two are discrete, last two form a range.
                let range_start = nums[nums.len() - 2];
                let range_end = nums[nums.len() - 1];
                for s in &nums[..nums.len() - 2] {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(1),
                    );
                }
                if range_end > range_start {
                    for s in range_start..=range_end {
                        matches.push(
                            MatchSpan::new(
                                full.start(),
                                full.end(),
                                Property::Season,
                                s.to_string(),
                            )
                            .with_priority(1),
                        );
                    }
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
        for cap in SEASON_ROMAN.captures_iter(input) {
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
        for cap in SEASON_DIR.captures_iter(input) {
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
        for cap in S_ONLY.captures_iter(input) {
            let full = cap.get(0).unwrap();
            // Reject if followed by digit, E+digit, or x/X+digit.
            let rest = &input[full.end()..];
            let next_char = rest.as_bytes().first().copied();
            if matches!(next_char, Some(b'0'..=b'9')) {
                continue;
            }
            if let Some(c) = next_char
                && (c == b'E' || c == b'e' || c == b'x' || c == b'X')
                && rest.len() > 1
                && rest.as_bytes()[1].is_ascii_digit()
            {
                continue;
            }
            let season = parse_num(&cap, "season");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, season).with_priority(1),
            );
        }
    }

    // Episode standalone patterns.
    if !has_property(&matches, Property::Episode) {
        for cap in EP_ONLY.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let ep_start: u32 = parse_num(&cap, "ep_start").parse().unwrap_or(0);
            let ep_rest = cap.name("ep_rest").map(|m| m.as_str()).unwrap_or("");

            if ep_rest.is_empty() {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(2),
                );
            } else {
                let episodes = parse_multi_episodes(ep_start, ep_rest);
                for ep in &episodes {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, ep.to_string())
                            .with_priority(2),
                    );
                }
            }
        }
    }

    if !has_property(&matches, Property::Episode) {
        // "Episode 1" or "Episodes 1-12" (word-based, possibly with range).
        for cap in EPISODE_WORD.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let ep_start: u32 = parse_num(&cap, "episode").parse().unwrap_or(0);
            let ep_end = cap
                .name("ep_end")
                .and_then(|m| m.as_str().parse::<u32>().ok());
            if let Some(end) = ep_end {
                if end > ep_start {
                    matches.extend(episode_range(ep_start, end, full.start(), full.end(), 2));
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

    // 7. Anime-style episode: `Show - 03` or `Show - 003`.
    if !has_property(&matches, Property::Episode) {
        match_episode(&ANIME_EPISODE, input, 1, &mut matches);
    }

    // 8. Bare episode after dots: `Show.05.Title`.
    if !has_property(&matches, Property::Episode)
        && let Some(cap) = BARE_EPISODE.captures(input)
    {
        let full = cap.get(0).unwrap();
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode).with_priority(-1),
        );
    }

    // 9. Versioned episode: `Show.07v4` or `312v1`.
    if !has_property(&matches, Property::Episode)
        && let Some(cap) = VERSIONED_EPISODE.captures(input)
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
        for cap in LEADING_EPISODE.captures_iter(filename) {
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

    // 9c. Spanish Cap.NNN pattern: [Cap.102] → S1E02, [Cap.1503] → S15E03.
    // Cap patterns provide episode (and optionally verify season) via digit decomposition.
    if !has_property(&matches, Property::Episode) {
        for cap in CAP_PATTERN.captures_iter(input) {
            let full = cap.get(0).unwrap();
            let num1_str = cap.name("num1").unwrap().as_str();
            let num1: u32 = num1_str.parse().unwrap_or(0);
            if num1 == 0 {
                continue;
            }
            let (_, ep1) = (num1 / 100, num1 % 100);
            if ep1 == 0 {
                continue;
            }

            // Check for range: Cap.102_104 → episodes 2..4.
            if let Some(num2_match) = cap.name("num2") {
                let num2: u32 = num2_match.as_str().parse().unwrap_or(0);
                let ep2 = num2 % 100;
                if ep2 > ep1 {
                    matches.extend(episode_range(ep1, ep2, full.start(), full.end(), 3));
                } else {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Episode,
                            ep1.to_string(),
                        )
                        .with_priority(3),
                    );
                }
            } else {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, ep1.to_string())
                        .with_priority(3),
                );
            }
        }
    }

    // 10. 3/4-digit episode number decomposition: 101→S1E01, 2401→S24E01.
    // Only fires when no season/episode found yet.
    // Must appear after the title portion (not in first 5 chars of filename).
    // Skipped for anime-style filenames (bracket groups, underscore separators)
    // where 3-digit numbers are absolute episode counts.
    if !has_property(&matches, Property::Season) && !has_property(&matches, Property::Episode) {
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let filename = &input[fn_start..];
        let is_anime_style = filename.starts_with('[') || filename.contains('_');

        if is_anime_style {
            // For anime, try to find a bare 3-digit episode number after the title.
            for cap in THREE_DIGIT.captures_iter(input) {
                let num_m = cap.name("num").unwrap();
                if num_m.start() < fn_start + 5 || !three_digit_trailing_ok(input, num_m.end()) {
                    continue;
                }
                let num_str = num_m.as_str();
                let num: u32 = num_str.parse().unwrap_or(0);
                if num == 0 || num_str.len() == 4 && (1920..=2039).contains(&num) {
                    continue;
                }
                if num == 264 || num == 265 || num == 128 {
                    continue;
                }
                // Emit as absolute episode (no season decomposition).
                matches.push(
                    MatchSpan::new(
                        num_m.start(),
                        num_m.end(),
                        Property::Episode,
                        num.to_string(),
                    )
                    .with_priority(0),
                );
                break;
            }
        } else {
            for cap in THREE_DIGIT.captures_iter(input) {
                let num_m = cap.name("num").unwrap();
                // Skip if too close to the start of the filename (likely title).
                if num_m.start() < fn_start + 5 || !three_digit_trailing_ok(input, num_m.end()) {
                    continue;
                }
                let num_str = num_m.as_str();
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
                        num_m.start(),
                        num_m.end(),
                        Property::Season,
                        season.to_string(),
                    )
                    .with_priority(0),
                );
                matches.push(
                    MatchSpan::new(
                        num_m.start(),
                        num_m.end(),
                        Property::Episode,
                        episode.to_string(),
                    )
                    .with_priority(0),
                );
                break; // Only decompose the first occurrence.
            }
        }
    }

    // 11. Absolute episode detection.
    // When we have BOTH season+episode from S/E markers AND standalone
    // number ranges nearby, the standalone numbers are absolute episodes.
    // e.g., "Show.Name.313-315.s16e03-05" → episode=[3,4,5], absolute_episode=[313,314,315]
    detect_absolute_episodes(input, &mut matches);

    // 12. Week detection: "Week 45", "Week.12".
    for cap in WEEK.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let week = parse_num(&cap, "week");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Week, week).with_priority(1),
        );
    }

    matches
}

/// Detect absolute episode numbers when both S/E markers and standalone
/// number ranges coexist.
///
/// Pattern: `Show.Name.313-315.s16e03-05`
///   → episode=[3,4,5] (from S/E), absolute_episode=[313,314,315] (standalone)
///
/// Also handles parenthesized: `Title - 16-20 (191-195)`
///   → episode=[16..20], absolute_episode=[191..195]
fn detect_absolute_episodes(input: &str, matches: &mut Vec<MatchSpan>) {
    let has_season = has_property(matches, Property::Season);
    let has_episode = has_property(matches, Property::Episode);
    if !has_season || !has_episode {
        return;
    }

    let max_episode: u32 = matches
        .iter()
        .filter(|m| m.property == Property::Episode)
        .filter_map(|m| m.value.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    // Positions already claimed by season/episode matches.
    let se_spans: Vec<(usize, usize)> = matches
        .iter()
        .filter(|m| m.property == Property::Season || m.property == Property::Episode)
        .map(|m| (m.start, m.end))
        .collect();
    let in_se_span = |pos: usize| -> bool { se_spans.iter().any(|(s, e)| pos >= *s && pos < *e) };

    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let bytes = input.as_bytes();

    // Find number ranges: "313-314", "313-315", or bare "313".
    static NUM_RANGE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?P<start>\d{2,4})(?:-(?P<end>\d{2,4}))?").unwrap());

    for cap in NUM_RANGE.captures_iter(input) {
        let start_m = cap.name("start").unwrap();
        let num_start: u32 = match start_m.as_str().parse() {
            Ok(n) if n > 0 => n,
            _ => continue,
        };

        // Boundary checks: must be preceded and followed by separator or boundary.
        if start_m.start() > 0
            && !matches!(
                bytes[start_m.start() - 1],
                b'.' | b'-' | b'_' | b' ' | b'(' | b'['
            )
        {
            continue;
        }

        let range_end_pos = cap.name("end").map(|m| m.end()).unwrap_or(start_m.end());
        if range_end_pos < bytes.len()
            && !matches!(
                bytes[range_end_pos],
                b'.' | b'-' | b'_' | b' ' | b')' | b']'
            )
        {
            continue;
        }

        // Skip if inside a season/episode match span.
        if in_se_span(start_m.start()) {
            continue;
        }

        // Skip if before the filename or too close to start.
        if start_m.start() < fn_start + 3 {
            continue;
        }

        // Skip year-like and codec-related numbers.
        if (1920..=2039).contains(&num_start) || num_start == 264 || num_start == 265 {
            continue;
        }

        // Must be larger than relative episodes to be "absolute".
        if num_start <= max_episode || num_start < 100 {
            continue;
        }

        let full_start = start_m.start();

        // Emit start number.
        matches.push(MatchSpan::new(
            full_start,
            range_end_pos,
            Property::AbsoluteEpisode,
            num_start.to_string(),
        ));

        // Emit range if present.
        if let Some(end_m) = cap.name("end")
            && let Ok(num_end) = end_m.as_str().parse::<u32>()
            && num_end > num_start
            && num_end - num_start < 50
        {
            for n in (num_start + 1)..=num_end {
                matches.push(MatchSpan::new(
                    full_start,
                    range_end_pos,
                    Property::AbsoluteEpisode,
                    n.to_string(),
                ));
            }
        }
    }
}
