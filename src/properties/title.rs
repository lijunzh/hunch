//! Title extraction — positional rule ("whatever's left" after other matchers).
//!
//! This runs as a post-processing step rather than a regex matcher. The title
//! is typically everything before the first recognized property token.
//!
//! Handles:
//! - Path-aware extraction (prefers parent dir for abbreviated filenames)
//! - Stripping leading bracket groups like `[XCT]`
//! - Stripping parenthesized text after the first recognized word group

use crate::matcher::span::{MatchSpan, Property};

/// Separators used in media filenames.
const SEPS: &[char] = &['.', ' ', '_', '-', '+'];

/// Characters we strip from title boundaries.
const BRACKETS: &[char] = &['(', ')', '[', ']', '{', '}'];

/// Extract title from the input string by finding the gap before the first
/// recognized match. This is a post-processing step, not a `PropertyMatcher`.
pub fn extract_title(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
    // Find the filename portion (after last path separator).
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];

    // First match in the filename portion (skip extension matches).
    let first_match_in_filename = matches
        .iter()
        .filter(|m| m.start >= filename_start)
        .filter(|m| !m.is_extension)
        .min_by_key(|m| m.start);

    let title_end_abs = match first_match_in_filename {
        Some(m) => m.start,
        None => {
            // All matches are extensions or outside filename.
            // Only strip if the trailing segment looks like a real extension.
            let ext_start = filename.rfind('.').unwrap_or(filename.len());
            if ext_start < filename.len() {
                let candidate_ext = &filename[ext_start + 1..];
                if is_likely_extension(&candidate_ext.to_lowercase()) {
                    filename_start + ext_start
                } else {
                    filename_start + filename.len()
                }
            } else {
                filename_start + filename.len()
            }
        }
    };

    if title_end_abs <= filename_start {
        // Title is empty in the filename — try parent directory.
        return extract_title_from_parent(input, matches);
    }

    let raw_title = &input[filename_start..title_end_abs];
    let cleaned = clean_title(raw_title);

    if cleaned.is_empty() {
        // Try anime-style: [Group] Title - Episode.
        // Look for text between the first bracket group and the next property.
        if let Some(title) = extract_after_bracket_group(input, matches, filename_start) {
            return Some(title);
        }
        return extract_title_from_parent(input, matches);
    }

    // If parent directory has the same title (case-insensitive), pick the version
    // with better casing: prefer proper title case over ALL CAPS or all lowercase.
    if has_parent_dir(input)
        && let Some(parent_match) = extract_title_from_parent(input, matches)
        && parent_match.value.to_lowercase() == cleaned.to_lowercase()
        && parent_match.value != cleaned
    {
        let best = pick_better_casing(&cleaned, &parent_match.value);
        if best != cleaned {
            return Some(MatchSpan::new(
                filename_start,
                title_end_abs,
                Property::Title,
                best,
            ));
        }
    }

    // If the filename looks like a scene abbreviation (very short, no spaces/dots
    // in the cleaned result), prefer the parent directory.
    if is_abbreviated(&cleaned)
        && has_parent_dir(input)
        && let Some(parent_title) = extract_title_from_parent(input, matches)
    {
        return Some(parent_title);
    }

    Some(MatchSpan::new(
        filename_start,
        title_end_abs,
        Property::Title,
        cleaned,
    ))
}

/// Try to extract the title from the parent directory name.
fn extract_title_from_parent(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
    let parts: Vec<&str> = input.split(['/', '\\']).collect();
    if parts.len() < 2 {
        // No parent directory.
        if matches.is_empty() {
            let stripped = strip_extension(input);
            let cleaned = clean_title(stripped);
            if !cleaned.is_empty() {
                return Some(MatchSpan::new(0, stripped.len(), Property::Title, cleaned));
            }
        }
        return None;
    }

    // Walk from the deepest non-filename dir upward looking for a good title.
    let mut offset = 0;
    #[allow(clippy::needless_range_loop)]
    for i in 0..parts.len() - 1 {
        let dir_name = parts[i];
        let dir_start = offset;
        let dir_end = dir_start + dir_name.len();
        offset = dir_end + 1; // +1 for separator

        if dir_name.is_empty() || is_generic_dir(dir_name) {
            continue;
        }

        // Find the first property match that falls within this directory's span.
        let first_match_in_dir = matches
            .iter()
            .filter(|m| m.start >= dir_start && m.start < dir_end)
            .filter(|m| !m.is_extension)
            .filter(|m| !m.is_path_based)
            .min_by_key(|m| m.start);

        let title_end = match first_match_in_dir {
            Some(m) => m.start,
            None => dir_end,
        };

        if title_end <= dir_start {
            continue;
        }

        let raw_title = &input[dir_start..title_end];
        let cleaned = clean_title(raw_title);
        if !cleaned.is_empty() {
            return Some(MatchSpan::new(
                dir_start,
                title_end,
                Property::Title,
                cleaned,
            ));
        }
    }

    None
}

/// For anime-style: `[Group] Title - 04 [480p]`, extract "Title" from
/// the gap between the bracket group and the next property match.
fn extract_after_bracket_group(
    input: &str,
    matches: &[MatchSpan],
    filename_start: usize,
) -> Option<MatchSpan> {
    let filename = &input[filename_start..];
    let filename_end = filename_start + filename.len();

    // Find the end of leading bracket groups in the filename.
    let mut pos = 0;
    while pos < filename.len() && filename[pos..].starts_with('[') {
        if let Some(close) = filename[pos..].find(']') {
            pos += close + 1;
            // Skip trailing separators.
            while pos < filename.len() && SEPS.contains(&(filename.as_bytes()[pos] as char)) {
                pos += 1;
            }
        } else {
            break;
        }
    }

    if pos == 0 || pos >= filename.len() {
        return None;
    }

    let title_start_abs = filename_start + pos;

    // Find the next property match after this position.
    let next_match = matches
        .iter()
        .filter(|m| m.start >= title_start_abs && m.start < filename_end)
        .filter(|m| !m.is_extension)
        .min_by_key(|m| m.start);

    let title_end_abs = match next_match {
        Some(m) => m.start,
        None => {
            let has_ext = matches
                .iter()
                .any(|m| m.property == Property::Container && m.start >= filename_start);
            if has_ext {
                if let Some(dot) = filename.rfind('.') {
                    filename_start + dot
                } else {
                    filename_end
                }
            } else {
                filename_end
            }
        }
    };

    if title_end_abs <= title_start_abs {
        return None;
    }

    let raw = &input[title_start_abs..title_end_abs];
    let cleaned = clean_title(raw);
    if cleaned.is_empty() {
        return None;
    }

    Some(MatchSpan::new(
        title_start_abs,
        title_end_abs,
        Property::Title,
        cleaned,
    ))
}

fn has_parent_dir(input: &str) -> bool {
    input.contains('/') || input.contains('\\')
}

/// Check if a directory name is generic (should be skipped for title).
fn is_generic_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
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
            | "downloads"
            | "download"
            | "mnt"
            | "nas"
            | "share"
            | "shares"
            | "data"
            | "public"
            | "home"
    ) || lower.starts_with("season")
        || lower.starts_with("saison")
        || lower.starts_with("temporada")
}

/// Detect if a title looks like a scene abbreviation (e.g., "dmd", "wthd-cab", "i-smwhr").
fn is_abbreviated(title: &str) -> bool {
    // Split on whitespace AND hyphens to check individual segments.
    let segments: Vec<&str> = title
        .split(|c: char| c.is_whitespace() || c == '-')
        .collect();
    // All segments short and lowercase → probably abbreviated.
    segments.iter().all(|w| {
        w.len() <= 6
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    }) && title.len() <= 20
}

/// Pick the string with better casing when two titles match case-insensitively.
///
/// Prefers proper case ("Some Title") over ALL CAPS ("SOME TITLE") or all
/// lowercase ("some title"). Scores by counting words that start with an
/// uppercase letter followed by lowercase (proper-cased words).
fn pick_better_casing<'a>(a: &'a str, b: &'a str) -> &'a str {
    fn casing_score(s: &str) -> i32 {
        // Penalize ALL CAPS heavily.
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
        {
            return -10;
        }
        // Penalize all lowercase.
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase())
        {
            return -5;
        }
        // Score: count words starting with uppercase.
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

/// Clean up a raw title: replace separators with spaces, strip brackets, trim.
fn clean_title(raw: &str) -> String {
    let mut s = raw.to_string();

    // Strip leading bracket groups: [XCT], [阿维达], etc.
    while s.starts_with('[') {
        if let Some(end) = s.find(']') {
            s = s[end + 1..].to_string();
            // Also strip separator after bracket.
            s = s.trim_start_matches(SEPS).to_string();
        } else {
            break;
        }
    }

    // Strip parenthesized year at the end: "Movie (2005)" → "Movie"
    let re_paren_year = fancy_regex::Regex::new(r"\s*\((?:19|20)\d{2}\)\s*$").unwrap();
    if let Ok(Some(m)) = re_paren_year.find(&s) {
        s = s[..m.start()].to_string();
    }

    // Strip all parenthesized groups (alternative titles, countries, etc.).
    // e.g., "Le Prestige (The Prestige)" → "Le Prestige"
    //        "The Office (US)" → "The Office"
    let re_paren = fancy_regex::Regex::new(r"\s*\([^)]*\)\s*").unwrap();
    let before_paren_strip = s.clone();
    s = re_paren.replace_all(&s, " ").to_string();
    // If stripping removed everything, revert.
    if s.trim().is_empty() {
        s = before_paren_strip;
    }

    // Replace separators with spaces, but preserve hyphens between letters
    // and dot-acronyms like S.H.I.E.L.D. or T.I.T.L.E.
    let dot_acronym_re = fancy_regex::Regex::new(
        r"(?:^|(?<=[\s._]))([A-Za-z](?:\.[A-Za-z]){2,}\.?)"
    ).unwrap();

    // Find dot-acronym byte ranges to protect from separator replacement.
    let mut protected_ranges: Vec<(usize, usize)> = Vec::new();
    // Use captures_iter pattern from fancy_regex.
    let mut search_pos = 0;
    while search_pos < s.len() {
        match dot_acronym_re.find(&s[search_pos..]) {
            Ok(Some(m)) => {
                protected_ranges.push((search_pos + m.start(), search_pos + m.end()));
                search_pos += m.end();
            }
            _ => break,
        }
    }

    let in_protected = |pos: usize| -> bool {
        protected_ranges.iter().any(|(s, e)| pos >= *s && pos < *e)
    };

    let chars: Vec<char> = s.chars().collect();
    // Build byte-position map for checking protected ranges.
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
                // Preserve dots in acronyms.
                '.'
            } else if SEPS.contains(&c) || BRACKETS.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .collect();

    // Collapse multiple spaces and trim.
    let mut result = collapse_spaces(&cleaned);

    // Strip trailing "Part" + optional roman/number: "The Godfather Part III" → "The Godfather".
    let re_part =
        fancy_regex::Regex::new(r"(?i)\s+Part\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?\s*$")
            .unwrap();
    if let Ok(Some(m)) = re_part.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    // Strip trailing season words: "Dexter Saison VII" → "Dexter".
    let re_season_word = fancy_regex::Regex::new(
        r"(?i)\s+(?:Saison|Temporada|Tem\.?|Season|Seasons?)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?(?:\s*(?:&|and)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+))?\s*$"
    ).unwrap();
    if let Ok(Some(m)) = re_season_word.find(&result) {
        let stripped = result[..m.start()].to_string();
        if !stripped.trim().is_empty() {
            result = stripped;
        }
    }

    result
}

/// Collapse multiple spaces into one and trim.
fn collapse_spaces(s: &str) -> String {
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
/// Only strips if the extension looks like a real file extension
/// (lowercase, known media/subtitle/metadata format).
fn strip_extension(s: &str) -> &str {
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
fn is_likely_extension(ext: &str) -> bool {
    matches!(
        ext,
        "mkv" | "mp4" | "avi" | "wmv" | "flv" | "mov" | "webm" | "ogm" | "ogv"
            | "ts" | "m2ts" | "m4v" | "mpg" | "mpeg" | "vob" | "divx" | "3gp"
            | "srt" | "sub" | "ssa" | "ass" | "idx" | "sup" | "vtt"
            | "nfo" | "txt" | "jpg" | "jpeg" | "png" | "nzb" | "par" | "par2"
            | "iso" | "img" | "rar" | "zip" | "7z"
    )
}

/// Extract episode title: the text between the last episode/season marker
/// and the next technical property in the filename portion.
pub fn extract_episode_title(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];
    let filename_end = filename_start + filename.len();

    // Must have an actual episode match (not just season).
    let has_episode = matches
        .iter()
        .any(|m| m.property == Property::Episode && m.start >= filename_start);
    if !has_episode {
        return None;
    }

    // Find the last episode/season match in the filename.
    let last_ep_match = matches
        .iter()
        .filter(|m| {
            m.start >= filename_start
                && (m.property == Property::Episode || m.property == Property::Season)
        })
        .max_by_key(|m| m.end)?;

    let ep_title_start = last_ep_match.end;

    // Find the next "technical" property match after the episode marker.
    // Exclude ReleaseGroup — it's positional (last word) and would eat the
    // episode title's last word otherwise.
    let technical_props = [
        Property::VideoCodec,
        Property::AudioCodec,
        Property::Source,
        Property::ScreenSize,
        Property::Edition,
        Property::Other,
        Property::AudioChannels,
        Property::Language,
        Property::Container,
        Property::StreamingService,
        Property::Year,
    ];

    let next_tech = matches
        .iter()
        .filter(|m| {
            m.start >= ep_title_start
                && m.start < filename_end
                && technical_props.contains(&m.property)
        })
        .min_by_key(|m| m.start);

    let ep_title_end = match next_tech {
        Some(m) => m.start,
        None => {
            // No technical property — check for extension via container matches.
            let has_container = matches
                .iter()
                .any(|m| m.property == Property::Container && m.start >= filename_start);
            if has_container {
                // Use position of last dot as extension separator.
                let ext_dot = filename.rfind('.');
                match ext_dot {
                    Some(pos) => filename_start + pos,
                    None => filename_end,
                }
            } else {
                filename_end
            }
        }
    };

    // Further trim: stop at opening brackets/parens that likely indicate
    // metadata, not title content (e.g., "[tvu.org.ru]", "(14-01-2008)").
    let ep_title_end = {
        let region = &input[ep_title_start..ep_title_end];
        let bracket_pos = region.find('[').or_else(|| region.find('('));
        match bracket_pos {
            Some(pos) if pos > 0 => ep_title_start + pos,
            _ => ep_title_end,
        }
    };

    if ep_title_end <= ep_title_start {
        return None;
    }

    let raw = &input[ep_title_start..ep_title_end];
    let cleaned = clean_title(raw);

    if cleaned.is_empty() {
        return None;
    }

    // Reject if too short/noisy.
    let trimmed = cleaned.trim();
    if trimmed.len() <= 1 {
        return None; // Too short to be meaningful.
    }
    // Reject if it looks like a season reference.
    let lower = trimmed.to_lowercase();
    if lower.starts_with("season") || lower.starts_with("saison") || lower.starts_with("tem") {
        return None;
    }
    // Reject if it contains another episode/season match within it.
    let has_ep_in_gap = matches.iter().any(|m| {
        m.start >= ep_title_start
            && m.end <= ep_title_end
            && (m.property == Property::Episode || m.property == Property::Season)
    });
    if has_ep_in_gap {
        return None;
    }

    Some(MatchSpan::new(
        ep_title_start,
        ep_title_end,
        Property::EpisodeTitle,
        cleaned,
    ))
}

/// Infer media type from the set of matched properties.
pub fn infer_media_type(matches: &[MatchSpan]) -> &'static str {
    let has_episode = matches.iter().any(|m| m.property == Property::Episode);
    let has_season = matches.iter().any(|m| m.property == Property::Season);
    let has_date = matches.iter().any(|m| m.property == Property::Date);

    if has_episode || has_season || has_date {
        "episode"
    } else {
        "movie"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_before_year() {
        let matches = vec![MatchSpan::new(11, 15, Property::Year, "1999")];
        let title = extract_title("The.Matrix.1999.1080p.mkv", &matches).unwrap();
        assert_eq!(title.value, "The Matrix");
    }

    #[test]
    fn test_title_no_matches() {
        let title = extract_title("JustATitle.mkv", &[]).unwrap();
        assert_eq!(title.value, "JustATitle");
    }

    #[test]
    fn test_title_with_path() {
        let matches = vec![MatchSpan::new(22, 26, Property::Year, "2020")];
        let title = extract_title("/movies/dir/The.Movie.2020.mkv", &matches).unwrap();
        assert_eq!(title.value, "The Movie");
    }

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

    #[test]
    fn test_abbreviated_fallback() {
        // Abbreviated filename should fall back to parent dir.
        // The parent dir "Alice in Wonderland DVDRip.XviD-DiAMOND" has property
        // matches in it, so the title should stop at the first match.
        let matches = vec![
            // DVDRip match in parent dir portion.
            MatchSpan::new(27, 34, Property::Source, "DVD"),
        ];
        let title = extract_title(
            "Movies/Alice in Wonderland DVDRip.XviD-DiAMOND/dmd-aw.avi",
            &matches,
        );
        assert!(title.is_some());
        let t = title.unwrap();
        assert_eq!(t.value, "Alice in Wonderland");
    }

    #[test]
    fn test_infer_episode() {
        let matches = vec![
            MatchSpan::new(0, 5, Property::Season, "1"),
            MatchSpan::new(5, 10, Property::Episode, "3"),
        ];
        assert_eq!(infer_media_type(&matches), "episode");
    }

    #[test]
    fn test_infer_movie() {
        let matches = vec![MatchSpan::new(0, 4, Property::Year, "2024")];
        assert_eq!(infer_media_type(&matches), "movie");
    }
}
