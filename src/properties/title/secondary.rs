//! Secondary title extractors — episode title, film title, alternative title.

use super::clean::{clean_episode_title, clean_title};
use super::find_title_boundary;
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;

/// Extract episode title: the text between the last episode/season marker
/// and the next technical property in the filename portion.
///
/// Uses resolved matches for suspicious match detection (v0.3):
/// if an `Other` match is surrounded by non-tech words, it's likely
/// title content (e.g., "Proper" in "Proper Pigs"), not metadata.
pub fn extract_episode_title(
    input: &str,
    matches: &[MatchSpan],
    _token_stream: &TokenStream,
) -> Option<MatchSpan> {
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let filename = &input[filename_start..];
    let filename_end = filename_start + filename.len();

    let has_anchor = matches.iter().any(|m| {
        m.start >= filename_start
            && matches!(
                m.property,
                Property::Episode | Property::Season | Property::Date
            )
    });
    if !has_anchor {
        return None;
    }

    // Find the last episode/season/date/episode_count marker.
    // Include EpisodeCount and SeasonCount so "14.of.21.Title" starts
    // the episode title after "21", not after "14".
    let last_ep_match = matches
        .iter()
        .filter(|m| {
            m.start >= filename_start
                && matches!(
                    m.property,
                    Property::Episode
                        | Property::Season
                        | Property::Date
                        | Property::EpisodeCount
                        | Property::SeasonCount
                )
        })
        .max_by_key(|m| m.end)?;

    let ep_title_start = last_ep_match.end;

    // Properties that should stop episode title extraction.
    // Part is intentionally excluded — episode titles often contain "Part N".
    let technical_props = [
        Property::VideoCodec,
        Property::AudioCodec,
        Property::Source,
        Property::ScreenSize,
        Property::Edition,
        Property::Language,
        Property::AudioChannels,
        Property::Container,
        Property::StreamingService,
        Property::Year,
        Property::FrameRate,
        Property::ColorDepth,
        Property::VideoProfile,
        // NOTE: Property::Other is handled below with suspicious match detection.
    ];

    // Find Other matches in the episode title zone and check if they're suspicious.
    // An Other match is "suspicious" (likely title content) if the word after it
    // is NOT a tech token. E.g., "Proper.Pigs" → "Proper" is title, not Other.
    let _other_matches_in_zone: Vec<&MatchSpan> = matches
        .iter()
        .filter(|m| {
            m.property == Property::Other && m.start >= ep_title_start && m.start < filename_end
        })
        .collect();

    // Find first stopping tech property (including non-suspicious Other matches).
    let next_tech = matches
        .iter()
        .filter(|m| {
            m.start >= ep_title_start
                && m.start < filename_end
                && (technical_props.contains(&m.property)
                    || (m.property == Property::Other && !is_suspicious_other(m, input, matches)))
        })
        .min_by_key(|m| m.start);

    let ep_title_end = match next_tech {
        Some(m) => m.start,
        None => {
            let has_container = matches
                .iter()
                .any(|m| m.property == Property::Container && m.start >= filename_start);
            if has_container {
                filename
                    .rfind('.')
                    .map(|pos| filename_start + pos)
                    .unwrap_or(filename_end)
            } else {
                filename_end
            }
        }
    };

    // Trim at opening brackets/parens (metadata, not title content).
    // But skip parens whose content starts with digits (date references like "(14-01...").
    let ep_title_end = {
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
    // Handles: "2x05 - Pure Laine - Je Me Souviens" → ep_title = "Je Me Souviens"
    let raw = split_ep_title_at_show_repeat(raw, matches);

    let cleaned = clean_episode_title(raw);
    if cleaned.is_empty() {
        return None;
    }

    // Strip trailing "Part N" from episode titles.
    // Part in the MIDDLE of a title is kept ("Harry Potter Part 2 The Quest"),
    // but trailing Part is always a separate metadata property.
    let re_trailing_part =
        regex::Regex::new(r"(?i)\s+Part\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)\s*$").unwrap();
    let cleaned = re_trailing_part.replace(&cleaned, "").trim().to_string();
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

    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

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
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

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
pub fn infer_media_type(matches: &[MatchSpan]) -> &'static str {
    let has_episode = matches.iter().any(|m| m.property == Property::Episode);
    let has_season = matches.iter().any(|m| m.property == Property::Season);
    let has_date = matches.iter().any(|m| m.property == Property::Date);
    // Bonus without Film or Year = TV series bonus (episode), not movie extra.
    // Movie extras typically have years: Moon_(2009)-x02-Making_Of
    let has_bonus_no_film = matches.iter().any(|m| m.property == Property::Bonus)
        && !matches.iter().any(|m| m.property == Property::Film)
        && !matches.iter().any(|m| m.property == Property::Year);

    if has_episode || has_season || has_date || has_bonus_no_film {
        "episode"
    } else {
        "movie"
    }
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
    // READNFO, REAL, PROPER produce Other:Proper via side effects but the
    // original text is obviously metadata, not a title word.
    if other_match.end > other_match.start && other_match.end <= input.len() {
        let original_text = input[other_match.start..other_match.end].to_lowercase();
        if matches!(
            original_text.as_str(),
            "repack" | "readnfo" | "real" | "proper" | "rerip" | "internal"
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
            let before_clean = before.replace(['.', '_'], " ").trim().to_lowercase();

            // If the text before this separator matches the show title,
            // the episode title starts after it.
            if before_clean == show_title || show_title.contains(&before_clean) {
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
