//! Title extraction — positional rule ("whatever's left" after other matchers).
//!
//! This runs as a post-processing step rather than a regex matcher. The title
//! is typically everything before the first recognized property token.

use crate::matcher::span::{MatchSpan, Property};

/// Separators used in media filenames.
const SEPS: &[char] = &[
    '.', ' ', '_', '-', '+', '(', ')', '[', ']', '{', '}',
];

/// Extract title from the input string by finding the gap before the first
/// recognized match. This is a post-processing step, not a `PropertyMatcher`.
pub fn extract_title(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
    if matches.is_empty() {
        // No properties found — the whole thing is the title (strip extension-like suffix).
        let title = strip_extension(input);
        let cleaned = clean_title(title);
        if cleaned.is_empty() {
            return None;
        }
        return Some(MatchSpan::new(0, title.len(), Property::Title, cleaned));
    }

    // Find the filename portion (after last path separator).
    let filename_start = input
        .rfind(['/', '\\'])
        .map(|i| i + 1)
        .unwrap_or(0);

    // First match in the filename portion (skip extension matches at the very end).
    let first_match_in_filename = matches
        .iter()
        .filter(|m| m.start >= filename_start)
        .filter(|m| !m.tags.contains(&"extension".to_string()))
        .min_by_key(|m| m.start);

    let title_end = match first_match_in_filename {
        Some(m) => m.start,
        None => {
            // All matches are extensions or outside filename — title is whole filename.
            let ext_start = input.rfind('.').unwrap_or(input.len());
            ext_start
        }
    };

    if title_end <= filename_start {
        return None;
    }

    let raw_title = &input[filename_start..title_end];
    let cleaned = clean_title(raw_title);
    if cleaned.is_empty() {
        return None;
    }

    Some(MatchSpan::new(
        filename_start,
        title_end,
        Property::Title,
        cleaned,
    ))
}

/// Clean up a raw title: replace separators with spaces, trim.
fn clean_title(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if SEPS.contains(&c) { ' ' } else { c })
        .collect();
    // Collapse multiple spaces and trim.
    let mut result = String::new();
    let mut prev_space = true;
    for c in cleaned.chars() {
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
        // Use span 0..0 as a synthetic match.
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
