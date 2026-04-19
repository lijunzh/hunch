//! Season / episode detection.
//!
//! Supports S01E02, 1x03, Season/Saison, Episode, 3/4-digit decomposition,
//! anime-style, path-based seasons, and Roman numeral seasons.
//!
//! The main `find_matches` orchestrates pattern groups in priority order:
//! 1. SxxExx family (structural: S01E02, S03-E01, etc.)
//! 2. NxN compact notation (1x03, 5x44x45)
//! 3. Season patterns (word-based, ranges, roman numerals, paths)
//! 4. Episode standalone (E01, "Episode 1", anime-style, versioned)
//! 5. Digit decomposition (101→S1E01)  ⚠️ HEURISTIC — see note below
//! 6. Post-processing (absolute episodes, week detection)
//!
//! ## Principle alignment (D6: Dumb engine, smart context)
//!
//! Groups 1–2 are **structural patterns** — unambiguous, context-free.
//! Groups 3–4 are **vocabulary patterns** — keyword-driven, low risk.
//! Group 5 (digit decomposition) and parts of Group 4 (anime detection
//! via `filename.starts_with('[')`) are **fragile heuristics** that
//! guess based on position and format conventions. These should be
//! superseded by cross-file context when available (see DESIGN.md
//! context when available (see DESIGN.md, Cross-file context).
//! Until then, they run at low priority as last-resort fallbacks.

mod patterns;
#[cfg(test)]
mod tests;

use crate::matcher::span::{MatchSpan, Property, Source};
use log::trace;
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
        let full = cap.get(0).expect("group 0 always present");
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
        let full = cap.get(0).expect("group 0 always present");
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
        let full = cap.get(0).expect("group 0 always present");
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(priority),
        );
    }
}

fn has_property(matches: &[MatchSpan], property: Property) -> bool {
    matches.iter().any(|m| m.property == property)
}

/// Parse multi-episode suffixes like "-E05", "+E06", "E07".
fn parse_multi_episodes(first_ep: u32, rest: &str) -> Vec<u32> {
    let mut episodes = vec![first_ep];
    let mut num_buf = String::new();
    let mut last_ep = first_ep;
    let mut pending_range = false;

    for c in rest.chars() {
        if c.is_ascii_digit() {
            num_buf.push(c);
        } else {
            if !num_buf.is_empty() {
                let n: u32 = num_buf.parse().unwrap_or(0);
                if pending_range && n > last_ep {
                    for ep in (last_ep + 1)..=n {
                        episodes.push(ep);
                    }
                } else {
                    episodes.push(n);
                }
                last_ep = n;
                num_buf.clear();
                pending_range = false;
            }
            if c == '-' {
                pending_range = true;
            }
        }
    }
    if !num_buf.is_empty() {
        let n: u32 = num_buf.parse().unwrap_or(0);
        if pending_range && n > last_ep {
            for ep in (last_ep + 1)..=n {
                episodes.push(ep);
            }
        } else {
            episodes.push(n);
        }
    }
    episodes
}

fn episode_range(
    start: u32,
    end: u32,
    span_start: usize,
    span_end: usize,
    priority: i32,
) -> Vec<MatchSpan> {
    (start..=end)
        .map(|ep| {
            MatchSpan::new(span_start, span_end, Property::Episode, ep.to_string())
                .with_priority(priority)
        })
        .collect()
}

fn parse_num(cap: &regex::Captures, name: &str) -> String {
    cap.name(name)
        .map(|m| {
            let s = m.as_str();
            // Strip leading zeros for clean output.
            let n: u32 = s.parse().unwrap_or(0);
            n.to_string()
        })
        .unwrap_or_default()
}

fn roman_to_int(s: &str) -> Option<u32> {
    let upper = s.to_uppercase();
    let mut total = 0u32;
    let mut prev = 0u32;
    for c in upper.chars().rev() {
        let val = match c {
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
            total -= val;
        } else {
            total += val;
        }
        prev = val;
    }
    if total == 0 { None } else { Some(total) }
}

// ── Main orchestrator ──────────────────────────────────────────────────

/// Scan for season/episode patterns (e.g., `S01E02`, `1x03`, `Ep 5`) and return matches.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // High confidence: structural markers (always run)
    try_sxxexx_family(input, &mut matches);

    // Medium-high: compact notation (only if no SxxExx found)
    if matches.is_empty() {
        try_nxn(input, &mut matches);
    }

    // Season patterns: word-based, ranges, roman, paths
    try_season_patterns(input, &mut matches);

    // Episode standalone: E01, "Episode 1", anime, versioned, Cap.NNN
    try_episode_standalone(input, &mut matches);

    // CJK fansub bracket episode: [Group][Title][01][1080P]...
    if !has_property(&matches, Property::Episode) {
        try_cjk_bracket_episode(input, &mut matches);
    }

    // CJK ordinal episode markers: 第N話, 第N集, 第N话, 第N回
    if !has_property(&matches, Property::Episode) {
        try_cjk_episode_marker(input, &mut matches);
    }

    // Low confidence: digit decomposition (only if nothing found)
    if !has_property(&matches, Property::Season) && !has_property(&matches, Property::Episode) {
        try_digit_decomposition(input, &mut matches);
    }

    // Post-processing
    detect_absolute_episodes(input, &mut matches);

    // Week detection
    for cap in WEEK.captures_iter(input) {
        let full = cap.get(0).expect("group 0 always present");
        let week = parse_num(&cap, "week");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Week, week)
                .with_priority(crate::priority::VOCABULARY),
        );
    }

    matches
}

// ── Group 1: SxxExx family ───────────────────────────────────────────

/// S01E02, S01E02-E05, S03-E01, S06xE01, S03-X01.
fn try_sxxexx_family(input: &str, matches: &mut Vec<MatchSpan>) {
    // S01E01-S01E21 full range (must come before SxxExx to win).
    for cap in SXXEXX_TO_SXXEXX.captures_iter(input) {
        let full = cap.get(0).expect("group 0 always present");
        let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
        let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
        let e1: u32 = parse_num(&cap, "e1").parse().unwrap_or(0);
        let e2: u32 = parse_num(&cap, "e2").parse().unwrap_or(0);
        if s1 == s2 && e2 >= e1 {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s1.to_string())
                    .with_priority(crate::priority::STRUCTURAL),
            );
            matches.extend(episode_range(e1, e2, full.start(), full.end(), 5));
        }
    }

    // S01E02 (highest priority, with multi-episode support).
    for cap in SXXEXX.captures_iter(input) {
        let full = cap.get(0).expect("group 0 always present");
        let season: u32 = cap
            .name("season")
            .expect("season group always present")
            .as_str()
            .parse()
            .unwrap_or(0);
        let ep_start: u32 = cap
            .name("ep_start")
            .expect("ep_start group always present")
            .as_str()
            .parse()
            .unwrap_or(0);

        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Season,
                season.to_string(),
            )
            .with_priority(crate::priority::HEURISTIC),
        );

        let ep_rest = cap.name("ep_rest").map(|m| m.as_str()).unwrap_or("");
        if ep_rest.is_empty() {
            matches.push(
                MatchSpan::new(
                    full.start(),
                    full.end(),
                    Property::Episode,
                    ep_start.to_string(),
                )
                .with_priority(crate::priority::STRUCTURAL),
            );
        } else {
            for ep in &parse_multi_episodes(ep_start, ep_rest) {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, ep.to_string())
                        .with_priority(crate::priority::STRUCTURAL),
                );
            }
        }
    }

    // S03-E01 (dash separated).
    if matches.is_empty() {
        match_season_episode(
            &SXX_DASH_EXX,
            input,
            crate::priority::STRUCTURAL - 1,
            matches,
        );
    }
    // S06xE01.
    if matches.is_empty() {
        match_season_episode(&SXX_X_EXX, input, crate::priority::STRUCTURAL - 1, matches);
    }
    // S03-X01 for bonus.
    if matches.is_empty() {
        match_season_episode(
            &SXX_DASH_XXX,
            input,
            crate::priority::STRUCTURAL - 1,
            matches,
        );
    }
}

// ── Group 2: NxN compact notation ──────────────────────────────────

/// 1x03, 5x44x45, 4x05-06.
fn try_nxn(input: &str, matches: &mut Vec<MatchSpan>) {
    for cap in NXN.captures_iter(input) {
        let full = cap.get(0).expect("group 0 always present");
        let season: u32 = cap
            .name("season")
            .expect("season group always present")
            .as_str()
            .parse()
            .unwrap_or(0);
        let ep_start: u32 = cap
            .name("ep_start")
            .expect("ep_start group always present")
            .as_str()
            .parse()
            .unwrap_or(0);

        matches.push(
            MatchSpan::new(
                full.start(),
                full.end(),
                Property::Season,
                season.to_string(),
            )
            .with_priority(crate::priority::PATTERN),
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
                    .with_priority(crate::priority::PATTERN),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, end.to_string())
                        .with_priority(crate::priority::PATTERN),
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
                    .with_priority(crate::priority::PATTERN),
                );
            }
        }
    }
}

// ── Group 3: Season patterns ───────────────────────────────────────

/// Season ranges, concatenated, word-based, roman, path-based.
fn try_season_patterns(input: &str, matches: &mut Vec<MatchSpan>) {
    // Multi-season S-prefixed patterns (must come before single season).
    if !has_property(matches, Property::Season) {
        try_s_prefix_ranges(input, matches);
    }

    // Word-based season patterns.
    if !has_property(matches, Property::Season) {
        try_season_words(input, matches);
    }

    // Season from path directory.
    if !has_property(matches, Property::Season) {
        for cap in SEASON_DIR.captures_iter(input) {
            let full = cap.get(0).expect("group 0 always present");
            let season = parse_num(&cap, "season");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, season)
                    .with_path_based()
                    .with_priority(crate::priority::POSITIONAL),
            );
        }
    }

    // S01-only (without episode).
    if !has_property(matches, Property::Season) {
        for cap in S_ONLY.captures_iter(input) {
            let full = cap.get(0).expect("group 0 always present");
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
                MatchSpan::new(full.start(), full.end(), Property::Season, season)
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
    }
}

/// S01-S10, S01.to.S04, S01S02S03, S01-02-03, etc.
fn try_s_prefix_ranges(input: &str, matches: &mut Vec<MatchSpan>) {
    // S01-S10 range.
    if let Some(cap) = S_RANGE.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
        let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
        for s in s1..=s2 {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
        return;
    }

    // S01.to.S04 range.
    if let Some(cap) = S_TO_S.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
        let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
        for s in s1..=s2 {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
        return;
    }

    // S01S02S03 concatenated.
    static S_NUM_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?i)S(\d{1,3})").unwrap());
    if let Some(cap) = S_CONCAT.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let text = &input[full.start()..full.end()];
        for num_cap in S_NUM_RE.find_iter(text) {
            let num_str = &num_cap.as_str()[1..];
            if let Ok(s) = num_str.parse::<u32>() {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
        return;
    }

    // S01-02-03 (S prefix + dash/space separated numbers).
    if let Some(cap) = S_MULTI_NUM.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let seasons_str = cap.name("seasons").unwrap().as_str();
        let nums: Vec<u32> = seasons_str
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        for s in &nums {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
    }
}

/// "Season 1", "Saison VII", "Season 1-3", "Season 1&3", "Season 1.2.3~5", etc.
fn try_season_words(input: &str, matches: &mut Vec<MatchSpan>) {
    // Season 1 to 3, Saison 1~3 (word-based range).
    if let Some(cap) = SEASON_RANGE_WORD.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let s1: u32 = parse_num(&cap, "s1").parse().unwrap_or(0);
        let s2: u32 = parse_num(&cap, "s2").parse().unwrap_or(0);
        if s2 > s1 {
            for s in s1..=s2 {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
            return;
        }
    }

    // Season 1.3 and 5, Season 1.3&5.
    if let Some(cap) = SEASON_LIST_AND.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
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
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
        return;
    }

    // Season 1.2.3~5 (discrete prefix + range end).
    if let Some(cap) = SEASON_MULTI_RANGE.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let prefix_str = cap.name("prefix").unwrap().as_str();
        let end: u32 = parse_num(&cap, "end").parse().unwrap_or(0);
        let prefix_nums: Vec<u32> = prefix_str
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        for s in &prefix_nums {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                    .with_priority(crate::priority::VOCABULARY),
            );
        }
        if let Some(&last) = prefix_nums.last() {
            for s in (last + 1)..=end {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
        return;
    }

    // Season 1-3, Season 1&3, Season 1.3.4 (generic multi-season).
    if let Some(cap) = SEASON_MULTI.captures(input) {
        let full = cap.get(0).expect("group 0 always present");
        let seasons_str = cap.name("seasons").unwrap().as_str();
        let nums: Vec<u32> = seasons_str
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        let last_sep_is_range = seasons_str
            .rfind(|c: char| !c.is_ascii_digit())
            .map(|i| matches!(seasons_str[i..].chars().next().unwrap_or(' '), '-' | '~'))
            .unwrap_or(false);

        if last_sep_is_range && nums.len() >= 2 {
            let range_start = nums[nums.len() - 2];
            let range_end = nums[nums.len() - 1];
            for s in &nums[..nums.len() - 2] {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
            if range_end > range_start {
                for s in range_start..=range_end {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                            .with_priority(crate::priority::VOCABULARY),
                    );
                }
            }
        } else {
            for s in &nums {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, s.to_string())
                        .with_priority(crate::priority::VOCABULARY),
                );
            }
        }
        return;
    }

    // Standalone: Season N, Saison N.
    match_season(&SEASON_ONLY, input, 2, matches);
    if has_property(matches, Property::Season) {
        return;
    }

    // Roman numeral: Season VII.
    for cap in SEASON_ROMAN.captures_iter(input) {
        let full = cap.get(0).expect("group 0 always present");
        let roman_str = cap
            .name("season")
            .expect("season group always present")
            .as_str();
        if let Some(num) = roman_to_int(roman_str) {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Season, num.to_string())
                    .with_priority(crate::priority::KEYWORD),
            );
        }
    }
}

// ── Group 4: Episode standalone ───────────────────────────────────

/// E01, "Episode 1", anime-style, versioned, Cap.NNN.
fn try_episode_standalone(input: &str, matches: &mut Vec<MatchSpan>) {
    // E01, Ep01, E02-E03.
    if !has_property(matches, Property::Episode) {
        for cap in EP_ONLY.captures_iter(input) {
            let full = cap.get(0).expect("group 0 always present");
            let ep_start: u32 = parse_num(&cap, "ep_start").parse().unwrap_or(0);
            let ep_rest = cap.name("ep_rest").map(|m| m.as_str()).unwrap_or("");

            // Extend match end for space-separated zero-padded episodes:
            // E01 02 03 → episodes [1, 2, 3]
            let mut extended_end = full.end();
            let mut extra_eps: Vec<u32> = Vec::new();
            let remaining = &input[full.end()..];
            let mut pos = 0;
            while pos < remaining.len() && remaining.as_bytes()[pos] == b' ' {
                let num_start = pos + 1;
                if num_start < remaining.len() && remaining.as_bytes()[num_start] == b'0' {
                    let num_end = remaining[num_start..]
                        .find(|c: char| !c.is_ascii_digit())
                        .map(|p| num_start + p)
                        .unwrap_or(remaining.len());
                    if num_end > num_start
                        && let Ok(n) = remaining[num_start..num_end].parse::<u32>()
                    {
                        extra_eps.push(n);
                        extended_end = full.end() + num_end;
                        pos = num_end;
                        continue;
                    }
                }
                break;
            }

            if ep_rest.is_empty() && extra_eps.is_empty() {
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(crate::priority::KEYWORD),
                );
            } else if !extra_eps.is_empty() {
                // Space-separated zero-padded episodes: E01 02 03
                matches.push(
                    MatchSpan::new(
                        full.start(),
                        extended_end,
                        Property::Episode,
                        ep_start.to_string(),
                    )
                    .with_priority(crate::priority::KEYWORD),
                );
                for ep in &extra_eps {
                    matches.push(
                        MatchSpan::new(
                            full.start(),
                            extended_end,
                            Property::Episode,
                            ep.to_string(),
                        )
                        .with_priority(crate::priority::KEYWORD),
                    );
                }
            } else {
                for ep in &parse_multi_episodes(ep_start, ep_rest) {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, ep.to_string())
                            .with_priority(crate::priority::KEYWORD),
                    );
                }
            }
        }
    }

    // "Episode 1" / "Episodes 1-12".
    if !has_property(matches, Property::Episode) {
        for cap in EPISODE_WORD.captures_iter(input) {
            let full = cap.get(0).expect("group 0 always present");
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
                    .with_priority(crate::priority::KEYWORD),
                );
            }
        }
    }

    // Anime-style: `Show - 03`.
    if !has_property(matches, Property::Episode) {
        match_episode(&ANIME_EPISODE, input, 1, matches);
    }

    // Bare episode after dots: `Show.05.Title`.
    if !has_property(matches, Property::Episode)
        && let Some(cap) = BARE_EPISODE.captures(input)
    {
        let full = cap.get(0).expect("group 0 always present");
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(crate::priority::HEURISTIC),
        );
    }

    // Versioned episode: `Show.07v4`.
    if !has_property(matches, Property::Episode)
        && let Some(cap) = VERSIONED_EPISODE.captures(input)
    {
        let full = cap.get(0).expect("group 0 always present");
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(crate::priority::VOCABULARY),
        );
    }

    // Leading episode: `01 - Ep Name`.
    if !has_property(matches, Property::Episode) {
        let fn_start = crate::filename_start(input);
        let filename = &input[fn_start..];
        for cap in LEADING_EPISODE.captures_iter(filename) {
            let ep_match = cap
                .name("episode")
                .expect("episode group always present in CJK_EPISODE_MARKER regex");
            let ep_num: u32 = ep_match.as_str().parse().unwrap_or(0);
            if ep_num == 0 || ep_num > 999 || (1900..=2039).contains(&ep_num) {
                continue;
            }
            let abs_start = fn_start + ep_match.start();
            let abs_end = fn_start + ep_match.end();
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::Episode, ep_num.to_string())
                    .with_priority(crate::priority::DEFAULT),
            );
            break;
        }
    }

    // Spanish Cap.NNN: [Cap.102] → S1E02.
    if !has_property(matches, Property::Episode) {
        for cap in CAP_PATTERN.captures_iter(input) {
            let full = cap.get(0).expect("group 0 always present");
            let num1: u32 = cap.name("num1").unwrap().as_str().parse().unwrap_or(0);
            if num1 == 0 {
                continue;
            }
            let ep1 = num1 % 100;
            if ep1 == 0 {
                continue;
            }

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
                        .with_priority(crate::priority::PATTERN),
                    );
                }
            } else {
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, ep1.to_string())
                        .with_priority(crate::priority::PATTERN),
                );
            }
        }
    }
}

// ── Group 4b: CJK fansub bracket episode ────────────────────────────

/// CJK fansub: `[Group][Title][01][1080P]...
/// Detects bare 1-3 digit numbers in brackets between other brackets.
fn try_cjk_bracket_episode(input: &str, matches: &mut Vec<MatchSpan>) {
    let fn_start = crate::filename_start(input);
    let filename = &input[fn_start..];

    // Only apply to filenames starting with `[` (CJK fansub style).
    if !filename.starts_with('[') {
        return;
    }

    for cap in CJK_BRACKET_EPISODE.captures_iter(filename) {
        let ep_match = cap.name("episode").unwrap();
        let ep_num: u32 = match ep_match.as_str().parse() {
            Ok(n) if n > 0 && n < 1000 => n,
            _ => continue,
        };
        // Skip values that look like resolutions (720, 1080, 480, 2160).
        if matches!(ep_num, 480 | 720 | 1080 | 2160) {
            continue;
        }
        let abs_start = fn_start + ep_match.start();
        let abs_end = fn_start + ep_match.end();
        matches.push(
            MatchSpan::new(abs_start, abs_end, Property::Episode, ep_num.to_string())
                .with_priority(crate::priority::VOCABULARY),
        );
    }
}

// ── Group 4c: CJK ordinal episode markers ─────────────────────────────

/// CJK ordinal episode markers: 第N話 (Japanese), 第N集 (Chinese), 第N话, 第N回.
///
/// Examples:
/// - `第13話` → Episode 13 (Japanese)
/// - `第1集` → Episode 1 (Chinese)
/// - `(BD)十二国記 第13話「月の影...」(...).mkv`
fn try_cjk_episode_marker(input: &str, matches: &mut Vec<MatchSpan>) {
    for cap in CJK_EPISODE_MARKER.captures_iter(input) {
        let ep_match = cap.name("episode").unwrap();
        // Normalize full-width digits (０-９) to ASCII (0-9).
        let normalized: String = ep_match
            .as_str()
            .chars()
            .map(|c| match c {
                '\u{ff10}'..='\u{ff19}' => (b'0' + (c as u32 - 0xff10) as u8) as char,
                _ => c,
            })
            .collect();
        let ep_num: u32 = match normalized.parse() {
            Ok(n) if n > 0 => n,
            _ => continue,
        };
        let abs_start = ep_match.start();
        let abs_end = ep_match.end();
        matches.push(
            MatchSpan::new(abs_start, abs_end, Property::Episode, ep_num.to_string())
                .with_priority(crate::priority::KEYWORD),
        );
    }
}

// ── Group 5: Digit decomposition (⚠️ HEURISTIC) ────────────────────

/// 3/4-digit decomposition: 101→S1E01, 2401→S24E01.
///
/// ⚠️ **Fragile heuristic** (D6 violation) — this guesses season/episode
/// from bare numbers using digit splitting. It's a last-resort fallback
/// that only runs when no structural patterns (SxxExx, NxN) matched.
///
/// The `is_anime_style` check (`filename.starts_with('[')`) is also
/// fragile — it assumes bracket-prefixed filenames are anime.
///
/// **Principled fix:** Use cross-file context (DESIGN.md, Cross-file context) to
/// detect episode numbering patterns across siblings instead of guessing
/// from digit positions in a single filename.
fn try_digit_decomposition(input: &str, matches: &mut Vec<MatchSpan>) {
    let fn_start = crate::filename_start(input);
    let filename = &input[fn_start..];
    // ⚠️ Fragile: assumes bracket-prefix = anime. Should use context instead.
    let is_anime_style = filename.starts_with('[') || filename.contains('_');

    for cap in THREE_DIGIT.captures_iter(input) {
        let num_m = cap.name("num").unwrap();
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
        if crate::CODEC_NUMBERS.contains(&num) {
            continue;
        }

        if is_anime_style {
            // Anime: emit as absolute episode (no season decomposition).
            trace!(
                "[HEURISTIC] digit decomposition: {} → Episode={} (anime-style)",
                num, num
            );
            matches.push(
                MatchSpan::new(
                    num_m.start(),
                    num_m.end(),
                    Property::Episode,
                    num.to_string(),
                )
                .with_priority(crate::priority::DEFAULT)
                .with_source(Source::Heuristic),
            );
            break;
        } else {
            // Scene: decompose e.g. 501 → S5E01.
            let (season, episode) = (num / 100, num % 100);
            if season == 0 || episode == 0 || season > 30 || episode > 99 {
                continue;
            }
            trace!(
                "[HEURISTIC] digit decomposition: {} → S{:02}E{:02}",
                num, season, episode
            );
            matches.push(
                MatchSpan::new(
                    num_m.start(),
                    num_m.end(),
                    Property::Season,
                    season.to_string(),
                )
                .with_priority(crate::priority::DEFAULT)
                .with_source(Source::Heuristic),
            );
            matches.push(
                MatchSpan::new(
                    num_m.start(),
                    num_m.end(),
                    Property::Episode,
                    episode.to_string(),
                )
                .with_priority(crate::priority::DEFAULT)
                .with_source(Source::Heuristic),
            );
            break;
        }
    }
}

// ── Group 6: Post-processing ──────────────────────────────────────

/// Detect absolute episode numbers when both S/E markers and standalone
/// number ranges coexist.
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

    let se_spans: Vec<(usize, usize)> = matches
        .iter()
        .filter(|m| m.property == Property::Season || m.property == Property::Episode)
        .map(|m| (m.start, m.end))
        .collect();
    let in_se_span = |pos: usize| -> bool { se_spans.iter().any(|(s, e)| pos >= *s && pos < *e) };

    let first_se_start = se_spans.iter().map(|(s, _)| *s).min().unwrap_or(usize::MAX);
    let fn_start = crate::filename_start(input);
    let bytes = input.as_bytes();

    static NUM_RANGE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?P<start>\d{2,4})(?:-(?P<end>\d{2,4}))?").unwrap());

    for cap in NUM_RANGE.captures_iter(input) {
        let start_m = cap.name("start").unwrap();
        let num_start: u32 = match start_m.as_str().parse() {
            Ok(n) if n > 0 => n,
            _ => continue,
        };

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

        if in_se_span(start_m.start()) {
            continue;
        }
        if start_m.start() < fn_start + 3 {
            continue;
        }
        if start_m.start() < first_se_start {
            continue;
        }
        if (1920..=2039).contains(&num_start) || num_start == 264 || num_start == 265 {
            continue;
        }
        if num_start <= max_episode || num_start < 100 {
            continue;
        }

        let full_start = start_m.start();
        matches.push(MatchSpan::new(
            full_start,
            range_end_pos,
            Property::AbsoluteEpisode,
            num_start.to_string(),
        ));

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
