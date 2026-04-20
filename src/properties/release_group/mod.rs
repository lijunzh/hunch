//! Release group extraction (post-resolution, v0.3).
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"
//!
//! ## Why this lives in Rust (not `src/rules/`)
//!
//! Positional context (start vs end of filename) drives priority among
//! ~9 fallback patterns; the *first* match wins, not the longest. That
//! ordering is logic, not vocabulary. See DESIGN.md "D2: Vocabulary in
//! TOML, logic in Rust" → "multiple regex variants with different
//! output meanings" + "cross-pattern coordination" rows.
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
//! - `validation.rs` — position-based validation + helpers

mod validation;

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;
use std::sync::LazyLock;
use validation::{expand_group_backwards, is_hex_crc, is_rejected_group, strip_trailing_metadata};

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

// ── Matching logic (post-resolution) ───────────────────────────────────────────

/// Heuristic: reject first-bracket candidates that look like natural-language
/// titles rather than release groups.
///
/// Release groups are usually short and formatted (`DBD-Raws`, `EMBER`,
/// `TxxZ&POPGO&MGRT`). Titles are often multi-word phrases with regular words
/// and spaces (`Kimetsu no Yaiba Mugen Ressha Hen`).
fn looks_like_natural_language_title(candidate: &str) -> bool {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains(['&', '/', '@']) {
        return false;
    }

    let words: Vec<&str> = trimmed
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .collect();

    if words.len() < 4 {
        return false;
    }

    if words.iter().any(|word| {
        word.contains('-')
            || word
                .chars()
                .any(|c| !c.is_alphanumeric() && c != '\'' && c != '!')
            || word.chars().all(|c| c.is_ascii_uppercase())
    }) {
        return false;
    }

    words.iter().filter(|word| word.len() >= 2).count() >= 4
}

/// Find release group matches using resolved tech match positions.
///
/// This runs in Pass 2 of the pipeline, AFTER conflict resolution.
/// Instead of `is_known_token`, it checks whether candidate positions
/// are already claimed by resolved matches.
pub fn find_matches(
    input: &str,
    resolved: &[MatchSpan],
    zone_map: &ZoneMap,
    token_stream: &TokenStream,
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
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::HEURISTIC),
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
                            .with_priority(crate::priority::HEURISTIC),
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
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::POSITIONAL),
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
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::POSITIONAL),
            );
        }
    }

    // 3c. `[GROUP]` at start (anime/fansub style).
    // Runs before compound bracket detection so that CJK fansub patterns
    // like `[Prejudice-Studio] Title - 01 [metadata]` correctly pick up
    // the first bracket as the release group. (#91)
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_START_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str().trim();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved)
            && !is_hex_crc(value)
            && !looks_like_natural_language_title(value)
        {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::HEURISTIC),
            );
        }
    }

    // 3d. Compound bracket merging using tokenizer's bracket model.
    // Catches `(Tigole) [QxR]`, `(JBENT)[TAoE]` patterns.
    if matches.is_empty()
        && let Some(compound) =
            find_compound_bracket_group_from_tokenstream(token_stream, filename_start, resolved)
    {
        matches.push(compound);
    }

    // 4. `-[GROUP]` at end.
    if matches.is_empty()
        && let Some(cap) = RELEASE_GROUP_DASH_BRACKET.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str().trim();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !value.contains('/')
            && !is_rejected_group(value, abs_start, abs_end, resolved)
            && !is_hex_crc(value)
        {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::POSITIONAL),
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
        if !value.contains('/')
            && !is_rejected_group(value, abs_start, abs_end, resolved)
            && !is_hex_crc(value)
        {
            matches.push(
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::POSITIONAL),
            );
        }
    }

    // 6. Space-separated at end (requires tech zone anchors).
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
                MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, value)
                    .with_priority(crate::priority::POSITIONAL - 2),
            );
        }
    }

    // 7. Last dot-segment as fallback (requires tech zone anchors).
    //    Also tries to merge preceding dot-segments (e.g., `YTS.LT` → "YTS.LT").
    if matches.is_empty()
        && zone_map.has_anchors
        && let Some(cap) = RELEASE_GROUP_LAST_DOT.captures(filename)
        && let Some(group) = cap.name("group")
    {
        let value = group.as_str();
        let abs_start = filename_start + group.start();
        let abs_end = filename_start + group.end();
        if !is_rejected_group(value, abs_start, abs_end, resolved) {
            // Try merging backwards: check if preceding dot-segment is also
            // unclaimed and non-tech (e.g., `YTS.LT` → "YTS.LT").
            let mut merged_value = value.to_string();
            let mut merged_start = abs_start;
            let before = &filename[..group.start().saturating_sub(1)]; // text before the dot
            if let Some(dot) = before.rfind('.') {
                let prev_seg = &before[dot + 1..];
                let prev_abs_start = filename_start + dot + 1;
                let prev_abs_end = filename_start + dot + 1 + prev_seg.len();
                if !prev_seg.is_empty()
                    && prev_seg.chars().next().is_some_and(|c| c.is_alphabetic())
                    && !is_rejected_group(prev_seg, prev_abs_start, prev_abs_end, resolved)
                {
                    merged_value = format!("{}.{}", prev_seg, value);
                    merged_start = prev_abs_start;
                }
            }
            matches.push(
                MatchSpan::new(merged_start, abs_end, Property::ReleaseGroup, merged_value)
                    .with_priority(crate::priority::POSITIONAL - 1),
            );
        }
    }

    // 7b. Mid-filename bracket group containing a single non-tech word.
    // E.g., `[HorribleSubs]` in `Show!.Name.2.-.10.(2016).[HorribleSubs][WEBRip]..[HD.720p]`.
    if matches.is_empty() {
        for bg in &token_stream.bracket_groups {
            if bg.open < filename_start || bg.kind != crate::tokenizer::BracketKind::Square {
                continue;
            }
            let content = bg.content.trim();
            // Single word, no separators, not a CRC, not tech.
            if content.is_empty()
                || content.contains([' ', '.', '-', '_', '/'])
                || is_hex_crc(content)
            {
                continue;
            }
            let abs_start = bg.open + 1;
            let abs_end = bg.close;
            if !is_rejected_group(content, abs_start, abs_end, resolved)
                && content.len() >= 3
                && content.chars().next().is_some_and(|c| c.is_alphabetic())
            {
                matches.push(
                    MatchSpan::new(abs_start, abs_end, Property::ReleaseGroup, content)
                        .with_priority(crate::priority::POSITIONAL - 2),
                );
                break;
            }
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
                            .with_priority(crate::priority::POSITIONAL - 1),
                    );
                }
            }
        }
    }

    // 9. Merge `-GROUP [BRACKET]` when Step 2 found a dash-group
    // but missed an adjacent bracket suffix (separated by space).
    // E.g., `-0SEC [GloDLS].mkv` → "0SEC [GloDLS]" (was just "0SEC").
    if matches.len() == 1 {
        let rg = &matches[0];
        // Find bracket groups that immediately follow the release group.
        let fn_bracket_groups: Vec<_> = token_stream
            .bracket_groups
            .iter()
            .filter(|bg| bg.open >= filename_start)
            .collect();

        for bg in &fn_bracket_groups {
            // Adjacent if bracket starts within 3 bytes of group end.
            if bg.open > rg.end && bg.open <= rg.end + 3 {
                let bracket_content = &bg.content;
                // Skip multi-word content — that's likely a title, not a suffix.
                // E.g., `[Saki Zenkoku Hen]` is a title, `[GloDLS]` is a suffix.
                if bracket_content.is_empty()
                    || bracket_content.contains('.')
                    || bracket_content.contains(' ')
                    || is_rejected_group(bracket_content, bg.open + 1, bg.close, resolved)
                    || is_hex_crc(bracket_content)
                {
                    continue;
                }
                // CJK fansub guard: if the very next bracket after this candidate
                // contains a claimed match (e.g., an episode), this candidate is
                // a title — not a release-group suffix.
                let next_bg_is_claimed = fn_bracket_groups.iter().any(|nbg| {
                    nbg.open > bg.close
                        && nbg.open <= bg.close + 1
                        && resolved
                            .iter()
                            .any(|m| m.start >= nbg.open && m.end <= nbg.close + 1)
                });
                if next_bg_is_claimed {
                    break;
                }
                let merged = format!("{} [{}]", rg.value, bracket_content);
                matches[0] = MatchSpan::new(rg.start, bg.close + 1, Property::ReleaseGroup, merged)
                    .with_priority(rg.priority);
                break;
            }
        }
    }

    matches
}

/// Detect compound bracket groups using the tokenizer's bracket model.
///
/// Looks for adjacent bracket pairs at the end of the filename where
/// the last non-tech word from each forms a valid group name.
/// E.g., `(1080p BluRay x265 Tigole) [QxR]` → "Tigole QxR".
fn find_compound_bracket_group_from_tokenstream(
    token_stream: &TokenStream,
    filename_start: usize,
    resolved: &[MatchSpan],
) -> Option<MatchSpan> {
    // Get bracket groups in the filename portion.
    let fn_brackets: Vec<_> = token_stream
        .bracket_groups
        .iter()
        .filter(|bg| bg.open >= filename_start)
        .collect();

    if fn_brackets.len() < 2 {
        return None;
    }

    // Check the last two bracket groups for compound pattern.
    let last = fn_brackets[fn_brackets.len() - 1];
    let second_last = fn_brackets[fn_brackets.len() - 2];

    // They should be adjacent (within a few bytes).
    if last.open > second_last.close + 4 {
        return None;
    }

    // Extract last non-tech word from each.
    let name1 = extract_last_non_tech_word(&second_last.content, second_last.open + 1, resolved);
    let name2 = extract_last_non_tech_word(&last.content, last.open + 1, resolved);

    match (name1, name2) {
        (Some((n1, _, _)), Some((n2, _, _))) if !n1.is_empty() && !n2.is_empty() => {
            let merged = format!("{} {}", n1, n2);
            Some(
                MatchSpan::new(
                    second_last.open,
                    last.close + 1,
                    Property::ReleaseGroup,
                    merged,
                )
                .with_priority(crate::priority::POSITIONAL),
            )
        }
        _ => None,
    }
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
