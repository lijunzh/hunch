//! Season / episode detection.
//! Supports S01E02, 1x03, Season/Saison, Episode, 3/4-digit decomposition,
//! anime-style, path-based seasons, and Roman numeral seasons.
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;
use fancy_regex::Regex;
use std::sync::LazyLock;

/// S01E02, S01E02E03, S01E02-E05, S01E02-05, S01E02+E03, S01.E02.E03.
static SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[. ]?E(?P<ep_start>\d{1,4})(?:(?:[-+. ]E?|E)(?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap()
});

/// S03-E01 (dash between S and E).
static SXX_DASH_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+E(?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

/// S06xE01 (x separator).
static SXX_X_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[xX]E(?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

/// NxN format: 1x03, 5x9, 5x44x45.
static NXN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])(?P<season>\d{1,2})[xX](?P<ep_start>\d{1,4})(?:[xX](?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap()
});

/// Standalone episode: E01, Ep01, Ep.01, E02-03, E02-E03.
static EP_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<ep_start>\d{1,4})(?:[-+]E?(?P<ep2>\d{1,4}))?(?![a-z0-9])"
    ).unwrap()
});

static SEASON_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Tem\.?)\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])",
    )
    .unwrap()
});

static SEASON_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>(?:X{0,3})(?:IX|IV|V?I{0,3}))(?![a-z])"
    ).unwrap()
});

static SEASON_DIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>\d{1,2})(?:[/\\])").unwrap()
});

/// Episode-only: Episode 1, Episode.01.
static EPISODE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z])Episode\s*\.?\s*(?P<episode>\d{1,4})(?![a-z0-9])").unwrap()
});

/// 3-4 digit episode number: 101, 117, 2401 → season/episode decomposition.
/// Must be preceded by a separator and not be a year (1900-2099).
/// Only matches after a dot/dash/space and NOT at the very start of filename.
static THREE_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<=[.\-_ ])(?P<num>\d{3,4})(?=[.\-_ ]|$)").unwrap());

/// Anime episode: `- 01`, `- 001` (preceded by dash + space).
static ANIME_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])[-]\s+(?P<episode>\d{1,4})(?:\s|[.]|$)").unwrap());

/// Bare episode number after dots: `Show.05.Title` → episode 5.
/// Very weak, only leading-zero or two-digit, must be between dots.
static BARE_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(?P<episode>0\d|\d{2})\.(?![0-9])").unwrap());

/// S01-only without episode (e.g., `S01Extras`, `S01.Special`).
/// The lookahead avoids matching S01E02 (which is handled by SXXEXX).
static S_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})(?!\d|E\d|[xX]\d)").unwrap()
});

/// S01-S10 multi-season range.
static S_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})[-]S(?P<s2>\d{1,3})(?![a-z0-9])").unwrap()
});

/// Season 1-3, Season 1&3, Season 1.3.4 (word-based multi-season).
static SEASON_MULTI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<seasons>\d{1,2}(?:\s*[-&.,]\s*\d{1,2})+)(?![a-z0-9])"
    ).unwrap()
});

/// S03-X01 for bonus/extras (x as episode prefix).
static SXX_DASH_XXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+[xX](?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

/// Versioned episode: `07v4`, `312v1` → episode is the number before 'v'.
static VERSIONED_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])(?P<episode>\d{1,4})v\d{1,2}(?![a-z0-9])").unwrap());

/// Leading episode number: `01 - Ep Name`, `003. Show Name`.
/// Only matches at the very start of the filename portion.
static LEADING_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<episode>0\d{1,3}|\d{1,3})(?:\s*[-.]\s+[A-Za-z])").unwrap());

/// Helper: iterate fancy_regex captures.
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

/// Generate a range of episode numbers as MatchSpans.
fn episode_range(
    start_ep: u32,
    end_ep: u32,
    span_start: usize,
    span_end: usize,
    priority: i32,
) -> Vec<MatchSpan> {
    let mut out = Vec::new();
    for ep in start_ep..=end_ep {
        out.push(
            MatchSpan::new(span_start, span_end, Property::Episode, ep.to_string())
                .with_priority(priority),
        );
    }
    out
}

/// Parse a named capture group as a u32 and return as String (strips leading zeros).
fn parse_num(cap: &fancy_regex::Captures, name: &str) -> String {
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

pub struct EpisodeMatcher;

impl PropertyMatcher for EpisodeMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
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
                        MatchSpan::new(
                            full.start(),
                            full.end(),
                            Property::Episode,
                            end.to_string(),
                        )
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
            for cap in captures_iter(&SXX_DASH_EXX, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(4),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(4),
                );
            }
        }

        // 3. S06xE01.
        if matches.is_empty() {
            for cap in captures_iter(&SXX_X_EXX, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(4),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(4),
                );
            }
        }

        // 4. S03-X01 for bonus.
        if matches.is_empty() {
            for cap in captures_iter(&SXX_DASH_XXX, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(4),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(4),
                );
            }
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
        if !matches.iter().any(|m| m.property == Property::Season) {
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

        if !matches.iter().any(|m| m.property == Property::Season) {
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
                            MatchSpan::new(
                                full.start(),
                                full.end(),
                                Property::Season,
                                s.to_string(),
                            )
                            .with_priority(1),
                        );
                    }
                } else {
                    for s in &nums {
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
            }
        }

        // 6b. Standalone season/episode words.
        if !matches.iter().any(|m| m.property == Property::Season) {
            for cap in captures_iter(&SEASON_ONLY, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(2),
                );
            }
        }

        // Roman numeral seasons.
        if !matches.iter().any(|m| m.property == Property::Season) {
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
        if !matches.iter().any(|m| m.property == Property::Season) {
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
        if !matches.iter().any(|m| m.property == Property::Season) {
            for cap in captures_iter(&S_ONLY, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_priority(1),
                );
            }
        }

        // Episode standalone patterns.
        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&EP_ONLY, input) {
                let full = cap.get(0).unwrap();
                let ep_start: u32 = parse_num(&cap, "ep_start").parse().unwrap_or(0);
                let ep2: Option<u32> = cap.name("ep2").and_then(|m| m.as_str().parse().ok());
                if let Some(ep_end) = ep2 {
                    // Multi-episode: E02-03 or E02-E03.
                    for ep in ep_start..=ep_end {
                        matches.push(
                            MatchSpan::new(
                                full.start(),
                                full.end(),
                                Property::Episode,
                                ep.to_string(),
                            )
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

        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&EPISODE_WORD, input) {
                let full = cap.get(0).unwrap();
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(2),
                );
            }
        }

        // 7. Anime-style episode: `Show - 03` or `Show - 003`.
        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&ANIME_EPISODE, input) {
                let full = cap.get(0).unwrap();
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(1),
                );
            }
        }

        // 8. Bare episode after dots: `Show.05.Title`.
        if !matches.iter().any(|m| m.property == Property::Episode)
            && let Some(cap) = captures_iter(&BARE_EPISODE, input).into_iter().next()
        {
            let full = cap.get(0).unwrap();
            let episode = parse_num(&cap, "episode");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                    .with_priority(-1),
            );
        }

        // 9. Versioned episode: `Show.07v4` or `312v1`.
        if !matches.iter().any(|m| m.property == Property::Episode)
            && let Some(cap) = captures_iter(&VERSIONED_EPISODE, input).into_iter().next()
        {
            let full = cap.get(0).unwrap();
            let episode = parse_num(&cap, "episode");
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                    .with_priority(1),
            );
        }

        // 9b. Leading episode: `01 - Ep Name`, `003. Show Name`.
        if !matches.iter().any(|m| m.property == Property::Episode) {
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
        if !matches.iter().any(|m| m.property == Property::Season)
            && !matches.iter().any(|m| m.property == Property::Episode)
        {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s01e02() {
        let m = EpisodeMatcher.find_matches("Show.S01E02.mkv");
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
        let m = EpisodeMatcher.find_matches("Show.S01E01E02.mkv");
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
        let m = EpisodeMatcher.find_matches("Show.S03E01-02.mkv");
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
        let m = EpisodeMatcher.find_matches("Show.S03E01-E02.mkv");
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
        let m = EpisodeMatcher.find_matches("Show.1x03.mkv");
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
        let m = EpisodeMatcher.find_matches("Parks_and_Recreation-s03-e01.mkv");
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
        let m = EpisodeMatcher.find_matches("The Office - S06xE01.avi");
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
        let m = EpisodeMatcher.find_matches("Show Season 2 Episode 5");
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
        let m = EpisodeMatcher.find_matches("Show.E05.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "5")
        );
    }

    #[test]
    fn test_season_dir() {
        let m = EpisodeMatcher.find_matches("TV/Show/Season 6/file.avi");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "6")
        );
    }

    #[test]
    fn test_s01_only() {
        let m = EpisodeMatcher.find_matches("Show.S01Extras.mkv");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Season && x.value == "1")
        );
    }

    #[test]
    fn test_s03_dash_x01() {
        let m = EpisodeMatcher.find_matches("Parks_and_Recreation-s03-x01.mkv");
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
        let m = EpisodeMatcher.find_matches("the.mentalist.501.hdtv.mkv");
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
        let m = EpisodeMatcher.find_matches("new.girl.117.hdtv.mkv");
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
        let m = EpisodeMatcher.find_matches("the.simpsons.2401.hdtv.mkv");
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
        let m = EpisodeMatcher.find_matches("Show Name - 03 Vostfr HD");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "3")
        );
    }

    #[test]
    fn test_bare_dot_episode() {
        let m = EpisodeMatcher.find_matches("Neverwhere.05.Down.Street.avi");
        assert!(
            m.iter()
                .any(|x| x.property == Property::Episode && x.value == "5")
        );
    }

    #[test]
    fn test_s_range() {
        let m = EpisodeMatcher.find_matches("Friends.S01-S10.COMPLETE.720p.BluRay.x264-PtM");
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
