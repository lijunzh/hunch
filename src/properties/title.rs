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
        .filter(|m| !m.tags.contains(&"extension".to_string()))
        .min_by_key(|m| m.start);

    let title_end_abs = match first_match_in_filename {
        Some(m) => m.start,
        None => {
            // All matches are extensions or outside filename.
            let ext_start = filename.rfind('.').unwrap_or(filename.len());
            filename_start + ext_start
        }
    };

    if title_end_abs <= filename_start {
        // Title is empty in the filename — try parent directory.
        return extract_title_from_parent(input, matches);
    }

    let raw_title = &input[filename_start..title_end_abs];
    let cleaned = clean_title(raw_title);

    if cleaned.is_empty() {
        return extract_title_from_parent(input, matches);
    }

    // If the filename looks like a scene abbreviation (very short, no spaces/dots
    // in the cleaned result), prefer the parent directory.
    if is_abbreviated(&cleaned) && has_parent_dir(input) {
        if let Some(parent_title) = extract_title_from_parent(input, matches) {
            return Some(parent_title);
        }
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
            .filter(|m| !m.tags.contains(&"extension".to_string()))
            .filter(|m| !m.tags.contains(&"path-season".to_string()))
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
            return Some(MatchSpan::new(dir_start, title_end, Property::Title, cleaned));
        }
    }

    None
}

fn has_parent_dir(input: &str) -> bool {
    input.contains('/') || input.contains('\\')
}

/// Check if a directory name is generic (should be skipped for title).
fn is_generic_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "movies" | "movie" | "series" | "tv shows" | "tv" | "media"
        | "video" | "videos" | "downloads" | "download"
    ) || lower.starts_with("season")
      || lower.starts_with("saison")
      || lower.starts_with("temporada")
}

/// Detect if a title looks like a scene abbreviation (e.g., "dmd", "wthd", "dmd aw").
fn is_abbreviated(title: &str) -> bool {
    let words: Vec<&str> = title.split_whitespace().collect();
    // All words short and lowercase → probably abbreviated.
    words.iter().all(|w| w.len() <= 6 && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
        && title.len() <= 20
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
    // But keep non-year parens as part of the title.
    let re_paren_year =
        fancy_regex::Regex::new(r"\s*\((?:19|20)\d{2}\)\s*$").unwrap();
    if let Ok(Some(m)) = re_paren_year.find(&s) {
        s = s[..m.start()].to_string();
    }

    // Replace separators with spaces.
    let cleaned: String = s
        .chars()
        .map(|c| {
            if SEPS.contains(&c) || BRACKETS.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .collect();

    // Collapse multiple spaces and trim.
    collapse_spaces(&cleaned)
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
fn strip_extension(s: &str) -> &str {
    if let Some(dot) = s.rfind('.') {
        let ext = &s[dot + 1..];
        if ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
            return &s[..dot];
        }
    }
    s
}

/// Infer media type from the set of matched properties.
pub fn infer_media_type(matches: &[MatchSpan]) -> Option<MatchSpan> {
    let has_episode = matches.iter().any(|m| m.property == Property::Episode);
    let has_season = matches.iter().any(|m| m.property == Property::Season);

    if has_episode || has_season {
        Some(MatchSpan::new(0, 0, Property::MediaType, "episode"))
    } else {
        Some(MatchSpan::new(0, 0, Property::MediaType, "movie"))
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
        let mt = infer_media_type(&matches).unwrap();
        assert_eq!(mt.value, "episode");
    }

    #[test]
    fn test_infer_movie() {
        let matches = vec![MatchSpan::new(0, 4, Property::Year, "2024")];
        let mt = infer_media_type(&matches).unwrap();
        assert_eq!(mt.value, "movie");
    }
}
