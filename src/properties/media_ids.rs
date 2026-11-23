//! Database IDs and volume numbers.
//!
//! - `imdb_id`: `tt1234567`, also inside `{imdb-tt1234567}` / `[imdbid-tt1234567]`
//! - `tmdb_id`: `{tmdb-12345}`, `[tmdbid-12345]`, `tmdb=12345`
//! - `tvdb_id`: `{tvdb-67890}`, `[tvdbid-67890]`, `tvdb=67890`
//! - `volume`: `vol127`, `vol.3`, `Volume 2`
//!
//! The bracketed forms are the Plex/Jellyfin/Emby naming conventions that
//! media agents write into folder and file names, so reading them back lets
//! a caller match a file to its database entry without a lookup. The whole
//! bracket is claimed, so it never leaks into the title.
//!
//! A volume followed later by a year (`Guardians.of.the.Galaxy.Vol.2.2017`,
//! `Kill.Bill.Vol.1.2003`) is part of the title and is left alone.

use regex::Regex;
use std::sync::LazyLock;

use crate::matcher::span::{MatchSpan, Property};
use crate::priority::PATTERN;

static TAGGED_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[\[{(]?(?P<db>imdb|tmdb|tvdb)(?:id)?[-=_ ](?P<id>tt\d{7,9}|\d{1,9})[\]})]?")
        .expect("TAGGED_ID regex is valid")
});

static BARE_IMDB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)tt\d{7,9}").expect("BARE_IMDB regex is valid"));

static VOLUME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)vol(?:ume)?[ ._]?(?P<n>\d{1,4})").expect("VOLUME regex is valid")
});

static YEAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:19|20)\d{2}").expect("YEAR regex is valid"));

fn is_word_char(c: Option<char>) -> bool {
    c.is_some_and(char::is_alphanumeric)
}

fn bounded(input: &str, start: usize, end: usize) -> bool {
    !is_word_char(input[..start].chars().next_back()) && !is_word_char(input[end..].chars().next())
}

/// Scan for IMDb/TMDB/TVDB ids and volume numbers.
pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches: Vec<MatchSpan> = Vec::new();

    for cap in TAGGED_ID.captures_iter(input) {
        let full = cap.get(0).expect("group 0");
        if !bounded(input, full.start(), full.end()) {
            continue;
        }
        let id = cap.name("id").expect("id group").as_str();
        let (property, value) = match cap
            .name("db")
            .expect("db group")
            .as_str()
            .to_lowercase()
            .as_str()
        {
            "imdb" if id.to_lowercase().starts_with("tt") => (Property::ImdbId, id.to_lowercase()),
            "tmdb" if !id.to_lowercase().starts_with("tt") => (Property::TmdbId, id.to_string()),
            "tvdb" if !id.to_lowercase().starts_with("tt") => (Property::TvdbId, id.to_string()),
            _ => continue,
        };
        matches
            .push(MatchSpan::new(full.start(), full.end(), property, value).with_priority(PATTERN));
    }

    for m in BARE_IMDB.find_iter(input) {
        if !bounded(input, m.start(), m.end())
            || matches
                .iter()
                .any(|x| x.start <= m.start() && m.end() <= x.end)
        {
            continue;
        }
        matches.push(
            MatchSpan::new(
                m.start(),
                m.end(),
                Property::ImdbId,
                m.as_str().to_lowercase(),
            )
            .with_priority(PATTERN),
        );
    }

    for cap in VOLUME.captures_iter(input) {
        let full = cap.get(0).expect("group 0");
        if !bounded(input, full.start(), full.end()) {
            continue;
        }
        let rest = &input[full.end()..];
        let filename_rest = rest.rsplit(['/', '\\']).next().unwrap_or(rest);
        if !rest.contains(['/', '\\'])
            && YEAR
                .find_iter(filename_rest)
                .any(|y| bounded(filename_rest, y.start(), y.end()))
        {
            continue; // `Vol.2.2017`: part of the title.
        }
        if let Some(n) = cap.name("n").and_then(|n| n.as_str().parse::<u32>().ok())
            && n > 0
        {
            matches.push(
                MatchSpan::new(full.start(), full.end(), Property::Volume, n.to_string())
                    .with_priority(PATTERN),
            );
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(input: &str, property: Property) -> Vec<String> {
        find_matches(input)
            .into_iter()
            .filter(|m| m.property == property)
            .map(|m| m.value.to_string())
            .collect()
    }

    #[test]
    fn plex_style_ids() {
        assert_eq!(
            get("Movie (2020) {tmdb-12345}.mkv", Property::TmdbId),
            ["12345"]
        );
        assert_eq!(
            get("Movie (2020) {imdb-tt0133093}.mkv", Property::ImdbId),
            ["tt0133093"]
        );
        assert_eq!(
            get("Show.S01E02.[tvdbid-67890].mkv", Property::TvdbId),
            ["67890"]
        );
        assert_eq!(
            get("Movie.Name.2020.tt1234567.1080p.mkv", Property::ImdbId),
            ["tt1234567"]
        );
    }

    #[test]
    fn no_id_inside_a_word() {
        assert!(get("Pretty1234567.mkv", Property::ImdbId).is_empty());
        assert!(get("Attt1234567.mkv", Property::ImdbId).is_empty());
    }

    #[test]
    fn volume_before_tech_but_not_before_a_year() {
        assert_eq!(
            get("Show.Name.vol127.1080p.x264-GRP.mkv", Property::Volume),
            ["127"]
        );
        assert_eq!(
            get("Manga.Title.vol.3.720p.x264-GRP.mkv", Property::Volume),
            ["3"]
        );
        assert!(
            get(
                "Guardians.of.the.Galaxy.Vol.2.2017.1080p.mkv",
                Property::Volume
            )
            .is_empty()
        );
        assert!(get("Revolution.S01E01.mkv", Property::Volume).is_empty());
    }
}
