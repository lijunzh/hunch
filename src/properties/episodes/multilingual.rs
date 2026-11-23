//! Season and episode markers in other languages and scripts.
//!
//! The SxxExx family and the English/French/Spanish/Italian `Season N`
//! words live in `patterns.rs`. This module adds what guessit 4.5 reads
//! and hunch didn't:
//!
//! - season words: `Staffel 2`, `Seizoen 4`, `Sezon 1`, `Säsong 2`,
//!   `Kausi 3`, `Сезон 3`, and number-first forms `2. Staffel`,
//!   `04ª Temporada`, `1a. Temporada`, `3 Temporada`, `5-й сезон`
//! - episode words: `Episodio 5`, `Épisode 5`, `Capitulo 3`, `Folge 5`,
//!   `Aflevering 5`, `Odcinek 5`, `Avsnitt 5`, `Jakso 5`, `Bölüm 7`,
//!   `Эпизод 21`, `Серия 5`, and number-first `24 серия`, `09-я серия`,
//!   `7.Bolum`
//! - totals: `Capitulo 5 de 12`, `Temporada 1 de 5`, `Folge 5 von 12`,
//!   `5 серия из 12`
//! - CJK: `第2季` / `第二季` (season), `第3集` / `第二十三集` / `第195話`
//!   (episode, Chinese numerals included), `シーズン2` (season)
//!
//! Every span covers the whole marker (`第2季`, not just `2`) so the
//! marker text can't leak into the title.

use regex::Regex;
use std::sync::LazyLock;

use crate::matcher::span::{MatchSpan, Property};
use crate::priority::KEYWORD;

const SEASON_WORDS: &str =
    r"staffel|seizoen|sezon|säsong|sasong|sæson|sesong|kausi|сезон|sezóna|évad";
const SEASON_WORDS_NUMBER_FIRST: &str =
    r"temporada|staffel|seizoen|sezon|säsong|season|saison|stagione|сезон|сезона";
const EPISODE_WORDS: &str = r"episodio|épisode|capitulo|capítulo|folge|aflevering|odcinek|avsnitt|jakso|bölüm|bolum|эпизод|серия|серії|epizoda|epizód";
const EPISODE_WORDS_NUMBER_FIRST: &str = r"серия|bölüm|bolum|folge|odcinek|эпизод";
const COUNT_WORDS: &str = r"of|de|di|von|van|из|z|av|sur|od";
const ALL_SEASON_WORDS: &str =
    r"season|saison|temporada|stagione|staffel|seizoen|sezon|säsong|сезон";

static SEASON_WORD_FIRST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?i)(?:{SEASON_WORDS})[\s._]*(?P<n>\d{{1,2}})")).expect("valid regex")
});

static SEASON_NUMBER_FIRST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)(?P<n>\d{{1,2}})(?:ª|º|a\.?|-й|-го|-ий|\.)?[\s._]*(?:{SEASON_WORDS_NUMBER_FIRST})"
    ))
    .expect("valid regex")
});

static EPISODE_WORD_FIRST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?i)(?:{EPISODE_WORDS})[\s._]*(?P<n>\d{{1,4}})")).expect("valid regex")
});

static EPISODE_NUMBER_FIRST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)(?P<n>\d{{1,4}})(?:-я|-й|\.)?[\s._]*(?:{EPISODE_WORDS_NUMBER_FIRST})"
    ))
    .expect("valid regex")
});

/// `<marker> N <of-word> M`: the total that follows a marker.
static SEASON_TOTAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)(?:{ALL_SEASON_WORDS})[\s._]*\d{{1,2}}[\s._]+(?:{COUNT_WORDS})[\s._]+(?P<total>\d{{1,3}})"
    ))
    .expect("valid regex")
});

static EPISODE_TOTAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)(?:(?:{EPISODE_WORDS})[\s._]*\d{{1,4}}|\d{{1,4}}[\s._]*(?:{EPISODE_WORDS_NUMBER_FIRST}))[\s._]+(?:{COUNT_WORDS})[\s._]+(?P<total>\d{{1,4}})"
    ))
    .expect("valid regex")
});

const CJK_NUM: &str = r"[0-9０-９一二三四五六七八九十百零〇两]+";

static CJK_SEASON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"第(?P<n>{CJK_NUM})季|シーズン\s*(?P<k>[0-9０-９]+)"
    ))
    .expect("valid regex")
});

static CJK_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"第(?P<n>{CJK_NUM})[話集话回]")).expect("valid regex"));

/// Parse ASCII, full-width or Chinese-numeral digits (`23`, `２３`,
/// `二十三`, `十一`, `一百零五`).
pub(super) fn parse_cjk_number(s: &str) -> Option<u32> {
    let ascii: String = s
        .chars()
        .map(|c| match c {
            '\u{ff10}'..='\u{ff19}' => char::from(b'0' + (c as u32 - 0xff10) as u8),
            _ => c,
        })
        .collect();
    if let Ok(n) = ascii.parse::<u32>() {
        return Some(n);
    }
    let digit = |c: char| -> Option<u32> {
        Some(match c {
            '零' | '〇' => 0,
            '一' => 1,
            '二' | '两' => 2,
            '三' => 3,
            '四' => 4,
            '五' => 5,
            '六' => 6,
            '七' => 7,
            '八' => 8,
            '九' => 9,
            _ => return None,
        })
    };
    let (mut total, mut current) = (0u32, 0u32);
    for c in ascii.chars() {
        match c {
            '十' => {
                total += if current == 0 { 1 } else { current } * 10;
                current = 0;
            }
            '百' => {
                total += if current == 0 { 1 } else { current } * 100;
                current = 0;
            }
            _ => current = digit(c)?,
        }
    }
    let n = total + current;
    (n > 0).then_some(n)
}

/// True when the char before `start` can't continue a word or number.
fn left_ok(input: &str, start: usize) -> bool {
    input[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
}

/// True when the char after `end` can't continue a word or number.
fn right_ok(input: &str, end: usize) -> bool {
    input[end..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_alphanumeric())
}

fn push(matches: &mut Vec<MatchSpan>, start: usize, end: usize, property: Property, n: u32) {
    if n == 0 {
        return;
    }
    // A number belongs to one marker: `2.Sezon.7.Bolum` is season 2,
    // episode 7, never season 7.
    let overlaps = matches.iter().any(|m| {
        (m.property == property
            || matches!(m.property, Property::Season | Property::Episode)
                && matches!(property, Property::Season | Property::Episode))
            && m.start < end
            && start < m.end
    });
    if !overlaps {
        matches.push(MatchSpan::new(start, end, property, n.to_string()).with_priority(KEYWORD));
    }
}

/// Add multilingual and CJK season/episode markers to `matches`.
pub(super) fn find(input: &str, matches: &mut Vec<MatchSpan>) {
    // Episodes first, word-first before number-first, so a number between
    // two markers goes to the right one: `Staffel.2.Folge.5` (word-first
    // episode `Folge.5`), `2.Sezon.7.Bolum` (number-first episode `7.Bolum`,
    // which blocks the word-first season `Sezon.7`).
    let word_markers: [(&Regex, Property); 4] = [
        (&EPISODE_WORD_FIRST, Property::Episode),
        (&EPISODE_NUMBER_FIRST, Property::Episode),
        (&SEASON_WORD_FIRST, Property::Season),
        (&SEASON_NUMBER_FIRST, Property::Season),
    ];
    for (re, property) in word_markers {
        for cap in re.captures_iter(input) {
            let full = cap.get(0).expect("group 0");
            if !left_ok(input, full.start()) || !right_ok(input, full.end()) {
                continue;
            }
            let Some(n) = cap.name("n").and_then(|m| m.as_str().parse::<u32>().ok()) else {
                continue;
            };
            // `Bolum.2021`: a year after the word isn't an episode number.
            if (1900..=2099).contains(&n) {
                continue;
            }
            push(matches, full.start(), full.end(), property, n);
        }
    }

    for (re, property) in [
        (&SEASON_TOTAL, Property::SeasonCount),
        (&EPISODE_TOTAL, Property::EpisodeCount),
    ] {
        for cap in re.captures_iter(input) {
            let full = cap.get(0).expect("group 0");
            if !left_ok(input, full.start()) || !right_ok(input, full.end()) {
                continue;
            }
            let total = cap.name("total").expect("total group");
            if let Ok(n) = total.as_str().parse() {
                // The total's own span: the marker itself is already claimed.
                push(matches, total.start(), total.end(), property, n);
            }
        }
    }

    for cap in CJK_SEASON.captures_iter(input) {
        let full = cap.get(0).expect("group 0");
        let num = cap
            .name("n")
            .or_else(|| cap.name("k"))
            .expect("number group");
        if let Some(n) = parse_cjk_number(num.as_str()) {
            push(matches, full.start(), full.end(), Property::Season, n);
        }
    }
    for cap in CJK_EPISODE.captures_iter(input) {
        let full = cap.get(0).expect("group 0");
        if let Some(n) = cap.name("n").and_then(|m| parse_cjk_number(m.as_str())) {
            push(matches, full.start(), full.end(), Property::Episode, n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(input: &str, property: Property) -> Vec<String> {
        let mut m = Vec::new();
        find(input, &mut m);
        m.into_iter()
            .filter(|s| s.property == property)
            .map(|s| s.value.to_string())
            .collect()
    }

    #[test]
    fn chinese_numerals() {
        assert_eq!(parse_cjk_number("二"), Some(2));
        assert_eq!(parse_cjk_number("十一"), Some(11));
        assert_eq!(parse_cjk_number("二十三"), Some(23));
        assert_eq!(parse_cjk_number("一百零五"), Some(105));
        assert_eq!(parse_cjk_number("０３"), Some(3));
        assert_eq!(parse_cjk_number("季"), None);
    }

    #[test]
    fn cjk_markers_cover_the_whole_marker() {
        let mut m = Vec::new();
        find("庆余年 第二季 第3集.mp4", &mut m);
        let season = m.iter().find(|s| s.property == Property::Season).unwrap();
        assert_eq!(season.value.to_string(), "2");
        assert_eq!(
            &"庆余年 第二季 第3集.mp4"[season.start..season.end],
            "第二季"
        );
        assert_eq!(
            values("庆余年第十一季 第二十三集.mkv", Property::Episode),
            ["23"]
        );
        assert_eq!(
            values("アニメ シーズン2 第3話.mkv", Property::Season),
            ["2"]
        );
    }

    #[test]
    fn word_first_and_number_first_markers() {
        assert_eq!(
            values("Die.Simpsons.Staffel.2.Folge.5.German", Property::Season),
            ["2"]
        );
        assert_eq!(
            values("Die.Simpsons.Staffel.2.Folge.5.German", Property::Episode),
            ["5"]
        );
        assert_eq!(
            values("Die Simpsons 2. Staffel Folge 5", Property::Season),
            ["2"]
        );
        assert_eq!(
            values("Teen Wolf - 04ª Temporada 720p", Property::Season),
            ["4"]
        );
        assert_eq!(
            values("Проклятие 5-й сезон 09-я серия", Property::Episode),
            ["9"]
        );
        assert_eq!(
            values("The.White.Lotus.2.Sezon.7.Bolum.2021", Property::Episode),
            ["7"]
        );
    }

    #[test]
    fn totals_after_markers() {
        assert_eq!(
            values("Show.Name.Capitulo.5.de.12.HDTV", Property::EpisodeCount),
            ["12"]
        );
        assert_eq!(
            values("Serie.Temporada.1.de.5.1080p", Property::SeasonCount),
            ["5"]
        );
        assert_eq!(
            values("Show.Name.5.серия.из.12.WEB-DL", Property::EpisodeCount),
            ["12"]
        );
    }

    #[test]
    fn no_marker_inside_a_word() {
        assert!(values("Restaffel.2", Property::Season).is_empty());
        assert!(values("Show.Name.2019.Capitulos", Property::Episode).is_empty());
    }
}
