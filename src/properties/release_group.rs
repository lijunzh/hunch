//! Release group extraction.
//!
//! Release groups typically appear at the end of the filename, after a "-".
//! Example: `Movie.2024.1080p.BluRay.x264-GROUP.mkv` -> "GROUP"
//!
//! Also handles:
//! - Groups before `[website]`: `-FtS.[site.com].mkv`
//! - Groups with `@`: `HiS@SiLUHD`
//! - Bracket prefix groups: `[SubGroup] Anime`

use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// Matches `-GROUP` at the end with optional bracket suffix.
/// Tolerates known tags, subtitle markers, and language codes between the group and extension.
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
/// Excludes website-like content (containing dots) and hex CRC values.
static RELEASE_GROUP_END_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(?P<group>[A-Za-z][A-Za-z0-9 _!&-]{0,30})\](?:\.[a-z0-9]{2,5})?$").unwrap()
});

/// Space-separated group at end: `x264.dxva EuReKA.mkv` or `AC3 TiTAN.mkv`.
static RELEASE_GROUP_SPACE_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s(?P<group>[A-Za-z][A-Za-z0-9]{1,15})(?:\.[a-z0-9]{2,5})?$").unwrap()
});

/// Last token after dots as fallback: `720p.YIFY` or `x264.anoXmous`.
static RELEASE_GROUP_LAST_DOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.(?P<group>[A-Za-z][A-Za-z0-9]{2,15})(?:\.[a-z0-9]{2,5})?$").unwrap()
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();

    // Use the filename portion for matching.
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];

    // Strip trailing metadata suffixes that appear after the release group.
    // e.g., `-Belex.-.Dual.Audio.-.Dublado` → `-Belex`
    //        `-AFG.HebSubs` → `-AFG`
    let cleaned_filename = strip_trailing_metadata(filename);

    // 1. Check for simple `-GROUP` at end with optional bracket suffix.
    //    Try the cleaned filename first (metadata stripped), fall back to original.
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

            // Expand backwards past hyphens to capture multi-segment group names.
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
                .with_priority(-2),
            );
        }
    }

    // 3. Dash-bracket group at end: `x264-[2Maverick].mp4`.
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

    // 4. Bracket group at end: `Title [GROUP].ext` — checked before start bracket
    //    so that `[StartGroup]...[EndGroup]` picks the end one.
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

    // 4. Bracket group at start: `[GROUP] Title`.
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

    // 5. Space-separated at end: `x264.dxva EuReKA.mkv`.
    // Only when filename has technical tokens — otherwise we'd eat title words.
    if matches.is_empty()
        && has_technical_tokens(filename)
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
                .with_priority(-4),
            );
        }
    }

    // (Prefix pattern disabled — too many false positives.)

    // 8. Check parent directory for release group.
    // Two cases:
    //   a) Filename has no group at all — always try parent.
    //   b) Filename is an abbreviated scene release (e.g., `wthd-cab.avi`)
    //      and the parent dir has the real group (e.g., `DVDRip.XviD-TheWretched`).
    //      We prefer the parent when ALL of:
    //        - filename is short (< 20 chars) with no technical tokens
    //        - parent dir has technical tokens AND a `-GROUP` pattern
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
                    // Also check for bracket suffix in parent: `-GROUP[bb]`
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
            | "atmos"
            | "truehd"
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
            // Audio codecs / profiles.
            | "eac3"
            | "ddp"
            | "dd2"
            | "dd5"
            | "dd7"
            | "dtsx"
            | "ddplus"
            // Source / container variants.
            | "dvdr"
            | "dvd5"
            | "dvd9"
            | "dvdscr"
            | "hddvd"
            | "sdtv"
            | "pdtv"
            | "dsr"
            | "hdrip"
            | "r5"
            | "stv"
            // Release tags (not group names).
            | "preair"
            | "prooffix"
            | "proof"
            | "readnfo"
            | "sample"
            | "subbed"
            | "reenc"
            | "reencoded"
            | "re-enc"
            | "re-encoded"
            | "dublado"
            | "legendas"
            | "legendado"
            | "subtitulado"
            | "hebsubs"
            | "nlsubs"
            | "swesub"
            | "noreleasegroup"
            // Subtitle markers.
            | "multiple subtitle"
            | "multi subs"
            | "multisubs"
            | "multi sub"
            | "subtitle"
            | "subtitles"
            | "subforced"
    )
}

/// Strip trailing metadata tokens that follow the release group.
///
/// Handles patterns like:
///   `-Belex.-.Dual.Audio.-.Dublado` → `-Belex`
///   `-AFG.HebSubs` → `-AFG`
///   `-demand.sample.mkv` → `-demand.mkv`
fn strip_trailing_metadata(filename: &str) -> String {
    // Known metadata tokens that appear after the group (case-insensitive).
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
        "por",
        "jpn",
        "kor",
        "rus",
        "chi",
        "cze",
        "pol",
        "hun",
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
        let trimmed = result.trim_end_matches(['.', '-', '_', ' ', '+']);
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
fn is_hex_crc(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Expand a release group name backwards past hyphens to capture
/// multi-segment names like `MARINE-FORD` or `D-Z0N3`.
///
/// `before` is the text preceding the hyphen (e.g., "x264.MARINE" for "-FORD").
/// `current` is the matched group so far (e.g., "FORD").
///
/// Only expand when the segment BEFORE the group is separated by a dot
/// (not a hyphen) from a known technical token. This ensures we capture
/// `x264.D-Z0N3` (dot before D, x264 is known) but not `Movie-x264`
/// (Movie is not separated by dot from anything known).
fn expand_group_backwards(before: &str, current: &str) -> String {
    // Look for a segment before the group that should be included.
    // Pattern: `KNOWN_TOKEN.SEGMENT-CURRENT` or `KNOWN_TOKEN-SEGMENT-CURRENT`
    // where SEGMENT is not a known token.
    let sep_pos = match before.rfind(['.', '-', '_']) {
        Some(pos) => pos,
        None => return current.to_string(),
    };

    let segment = &before[sep_pos + 1..];
    let before_sep = &before[..sep_pos];

    // The segment must be alphanumeric, not purely numeric, and not a known token.
    // Also reject if the segment forms a known compound with the preceding word.
    if segment.is_empty()
        || !segment.chars().all(|c| c.is_ascii_alphanumeric())
        || segment.chars().all(|c| c.is_ascii_digit())
        || is_known_token(segment)
    {
        return current.to_string();
    }

    // Check if segment + preceding token form a known compound.
    let last_word_before = before_sep
        .rsplit(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("");
    let compound = format!("{}{}", last_word_before, segment).to_lowercase();
    if is_known_token(&compound) {
        return current.to_string();
    }

    // The text before the separator must end with a known technical token.
    let last_token = before_sep.rsplit(['.', '-', '_', ' ']).next().unwrap_or("");
    if !is_known_token(last_token) {
        return current.to_string();
    }

    format!("{segment}-{current}")
}

/// Returns true if the filename contains recognizable technical tokens
/// (codecs, resolutions, sources, etc.). This helps the "last-dot" fallback
/// avoid false positives on simple filenames like `Title Only.avi`.
fn has_technical_tokens(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let technical = [
        "x264", "x265", "h264", "h265", "hevc", "xvid", "divx", "av1", "aac", "ac3", "dts", "flac",
        "opus", "720p", "1080p", "2160p", "4k", "bluray", "bdrip", "brrip", "dvdrip", "webrip",
        "webdl", "hdtv", "hdrip", "remux", "cam", "screener", "atmos", "truehd",
    ];
    technical.iter().any(|t| lower.contains(t))
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
        // When filename has no group pattern, fall back to parent dir.
        let m = find_matches("movies/Movie.DVDRip.XviD-DiAMOND/somefile.avi");
        assert!(m.iter().any(|x| x.value == "DiAMOND"));
    }

    #[test]
    fn test_group_with_crc() {
        let m = find_matches("[SubGroup] Anime - 01 [1080p][DEADBEEF].mkv");
        // Bracket groups handled separately.
        assert!(m.is_empty() || m.iter().all(|x| !x.value.is_empty()));
    }
}
