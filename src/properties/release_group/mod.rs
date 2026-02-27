//! Release group extraction (post-resolution, v0.3).
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"
//!
//! ## v0.3 change: Two-pass pipeline
//!
//! Release group now runs AFTER conflict resolution (Pass 2), so it can
//! check resolved match positions instead of maintaining a 130+ token
//! exclusion list. `is_known_token` is replaced by `is_position_claimed`.
//!
//! Also handles:
//! - Groups before `[website]`: `-FtS.[site.com].mkv`
//! - Groups with `@`: `HiS@SiLUHD`
//! - Bracket prefix groups: `[SubGroup] Anime`
//! - Compound bracket groups: `(Tigole) [QxR]`
//! - `-by.Group[Suffix]` patterns
//!
//! ## Module structure
//! - `mod.rs` — regex patterns + find_matches (matching logic)
//! - `known_tokens.rs` — position-based validation + helpers

mod known_tokens;

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;
use known_tokens::{
    expand_group_backwards, is_hex_crc, is_rejected_group, strip_trailing_metadata,
};
use std::sync::LazyLock;

// ── Regex patterns ────────────────────────────────────────────────────────

/// Matches `-GROUP` at the end with optional bracket suffix.
static RELEASE_GROUP_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)-(?P<group>[A-Za-z0-9@µ!]+)(?:\[(?P<suffix>[A-Za-z0-9]+)\])?(?:\.(?:sample|proof|nfo|srt|sub|subs|proper|repack|real|dubbed|hebsubs|nlsubs|swesub|hardcoded|[a-z]{2,3}))*(?:\.[a-z0-9]{2,5})?$")
        .unwrap()
});

/// Matches `-by.GROUP[SUFFIX]` pattern.
static RELEASE_GROUP_BY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)-by\.(?P<group>[A-Za-z][A-Za-z0-9]+)(?:\[(?P<suffix>[A-Za-z0-9]+)\])?")
        .unwrap()
});

/// Matches `-GROUP` before a `[website]` suffix.
static RELEASE_GROUP_BEFORE_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-(?P<group>[A-Za-z0-9@µ!]+)\s*\.?\s*\[").unwrap());

/// Matches `.GROUP.[website]` (dot-separated before bracket).
static RELEASE_GROUP_DOT_BEFORE_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(?P<group>[A-Za-z][A-Za-z0-9@µ!]+)\.\[").unwrap());

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

/// Last token after dots as fallback: `720p.YIFY` or `HDTV.SC`.
static RELEASE_GROUP_LAST_DOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(?P<group>[A-Za-z][A-Za-z0-9]{1,15})(?:\.[a-z0-9]{2,5})?$").unwrap()
});

// ── Matching logic (post-resolution) ──────────────────────────────────────

/// Find release group matches using resolved tech match positions.
///
/// This runs in Pass 2 of the pipeline, AFTER conflict resolution.
/// Instead of `is_known_token`, it checks whether candidate positions
/// are already claimed by resolved matches.
pub fn find_matches(
    input: &str,
    resolved: &[MatchSpan],
    zone_map: &ZoneMap,
    _token_stream: &TokenStream,
) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];
    let cleaned_filename = strip_trailing_metadata(filename);

    // 1. `-by.GROUP[SUFFIX]` pattern (before generic `-GROUP` to avoid conflict).
    if let Some(cap) = RELEASE_GROUP_BY.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let mut value = group.as_str().to_string();
        let abs_start = filename_start + group.start();
        let mut abs_end = filename_start + group.end();

        if let Some(suffix) = cap.name("suffix") {
            value = format!("{}[{}]", value, suffix.as_str());
            abs_end = filename_start + suffix.end() + 1; // +1 for closing ]
        }

        if !is_rejected_group(&value, abs_start, abs_end, resolved) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-1),
            );
        }
    }

    // 2. `-GROUP` at end with optional bracket suffix.
    if matches.is_empty() {
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
                let expanded =
                    expand_group_backwards(before_group, &value, filename_start, resolved);
                if expanded != value {
                    start = start.saturating_sub(expanded.len() - value.len());
                    value = expanded;
                }

                if let Some(suffix) = cap.name("suffix") {
                    value = format!("{}[{}]", value, suffix.as_str());
                }

                let abs_start = filename_start + start;
                let abs_end = cap
                    .name("suffix")
                    .map(|s| filename_start + s.end() + 1)
                    .unwrap_or(filename_start + group.end());

                if !is_rejected_group(&value, abs_start, abs_end, resolved) {
                    matches.push(
                        MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                            .with_priority(-1),
                    );
                }
            }
        }
    }

    // 3. `-GROUP[website]` or `-GROUP.[website]`.
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_BEFORE_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-2),
            );
        }
    }

    // 3b. `.GROUP.[website]` (dot-separated before bracket).
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_DOT_BEFORE_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-2),
            );
        }
    }

    // 4. `-[GROUP]` at end.
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_DASH_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str().trim();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) && !is_hex_crc(value) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-2),
            );
        }
    }

    // 5. `[GROUP]` at end.
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_END_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str().trim();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) && !is_hex_crc(value) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-2),
            );
        }
    }

    // 6. `[GROUP]` at start (anime style).
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_START_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str().trim();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) && !is_hex_crc(value) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-1),
            );
        }
    }

    // 7. Space-separated at end (requires tech zone anchors).
    if matches.is_empty()
        && zone_map.has_anchors
        && let Some(cap) = RELEASE_GROUP_SPACE_END.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) && value.len() >= 3 {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-4),
            );
        }
    }

    // 8. Last dot-segment as fallback (requires tech zone anchors).
    if matches.is_empty()
        && zone_map.has_anchors
        && let Some(cap) = RELEASE_GROUP_LAST_DOT.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value).with_priority(-3),
            );
        }
    }

    // 9. Check parent directory for release group.
    if filename_start > 0 {
        let parent = &input[..filename_start.saturating_sub(1)];
        let parent_name = parent.rsplit(['/', '\\']).next().unwrap_or("");
        if let Some(cap) = RELEASE_GROUP_END.captures(parent_name)
            && let Some(group) = cap.name("group")
        {
            let value = group.as_str();
            let abs_start = parent.len() - parent_name.len() + group.start();
            let abs_end = parent.len() - parent_name.len() + group.end();
            if !is_rejected_group(value, abs_start, abs_end, resolved) {
                let filename_is_abbreviated = !zone_map.has_anchors && filename.len() < 20;

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

    // 10. Compound bracket merging: `(GroupA) [GroupB]` → "GroupA GroupB".
    if matches.is_empty()
        && let Some(compound) = find_compound_bracket_group(filename, filename_start, resolved)
    {
        matches.push(compound);
    }

    matches
}

/// Detect compound bracket groups like `(Tigole) [QxR]` or `(JBENT)[TAoE]`.
///
/// Scans for adjacent parenthesized and bracketed groups in the tech zone,
/// merging them into a single release group value.
fn find_compound_bracket_group(
    filename: &str,
    filename_start: usize,
    resolved: &[MatchSpan],
) -> Option<MatchSpan> {
    // Look for pattern: (...GROUP...) optional-space [GROUP2]
    // The paren group may contain tech tokens before the actual group name.
    let mut bracket_groups: Vec<(usize, usize, String)> = Vec::new();

    let bytes = filename.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let (_open, close) = match bytes[i] {
            b'(' => (b'(', b')'),
            b'[' => (b'[', b']'),
            _ => {
                i += 1;
                continue;
            }
        };

        // Find matching close.
        if let Some(end_offset) = filename[i + 1..].find(close as char) {
            let content = &filename[i + 1..i + 1 + end_offset];
            let abs_start = filename_start + i;
            let abs_end = filename_start + i + 1 + end_offset + 1;

            // Extract the last non-tech token from the bracket content.
            // For "(1080p AMZN Webrip x265 10bit EAC3 5.1 - JBENT)",
            // we want "JBENT".
            let last_word = extract_last_non_tech_word(content, abs_start + 1, resolved);

            if let Some((word, _word_start, _word_end)) = last_word {
                bracket_groups.push((abs_start, abs_end, word));
            }

            i += 1 + end_offset + 1;
        } else {
            i += 1;
        }
    }

    // Merge adjacent bracket groups.
    if bracket_groups.len() >= 2 {
        // Take the last two bracket groups (most likely to be group + indexer).
        let len = bracket_groups.len();
        let (start1, _, ref name1) = bracket_groups[len - 2];
        let (_, end2, ref name2) = bracket_groups[len - 1];

        if !name1.is_empty() && !name2.is_empty() {
            let merged = format!("{} {}", name1, name2);
            return Some(
                MatchSpan::new(start1, end2, Property::ReleaseGroup, merged).with_priority(-2),
            );
        }
    }

    None
}

/// Extract the last non-tech word from bracket content.
///
/// Given content like `1080p AMZN Webrip x265 10bit EAC3 5.1 - JBENT`,
/// returns `("JBENT", abs_start, abs_end)`.
fn extract_last_non_tech_word(
    content: &str,
    content_abs_start: usize,
    resolved: &[MatchSpan],
) -> Option<(String, usize, usize)> {
    // Split on spaces, dots, hyphens, and find the last unclaimed word.
    let words: Vec<&str> = content.split([' ', '.', '-', '_']).collect();

    for word in words.iter().rev() {
        let word = word.trim();
        if word.is_empty() || word.chars().all(|c| c.is_ascii_digit() || c == '.') {
            continue;
        }

        // Find position of this word in content.
        if let Some(pos) = content.rfind(word) {
            let abs_start = content_abs_start + pos;
            let abs_end = abs_start + word.len();

            if !is_rejected_group(word, abs_start, abs_end, resolved) {
                return Some((word.to_string(), abs_start, abs_end));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;
    use crate::zone_map;

    fn test_find(input: &str) -> Vec<MatchSpan> {
        let ts = tokenizer::tokenize(input);
        let zm = zone_map::build_zone_map(input, &ts);
        find_matches(input, &[], &zm, &ts)
    }

    fn test_find_with_resolved(input: &str, resolved: Vec<MatchSpan>) -> Vec<MatchSpan> {
        let ts = tokenizer::tokenize(input);
        let zm = zone_map::build_zone_map(input, &ts);
        find_matches(input, &resolved, &zm, &ts)
    }

    #[test]
    fn test_group_at_end() {
        let m = test_find("Movie.2024.1080p.BluRay.x264-SPARKS.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "SPARKS");
    }

    #[test]
    fn test_group_no_extension() {
        let m = test_find("Movie.2024.1080p.BluRay.x264-YTS");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "YTS");
    }

    #[test]
    fn test_no_false_positive_codec() {
        // x264 is a Tier 2 token → rejected.
        let m = test_find("Movie-x264.mkv");
        assert!(m.is_empty(), "x264 should not be a release group");
    }

    #[test]
    fn test_group_with_at() {
        let m = test_find("Movie.BDRip.720p-HiS@SiLUHD.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "HiS@SiLUHD");
    }

    #[test]
    fn test_group_before_bracket_website() {
        let m = test_find("Movie.x264-FtS.[sharethefiles.com].mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "FtS");
    }

    #[test]
    fn test_group_from_parent_dir() {
        let m = test_find("movies/Movie.DVDRip.XviD-DiAMOND/somefile.avi");
        assert!(m.iter().any(|x| x.value == "DiAMOND"));
    }

    #[test]
    fn test_group_with_crc() {
        let m = test_find("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }

    #[test]
    fn test_fansub_not_group() {
        let m = test_find("XViD.Fansub");
        assert!(
            m.is_empty(),
            "Fansub should not be detected as release group"
        );
    }

    #[test]
    fn test_position_claimed_rejects_codec() {
        let resolved = vec![MatchSpan::new(6, 10, Property::VideoCodec, "H.264")];
        let m = test_find_with_resolved("Movie-x264.mkv", resolved);
        assert!(
            m.is_empty(),
            "x264 should be rejected (claimed by VideoCodec)"
        );
    }

    #[test]
    fn test_by_group_pattern() {
        let m = test_find("Some.Title.XViD-by.Artik[SEDG].avi");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "Artik[SEDG]");
    }
}
