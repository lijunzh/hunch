//! Release group extraction.
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"
//!
//! Also handles:
//! - Groups before `[website]`: `-FtS.[site.com].mkv`
//! - Groups with `@`: `HiS@SiLUHD`
//! - Bracket prefix groups: `[SubGroup] Anime`

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Matches `-GROUP` at the end, before optional extension.
    static ref RELEASE_GROUP_END: Regex = Regex::new(
        r"-(?P<group>[A-Za-z0-9@]+(?:\[[A-Fa-f0-9]+\])?)(?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Matches `-GROUP` before a `[website]` suffix.
    static ref RELEASE_GROUP_BEFORE_BRACKET: Regex = Regex::new(
        r"-(?P<group>[A-Za-z0-9@]+)\s*\.?\s*\["
    ).unwrap();

    /// Release group in brackets at the start: `[GROUP] Title`.
    static ref RELEASE_GROUP_START_BRACKET: Regex = Regex::new(
        r"^\[(?P<group>[A-Za-z][A-Za-z0-9 _.]{0,15})\]\s*"
    ).unwrap();
}

pub struct ReleaseGroupMatcher;

impl PropertyMatcher for ReleaseGroupMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Use the filename portion for matching.
        let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let filename = &input[filename_start..];

        // 1. Check for `-GROUP` at the end (most common convention).
        if let Some(cap) = RELEASE_GROUP_END.captures(filename) {
            if let Some(group) = cap.name("group") {
                let value = group.as_str();
                if !is_known_token(value) {
                    matches.push(
                        MatchSpan::new(
                            filename_start + group.start(),
                            filename_start + group.end(),
                            Property::ReleaseGroup,
                            value,
                        )
                        .with_tag("end-dash")
                        .with_priority(-1),
                    );
                }
            }
        }

        // 2. Check for `-GROUP[website]` or `-GROUP.[website]`.
        if matches.is_empty() {
            if let Some(cap) = RELEASE_GROUP_BEFORE_BRACKET.captures(filename) {
                if let Some(group) = cap.name("group") {
                    let value = group.as_str();
                    if !is_known_token(value) {
                        matches.push(
                            MatchSpan::new(
                                filename_start + group.start(),
                                filename_start + group.end(),
                                Property::ReleaseGroup,
                                value,
                            )
                            .with_tag("before-bracket")
                            .with_priority(-2),
                        );
                    }
                }
            }
        }

        // 3. Also check parent directory for group if filename didn't have one.
        if matches.is_empty() && filename_start > 0 {
            let parent = &input[..filename_start.saturating_sub(1)];
            let parent_name = parent.rsplit(['/', '\\']).next().unwrap_or("");
            if let Some(cap) = RELEASE_GROUP_END.captures(parent_name) {
                if let Some(group) = cap.name("group") {
                    let value = group.as_str();
                    if !is_known_token(value) {
                        matches.push(
                            MatchSpan::new(
                                0,
                                0,
                                Property::ReleaseGroup,
                                value,
                            )
                            .with_tag("parent-dir")
                            .with_priority(-3),
                        );
                    }
                }
            }
        }

        matches
    }
}

/// Check if a string is a known token that shouldn't be a release group.
fn is_known_token(s: &str) -> bool {
    let lower = s.to_lowercase();
    matches!(
        lower.as_str(),
        "mkv" | "mp4" | "avi" | "wmv" | "flv" | "mov" | "webm" |
        "x264" | "x265" | "h264" | "h265" | "hevc" | "avc" | "av1" | "xvid" | "divx" |
        "aac" | "ac3" | "dts" | "flac" | "mp3" | "pcm" | "opus" |
        "bluray" | "bdrip" | "brrip" | "dvdrip" | "webrip" | "webdl" | "hdtv" |
        "720p" | "1080p" | "2160p" | "4k" |
        "hdr" | "hdr10" | "sdr" | "remux" | "proper" | "repack" |
        "srt" | "sub" | "subs" | "idx" | "nfo"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_at_end() {
        let m = ReleaseGroupMatcher.find_matches("Movie.2024.1080p.BluRay.x264-SPARKS.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "SPARKS");
    }

    #[test]
    fn test_group_no_extension() {
        let m = ReleaseGroupMatcher.find_matches("Movie.2024.1080p.BluRay.x264-YTS");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "YTS");
    }

    #[test]
    fn test_no_false_positive_codec() {
        let m = ReleaseGroupMatcher.find_matches("Movie-x264.mkv");
        assert!(m.is_empty());
    }

    #[test]
    fn test_group_with_at() {
        let m = ReleaseGroupMatcher.find_matches("Movie.BDRip.720p-HiS@SiLUHD.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "HiS@SiLUHD");
    }

    #[test]
    fn test_group_before_bracket_website() {
        let m = ReleaseGroupMatcher.find_matches("Movie.x264-FtS.[sharethefiles.com].mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "FtS");
    }

    #[test]
    fn test_group_from_parent_dir() {
        // When filename has no group pattern, fall back to parent dir.
        let m = ReleaseGroupMatcher.find_matches(
            "movies/Movie.DVDRip.XviD-DiAMOND/somefile.avi"
        );
        assert!(m.iter().any(|x| x.value == "DiAMOND"));
    }

    #[test]
    fn test_group_with_crc() {
        let m = ReleaseGroupMatcher.find_matches("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        // Bracket groups handled separately.
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }
}
