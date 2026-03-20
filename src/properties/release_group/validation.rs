//! Release group validation: position-based overlap detection and token filtering.
//!
//! # Architecture
//!
//! v0.3 replaced the old 130+ token exclusion list with position-based
//! overlap detection (`is_position_claimed`). Release group extraction now
//! runs AFTER conflict resolution, so we check whether a candidate span is
//! already claimed by a resolved tech match (VideoCodec, Source, etc.).
//!
//! A small curated list of non-tech tokens (`is_non_group_token`) is retained
//! for tokens not covered by TOML rules:
//!
//! - **Container extensions** (mkv, mp4, etc.): These exist in `container.toml`
//!   but container detection uses the extension path (PATH A), not token
//!   matching. So `is_position_claimed()` won't catch them mid-filename.
//!
//! - **Subtitle/metadata markers** (fansub, dublado, etc.): Not covered by
//!   any TOML rule — they're purely release-group-exclusion tokens.

use crate::matcher::span::{MatchSpan, Property};
use crate::zone_map;

/// Check if a byte range in the input is already claimed by resolved matches.
///
/// Returns true when resolved tech matches **collectively** cover ≥50% of
/// the candidate span `[start, end)`. This catches both:
/// - A single match covering most of the candidate (original behavior)
/// - Multiple matches covering a compound string like `H264_FLACx3_DTS-HDMA`
///   where three smaller matches together cover the full span (#37)
pub fn is_position_claimed(start: usize, end: usize, resolved: &[MatchSpan]) -> bool {
    let candidate_len = end.saturating_sub(start);
    if candidate_len == 0 {
        return false;
    }

    // Aggregate non-overlapping coverage from all tech matches.
    let total_overlap: usize = resolved
        .iter()
        .filter(|m| {
            // Skip positional/aggregate properties — these claim broad text
            // spans that may overlap with the release group. Only tech
            // properties (codecs, sources, etc.) are reliable.
            !matches!(
                m.property,
                Property::ReleaseGroup
                    | Property::Title
                    | Property::EpisodeTitle
                    | Property::FilmTitle
                    | Property::BonusTitle
                    | Property::AlternativeTitle
            )
        })
        .filter_map(|m| {
            // Only count overlapping ranges.
            if start >= m.end || end <= m.start {
                return None;
            }
            let overlap_start = start.max(m.start);
            let overlap_end = end.min(m.end);
            Some(overlap_end.saturating_sub(overlap_start))
        })
        .sum();

    // Require at least 50% aggregate coverage.
    // This prevents over-broad regex matches (like VideoCodec "hevc.+"
    // spanning "HEVC.Atmos-EPSiLON") from blocking the release group,
    // while still catching compound codec strings where multiple matches
    // together cover the full span.
    total_overlap * 2 >= candidate_len
}

/// Check if a candidate group name should be rejected.
///
/// This is a small curated list of tokens that:
/// 1. Are NOT tech tokens (so they won't appear in resolved matches)
/// 2. Should NEVER be release group names
///
/// This replaces the old 130+ token `is_known_token` list. Most of those
/// tokens are now caught by `is_position_claimed` instead.
pub fn is_non_group_token(s: &str) -> bool {
    let lower = s.to_lowercase();
    // Subtitle/metadata markers not covered by TOML rules.
    // Container extensions (not always claimed as resolved matches,
    // since container detection uses the extension path, not token matching).
    if matches!(
        lower.as_str(),
        "mkv"
            | "mp4"
            | "avi"
            | "wmv"
            | "flv"
            | "mov"
            | "webm"
            | "ogm"
            | "srt"
            | "sub"
            | "subs"
            | "idx"
            | "nfo"
            | "iso"
            | "par"
            | "par2"
    ) {
        return true;
    }
    // Subtitle / metadata markers not covered by TOML rules.
    matches!(
        lower.as_str(),
        "fansub"
            | "fansubbed"
            | "fastsub"
            | "multisubs"
            | "multi subs"
            | "multi sub"
            | "subtitle"
            | "subtitles"
            | "subforced"
            | "noreleasegroup"
            | "dublado"
            | "legendas"
            | "legendado"
            | "subtitulado"
    )
}

/// Combined check: is the candidate rejected either by position overlap
/// or by being a known non-group token?
pub fn is_rejected_group(
    candidate: &str,
    abs_start: usize,
    abs_end: usize,
    resolved: &[MatchSpan],
) -> bool {
    is_position_claimed(abs_start, abs_end, resolved)
        || is_non_group_token(candidate)
        || zone_map::is_tier2_token(candidate)
        || is_suffixed_resolution(candidate)
}

/// Strip trailing metadata tokens that follow the release group.
///
/// Handles patterns like:
///   `-Belex.-.Dual.Audio.-.Dublado` → `-Belex`
///   `-AFG.HebSubs` → `-AFG`
///   `-demand.sample.mkv` → `-demand.mkv`
///
/// This uses word-level checking (not position-based) because trailing
/// metadata can span multiple dot-segments mixed with the group name,
/// and we need to strip them before regex matching.
pub fn strip_trailing_metadata(filename: &str) -> String {
    static META_TOKENS: &[&str] = &[
        "dual",
        "audio",
        "dublado",
        "legendas",
        "legendado",
        "subtitulado",
        "hebsubs",
        "nlsubs",
        "swesub",
        "subbed",
        "dubbed",
        "sample",
        "proof",
        "proper",
        "repack",
        "real",
        "internal",
        "hardcoded",
        "eng",
        "fre",
        "fra",
        "spa",
        "ger",
        "deu",
        "ita",
        "jpn",
        "kor",
        "rus",
        "por",
        "ara",
        "hin",
        "chi",
        "hun",
        "multi",
        "vff",
        "vost",
        "vostfr",
        "truefrench",
        "flemish",
        "cze",
        "pol",
        "swe",
        "nor",
        "dan",
        "fin",
        "espanol",
        "esp",
    ];

    // Strip the file extension first.
    let (base, ext) = match filename.rfind('.') {
        Some(dot) if filename.len() - dot <= 6 => (&filename[..dot], &filename[dot..]),
        _ => (filename, ""),
    };

    // Walk backwards through dot-separated segments stripping metadata.
    let mut result = base.to_string();
    loop {
        // Strip trailing `.-.` separators.
        let trimmed = result.trim_end_matches(['.', '-', '_', ' ', '+', '[', ']']);
        if trimmed.len() < result.len() {
            result = trimmed.to_string();
            continue;
        }

        // Check if the last dot-segment is a metadata token.
        if let Some(dot) = result.rfind('.') {
            let segment = &result[dot + 1..];
            if META_TOKENS.iter().any(|t| segment.eq_ignore_ascii_case(t)) {
                result = result[..dot].to_string();
                continue;
            }
        }
        break;
    }

    format!("{result}{ext}")
}

/// Check if a string looks like a CRC32 hex value.
pub fn is_hex_crc(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Check if a string looks like a suffixed resolution (Tier 1 anchor).
/// E.g., 720p, 1080p, 2160p, 480i — these are structural anchors that
/// should never be release group names.
pub fn is_suffixed_resolution(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 4 || bytes.len() > 5 {
        return false;
    }
    let last = bytes[bytes.len() - 1].to_ascii_lowercase();
    if last != b'p' && last != b'i' {
        return false;
    }
    bytes[..bytes.len() - 1].iter().all(|b| b.is_ascii_digit())
}

/// Check if a byte range is claimed by a tech property (codec, source, etc.).
///
/// Unlike `is_position_claimed`, this ONLY counts structural tech properties,
/// NOT contextual ones like Language, SubtitleLanguage, Country, Year, etc.
/// Used by `expand_group_backwards` to decide if a preceding token is a
/// tech anchor (which would justify backwards expansion).
fn is_tech_claimed(start: usize, end: usize, resolved: &[MatchSpan]) -> bool {
    resolved.iter().any(|m| {
        if !matches!(
            m.property,
            Property::VideoCodec
                | Property::AudioCodec
                | Property::Source
                | Property::ScreenSize
                | Property::AudioChannels
                | Property::AudioProfile
                | Property::VideoProfile
                | Property::FrameRate
                | Property::ColorDepth
                | Property::StreamingService
                | Property::Container
                | Property::Other
                | Property::Edition
        ) {
            return false;
        }
        start < m.end && end > m.start
    })
}

/// Expand a release group name backwards past hyphens to capture
/// multi-segment names like `MARINE-FORD` or `D-Z0N3`.
///
/// Uses resolved match positions + Tier 2 tokens to decide whether
/// the preceding segment is a tech token (stop expanding) or part
/// of the group name (keep expanding).
pub fn expand_group_backwards(
    before: &str,
    current: &str,
    filename_start: usize,
    resolved: &[MatchSpan],
) -> String {
    let sep_pos = match before.rfind(['.', '-', '_']) {
        Some(pos) => pos,
        None => return current.to_string(),
    };

    let segment = &before[sep_pos + 1..];
    let before_sep = &before[..sep_pos];

    if segment.is_empty()
        || !segment.chars().all(|c| c.is_ascii_alphanumeric())
        || segment.chars().all(|c| c.is_ascii_digit())
    {
        return current.to_string();
    }

    // Check if the segment is claimed by a resolved match or is a tech token.
    let seg_abs_start = filename_start + sep_pos + 1;
    let seg_abs_end = filename_start + sep_pos + 1 + segment.len();
    if is_position_claimed(seg_abs_start, seg_abs_end, resolved)
        || zone_map::is_tier2_token(segment)
        || is_non_group_token(segment)
        || is_suffixed_resolution(segment)
    {
        return current.to_string();
    }

    // Check if the segment combined with the preceding token forms a
    // known compound (e.g., "DVD" + "R" = "dvdr" is a Tier 2 token).
    let last_word_before = before_sep
        .rsplit(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("");
    if !last_word_before.is_empty() {
        let compound = format!("{}{}", last_word_before, segment).to_lowercase();
        if zone_map::is_tier2_token(&compound) || is_non_group_token(&compound) {
            return current.to_string();
        }
    }

    // Check that the token BEFORE the segment IS a tech token
    // (otherwise we'd be expanding into title territory).
    // Only count real tech properties (codecs, sources), not Language/Year.
    let last_token = before_sep.rsplit(['.', '-', '_', ' ']).next().unwrap_or("");
    if !last_token.is_empty() {
        let lt_abs_start = filename_start + before_sep.len() - last_token.len();
        let lt_abs_end = filename_start + before_sep.len();
        let last_is_tech = is_tech_claimed(lt_abs_start, lt_abs_end, resolved)
            || zone_map::is_tier2_token(last_token);
        if !last_is_tech {
            return current.to_string();
        }
    }

    format!("{segment}-{current}")
}
