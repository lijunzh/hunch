//! Container / file extension detection.

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

const VIDEO_EXTS: &[&str] = &[
    "3g2", "3gp", "asf", "avi", "divx", "flv", "m2ts", "m4v", "mk3d", "mkv", "mov", "mp4", "mpeg",
    "mpg", "mts", "ogm", "ogv", "rm", "rmvb", "ts", "vob", "webm", "wmv",
];

const SUBTITLE_EXTS: &[&str] = &[
    "aqt", "ass", "idx", "mpl", "pjs", "psb", "rt", "smi", "srt", "ssa", "stl", "sub", "sup",
    "usf", "vtt",
];

const INFO_EXTS: &[&str] = &["nfo"];
const TORRENT_EXTS: &[&str] = &["torrent"];
const NZB_EXTS: &[&str] = &["nzb"];

static EXT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let all_exts: Vec<&str> = VIDEO_EXTS
        .iter()
        .chain(SUBTITLE_EXTS)
        .chain(INFO_EXTS)
        .chain(TORRENT_EXTS)
        .chain(NZB_EXTS)
        .copied()
        .collect();
    let pattern = format!(r"(?i)\.({})$", all_exts.join("|"));
    Regex::new(&pattern).unwrap()
});

/// Match container as standalone uppercase token (e.g., MP4-GUSH, WMV-NOVO).
/// Also matches bare extension as entire input (e.g., "mkv", "avi").
static EXT_STANDALONE: LazyLock<Regex> = LazyLock::new(|| {
    let all_exts: Vec<&str> = VIDEO_EXTS.iter().chain(SUBTITLE_EXTS).copied().collect();
    let pattern = format!(r"(?i)({})", all_exts.join("|"));
    Regex::new(&pattern).unwrap()
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    if let Some(cap) = EXT_REGEX.find(input) {
        let ext = &input[cap.start() + 1..cap.end()];
        matches.push(
            MatchSpan::new(
                cap.start() + 1,
                cap.end(),
                Property::Container,
                ext.to_lowercase(),
            )
            .as_extension()
            .with_priority(10),
        );
    }
    // Fallback: standalone container token (e.g., "MP4-GUSH", "[.mp4]").
    if matches.is_empty() {
        // Replicate the lookbehind (?<=[-. \[]) or start-of-string manually.
        for cap in EXT_STANDALONE.captures_iter(input) {
            let m = cap.get(0).unwrap();
            let at_start = m.start() == 0;
            let preceded_by_separator = m.start() > 0
                && matches!(
                    input.as_bytes()[m.start() - 1],
                    b'.' | b'-' | b'_' | b' ' | b'['
                );
            let at_end = m.end() == input.len();
            let followed_by_separator = m.end() < input.len()
                && matches!(
                    input.as_bytes()[m.end()],
                    b'.' | b'-' | b'_' | b' ' | b']' | b')'
                );
            if (at_start || preceded_by_separator) && (at_end || followed_by_separator) {
                let ext = &input[m.start()..m.end()];
                matches.push(
                    MatchSpan::new(m.start(), m.end(), Property::Container, ext.to_lowercase())
                        .with_priority(5),
                );
                break;
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkv() {
        let m = find_matches("Movie.2020.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "mkv");
    }

    #[test]
    fn test_srt() {
        let m = find_matches("Movie.srt");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "srt");
    }

    #[test]
    fn test_no_extension() {
        let m = find_matches("Movie 2020 1080p");
        assert!(m.is_empty());
    }
}
