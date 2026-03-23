//! Title string cleaning — separator replacement, bracket stripping, etc.

use super::{BRACKETS, SEPS};

/// Clean up a raw title: replace separators with spaces, strip brackets, trim.
pub(super) fn clean_title(raw: &str) -> String {
    clean_title_inner(raw, true)
}

pub(super) fn clean_episode_title(raw: &str) -> String {
    let trimmed = raw.trim_start_matches(['.', '_', ' ', '-']);
    clean_title_inner(trimmed, false)
}

fn clean_title_inner(raw: &str, strip_season_part: bool) -> String {
    let mut s = raw.to_string();

    // Strip leading bracket groups: [XCT], [阿维达], etc.
    while s.starts_with('[') {
        if let Some(end) = s.find(']') {
            s = s[end + 1..].to_string();
            s = s.trim_start_matches(SEPS).to_string();
        } else {
            break;
        }
    }

    // Strip parenthesized year at the end: "Movie (2005)" → "Movie"
    let re_paren_year = regex::Regex::new(r"\s*\((?:19|20)\d{2}\)\s*$").unwrap();
    if let Some(m) = re_paren_year.find(&s) {
        s = s[..m.start()].to_string();
    }

    // Strip all parenthesized groups (alternative titles, countries, etc.).
    let re_paren = regex::Regex::new(r"\s*\([^)]*\)\s*").unwrap();
    let before_paren_strip = s.clone();
    s = re_paren.replace_all(&s, " ").to_string();
    if s.trim().is_empty() {
        s = before_paren_strip;
    }

    // Replace separators with spaces, preserving hyphens between letters
    // and dot-acronyms like S.H.I.E.L.D.
    let dot_acronym_re =
        regex::Regex::new(r"(?:^|[\s._])([A-Za-z0-9](?:\.[A-Za-z0-9]){2,}\.?)").unwrap();

    let mut protected_ranges: Vec<(usize, usize)> = Vec::new();
    for m in dot_acronym_re.find_iter(&s) {
        let actual_start =
            if m.start() > 0 && matches!(s.as_bytes()[m.start()], b' ' | b'\t' | b'.' | b'_') {
                m.start() + 1
            } else {
                m.start()
            };
        protected_ranges.push((actual_start, m.end()));
    }

    let in_protected =
        |pos: usize| -> bool { protected_ranges.iter().any(|(s, e)| pos >= *s && pos < *e) };

    let chars: Vec<char> = s.chars().collect();
    let mut byte_positions: Vec<usize> = Vec::with_capacity(chars.len());
    let mut byte_pos = 0;
    for &c in &chars {
        byte_positions.push(byte_pos);
        byte_pos += c.len_utf8();
    }

    let cleaned: String = chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c == '-' {
                let prev_alnum = i > 0 && chars[i - 1].is_alphanumeric();
                let next_alnum = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
                if prev_alnum && next_alnum { '-' } else { ' ' }
            } else if c == '.' && in_protected(byte_positions[i]) {
                '.'
            } else if SEPS.contains(&c) || BRACKETS.contains(&c) || c == '*' {
                ' '
            } else {
                c
            }
        })
        .collect();

    let mut result = collapse_spaces(&cleaned);

    // Strip trailing punctuation that leaks from separator boundaries.
    result = result
        .trim_end_matches([':', '-', ',', ';'])
        .trim()
        .to_string();

    if strip_season_part {
        result = strip_trailing_keywords(&result);
    }

    result
}

/// Strip trailing Part, Season, Episode keywords and bonus markers from titles.
fn strip_trailing_keywords(result: &str) -> String {
    let mut result = result.to_string();

    // Strip trailing "Part" + optional roman/number.
    let re_part =
        regex::Regex::new(r"(?i)\s+Part\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?\s*$").unwrap();
    if let Some(m) = re_part.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    // Strip trailing season words.
    let re_season_word = regex::Regex::new(
        r"(?i)\s+(?:Saison|Temporada|Stagione|Tem\.?|Season|Seasons?)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?(?:\s*(?:&|and)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+))?\s*$"
    ).unwrap();
    if let Some(m) = re_season_word.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    // Strip trailing episode keywords.
    let re_ep_word = regex::Regex::new(r"(?i)\s+(?:Episodes?|Ep\.?)\s*$").unwrap();
    if let Some(m) = re_ep_word.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    // Strip trailing bonus markers.
    let re_bonus = regex::Regex::new(r"(?i)[-]x\d{1,3}\s*$").unwrap();
    if let Some(m) = re_bonus.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    result
}

/// Collapse multiple spaces into one and trim.
pub(super) fn collapse_spaces(s: &str) -> String {
    let mut result = String::new();
    let mut prev_space = true;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(' ');
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

/// Strip a file extension from the end of a string.
pub(super) fn strip_extension(s: &str) -> &str {
    if let Some(dot) = s.rfind('.') {
        let ext = &s[dot + 1..];
        let ext_lower = ext.to_lowercase();
        if ext.len() <= 5 && is_likely_extension(&ext_lower) {
            return &s[..dot];
        }
    }
    s
}

/// Check if a string looks like a real file extension.
pub(super) fn is_likely_extension(ext: &str) -> bool {
    matches!(
        ext,
        "mkv"
            | "mp4"
            | "avi"
            | "wmv"
            | "flv"
            | "mov"
            | "webm"
            | "ogm"
            | "ogv"
            | "ts"
            | "m2ts"
            | "m4v"
            | "mpg"
            | "mpeg"
            | "vob"
            | "divx"
            | "3gp"
            | "srt"
            | "sub"
            | "ssa"
            | "ass"
            | "idx"
            | "sup"
            | "vtt"
            | "nfo"
            | "txt"
            | "jpg"
            | "jpeg"
            | "png"
            | "nzb"
            | "par"
            | "par2"
            | "iso"
            | "img"
            | "rar"
            | "zip"
            | "7z"
    )
}

/// Detect if a title looks like a scene abbreviation.
pub(super) fn is_abbreviated(title: &str) -> bool {
    let segments: Vec<&str> = title
        .split(|c: char| c.is_whitespace() || c == '-')
        .collect();
    segments.iter().all(|w| {
        w.len() <= 6
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    }) && title.len() <= 20
}

/// Pick the string with better casing when two titles match case-insensitively.
pub(super) fn pick_better_casing<'a>(a: &'a str, b: &'a str) -> &'a str {
    fn casing_score(s: &str) -> i32 {
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
        {
            return -10;
        }
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase())
        {
            return -5;
        }
        s.split_whitespace()
            .filter(|w| w.starts_with(|c: char| c.is_uppercase()))
            .count() as i32
    }
    if casing_score(a) >= casing_score(b) {
        a
    } else {
        b
    }
}

/// Check if a directory name is generic (should be skipped for title).
///
/// Generic directories are structural (e.g., "Season 1", "Extras") or
/// organizational (e.g., "Movies", "Downloads"). When walking parent
/// directories for title fallback, these are skipped so the real title
/// directory (e.g., "Transformers 1984") is found.
pub(super) fn is_generic_dir(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Exact matches (case-insensitive).
    if matches!(
        lower.as_str(),
        // Library root / organizational
        "movies"
            | "movie"
            | "films"
            | "film"
            | "series"
            | "tv shows"
            | "tvshows"
            | "tv"
            | "media"
            | "video"
            | "videos"
            | "anime"
            | "donghua"
            | "kids"
            | "cartoons"
            | "shows"
            | "documentary"
            | "documentaries"
            | "music"
            | "concert"
            | "concerts"
            // Language categories (library organization, not show titles)
            | "chinese"
            | "english"
            | "japanese"
            | "korean"
            | "french"
            | "german"
            | "spanish"
            | "italian"
            | "portuguese"
            | "russian"
            | "thai"
            | "hindi"
            | "arabic"
            // Download / system
            | "downloads"
            | "download"
            | "completed"
            | "mnt"
            | "nas"
            | "share"
            | "shares"
            | "data"
            | "public"
            | "home"
            | "tmp"
            | "temp"
            // Bonus / extras
            | "extras"
            | "extra"
            | "specials"
            | "special"
            | "bonus"
            | "featurettes"
            | "featurette"
            | "behind the scenes"
            | "deleted scenes"
            | "interviews"
            | "interview"
            | "trailers"
            | "trailer"
            | "samples"
            | "sample"
            // CJK bonus / extras directory names
            | "特典映像"  // tokuten eizou — special footage (JP)
            | "特典"      // tokuten — bonus/extras (JP)
            | "映像特典"  // eizou tokuten — video bonus (JP)
            | "sp"
            // Subtitles / audio
            | "subs"
            | "subtitles"
            | "subtitle"
            | "ost"
            | "soundtrack"
            | "soundtracks"
    ) {
        return true;
    }

    // Prefix matches (e.g., "Season 1", "Disc 2", "CD1").
    if lower.starts_with("season")
        || lower.starts_with("saison")
        || lower.starts_with("temporada")
        || lower.starts_with("stagione")
        || lower.starts_with("disc")
        || lower.starts_with("disk")
        || lower.starts_with("dvd")
    {
        return true;
    }

    // CD1, CD2, etc.
    if lower.starts_with("cd") && lower[2..].chars().all(|c| c.is_ascii_digit()) && lower.len() <= 4
    {
        return true;
    }

    // Quality-as-dir: "1080p", "720p", "2160p", "4K", "4k"
    if lower.ends_with('p') && lower[..lower.len() - 1].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if lower == "4k" {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title_dots() {
        assert_eq!(clean_title("The.Matrix"), "The Matrix");
    }

    #[test]
    fn test_clean_title_underscores() {
        assert_eq!(clean_title("The_Matrix_Reloaded"), "The Matrix Reloaded");
    }

    #[test]
    fn test_strip_leading_bracket() {
        assert_eq!(clean_title("[XCT].Le.Prestige"), "Le Prestige");
    }

    #[test]
    fn test_strip_paren_year() {
        assert_eq!(clean_title("Movie Name (2005)"), "Movie Name");
    }

    // ── is_generic_dir ──────────────────────────────────────────────

    #[test]
    fn generic_dir_originals() {
        // Original entries still work.
        assert!(is_generic_dir("Movies"));
        assert!(is_generic_dir("tv"));
        assert!(is_generic_dir("Season 1"));
        assert!(is_generic_dir("Saison 03"));
    }

    #[test]
    fn generic_dir_extras_and_bonus() {
        assert!(is_generic_dir("Extras"));
        assert!(is_generic_dir("Specials"));
        assert!(is_generic_dir("Bonus"));
        assert!(is_generic_dir("Featurettes"));
        assert!(is_generic_dir("Behind The Scenes"));
        assert!(is_generic_dir("Deleted Scenes"));
        assert!(is_generic_dir("Trailers"));
        assert!(is_generic_dir("Sample"));
    }

    #[test]
    fn generic_dir_disc_and_cd() {
        assert!(is_generic_dir("Disc 1"));
        assert!(is_generic_dir("Disc2"));
        assert!(is_generic_dir("Disk 3"));
        assert!(is_generic_dir("DVD1"));
        assert!(is_generic_dir("CD1"));
        assert!(is_generic_dir("CD2"));
        assert!(!is_generic_dir("CD123")); // too long for CD pattern
    }

    #[test]
    fn generic_dir_quality() {
        assert!(is_generic_dir("1080p"));
        assert!(is_generic_dir("720p"));
        assert!(is_generic_dir("2160p"));
        assert!(is_generic_dir("4K"));
    }

    #[test]
    fn generic_dir_subtitles_and_audio() {
        assert!(is_generic_dir("Subs"));
        assert!(is_generic_dir("Subtitles"));
        assert!(is_generic_dir("OST"));
        assert!(is_generic_dir("Soundtrack"));
    }

    #[test]
    fn generic_dir_structural() {
        assert!(is_generic_dir("Anime"));
        assert!(is_generic_dir("Kids"));
        assert!(is_generic_dir("Cartoons"));
        assert!(is_generic_dir("Shows"));
        assert!(is_generic_dir("Documentary"));
        assert!(is_generic_dir("Documentaries"));
        assert!(is_generic_dir("Music"));
        assert!(is_generic_dir("Concert"));
        assert!(is_generic_dir("Concerts"));
    }

    #[test]
    fn generic_dir_language_categories() {
        assert!(is_generic_dir("Chinese"));
        assert!(is_generic_dir("English"));
        assert!(is_generic_dir("Japanese"));
        assert!(is_generic_dir("Korean"));
        assert!(is_generic_dir("French"));
        assert!(is_generic_dir("German"));
        assert!(is_generic_dir("Spanish"));
        assert!(is_generic_dir("Italian"));
        assert!(is_generic_dir("Portuguese"));
        assert!(is_generic_dir("Russian"));
        assert!(is_generic_dir("Thai"));
        assert!(is_generic_dir("Hindi"));
        assert!(is_generic_dir("Arabic"));
    }

    #[test]
    fn generic_dir_cjk_bonus() {
        assert!(is_generic_dir("特典映像"));
        assert!(is_generic_dir("特典"));
        assert!(is_generic_dir("映像特典"));
        assert!(is_generic_dir("SP"));
    }

    #[test]
    fn non_generic_dirs() {
        // Real titles should NOT be generic.
        assert!(!is_generic_dir("Paw Patrol"));
        assert!(!is_generic_dir("Transformers 1984"));
        assert!(!is_generic_dir("Breaking Bad"));
        assert!(!is_generic_dir("十二国記"));
    }
}
