//! Regex patterns and helpers for season/episode detection.

pub use crate::matcher::regex_utils::captures_iter;
use crate::matcher::span::{MatchSpan, Property};
use fancy_regex::Regex;
use std::sync::LazyLock;

// ── SxxExx patterns ──

/// S01E02, S01E02E03, S01E02-E05, S01E02-05, S01E02+E03, S01.E02.E03.
pub static SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[. ]?E(?P<ep_start>\d{1,4})(?:(?:[-+. ]E?|E)(?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap()
});

/// S03-E01 (dash between S and E).
pub static SXX_DASH_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+E(?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

/// S06xE01 (x separator).
pub static SXX_X_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[xX]E(?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

/// S03-X01 for bonus/extras (x as episode prefix).
pub static SXX_DASH_XXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+[xX](?P<episode>\d{1,4})(?![a-z0-9])")
        .unwrap()
});

// ── NxN patterns ──

/// NxN format: 1x03, 5x9, 5x44x45.
pub static NXN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])(?P<season>\d{1,2})[xX](?P<ep_start>\d{1,4})(?:[xX](?P<ep2>\d{1,4}))*(?![a-z0-9])"
    ).unwrap()
});

// ── Standalone episode patterns ──

/// Standalone episode: E01, Ep01, Ep.01, E02-03, E02-E03.
pub static EP_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<ep_start>\d{1,4})(?:[-+]E?(?P<ep2>\d{1,4}))?(?![a-z0-9])"
    ).unwrap()
});

/// Episode-only: Episode 1, Episode.01.
pub static EPISODE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z])Episode\s*\.?\s*(?P<episode>\d{1,4})(?![a-z0-9])").unwrap()
});

/// Versioned episode: `07v4`, `312v1` → episode is the number before 'v'.
pub static VERSIONED_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])(?P<episode>\d{1,4})v\d{1,2}(?![a-z0-9])").unwrap());

/// Leading episode number: `01 - Ep Name`, `003. Show Name`.
/// Only matches at the very start of the filename portion.
pub static LEADING_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<episode>0\d{1,3}|\d{1,3})(?:\s*[-.]\s+[A-Za-z])").unwrap());

/// Anime episode: `- 01`, `- 001` (preceded by dash + space).
pub static ANIME_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])[-]\s+(?P<episode>\d{1,4})(?:\s|[.]|$)").unwrap());

/// Bare episode number after dots: `Show.05.Title` → episode 5.
/// Very weak, only leading-zero or two-digit, must be between dots.
pub static BARE_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(?P<episode>0\d|\d{2})\.(?![0-9])").unwrap());

// ── Season patterns ──

pub static SEASON_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Tem\.?)\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])",
    )
    .unwrap()
});

pub static SEASON_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>(?:X{0,3})(?:IX|IV|V?I{0,3}))(?![a-z])"
    ).unwrap()
});

pub static SEASON_DIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Season|Saison|Temporada)\s*\.?\s*(?P<season>\d{1,2})(?:[/\\])").unwrap()
});

/// S01-only without episode (e.g., `S01Extras`, `S01.Special`).
pub static S_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})(?!\d|E\d|[xX]\d)").unwrap()
});

/// S01-S10 multi-season range.
pub static S_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})[-]S(?P<s2>\d{1,3})(?![a-z0-9])").unwrap()
});

/// Season 1-3, Season 1&3, Season 1.3.4 (word-based multi-season).
pub static SEASON_MULTI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"(?i)(?<![a-z])(?:Season|Saison|Temporada)\s*\.?\s*(?P<seasons>\d{1,2}(?:\s*[-&.,]\s*\d{1,2})+)(?![a-z0-9])"
    ).unwrap()
});

// ── Digit decomposition ──

/// 3-4 digit episode number: 101, 117, 2401 → season/episode decomposition.
pub static THREE_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<=[.\-_ ])(?P<num>\d{3,4})(?=[.\-_ ]|$)").unwrap());

/// Match a simple season+episode pair from a regex with named groups `season` and `episode`.
pub fn match_season_episode(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in captures_iter(re, input) {
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
pub fn match_season(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in captures_iter(re, input) {
        let full = cap.get(0).unwrap();
        let season = parse_num(&cap, "season");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Season, season)
                .with_priority(priority),
        );
    }
}

/// Match an episode-only value from a regex with named group `episode`.
pub fn match_episode(re: &Regex, input: &str, priority: i32, matches: &mut Vec<MatchSpan>) {
    for cap in captures_iter(re, input) {
        let full = cap.get(0).unwrap();
        let episode = parse_num(&cap, "episode");
        matches.push(
            MatchSpan::new(full.start(), full.end(), Property::Episode, episode)
                .with_priority(priority),
        );
    }
}

/// Check if matches contain a given property.
pub fn has_property(matches: &[MatchSpan], property: Property) -> bool {
    matches.iter().any(|m| m.property == property)
}

// ── Helpers ──
/// Generate a range of episode numbers as MatchSpans.
pub fn episode_range(
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
pub fn parse_num(cap: &fancy_regex::Captures, name: &str) -> String {
    cap.name(name)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap_or(0)
        .to_string()
}

/// Parse a Roman numeral string to an integer.
pub fn roman_to_int(s: &str) -> Option<u32> {
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
