//! Title extraction — positional rule ("whatever's left" after other matchers).
//!
//! Split into submodules:
//! - `clean` — string cleaning (separators, brackets, casing)
//! - `secondary` — episode_title, film_title, alternative_title, media_type

mod clean;
mod secondary;
mod strategies;

pub(crate) use clean::is_generic_dir;
pub use secondary::{
    extract_alternative_titles, extract_episode_title, extract_film_title, infer_media_type,
};

use crate::FILENAME_SEPS as SEPS;
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;
use clean::{clean_title, is_abbreviated, is_likely_extension, pick_better_casing};
use strategies::{StrategyContext, TitleStrategy};

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
    let title_end_abs = find_first_structural_separator(raw_title)
        .map(|offset| filename_start + offset)
        .unwrap_or(title_end_abs);
    let raw_title = &input[filename_start..title_end_abs];

    let cleaned = clean_title(raw_title);

    if cleaned.is_empty() {
        let ctx = StrategyContext {
            input,
            matches,
            filename_start,
        };
        if let Some(title) = strategies::run_fallback_ladder(&ctx) {
            return Some(title);
        }
        return None;
    }

    // Prefer parent dir casing when titles match case-insensitively.
    if has_parent_dir(input)
        && let Some(parent_match) = strategies::ParentDir.try_extract(&StrategyContext {
            input,
            matches,
            filename_start,
        })
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
        && let Some(parent_title) = strategies::ParentDir.try_extract(&StrategyContext {
            input,
            matches,
            filename_start,
        })
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
    // Unclaimed bracket content: when everything is in brackets and one
    // bracket group isn't claimed by any matcher, it's likely the title.
    // E.g., [DBD-Raws][4K_HDR][ready.player.one][2160P][...].mkv
    let ctx = StrategyContext {
        input,
        matches,
        filename_start,
    };
    if let Some(title) = strategies::UnclaimedBracket.try_extract(&ctx) {
        return Some(title);
    }
    strategies::ParentDir.try_extract(&ctx)
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

fn has_parent_dir(input: &str) -> bool {
    input.contains('/') || input.contains('\\')
}

/// Return the byte offset of the **first** structural separator in `raw`,
/// or `None` if the input has no separator that qualifies (or one occurs
/// inside the leading 3 bytes — too short to be a real title prefix).
///
/// "Structural separators" are the punctuation patterns release-naming
/// conventions use to split a title from its trailing metadata: `" ("`,
/// `" - "`, `"--"`, and their `_`/`.`-flanked equivalents.
///
/// # Semantics: first wins
///
/// **All current callers want this**: a parenthesized year, alt-title,
/// or `" - "` segment marks the *end* of the canonical title; everything
/// after it is metadata or a sub-title. So the function returns the
/// EARLIEST qualifying offset (`min` over per-separator `find` results).
///
/// # When NOT to use this
///
/// Some inputs legitimately contain `" - "` *inside* the title:
///
/// - Anime multi-segment releases:
///   `[Group] Show - Sub-arc Part 2 - 13 [tags].mkv` — here the first
///   `" - "` separates two title segments, NOT title from metadata.
/// - Spider-style hyphenated names that survive `normalize_separators`
///   are a separate concern (they keep the `-`, no surrounding spaces).
///
/// In those cases the caller already knows the boundary structurally
/// (e.g. an Episode `MatchSpan` after the title) and should compute the
/// trim point directly rather than asking this function. See
/// `strategies::AfterBracketGroup` for the canonical example: it skips
/// `find_first_structural_separator` on the anime-episode branch and
/// trims trailing separators by hand instead. See also #124 / #127.
pub(super) fn find_first_structural_separator(raw: &str) -> Option<usize> {
    /// Minimum length the title prefix must have for a separator to count.
    /// Guards against pathological inputs like `"a - b"` where `" - "`
    /// at offset 1 would yield an empty title.
    const MIN_TITLE_LEN: usize = 3;

    // The earliest hit across any separator wins.
    const SEPARATORS: &[&str] = &[" (", "_(", ".(", " - ", "_-_", ".-.", "--"];

    SEPARATORS
        .iter()
        .filter_map(|sep| raw.find(sep).filter(|&pos| pos >= MIN_TITLE_LEN))
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

    // ── find_first_structural_separator ──────────────────────────
    //
    // These tests pin the "first wins" semantic. They exist so that
    // anyone tempted to change `min` to `max` (or to add an EpisodeAware
    // mode without a use case) reads this comment first. See the rustdoc
    // on the function for the rationale.

    #[test]
    fn first_separator_wins_picks_earliest_offset() {
        // " - " at offset 4 wins over " (" at offset 12.
        assert_eq!(
            find_first_structural_separator("Show - Subtitle (2020)"),
            Some(4)
        );
    }

    #[test]
    fn first_separator_skips_too_short_prefix() {
        // "a - b": " - " at offset 1 < MIN_TITLE_LEN, so None.
        assert_eq!(find_first_structural_separator("a - b"), None);
        // "abc - d": offset 3 ≥ MIN_TITLE_LEN, accepted.
        assert_eq!(find_first_structural_separator("abc - d"), Some(3));
    }

    #[test]
    fn first_separator_returns_none_on_separatorless_input() {
        assert_eq!(
            find_first_structural_separator("PlainTitleNoSeparator"),
            None
        );
    }

    #[test]
    fn first_separator_caveat_anime_multi_segment() {
        // KNOWN LIMITATION (documented on the function): for anime-style
        // multi-segment titles, the FIRST " - " is INSIDE the title, not at
        // the boundary. Callers facing this case must NOT use this function.
        // This test pins the limitation so a future "fix" doesn't silently
        // break `strategies::AfterBracketGroup`'s anime-episode branch.
        let raw = "Enen no Shouboutai - San no Shou Part 2";
        assert_eq!(
            find_first_structural_separator(raw),
            Some(18),
            "function returns the first \" - \"; AfterBracketGroup must \
             bypass it on the anime-episode branch (#124 / #127)"
        );
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
        assert_eq!(infer_media_type("Show.S01E03.mkv", &matches), "episode");
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
        assert_eq!(infer_media_type("Movie.2024.mkv", &matches), "movie");
    }

    #[test]
    fn test_movie_dir_suppresses_heuristic_episode() {
        // "Movie 10" in a movie/ directory: bare number is a franchise number,
        // not an episode. Path context should win over heuristic episode.
        let matches = vec![
            MatchSpan::new(52, 56, Property::Episode, "10")
                .with_priority(crate::priority::HEURISTIC),
        ];
        assert_eq!(
            infer_media_type(
                "movie/Japanese/Detective Conan/Detective.Conan.Movie.10.mkv",
                &matches
            ),
            "movie"
        );
    }

    #[test]
    fn test_movie_dir_keeps_strong_episode() {
        // SxxExx in a movie/ directory: strong signal overrides path context.
        let matches = vec![
            MatchSpan::new(0, 6, Property::Season, "1"),
            MatchSpan::new(0, 6, Property::Episode, "3").with_priority(crate::priority::STRUCTURAL),
        ];
        assert_eq!(
            infer_media_type("movie/Show.S01E03.mkv", &matches),
            "episode"
        );
    }
}
