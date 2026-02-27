//! Known tokens that should NOT be treated as release group names.
//!
//! This is a manually-maintained exclusion list. When tokens are added to
//! TOML rule files, they should also be added here to prevent false-positive
//! release group detection.
//!
//! TODO (v0.3.x): Replace with post-resolution overlap detection against
//! resolved MatchSpan positions. See ARCHITECTURE.md Phase E1.

/// Check if a string is a known token that shouldn't be a release group.
pub fn is_known_token(s: &str) -> bool {
    let lower = s.to_lowercase();
    // Containers.
    if matches!(
        lower.as_str(),
        "mkv" | "mp4" | "avi" | "wmv" | "flv" | "mov" | "webm" | "ogm"
            | "srt" | "sub" | "subs" | "idx" | "nfo" | "iso" | "par" | "par2"
    ) {
        return true;
    }
    // Video codecs.
    if matches!(
        lower.as_str(),
        "x264" | "x265" | "h264" | "h265" | "hevc" | "avc" | "av1"
            | "xvid" | "divx" | "dvdivx"
    ) {
        return true;
    }
    // Audio codecs / profiles.
    if matches!(
        lower.as_str(),
        "aac" | "ac3" | "dts" | "flac" | "mp3" | "pcm" | "opus"
            | "atmos" | "truehd" | "ma"
            | "eac3" | "ddp" | "dd2" | "dd5" | "dd7" | "dtsx" | "ddplus"
    ) {
        return true;
    }
    // Sources.
    if matches!(
        lower.as_str(),
        "bluray" | "bdrip" | "brrip" | "dvdrip" | "webrip" | "webdl" | "hdtv"
            | "dvd" | "dvdr" | "dvd5" | "dvd9" | "dvdscr" | "hddvd"
            | "sdtv" | "pdtv" | "dsr" | "hdrip" | "vhsrip" | "cam"
            | "screener" | "scr" | "r5" | "stv"
    ) {
        return true;
    }
    // Screen sizes / quality.
    if matches!(
        lower.as_str(),
        "720p" | "1080p" | "2160p" | "4k" | "hdr" | "hdr10" | "sdr"
            | "hq" | "lq"
    ) {
        return true;
    }
    // Release tags.
    if matches!(
        lower.as_str(),
        "remux" | "proper" | "repack" | "internal" | "limited"
            | "unrated" | "extended" | "directors" | "cut" | "complete"
            | "season" | "disc" | "imax" | "edition"
            | "preair" | "prooffix" | "proof" | "readnfo" | "sample"
            | "subbed" | "reenc" | "reencoded" | "re-enc" | "re-encoded"
    ) {
        return true;
    }
    // Languages.
    if matches!(
        lower.as_str(),
        "english" | "french" | "spanish" | "german" | "italian"
            | "eng" | "fre" | "spa" | "multi" | "dual" | "dubbed"
            | "hun" | "ger" | "deu" | "ita" | "por" | "jpn" | "kor" | "rus"
            | "truefrench" | "vfi" | "flemish"
            | "pal" | "ntsc" | "dub" | "vostfr" | "vff" | "vost"
    ) {
        return true;
    }
    // Subtitle markers.
    if matches!(
        lower.as_str(),
        "fansub" | "fansubbed" | "fastsub"
            | "multiple subtitle" | "multi subs" | "multisubs" | "multi sub"
            | "subtitle" | "subtitles" | "subforced"
            | "dublado" | "legendas" | "legendado" | "subtitulado"
            | "hebsubs" | "nlsubs" | "swesub" | "noreleasegroup"
    ) {
        return true;
    }
    false
}

/// Strip trailing metadata tokens that follow the release group.
///
/// Handles patterns like:
///   `-Belex.-.Dual.Audio.-.Dublado` → `-Belex`
///   `-AFG.HebSubs` → `-AFG`
///   `-demand.sample.mkv` → `-demand.mkv`
pub fn strip_trailing_metadata(filename: &str) -> String {
    static META_TOKENS: &[&str] = &[
        "dual", "audio", "dublado", "legendas", "legendado", "subtitulado",
        "hebsubs", "nlsubs", "swesub", "subbed", "dubbed", "sample", "proof",
        "proper", "repack", "real", "internal", "hardcoded",
        "eng", "fre", "fra", "spa", "ger", "deu", "ita", "jpn", "kor", "rus",
        "por", "ara", "hin", "chi", "hun", "multi", "vff", "vost", "vostfr",
        "truefrench", "flemish", "cze", "pol", "swe", "nor", "dan", "fin",
        "espanol", "esp",
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
pub fn is_hex_crc(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Expand a release group name backwards past hyphens to capture
/// multi-segment names like `MARINE-FORD` or `D-Z0N3`.
pub fn expand_group_backwards(before: &str, current: &str) -> String {
    let sep_pos = match before.rfind(['.', '-', '_']) {
        Some(pos) => pos,
        None => return current.to_string(),
    };

    let segment = &before[sep_pos + 1..];
    let before_sep = &before[..sep_pos];

    if segment.is_empty()
        || !segment.chars().all(|c| c.is_ascii_alphanumeric())
        || segment.chars().all(|c| c.is_ascii_digit())
        || is_known_token(segment)
    {
        return current.to_string();
    }

    let last_word_before = before_sep
        .rsplit(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("");
    let compound = format!("{}{}", last_word_before, segment).to_lowercase();
    if is_known_token(&compound) {
        return current.to_string();
    }

    let last_token = before_sep.rsplit(['.', '-', '_', ' ']).next().unwrap_or("");
    if !is_known_token(last_token) {
        return current.to_string();
    }

    format!("{segment}-{current}")
}

/// Returns true if the filename contains recognizable technical tokens.
pub fn has_technical_tokens(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    const TECH: &[&str] = &[
        "x264", "x265", "h264", "h265", "hevc", "xvid", "divx", "av1",
        "aac", "ac3", "dts", "flac", "opus", "720p", "1080p", "2160p", "4k",
        "bluray", "bdrip", "brrip", "dvdrip", "webrip", "webdl", "hdtv",
        "hdrip", "remux", "cam", "screener", "atmos", "truehd",
    ];
    TECH.iter().any(|t| lower.contains(t))
}
