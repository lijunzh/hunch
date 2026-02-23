//! Season / episode detection.
//!
//! Supported patterns:
//! - S01E02, S01E02E03, S01E02-E05, S01E02-05
//! - s03-e01, s03-e02 (dash-separated)
//! - S06xE01 (x separator)
//! - 1x03, 5x9, 5x44x45x46 (NxN)
//! - E01, Ep01, Ep.01 (standalone episode)
//! - Season 1, Season.01 (standalone season)
//! - Episode 1, Episode.01 (episode word)
//! - 3-digit: 101 → S01E01, 117 → S01E17
//! - [401] bracket episode numbers
//! - Season from path: /Season 6/ → S06
//! - Anime: `- 01`, `- 001`

use lazy_static::lazy_static;
use fancy_regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// S01E02, S01E02E03, S01E02-E05, S01E02-05.
    static ref SXXEXX: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})\s*E(?P<ep_start>\d{1,4})(?:(?:[-]E?|E)(?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap();

    /// S03-E01 (dash between S and E).
    static ref SXX_DASH_EXX: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+E(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();

    /// S06xE01 (x separator).
    static ref SXX_X_EXX: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[xX]E(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();

    /// NxN format: 1x03, 5x9, 5x44x45.
    static ref NXN: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])(?P<season>\d{1,2})[xX](?P<ep_start>\d{1,4})(?:[xX](?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap();

    /// Standalone episode: E01, Ep01, Ep.01.
    static ref EP_ONLY: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();

    /// Season-only: Season 1, Season.01, Saison 2.
    static ref SEASON_ONLY: Regex = Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison)\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])"
    ).unwrap();

    /// Season from path directory: /Season 6/, /Season.02/.
    static ref SEASON_DIR: Regex = Regex::new(
        r"(?i)(?:Season|Saison)\s*\.?\s*(?P<season>\d{1,2})(?:[/\\])"
    ).unwrap();

    /// Episode-only: Episode 1, Episode.01.
    static ref EPISODE_WORD: Regex = Regex::new(
        r"(?i)(?<![a-z])Episode\s*\.?\s*(?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();

    /// 3-4 digit episode number: 101, 117, 2401 → season/episode decomposition.
    /// Must be preceded by a separator and not be a year (1900-2099).
    static ref THREE_DIGIT: Regex = Regex::new(
        r"(?<![0-9a-zA-Z])(?P<num>\d{3,4})(?=[^\d a-zA-Z]|$)"
    ).unwrap();

    /// Bracket episode: [401], [S01E02].
    static ref BRACKET_EPISODE: Regex = Regex::new(
        r"(?i)\[(?P<num>\d{3,4})\]"
    ).unwrap();

    /// Anime episode: `- 01`, `- 001` (preceded by dash + space).
    static ref ANIME_EPISODE: Regex = Regex::new(
        r"(?<![a-z0-9])[-]\s+(?P<episode>\d{1,4})(?:\s|[.]|$)"
    ).unwrap();

    /// Bare episode number after dots: `Show.05.Title` → episode 5.
    /// Very weak, only leading-zero or two-digit, must be between dots.
    static ref BARE_EPISODE: Regex = Regex::new(
        r"\.(?P<episode>0\d|\d{2})\.(?![0-9])"
    ).unwrap();

    /// S01-only without episode (e.g., `S01Extras`, `S01.Special`).
    /// The lookahead avoids matching S01E02 (which is handled by SXXEXX).
    static ref S_ONLY: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})(?!\d|E\d|[xX]\d)"
    ).unwrap();

    /// S03-X01 for bonus/extras (x as episode prefix).
    static ref SXX_DASH_XXX: Regex = Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+[xX](?P<episode>\d{1,4})(?![a-z0-9])"
    ).unwrap();
}

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
                MatchSpan::new(full.start(), full.end(), Property::Season, season.to_string())
                    .with_tag("SxxExx")
                    .with_priority(5),
            );

            // Check for multi-episode.
            let ep_end = cap
                .name("ep2")
                .and_then(|m| m.as_str().parse::<u32>().ok());

            match ep_end {
                Some(end) if end > ep_start => {
                    matches.extend(episode_range(ep_start, end, full.start(), full.end(), 5));
                }
                Some(end) => {
                    // E01E02 style (not a range, individual episodes).
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, ep_start.to_string())
                            .with_priority(5),
                    );
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, end.to_string())
                            .with_priority(5),
                    );
                }
                None => {
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::Episode, ep_start.to_string())
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
                    MatchSpan::new(full.start(), full.end(), Property::Season, season.to_string())
                        .with_priority(3),
                );

                let ep_end = cap
                    .name("ep2")
                    .and_then(|m| m.as_str().parse::<u32>().ok());

                match ep_end {
                    Some(end) if end > ep_start => {
                        matches.extend(episode_range(ep_start, end, full.start(), full.end(), 3));
                    }
                    Some(end) => {
                        matches.push(
                            MatchSpan::new(full.start(), full.end(), Property::Episode, ep_start.to_string())
                                .with_priority(3),
                        );
                        matches.push(
                            MatchSpan::new(full.start(), full.end(), Property::Episode, end.to_string())
                                .with_priority(3),
                        );
                    }
                    None => {
                        matches.push(
                            MatchSpan::new(full.start(), full.end(), Property::Episode, ep_start.to_string())
                                .with_priority(3),
                        );
                    }
                }
            }
        }

        // 6. Standalone season/episode words.
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

        // Season from path directory.
        if !matches.iter().any(|m| m.property == Property::Season) {
            for cap in captures_iter(&SEASON_DIR, input) {
                let full = cap.get(0).unwrap();
                let season = parse_num(&cap, "season");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season)
                        .with_tag("path-season")
                        .with_priority(1),
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
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_priority(2),
                );
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
                        .with_tag("anime")
                        .with_priority(1),
                );
            }
        }

        // 8. Bare episode after dots: `Show.05.Title`.
        if !matches.iter().any(|m| m.property == Property::Episode) {
            for cap in captures_iter(&BARE_EPISODE, input) {
                let full = cap.get(0).unwrap();
                let episode = parse_num(&cap, "episode");
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                        .with_tag("bare")
                        .with_priority(-1),
                );
                break; // Only the first bare number.
            }
        }

        // 9. 3/4-digit episode number decomposition: 101→S1E01, 2401→S24E01.
        // Only fires when no season/episode found yet.
        if !matches.iter().any(|m| m.property == Property::Season)
            && !matches.iter().any(|m| m.property == Property::Episode)
        {
            for cap in captures_iter(&THREE_DIGIT, input) {
                let full = cap.get(0).unwrap();
                let num_str = cap.name("num").unwrap().as_str();
                let num: u32 = num_str.parse().unwrap_or(0);
                if num == 0 {
                    continue;
                }
                // Skip year-like 4-digit numbers (1920-2039).
                if num_str.len() == 4 && (1920..=2039).contains(&num) {
                    continue;
                }
                // Decompose: e.g., 501 → S5E01, 117 → S1E17, 2401 → S24E01.
                let (season, episode) = if num_str.len() == 4 {
                    (num / 100, num % 100)
                } else {
                    // 3-digit: first digit is season, last two are episode.
                    (num / 100, num % 100)
                };
                if season == 0 || episode == 0 || season > 50 || episode > 99 {
                    continue;
                }
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Season, season.to_string())
                        .with_priority(0),
                );
                matches.push(
                    MatchSpan::new(full.start(), full.end(), Property::Episode, episode.to_string())
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
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "2"));
    }

    #[test]
    fn test_multi_episode_e01e02() {
        let m = EpisodeMatcher.find_matches("Show.S01E01E02.mkv");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "2"));
    }

    #[test]
    fn test_multi_episode_e01_dash_02() {
        let m = EpisodeMatcher.find_matches("Show.S03E01-02.mkv");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "2"));
    }

    #[test]
    fn test_multi_episode_e01_dash_e02() {
        let m = EpisodeMatcher.find_matches("Show.S03E01-E02.mkv");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "2"));
    }

    #[test]
    fn test_1x03() {
        let m = EpisodeMatcher.find_matches("Show.1x03.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "3"));
    }

    #[test]
    fn test_s03_dash_e01() {
        let m = EpisodeMatcher.find_matches("Parks_and_Recreation-s03-e01.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "3"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
    }

    #[test]
    fn test_s06xe01() {
        let m = EpisodeMatcher.find_matches("The Office - S06xE01.avi");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "6"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
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
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "5"));
    }

    #[test]
    fn test_season_dir() {
        let m = EpisodeMatcher.find_matches("TV/Show/Season 6/file.avi");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "6"));
    }

    #[test]
    fn test_s01_only() {
        let m = EpisodeMatcher.find_matches("Show.S01Extras.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "1"));
    }

    #[test]
    fn test_s03_dash_x01() {
        let m = EpisodeMatcher.find_matches("Parks_and_Recreation-s03-x01.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "3"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
    }

    #[test]
    fn test_three_digit_501() {
        let m = EpisodeMatcher.find_matches("the.mentalist.501.hdtv.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "5"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
    }

    #[test]
    fn test_three_digit_117() {
        let m = EpisodeMatcher.find_matches("new.girl.117.hdtv.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "1"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "17"));
    }

    #[test]
    fn test_four_digit_2401() {
        let m = EpisodeMatcher.find_matches("the.simpsons.2401.hdtv.mkv");
        assert!(m.iter().any(|x| x.property == Property::Season && x.value == "24"));
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "1"));
    }

    #[test]
    fn test_anime_dash_episode() {
        let m = EpisodeMatcher.find_matches("Show Name - 03 Vostfr HD");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "3"));
    }

    #[test]
    fn test_bare_dot_episode() {
        let m = EpisodeMatcher.find_matches("Neverwhere.05.Down.Street.avi");
        assert!(m.iter().any(|x| x.property == Property::Episode && x.value == "5"));
    }
}
