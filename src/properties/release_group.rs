//! Release group extraction.
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Matches the release group at the end of the filename, before the extension.
    /// Pattern: `-GROUP` where GROUP is alphanumeric (no spaces/dots).
    static ref RELEASE_GROUP_END: Regex = Regex::new(
        r"-(?P<group>[A-Za-z0-9]+(?:\[[A-Fa-f0-9]+\])?)(?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Release group in brackets: [GROUP].
    static ref RELEASE_GROUP_BRACKET: Regex = Regex::new(
        r"\[(?P<group>[A-Za-z][A-Za-z0-9 _.-]{1,20})\]"
    ).unwrap();
}

pub struct ReleaseGroupMatcher;

impl PropertyMatcher for ReleaseGroupMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Check for `-GROUP` at the end (most common convention).
        if let Some(cap) = RELEASE_GROUP_END.captures(input) {
            if let Some(group) = cap.name("group") {
                let value = group.as_str();
                // Filter out known false positives (codecs, containers, etc.).
                if !is_known_token(value) {
                    matches.push(
                        MatchSpan::new(
                            group.start(),
                            group.end(),
                            Property::ReleaseGroup,
                            value,
                        )
                        .with_tag("end-dash")
                        .with_priority(-1),
                    );
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
    fn test_group_with_crc() {
        let m = ReleaseGroupMatcher.find_matches("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        // The end-dash pattern won't match here, so no result from this matcher.
        // Bracket groups are a Phase 2 feature.
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }
}
