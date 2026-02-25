//! Season / episode detection.
//!
//! Supports S01E02, 1x03, Season/Saison, Episode, 3/4-digit decomposition,
//! anime-style, path-based seasons, and Roman numeral seasons.

use crate::matcher::regex_utils::BoundedRegex;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

type Regex = BoundedRegex;

// ── SxxExx patterns ──

/// S01E02, S01E02E03, S01E02-E05, S01E02-05, S01E02+E03, S01.E02.E03.
/// Captures the full multi-episode suffix for manual parsing.
static SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[. ]?E(?:P)?(?P<ep_start>\d{1,4})(?P<ep_rest>(?:(?:[-+]E?|[. ]E|E)\d{1,4})+)?(?![a-z0-9])",
    )
});

/// S03-E01 (dash between S and E).
static SXX_DASH_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+E(?P<episode>\d{1,4})(?![a-z0-9])")
});

/// S01E01-S01E21 full range (same season, episode range).
static SXXEXX_TO_SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})E(?P<e1>\d{1,4})[-]S(?P<s2>\d{1,3})E(?P<e2>\d{1,4})(?![a-z0-9])",
    )
});

/// S06xE01 (x separator).
static SXX_X_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[xX]E(?P<episode>\d{1,4})(?![a-z0-9])")
});

/// S03-X01 for bonus/extras (x as episode prefix).
static SXX_DASH_XXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+[xX](?P<episode>\d{1,4})(?![a-z0-9])")
});

// ── NxN patterns ──

/// NxN format: 1x03, 5x9, 5x44x45, 4x05-06.
static NXN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])(?P<season>\d{1,2})[xX](?P<ep_start>\d{1,4})(?:[-xX](?P<ep2>\d{1,4}))*(?![a-z0-9])",
    )
});

// ── Standalone episode patterns ──

/// Standalone episode: E01, Ep01, Ep.01, EP01, E02-03, E02-E03, E01-02-03.
/// Does not support space-separated episodes (too ambiguous, causes backtracking).
static EP_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<ep_start>\d{1,4})(?P<ep_rest>(?:(?:[-+]E?|E)\d{1,4})+)?(?![a-z0-9])",
    )
});

/// Episode-only: Episode 1, Episode.01.
static EPISODE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])Episodes?\s*\.?\s*(?P<episode>\d{1,4})(?:\s*[-~]\s*(?P<ep_end>\d{1,4}))?(?![a-z0-9])",
    )
});

/// Versioned episode: `07v4`, `312v1` → episode is the number before 'v'.
static VERSIONED_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])(?P<episode>\d{1,4})v\d{1,2}(?![a-z0-9])"));

/// Leading episode number: `01 - Ep Name`, `003. Show Name`.
/// Only matches at the very start of the filename portion.
static LEADING_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<episode>0\d{1,3}|\d{1,3})(?:\s*[-.]\s+[A-Za-z])"));

/// Anime episode: `- 01`, `- 001` (preceded by dash + space).
static ANIME_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])[-]\s+(?P<episode>\d{1,4})(?:\s|[.]|$)"));

/// Bare episode number after dots: `Show.05.Title` → episode 5.
/// Very weak, only leading-zero or two-digit, must be between dots.
static BARE_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(?P<episode>0\d|\d{2})\.(?![0-9])"));

// ── Season patterns ──

static SEASON_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Tem\.?)\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])",
    )
});

static SEASON_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>(?:X{0,3})(?:IX|IV|V?I{0,3}))(?![a-z])",
    )
});

static SEASON_DIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>\d{1,2})(?:[/\\])")
});

/// S01-only without episode (e.g., `S01Extras`, `S01.Special`).
/// Uses raw regex since we need custom post-match logic.
static S_ONLY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})"));

/// S01-S10 multi-season range.
static S_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})[-]S(?P<s2>\d{1,3})(?![a-z0-9])")
});

/// Season 1-3, Season 1&3, Season 1.3.4 (word-based multi-season).
static SEASON_MULTI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<seasons>\d{1,2}(?:\s*[-&.,]\s*\d{1,2})+)(?![a-z0-9])",
    )
});

/// Season 1.2.3~5, Season 1.2.3 to 5 (discrete list ending with range).
static SEASON_MULTI_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<prefix>\d{1,2}(?:[. ]\d{1,2})*)\s*[. ]?\s*(?:~|to)\s*\.?\s*(?P<end>\d{1,2})(?![a-z0-9])",
    )
});

/// Season 1 to 3, Season 1~3, Saison 1 a 3 (word-based range with separators).
static SEASON_RANGE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<s1>\d{1,2})\s*\.?\s*(?:to|~|a|\.\.)\s*\.?\s*(?P<s2>\d{1,2})(?![a-z0-9])",
    )
});

/// S01S02S03 (concatenated S-prefixed seasons).
static S_CONCAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<first>\d{1,3})(?:S(?P<rest>\d{1,3}))+(?![a-z0-9])")
});

/// S01-02-03 (S-prefixed dash/space separated multi-season without S prefix on rest).
/// Requires zero-padded 2+ digit numbers to avoid matching S03.1 (size context).
static S_MULTI_NUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<seasons>\d{2,3}(?:[-. ]\d{2,3})+)(?![a-z0-9])")
});

/// s01.to.s04, s01-to-s04 (S-prefixed range with "to").
static S_TO_S: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})\.?(?:to|\.to\.)\.?S(?P<s2>\d{1,3})(?![a-z0-9])")
});

/// Season word with "and" / "&" list: Season 1.3 and 5, Season 1.3&5.
static SEASON_LIST_AND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<nums>\d{1,2}(?:[. ]\d{1,2})*)[. ](?:and|&)\s*(?P<last>\d{1,2})(?![a-z0-9])",
    )
});

// ── Spanish Cap patterns ──

/// Spanish `[Cap.NNN]` or `[Cap.NNNN]`: e.g., Cap.102 → S1E02, Cap.1503 → S15E03.
/// Also handles ranges: `[Cap.102_104]` → episodes 2-4.
static CAP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])Cap\.?\s*(?P<num1>\d{3,4})(?:[_](?P<num2>\d{3,4}))?(?:\.[A-Za-z]|[\]\[]|$)",
    )
});

// ── Digit decomposition ──

/// 3-4 digit episode number: 101, 117, 2401 → season/episode decomposition.
/// Matches digits preceded by a separator; trailing check done manually.
static THREE_DIGIT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[.\-_ ](?P<num>\d{3,4})").unwrap());

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
    fn test_cap_single() {
        let m = find_matches("Show.Name.-.Temporada.1.720p.HDTV.x264[Cap.102]SPANISH.AUDIO");
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
    fn test_cap_range() {
        let m = find_matches("Show.Name.-.Temporada.1.720p.HDTV.x264[Cap.102_104]SPANISH.AUDIO");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "2")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "4")
        );
    }

    #[test]
    fn test_cap_four_digit() {
        let m = find_matches("Show.Name.-.Temporada.15.720p.HDTV.x264[Cap.1503]SPANISH.AUDIO");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "15")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
    }

    #[test]
    fn test_cap_four_digit_range() {
        let m = find_matches("Show.Name.-.Temporada.15.720p.HDTV.x264[Cap.1503_1506]SPANISH.AUDIO");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "6")
        );
    }

    #[test]
    fn test_s_range() {
        let m = find_matches("Friends.S01-S10.COMPLETE.720p.BluRay.x264-PtM");
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

    #[test]
    fn test_s_concat() {
        let m = find_matches("Some Series S01S02S03");
        let seasons: Vec<&str> = m
            .iter()
            .filter(|x| x.property == Property::Season)
            .map(|x| x.value.as_str())
            .collect();
        assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
    }

    #[test]
    fn test_s_multi_num() {
        let m = find_matches("Some Series S01-02-03");
        let seasons: Vec<&str> = m
            .iter()
            .filter(|x| x.property == Property::Season)
            .map(|x| x.value.as_str())
            .collect();
        assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
    }

    #[test]
    fn test_season_range_word() {
        let m = find_matches("Show.Name.-.Season.1.to.3.-.Mp4.1080p");
        let seasons: Vec<&str> = m
            .iter()
            .filter(|x| x.property == Property::Season)
            .map(|x| x.value.as_str())
            .collect();
        assert_eq!(seasons.len(), 3, "Expected 3 seasons, got: {:?}", seasons);
    }
}
