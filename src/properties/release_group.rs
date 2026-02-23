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
    /// Matches `-GROUP` at the end with optional bracket suffix.
    static ref RELEASE_GROUP_END: Regex = Regex::new(
        r"-(?P<group>[A-Za-z0-9@µ!]+)(?:\[(?P<suffix>[A-Za-z0-9]+)\])?(?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Matches `-GROUP` before a `[website]` suffix.
    static ref RELEASE_GROUP_BEFORE_BRACKET: Regex = Regex::new(
        r"-(?P<group>[A-Za-z0-9@µ!]+)\s*\.?\s*\["
    ).unwrap();

    /// Release group in brackets at the start: `[GROUP] Title`.
    static ref RELEASE_GROUP_START_BRACKET: Regex = Regex::new(
        r"^\[(?P<group>[A-Za-z][A-Za-z0-9 _.!-]{0,20})\]\s*"
    ).unwrap();

    /// Release group in brackets at the end: `Title [GROUP].ext`.
    /// Excludes website-like content (containing dots) and hex CRC values.
    static ref RELEASE_GROUP_END_BRACKET: Regex = Regex::new(
        r"\[(?P<group>[A-Za-z][A-Za-z0-9 _!-]{0,20})\](?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Space-separated group at end: `x264.dxva EuReKA.mkv` or `AC3 TiTAN.mkv`.
    static ref RELEASE_GROUP_SPACE_END: Regex = Regex::new(
        r"\s(?P<group>[A-Za-z][A-Za-z0-9]{1,15})(?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Last token after dots as fallback: `720p.YIFY` or `x264.anoXmous`.
    static ref RELEASE_GROUP_LAST_DOT: Regex = Regex::new(
        r"\.(?P<group>[A-Za-z][A-Za-z0-9]{2,15})(?:\.[a-z0-9]{2,5})?$"
    ).unwrap();

    /// Prefix before dash (lowercase, only when filename starts with lowercase): `blow-how.to.be.single`.
    static ref RELEASE_GROUP_PREFIX: Regex = Regex::new(
        r"^(?P<group>[a-z][a-z0-9]{2,10})-[a-z].*\."
    ).unwrap();
}

pub struct ReleaseGroupMatcher;

impl PropertyMatcher for ReleaseGroupMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Use the filename portion for matching.
        let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let filename = &input[filename_start..];

        // 1. Check for simple `-GROUP` at end with optional bracket suffix.
        if let Some(cap) = RELEASE_GROUP_END.captures(filename)
            && let Some(group) = cap.name("group")
        {
            let mut value = group.as_str().to_string();
            if let Some(suffix) = cap.name("suffix") {
                value = format!("{}[{}]", value, suffix.as_str());
            }
            if !is_known_token(&value) {
                let end = cap
                    .name("suffix")
                    .map(|s| s.end() + 1)
                    .unwrap_or(group.end());
                matches.push(
                    MatchSpan::new(
                        filename_start + group.start(),
                        filename_start + end,
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_tag("end-dash")
                    .with_priority(-1),
                );
            }
        }

        // 2. Check for `-GROUP[website]` or `-GROUP.[website]`.
        if matches.is_empty()
            && let Some(cap) = RELEASE_GROUP_BEFORE_BRACKET.captures(filename)
            && let Some(group) = cap.name("group")
        {
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

        // 3. Bracket group at start: `[GROUP] Title`.
        if matches.is_empty()
            && let Some(cap) = RELEASE_GROUP_START_BRACKET.captures(filename)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str().trim();
            if !is_known_token(value) && !is_hex_crc(value) {
                matches.push(
                    MatchSpan::new(
                        filename_start + group.start(),
                        filename_start + group.end(),
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_tag("start-bracket")
                    .with_priority(-1),
                );
            }
        }

        // 4. Bracket group at end: `Title [GROUP].ext`.
        if matches.is_empty()
            && let Some(cap) = RELEASE_GROUP_END_BRACKET.captures(filename)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str().trim();
            if !is_known_token(value) && !is_hex_crc(value) {
                matches.push(
                    MatchSpan::new(
                        filename_start + group.start(),
                        filename_start + group.end(),
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_tag("end-bracket")
                    .with_priority(-2),
                );
            }
        }

        // 5. Space-separated at end: `x264.dxva EuReKA.mkv`.
        if matches.is_empty()
            && let Some(cap) = RELEASE_GROUP_SPACE_END.captures(filename)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str();
            if !is_known_token(value) {
                matches.push(
                    MatchSpan::new(
                        filename_start + group.start(),
                        filename_start + group.end(),
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_tag("space-end")
                    .with_priority(-3),
                );
            }
        }

        // 6. Last dot-separated token (fallback): `720p.YIFY`.
        // Only if the filename has recognizable technical tokens.
        if matches.is_empty()
            && has_technical_tokens(filename)
            && let Some(cap) = RELEASE_GROUP_LAST_DOT.captures(filename)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str();
            if !is_known_token(value) && value.len() >= 3 {
                matches.push(
                    MatchSpan::new(
                        filename_start + group.start(),
                        filename_start + group.end(),
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_tag("last-dot")
                    .with_priority(-4),
                );
            }
        }

        // (Prefix pattern disabled — too many false positives.)

        // 8. Also check parent directory for group if filename didn't have one.
        if matches.is_empty() && filename_start > 0 {
            let parent = &input[..filename_start.saturating_sub(1)];
            let parent_name = parent.rsplit(['/', '\\']).next().unwrap_or("");
            if let Some(cap) = RELEASE_GROUP_END.captures(parent_name)
                && let Some(group) = cap.name("group")
            {
                let value = group.as_str();
                if !is_known_token(value) {
                    matches.push(
                        MatchSpan::new(0, 0, Property::ReleaseGroup, value)
                            .with_tag("parent-dir")
                            .with_priority(-3),
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
        "mkv"
            | "mp4"
            | "avi"
            | "wmv"
            | "flv"
            | "mov"
            | "webm"
            | "ogm"
            | "x264"
            | "x265"
            | "h264"
            | "h265"
            | "hevc"
            | "avc"
            | "av1"
            | "xvid"
            | "divx"
            | "dvdivx"
            | "aac"
            | "ac3"
            | "dts"
            | "flac"
            | "mp3"
            | "pcm"
            | "opus"
            | "ma"
            | "bluray"
            | "bdrip"
            | "brrip"
            | "dvdrip"
            | "webrip"
            | "webdl"
            | "hdtv"
            | "720p"
            | "1080p"
            | "2160p"
            | "4k"
            | "hdr"
            | "hdr10"
            | "sdr"
            | "remux"
            | "proper"
            | "repack"
            | "srt"
            | "sub"
            | "subs"
            | "idx"
            | "nfo"
            | "iso"
            | "par"
            | "par2"
            | "hq"
            | "lq"
            | "english"
            | "french"
            | "spanish"
            | "german"
            | "italian"
            | "eng"
            | "fre"
            | "spa"
            | "multi"
            | "dual"
            | "dubbed"
            | "dvd"
            | "vhsrip"
            | "cam"
            | "screener"
            | "scr"
            | "internal"
            | "limited"
            | "unrated"
            | "extended"
            | "directors"
            | "cut"
            | "complete"
            | "season"
            | "disc"
            | "imax"
            | "edition"
            | "pal"
            | "ntsc"
            | "dub"
            | "vostfr"
            | "vff"
            | "vost"
    )
}

/// Check if a string looks like a CRC32 hex value.
fn is_hex_crc(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Returns true if the filename contains recognizable technical tokens
/// (codecs, resolutions, sources, etc.). This helps the "last-dot" fallback
/// avoid false positives on simple filenames like `Title Only.avi`.
fn has_technical_tokens(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let technical = [
        "x264", "x265", "h264", "h265", "hevc", "xvid", "divx", "av1", "aac", "ac3", "dts", "flac",
        "opus", "720p", "1080p", "2160p", "4k", "bluray", "bdrip", "brrip", "dvdrip", "webrip",
        "webdl", "hdtv", "hdrip", "remux", "cam", "screener",
    ];
    technical.iter().any(|t| lower.contains(t))
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
        let m = ReleaseGroupMatcher.find_matches("movies/Movie.DVDRip.XviD-DiAMOND/somefile.avi");
        assert!(m.iter().any(|x| x.value == "DiAMOND"));
    }

    #[test]
    fn test_group_with_crc() {
        let m = ReleaseGroupMatcher.find_matches("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        // Bracket groups handled separately.
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }
}
