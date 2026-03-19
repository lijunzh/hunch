//! Title extraction — positional rule ("whatever's left" after other matchers).
//!
//! Split into submodules:
//! - `clean` — string cleaning (separators, brackets, casing)
//! - `secondary` — episode_title, film_title, alternative_title, media_type

mod clean;
mod secondary;

pub use secondary::{
    extract_alternative_titles, extract_episode_title, extract_film_title, infer_media_type,
};

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;
use clean::{
    clean_title, is_abbreviated, is_generic_dir, is_likely_extension, pick_better_casing,
    strip_extension,
};

/// Separators used in media filenames.
const SEPS: &[char] = &['.', ' ', '_', '-', '+'];

/// Characters we strip from title boundaries.
const BRACKETS: &[char] = &['(', ')', '[', ']', '{', '}'];

/// Whether a property is a technical metadata property (not a title word).
fn is_tech_property(p: Property) -> bool {
    matches!(
        p,
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
            | Property::Edition
            | Property::Other
    )
}

/// Extract title from the input string by finding the gap before the first
/// recognized match. This is a post-processing step, not a `PropertyMatcher`.
///
/// The `zone_map` is used for year-as-title disambiguation (e.g., "2001" in
/// "2001.A.Space.Odyssey.1968" is a title word, not the release year).
///
/// Reclaimable matches (marked by TOML `requires_nearby`) are transparent
/// to the title boundary: they don't stop the title, and if absorbed into
/// the title span they are removed from `matches`.
pub fn extract_title(
    input: &str,
    matches: &[MatchSpan],
    zone_map: &ZoneMap,
    _token_stream: &TokenStream,
) -> Option<MatchSpan> {
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];

    // Title boundary: first non-extension match in the filename.
    // Reclaimable matches are skipped ONLY if there's title content before
    // them (e.g., "Pacific.Rim.3D" → skip 3D, absorb into title).
    // If a reclaimable match starts at the filename beginning, it's treated
    // normally (e.g., "3D.2019" → 3D is Other, not title content).
    let first_match_in_filename = matches
        .iter()
        .filter(|m| {
            m.start >= filename_start
                && !m.is_extension
                && (!m.reclaimable || m.start == filename_start)
        })
        .min_by_key(|m| m.start);

    let title_end_abs = match first_match_in_filename {
        Some(m) => m.start,
        None => {
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
        return handle_empty_title(
            input,
            filename_start,
            filename,
            matches,
            zone_map,
            first_match_in_filename,
        );
    }

    let raw_title = &input[filename_start..title_end_abs];

    // Truncate at structural separators (" - ", "--", "(").
    let title_end_abs = find_title_boundary(raw_title)
        .map(|offset| filename_start + offset)
        .unwrap_or(title_end_abs);
    let raw_title = &input[filename_start..title_end_abs];

    let cleaned = clean_title(raw_title);

    if cleaned.is_empty() {
        if let Some(title) = extract_after_bracket_group(input, matches, filename_start) {
            return Some(title);
        }
        return extract_title_from_parent(input, matches);
    }

    // Prefer parent dir casing when titles match case-insensitively.
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

    // Abbreviated filenames fall back to parent directory.
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

/// Remove reclaimable matches that fall within the title span.
///
/// Called after title extraction. Any reclaimable match whose byte range
/// overlaps with the title is considered absorbed into the title.
pub fn absorb_reclaimable(title: &MatchSpan, matches: &mut Vec<MatchSpan>) {
    matches.retain(|m| {
        if !m.reclaimable {
            return true;
        }
        // Drop if this match falls within the title span.
        !(m.start >= title.start && m.end <= title.end)
    });
}

/// Handle the case where title_end_abs <= filename_start (empty title zone).
fn handle_empty_title(
    input: &str,
    filename_start: usize,
    filename: &str,
    matches: &[MatchSpan],
    zone_map: &ZoneMap,
    first_match_in_filename: Option<&MatchSpan>,
) -> Option<MatchSpan> {
    // Year-as-title via ZoneMap: e.g., "2001" in "2001.A.Space.Odyssey.1968".
    if let Some(ref yi) = zone_map.year
        && let Some(ty) = yi.title_years.iter().find(|ty| ty.start == filename_start)
        && let Some(title) =
            extract_title_after_position(input, ty.end, filename_start, filename, matches)
    {
        return Some(title);
    }
    // Fallback: first match is a Year at filename start.
    if let Some(first_m) = first_match_in_filename
        && first_m.property == Property::Year
        && first_m.start == filename_start
        && let Some(title) =
            extract_title_after_position(input, first_m.end, filename_start, filename, matches)
    {
        return Some(title);
    }
    // Leading tech tokens at filename start (e.g., "h265 - HEVC Riddick...").
    // Skip past all contiguous tech matches at the start to find the title gap.
    if let Some(first_m) = first_match_in_filename
        && first_m.start == filename_start
        && is_tech_property(first_m.property)
    {
        // Find the end of the last contiguous tech match at the start.
        let mut skip_end = first_m.end;
        loop {
            let next = matches.iter().find(|m| {
                m.start >= skip_end
                    && m.start <= skip_end + 3 // allow small separator gap
                    && m.start < filename_start + filename.len()
                    && !m.is_extension
                    && is_tech_property(m.property)
            });
            match next {
                Some(m) => skip_end = m.end,
                None => break,
            }
        }
        if let Some(title) =
            extract_title_after_position(input, skip_end, filename_start, filename, matches)
        {
            return Some(title);
        }
    }
    // Single short word with no path/extension → treat as title.
    if !input.contains(['/', '\\']) && !input.contains('.') && input.len() <= 10 {
        let cleaned = clean_title(input);
        if !cleaned.is_empty() {
            return Some(MatchSpan::new(0, input.len(), Property::Title, cleaned));
        }
    }
    extract_title_from_parent(input, matches)
}

/// Extract title from position `start` to the next match in the filename.
fn extract_title_after_position(
    input: &str,
    start: usize,
    filename_start: usize,
    filename: &str,
    matches: &[MatchSpan],
) -> Option<MatchSpan> {
    let next_match = matches
        .iter()
        .filter(|m| m.start > start && !m.is_extension)
        .min_by_key(|m| m.start);
    let title_end = next_match
        .map(|m| m.start)
        .unwrap_or(filename_start + filename.len());
    if title_end > start {
        let raw = &input[start..title_end];
        let cleaned = clean_title(raw);
        if !cleaned.is_empty() {
            return Some(MatchSpan::new(start, title_end, Property::Title, cleaned));
        }
    }
    None
}

/// Try to extract the title from the parent directory name.
fn extract_title_from_parent(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
    let parts: Vec<&str> = input.split(['/', '\\']).collect();
    if parts.len() < 2 {
        if matches.is_empty() {
            let stripped = strip_extension(input);
            let cleaned = clean_title(stripped);
            if !cleaned.is_empty() {
                return Some(MatchSpan::new(0, stripped.len(), Property::Title, cleaned));
            }
        }
        return None;
    }

    let mut offset = 0;
    let mut dir_spans: Vec<(usize, usize, &str)> = Vec::new();
    #[allow(clippy::needless_range_loop)]
    for i in 0..parts.len() - 1 {
        let dir_name = parts[i];
        let dir_start = offset;
        let dir_end = dir_start + dir_name.len();
        offset = dir_end + 1;
        dir_spans.push((dir_start, dir_end, dir_name));
    }

    // Iterate deepest-first.
    for &(dir_start, dir_end, dir_name) in dir_spans.iter().rev() {
        if dir_name.is_empty() || is_generic_dir(dir_name) {
            continue;
        }

        let first_match_in_dir = matches
            .iter()
            .filter(|m| m.start >= dir_start && m.start < dir_end)
            .filter(|m| !m.is_extension && !m.is_path_based)
            .min_by_key(|m| m.start);

        let title_end = first_match_in_dir.map(|m| m.start).unwrap_or(dir_end);
        if title_end <= dir_start {
            // Directory starts with a match (e.g., "S02 Some Series").
            // Try extracting title from content AFTER the first match.
            if let Some(first_m) = first_match_in_dir {
                let after = first_m.end;
                let next_match = matches
                    .iter()
                    .filter(|m| m.start > after && m.start < dir_end && !m.is_extension)
                    .min_by_key(|m| m.start);
                let after_end = next_match.map(|m| m.start).unwrap_or(dir_end);
                if after_end > after {
                    let raw = &input[after..after_end];
                    let cleaned = clean_title(raw);
                    if !cleaned.is_empty() {
                        return Some(MatchSpan::new(after, after_end, Property::Title, cleaned));
                    }
                }
            }
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

/// For anime-style: `[Group] Title - 04 [480p]`.
fn extract_after_bracket_group(
    input: &str,
    matches: &[MatchSpan],
    filename_start: usize,
) -> Option<MatchSpan> {
    let filename = &input[filename_start..];
    let filename_end = filename_start + filename.len();

    let mut pos = 0;
    while pos < filename.len() && filename[pos..].starts_with('[') {
        if let Some(close) = filename[pos..].find(']') {
            pos += close + 1;
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

    let next_match = matches
        .iter()
        .filter(|m| m.start >= title_start_abs && m.start < filename_end && !m.is_extension)
        .min_by_key(|m| m.start);

    let title_end_abs = match next_match {
        Some(m) => m.start,
        None => {
            let has_ext = matches
                .iter()
                .any(|m| m.property == Property::Container && m.start >= filename_start);
            if has_ext {
                filename
                    .rfind('.')
                    .map(|dot| filename_start + dot)
                    .unwrap_or(filename_end)
            } else {
                filename_end
            }
        }
    };

    if title_end_abs <= title_start_abs {
        return None;
    }

    let raw = &input[title_start_abs..title_end_abs];

    // Apply structural boundary detection (" - ", "--", "(").
    let title_end_abs = find_title_boundary(raw)
        .map(|offset| title_start_abs + offset)
        .unwrap_or(title_end_abs);
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

/// Find the first structural separator in a raw title span.
///
/// Returns the byte offset within `raw` where the title should be truncated.
fn find_title_boundary(raw: &str) -> Option<usize> {
    let min_title_len = 3;

    // Find the earliest structural separator across all types.
    let separators: &[&str] = &[" (", "_(", ".(", " - ", "_-_", ".-.", "--"];

    separators
        .iter()
        .filter_map(|sep| raw.find(sep).filter(|&pos| pos >= min_title_len))
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;
    use crate::zone_map;

    fn test_zone_map(input: &str) -> ZoneMap {
        let ts = tokenizer::tokenize(input);
        zone_map::build_zone_map(input, &ts)
    }

    fn test_ts(input: &str) -> tokenizer::TokenStream {
        tokenizer::tokenize(input)
    }

    #[test]
    fn test_title_before_year() {
        let input = "The.Matrix.1999.1080p.mkv";
        let matches = vec![MatchSpan::new(11, 15, Property::Year, "1999")];
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &matches, &zm, &ts).unwrap();
        assert_eq!(title.value, "The Matrix");
    }

    #[test]
    fn test_title_no_matches() {
        let input = "JustATitle.mkv";
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &[], &zm, &ts).unwrap();
        assert_eq!(title.value, "JustATitle");
    }

    #[test]
    fn test_title_with_path() {
        let input = "/movies/dir/The.Movie.2020.mkv";
        let matches = vec![MatchSpan::new(22, 26, Property::Year, "2020")];
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &matches, &zm, &ts).unwrap();
        assert_eq!(title.value, "The Movie");
    }

    #[test]
    fn test_abbreviated_fallback() {
        let input = "Movies/Alice in Wonderland DVDRip.XviD-DiAMOND/dmd-aw.avi";
        let matches = vec![MatchSpan::new(27, 34, Property::Source, "DVD")];
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &matches, &zm, &ts);
        assert!(title.is_some());
        assert_eq!(title.unwrap().value, "Alice in Wonderland");
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
    fn test_reclaimable_absorbed_into_title() {
        let input = "Harold.And.Kumar.3D.Christmas.mkv";
        let reclaimable_3d = MatchSpan::new(17, 19, Property::Other, "3D").as_reclaimable();
        let mut matches = vec![reclaimable_3d];
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &matches, &zm, &ts).unwrap();
        assert_eq!(title.value, "Harold And Kumar 3D Christmas");
        // Absorb should remove the reclaimable match.
        absorb_reclaimable(&title, &mut matches);
        assert!(matches.is_empty(), "reclaimable 3D should be absorbed");
    }

    #[test]
    fn test_confident_3d_stops_title() {
        // When 3D is NOT reclaimable (confident), it sets the title boundary.
        let input = "Pacific.Rim.3D.2013.BluRay.mkv";
        let confident_3d = MatchSpan::new(12, 14, Property::Other, "3D");
        let year = MatchSpan::new(15, 19, Property::Year, "2013");
        let matches = vec![confident_3d, year];
        let zm = test_zone_map(input);
        let ts = test_ts(input);
        let title = extract_title(input, &matches, &zm, &ts).unwrap();
        assert_eq!(title.value, "Pacific Rim");
    }

    #[test]
    fn test_infer_movie() {
        let matches = vec![MatchSpan::new(0, 4, Property::Year, "2024")];
        assert_eq!(infer_media_type(&matches), "movie");
    }
}
