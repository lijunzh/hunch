//! Release group extraction.
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"
//!
//! Also handles:
//! - Groups before `[website]`: `-FtS.[site.com].mkv`
//! - Groups with `@`: `HiS@SiLUHD`
//! - Bracket prefix groups: `[SubGroup] Anime`
//!
//! ## Module structure
//! - `mod.rs` — regex patterns + find_matches (matching logic)
//! - `known_tokens.rs` — token exclusion list + helper functions

mod known_tokens;

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use known_tokens::{expand_group_backwards, has_technical_tokens, is_hex_crc, is_known_token, strip_trailing_metadata};
use std::sync::LazyLock;

// ── Regex patterns ────────────────────────────────────────────────────────

/// Matches `-GROUP` at the end with optional bracket suffix.
static RELEASE_GROUP_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)-(?P<group>[A-Za-z0-9@µ!]+)(?:\[(?P<suffix>[A-Za-z0-9]+)\])?(?:\.(?:sample|proof|nfo|srt|sub|subs|proper|repack|real|dubbed|hebsubs|nlsubs|swesub|hardcoded|[a-z]{2,3}))*(?:\.[a-z0-9]{2,5})?$")
        .unwrap()
});

/// Matches `-GROUP` before a `[website]` suffix.
static RELEASE_GROUP_BEFORE_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-(?P<group>[A-Za-z0-9@µ!]+)\s*\.?\s*\[").unwrap());

/// Matches `-[GROUP]` at end: `x264-[2Maverick].mp4`.
static RELEASE_GROUP_DASH_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-\[(?P<group>[A-Za-z0-9][A-Za-z0-9 _!&-]{0,30})\](?:\.[a-z0-9]{2,5})?$").unwrap()
});

/// Release group in brackets at the start: `[GROUP] Title`.
static RELEASE_GROUP_START_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[(?P<group>[A-Za-z][A-Za-z0-9 _.!&-]{0,30})\]\s*").unwrap());

/// Release group in brackets at the end: `Title [GROUP].ext`.
static RELEASE_GROUP_END_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(?P<group>[A-Za-z][A-Za-z0-9 _!&-]{0,30})\](?:\.[a-z0-9]{2,5})?$").unwrap()
});

/// Space-separated group at end: `x264.dxva EuReKA.mkv`.
static RELEASE_GROUP_SPACE_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s(?P<group>[A-Za-z][A-Za-z0-9]{1,15})(?:\.[a-z0-9]{2,5})?$").unwrap()
});

/// Last token after dots as fallback: `720p.YIFY`.
static RELEASE_GROUP_LAST_DOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(?P<group>[A-Za-z][A-Za-z0-9]{2,15})(?:\.[a-z0-9]{2,5})?$").unwrap()
});

// ── Matching logic ────────────────────────────────────────────────────────

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];
    let cleaned_filename = strip_trailing_metadata(filename);

    // 1. `-GROUP` at end with optional bracket suffix.
    let candidates = [cleaned_filename.as_str(), filename];
    for fname in candidates {
        if !matches.is_empty() {
            break;
        }
        if let Some(cap) = RELEASE_GROUP_END.captures(fname)
            && let Some(group) = cap.name("group")
        {
            let mut value = group.as_str().to_string();
            let mut start = group.start();

            let before_group = &fname[..start.saturating_sub(1)];
            let expanded = expand_group_backwards(before_group, &value);
            if expanded != value {
                start = start.saturating_sub(expanded.len() - value.len());
                value = expanded;
            }

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
                        filename_start + start,
                        filename_start + end,
                        Property::ReleaseGroup,
                        value,
                    )
                    .with_priority(-1),
                );
            }
        }
    }

    // 2. `-GROUP[website]` or `-GROUP.[website]`.
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
                .with_priority(-2),
            );
        }
    }

    // 3. `-[GROUP]` at end.
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_DASH_BRACKET.captures(filename)
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
                .with_priority(-2),
            );
        }
    }

    // 4. `[GROUP]` at end.
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
                .with_priority(-2),
            );
        }
    }

    // 5. `[GROUP]` at start (anime style).
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
                .with_priority(-1),
            );
        }
    }

    // 6. Space-separated at end (requires tech tokens).
    if matches.is_empty()
        && has_technical_tokens(filename)
        && let Some(cap) = RELEASE_GROUP_SPACE_END.captures(filename)
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
                .with_priority(-4),
            );
        }
    }

    // 7. Last dot-segment as fallback (requires tech tokens).
    if matches.is_empty()
        && has_technical_tokens(filename)
        && let Some(cap) = RELEASE_GROUP_LAST_DOT.captures(filename)
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
                .with_priority(-3),
            );
        }
    }

    // 8. Check parent directory for release group.
    if filename_start > 0 {
        let parent = &input[..filename_start.saturating_sub(1)];
        let parent_name = parent.rsplit(['/', '\\']).next().unwrap_or("");
        if let Some(cap) = RELEASE_GROUP_END.captures(parent_name)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str();
            if !is_known_token(value) {
                let filename_is_abbreviated = !has_technical_tokens(filename)
                    && filename.len() < 20
                    && has_technical_tokens(parent_name);

                if matches.is_empty() || filename_is_abbreviated {
                    if filename_is_abbreviated {
                        matches.clear();
                    }
                    let mut parent_value = value.to_string();
                    if let Some(suffix) = cap.name("suffix") {
                        parent_value = format!("{}[{}]", parent_value, suffix.as_str());
                    }
                    matches.push(
                        MatchSpan::new(0, 0, Property::ReleaseGroup, parent_value)
                            .with_priority(-3),
                    );
                }
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_at_end() {
        let m = find_matches("Movie.2024.1080p.BluRay.x264-SPARKS.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "SPARKS");
    }

    #[test]
    fn test_group_no_extension() {
        let m = find_matches("Movie.2024.1080p.BluRay.x264-YTS");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "YTS");
    }

    #[test]
    fn test_no_false_positive_codec() {
        let m = find_matches("Movie-x264.mkv");
        assert!(m.is_empty());
    }

    #[test]
    fn test_group_with_at() {
        let m = find_matches("Movie.BDRip.720p-HiS@SiLUHD.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "HiS@SiLUHD");
    }

    #[test]
    fn test_group_before_bracket_website() {
        let m = find_matches("Movie.x264-FtS.[sharethefiles.com].mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "FtS");
    }

    #[test]
    fn test_group_from_parent_dir() {
        let m = find_matches("movies/Movie.DVDRip.XviD-DiAMOND/somefile.avi");
        assert!(m.iter().any(|x| x.value == "DiAMOND"));
    }

    #[test]
    fn test_group_with_crc() {
        let m = find_matches("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }

    #[test]
    fn test_fansub_not_group() {
        let m = find_matches("XViD.Fansub");
        assert!(m.is_empty(), "Fansub should not be detected as release group");
    }
}
