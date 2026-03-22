//! Secondary title extractors — episode title, film title, alternative title.

use super::clean::{clean_episode_title, clean_title};
use super::find_title_boundary;
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;

use std::sync::LazyLock;

/// Regex to strip trailing "Part N" from episode titles.
static RE_TRAILING_PART: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\s+Part\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)\s*$")
        .expect("RE_TRAILING_PART regex is valid")
});

/// Extract episode title: structure-aware extraction from whichever path
/// segment contains the episode/season anchor.
///
/// Instead of assuming the leaf filename always has the episode title,
/// we find the segment where the episode anchor lives and extract from
/// there. This handles organized libraries where episode metadata lives
/// in a parent directory:
///
/// ```text
/// Bones.S12E02.The.Brain.In.The.Bot.1080p-R2D2/161219_06.mkv
///   anchor segment: parent dir ──────────────┘
///   episode_title: "The Brain In The Bot" (from parent dir)
/// ```
pub fn extract_episode_title(
    input: &str,
    matches: &[MatchSpan],
    token_stream: &TokenStream,
) -> Option<MatchSpan> {
    // Build segment boundaries from the token stream.
    let segments: Vec<(usize, usize)> = token_stream
        .segments
        .iter()
        .map(|s| (s.start, s.end))
        .collect();

    // Find which segment(s) contain episode/season/date anchors.
    // Try each anchor segment, deepest first (prefer filename over parent dir).
    let mut anchor_segments: Vec<(usize, usize)> = segments
        .iter()
        .filter(|(seg_start, seg_end)| {
            matches.iter().any(|m| {
                m.start >= *seg_start
                    && m.end <= *seg_end
                    && matches!(
                        m.property,
                        Property::Episode | Property::Season | Property::Date
                    )
            })
        })
        .copied()
        .collect();

    // Deepest first: prefer closer-to-leaf segments.
    anchor_segments.sort_by(|a, b| b.0.cmp(&a.0));

    for (seg_start, seg_end) in &anchor_segments {
        if let Some(result) = extract_episode_title_in_segment(input, matches, *seg_start, *seg_end)
        {
            return Some(result);
        }
    }

    None
}

/// Core episode title extraction within a specific segment boundary.
///
/// Finds the last episode/season anchor in `[seg_start, seg_end)`, then
/// extracts the text between that anchor and the next technical property.
fn extract_episode_title_in_segment(
    input: &str,
    matches: &[MatchSpan],
    seg_start: usize,
    seg_end: usize,
) -> Option<MatchSpan> {
    // Find the last episode/season marker (preferred) or date marker
    // within this segment.
    let last_ep_season = matches
        .iter()
        .filter(|m| {
            m.start >= seg_start
                && m.end <= seg_end
                && matches!(
                    m.property,
                    Property::Episode
                        | Property::Season
                        | Property::EpisodeCount
                        | Property::SeasonCount
                )
        })
        .max_by_key(|m| m.end);

    let last_date = matches
        .iter()
        .filter(|m| m.start >= seg_start && m.end <= seg_end && m.property == Property::Date)
        .max_by_key(|m| m.end);

    // Prefer Episode/Season markers over Date when both exist.
    let last_ep_match = last_ep_season.or(last_date)?;
    let ep_title_start = last_ep_match.end;

    // Properties that stop episode title extraction.
    let technical_props = [
        Property::VideoCodec,
        Property::AudioCodec,
        Property::Source,
        Property::ScreenSize,
        Property::Edition,
        Property::Language,
        Property::SubtitleLanguage,
        Property::AudioChannels,
        Property::Container,
        Property::StreamingService,
        Property::Year,
        Property::Date,
        Property::FrameRate,
        Property::ColorDepth,
        Property::VideoProfile,
    ];

    // Find first stopping tech property (including non-suspicious Other).
    let next_tech = matches
        .iter()
        .filter(|m| {
            m.start >= ep_title_start
                && m.start < seg_end
                && (technical_props.contains(&m.property)
                    || (m.property == Property::Other && !is_suspicious_other(m, input, matches)))
        })
        .min_by_key(|m| m.start);

    let segment_text = &input[seg_start..seg_end];
    let ep_title_end = match next_tech {
        Some(m) => m.start,
        None => {
            // For segments with a trailing path separator or extension,
            // trim at the last dot (extension) or use the segment end.
            let has_container = matches
                .iter()
                .any(|m| m.property == Property::Container && m.start >= seg_start);
            if has_container {
                segment_text
                    .rfind('.')
                    .map(|pos| seg_start + pos)
                    .unwrap_or(seg_end)
            } else {
                seg_end
            }
        }
    };

    // Trim at opening brackets/parens (metadata, not title content).
    let ep_title_end = {
        if ep_title_end <= ep_title_start {
            return None;
        }
        let region = &input[ep_title_start..ep_title_end];
        let bracket_pos = region.find('[').or_else(|| {
            region.find('(').filter(|&pos| {
                let after = &region[pos + 1..];
                !after.starts_with(|c: char| c.is_ascii_digit())
            })
        });
        match bracket_pos {
            Some(pos) if pos > 0 => ep_title_start + pos,
            _ => ep_title_end,
        }
    };

    if ep_title_end <= ep_title_start {
        return None;
    }

    let raw = &input[ep_title_start..ep_title_end];

    // Split at " - " separator when the text before it matches the show title.
    let raw = split_ep_title_at_show_repeat(raw, matches);

    let cleaned = clean_episode_title(raw);
    if cleaned.is_empty() {
        return None;
    }

    let cleaned = RE_TRAILING_PART.replace(&cleaned, "").trim().to_string();
    if cleaned.is_empty() {
        return None;
    }

    let trimmed = cleaned.trim();
    if trimmed.len() <= 1 {
        return None;
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("season")
        || lower.starts_with("saison")
        || lower.starts_with("tem")
        || lower.starts_with("stagione")
    {
        return None;
    }
    // Check for episode/season markers inside the gap (would mean this
    // is not actually episode title content).
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

/// Extract film_title when a `film` marker (-fNN-) splits franchise from movie title.
pub fn extract_film_title(
    input: &str,
    matches: &[MatchSpan],
    _token_stream: &TokenStream,
) -> Option<(MatchSpan, MatchSpan)> {
    let film_match = matches.iter().find(|m| m.property == Property::Film)?;
    let _title_match = matches.iter().find(|m| m.property == Property::Title)?;

    let fn_start = crate::filename_start(input);

    if film_match.start <= fn_start {
        return None;
    }

    let film_title_raw = &input[fn_start..film_match.start];
    let film_title = clean_title(film_title_raw);
    if film_title.is_empty() {
        return None;
    }

    let after_film = film_match.end;
    let next_match_after_film = matches
        .iter()
        .filter(|m| {
            m.start > after_film
                && m.start >= fn_start
                && !m.is_extension
                && !matches!(
                    m.property,
                    Property::Title | Property::ReleaseGroup | Property::Bonus
                )
        })
        .min_by_key(|m| m.start);

    let title_end = next_match_after_film.map(|m| m.start).unwrap_or_else(|| {
        input[fn_start..]
            .rfind('.')
            .map(|p| fn_start + p)
            .unwrap_or(input.len())
    });

    if title_end <= after_film {
        return None;
    }

    let title_raw = &input[after_film..title_end];
    let title_cleaned = clean_title(title_raw);
    if title_cleaned.is_empty() {
        return None;
    }

    let title_end = find_title_boundary(&title_cleaned)
        .map(|offset| title_cleaned[..offset].trim().to_string())
        .unwrap_or(title_cleaned);

    Some((
        MatchSpan::new(fn_start, film_match.start, Property::FilmTitle, film_title),
        MatchSpan::new(
            after_film,
            title_end.len() + after_film,
            Property::Title,
            title_end,
        ),
    ))
}

/// Extract alternative titles from content after the title boundary.
///
/// When a title zone contains multiple `" - "` separated segments,
/// each segment after the first becomes a separate `AlternativeTitle` value.
/// E.g., `"Echec et Mort - Hard to Kill - Steven Seagal"` →
///   title: "Echec et Mort", alt_titles: ["Hard to Kill", "Steven Seagal"]
pub fn extract_alternative_titles(
    input: &str,
    matches: &[MatchSpan],
    _token_stream: &TokenStream,
) -> Vec<MatchSpan> {
    let filename_start = crate::filename_start(input);

    let first_match = matches
        .iter()
        .filter(|m| {
            m.start >= filename_start
                && !m.is_extension
                && !matches!(
                    m.property,
                    Property::Title
                        | Property::FilmTitle
                        | Property::AlternativeTitle
                        | Property::EpisodeTitle
                )
        })
        .min_by_key(|m| m.start);

    let filename = &input[filename_start..];
    let title_end_abs = match first_match {
        Some(m) => m.start,
        None => filename
            .rfind('.')
            .map(|p| filename_start + p)
            .unwrap_or(input.len()),
    };

    if title_end_abs <= filename_start {
        return Vec::new();
    }

    let raw_title = &input[filename_start..title_end_abs];
    let boundary_offset = match find_title_boundary(raw_title) {
        Some(offset) => offset,
        None => return Vec::new(),
    };

    let after = &raw_title[boundary_offset..];
    let sep_len =
        if after.starts_with(" - ") || after.starts_with("_-_") || after.starts_with(".-.") {
            3
        } else if after.starts_with("--")
            || after.starts_with(" (")
            || after.starts_with("_(")
            || after.starts_with(".(")
        {
            2
        } else {
            1
        };
    let sep_end = boundary_offset + sep_len;

    if sep_end >= raw_title.len() {
        return Vec::new();
    }

    let alt_raw = &raw_title[sep_end..];

    // Split on " - ", "_-_", ".-." to produce multiple alternative titles.
    let separators = [" - ", "_-_", ".-."];
    let segments = split_on_separators(alt_raw, &separators);

    let mut results = Vec::new();
    let mut offset = sep_end;
    for segment in &segments {
        let cleaned = clean_title(segment);
        if !cleaned.is_empty() {
            results.push(MatchSpan::new(
                filename_start + offset,
                filename_start + offset + segment.len(),
                Property::AlternativeTitle,
                cleaned,
            ));
        }
        // Advance past this segment and the next separator.
        offset += segment.len();
        // Skip the separator (find which one is next).
        let remaining = &raw_title[offset..];
        for sep in &separators {
            if remaining.starts_with(sep) {
                offset += sep.len();
                break;
            }
        }
    }

    results
}

/// Split a string on any of the given separators, preserving order.
fn split_on_separators<'a>(s: &'a str, separators: &[&str]) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut remaining = s;

    loop {
        // Find the earliest separator.
        let earliest = separators
            .iter()
            .filter_map(|sep| remaining.find(sep).map(|pos| (pos, *sep)))
            .min_by_key(|(pos, _)| *pos);

        match earliest {
            Some((pos, sep)) => {
                if pos > 0 {
                    result.push(&remaining[..pos]);
                }
                remaining = &remaining[pos + sep.len()..];
            }
            None => {
                if !remaining.is_empty() {
                    result.push(remaining);
                }
                break;
            }
        }
    }

    result
}

/// Infer media type from the set of matched properties.
pub fn infer_media_type(input: &str, matches: &[MatchSpan]) -> &'static str {
    // 1. Structural signals from matched properties.
    let has_season = matches.iter().any(|m| m.property == Property::Season);
    let has_date = matches.iter().any(|m| m.property == Property::Date);
    let has_episode_details = matches
        .iter()
        .any(|m| m.property == Property::EpisodeDetails);
    // Bonus without Film or Year = TV series bonus (episode), not movie extra.
    // Movie extras typically have years: Moon_(2009)-x02-Making_Of
    let has_bonus_no_film = matches.iter().any(|m| m.property == Property::Bonus)
        && !matches.iter().any(|m| m.property == Property::Film)
        && !matches.iter().any(|m| m.property == Property::Year);

    // Episode signal strength: only consider episodes above HEURISTIC priority
    // as strong evidence. Bare numbers (pri ≤ HEURISTIC) are guesses that
    // path context can override.
    let strong_episode = matches
        .iter()
        .any(|m| m.property == Property::Episode && m.priority > crate::priority::HEURISTIC);
    let weak_episode = !strong_episode && matches.iter().any(|m| m.property == Property::Episode);

    // Episode details (NCED, OP, SP, PV, CM, etc.) WITHOUT episode/season
    // markers are supplementary content — "extra", not "episode".
    // With episode/season (e.g., S01E00 Special), it's still an episode.
    if has_episode_details && !strong_episode && !weak_episode && !has_season && !has_date {
        return "extra";
    }

    // 2. Strong structural signals always win — SxxExx, "Episode 1", etc.
    if strong_episode || has_season || has_date || has_episode_details || has_bonus_no_film {
        return "episode";
    }

    // 3. Path-based context (D6: smart context overrides dumb engine).
    //    Movie directory context suppresses weak (heuristic) episode signals.
    //    Episode directory context promotes to episode even without structural markers.
    if path_hints_movie(input) {
        // Movie dir + only a heuristic episode guess → movie wins.
        // The bare number is likely a franchise number, not an episode.
        if weak_episode {
            return "movie";
        }
    }

    if path_hints_episode(input) {
        return "episode";
    }

    // 4. Weak episode signal with no path context → still episode.
    if weak_episode {
        return "episode";
    }

    "movie"
}

/// Check if the input path's directory components hint at movie content.
///
/// Recognises common media library directory conventions:
/// - `movie/`, `movies/`, `film/`, `films/`
///
/// Conservative: only unambiguous movie indicators.
fn path_hints_movie(input: &str) -> bool {
    let dir_part = match input.rfind(['/', '\\']) {
        Some(i) => &input[..i],
        None => return false,
    };
    let lower = dir_part.to_lowercase();
    lower
        .split(['/', '\\'])
        .any(|c| matches!(c, "movie" | "movies" | "film" | "films"))
}

/// Check if the input path's directory components hint at TV/episode content.
///
/// Recognises common media library directory conventions:
/// - `tv/`, `TV/`, `TV Shows/`, `Television/`
/// - `Series/`, `Anime/`
/// - Season directories: `Season 1/`, `S01/`
///
/// This is deliberately conservative — we only match well-known patterns
/// that are unambiguous evidence of episodic content.
fn path_hints_episode(input: &str) -> bool {
    // Only look at directory components (before the last separator).
    let dir_part = match input.rfind(['/', '\\']) {
        Some(i) => &input[..i],
        None => return false, // No path → no hints.
    };

    // Normalize to lowercase for case-insensitive matching.
    let lower = dir_part.to_lowercase();

    // Split into path components and check each.
    lower.split(['/', '\\']).any(is_episode_directory)
}

/// Returns true if a single directory component indicates episodic content.
fn is_episode_directory(component: &str) -> bool {
    matches!(
        component,
        "tv" | "tv shows" | "television" | "series" | "anime" | "donghua"
    ) || component.starts_with("season ")
        || component.starts_with("saison ")
        || component.starts_with("temporada ")
        || component.starts_with("stagione ")
        // S01, S02, etc. as directory names.
        // Uses strip_prefix for safe UTF-8 handling instead of byte indexing.
        || component
            .strip_prefix('s')
            .is_some_and(|rest| !rest.is_empty() && rest.len() <= 3 && rest.chars().all(|c| c.is_ascii_digit()))
}

// ── Episode title helpers ────────────────────────────────────────────────

/// Check if an `Other` match is "suspicious" — likely title content,
/// not actual metadata.
///
/// Only a curated set of Other values can be title content. Words like
/// "Proper", "Proof", "Line" are common English words that appear in titles.
/// Words like "REPACK", "Remux", "XXX" are never title content.
fn is_suspicious_other(other_match: &MatchSpan, input: &str, _matches: &[MatchSpan]) -> bool {
    // Only these Other values could plausibly be title words.
    const TITLE_AMBIGUOUS_OTHER: &[&str] = &[
        "Proper", // "Proper Pigs", "A Proper Lady"
        "Fix",    // "The Fix"
        "3D",     // "Step Up 3D"
        "HD",     // rare but possible
    ];

    if !TITLE_AMBIGUOUS_OTHER
        .iter()
        .any(|v| v.eq_ignore_ascii_case(&other_match.value))
    {
        return false;
    }

    // Check the original token text in the input. Release tags like REPACK,
    // READNFO, REAL produce Other:Proper via side effects but the
    // original text is obviously metadata, not a title word.
    // Note: "proper" is intentionally NOT in this list — we want the
    // next-word heuristic below to decide if standalone "Proper" is title
    // content (e.g., "Proper.Pigs") or metadata (e.g., "Proper.720p").
    if other_match.end > other_match.start && other_match.end <= input.len() {
        let original_text = input[other_match.start..other_match.end].to_lowercase();
        if matches!(
            original_text.as_str(),
            "repack" | "readnfo" | "real" | "rerip" | "internal"
        ) {
            return false;
        }
    }

    // Check that the next word after the match is NOT a tech token.
    let after_pos = other_match.end;
    if after_pos >= input.len() {
        return false;
    }

    let rest = &input[after_pos..];
    let next_word: String = rest
        .trim_start_matches(['.', '-', '_', ' '])
        .chars()
        .take_while(|c| c.is_alphanumeric())
        .collect();

    if next_word.is_empty() {
        return false;
    }

    // If the next word is NOT a tech token, this Other match is suspicious.
    !crate::zone_map::is_tier2_token(&next_word) && !is_tech_word(&next_word)
}

/// Quick check for common tech words that aren't in Tier 2.
fn is_tech_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    matches!(
        lower.as_str(),
        "720p" | "1080p" | "2160p" | "480p" | "hdr" | "hdr10" | "sdr"
    )
}

/// Split episode title at " - " when the text before it repeats the show title.
///
/// Pattern: `" - Show Title - Actual Episode Title"`
/// The show title appears in the parent dir or the Title match.
fn split_ep_title_at_show_repeat<'a>(raw: &'a str, matches: &[MatchSpan]) -> &'a str {
    // Get the show title for comparison.
    let show_title = matches
        .iter()
        .find(|m| m.property == Property::Title)
        .map(|m| m.value.to_lowercase());

    let show_title = match show_title {
        Some(t) => t,
        None => return raw,
    };

    // Look for " - " separators.
    let separators = [" - ", "_-_", ".-."];
    for sep in &separators {
        // Find all occurrences.
        let mut search_start = 0;
        while let Some(pos) = raw[search_start..].find(sep) {
            let abs_pos = search_start + pos;
            let before = raw[..abs_pos].trim();
            let before_clean = before
                .replace(['.', '_'], " ")
                .trim()
                .trim_start_matches(['-', ' '])
                .trim()
                .to_lowercase();

            // If the text before this separator matches the show title,
            // the episode title starts after it.
            // Guard: skip empty before_clean — an empty string is trivially
            // "contained" in every string, producing false positives.
            if !before_clean.is_empty()
                && (before_clean == show_title || show_title.contains(&before_clean))
            {
                let after = &raw[abs_pos + sep.len()..];
                // Look for another " - " after this one (nested separators).
                if let Some(next_pos) = after.find(sep) {
                    // Return the part after the LAST separator that follows the title.
                    return &raw[abs_pos + sep.len() + next_pos + sep.len()..];
                }
                return &raw[abs_pos + sep.len()..];
            }
            search_start = abs_pos + sep.len();
        }
    }

    raw
}
