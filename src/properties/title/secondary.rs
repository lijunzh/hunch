//! Secondary title extractors — episode title, film title, alternative title.

use super::clean::{clean_episode_title, clean_title};
use super::find_title_boundary;
use crate::matcher::span::{MatchSpan, Property};

/// Extract episode title: the text between the last episode/season marker
/// and the next technical property in the filename portion.
pub fn extract_episode_title(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
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

    let last_ep_match = matches
        .iter()
        .filter(|m| {
            m.start >= filename_start
                && matches!(
                    m.property,
                    Property::Episode | Property::Season | Property::Date
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
        Property::Other,
        Property::Language,
        Property::AudioChannels,
        Property::Container,
        Property::StreamingService,
        Property::Year,
        Property::FrameRate,
        Property::ColorDepth,
        Property::VideoProfile,
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
    let cleaned = clean_episode_title(raw);
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
pub fn extract_film_title(input: &str, matches: &[MatchSpan]) -> Option<(MatchSpan, MatchSpan)> {
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

/// Extract alternative_title from content after the title boundary.
pub fn extract_alternative_title(input: &str, matches: &[MatchSpan]) -> Option<MatchSpan> {
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
        return None;
    }

    let raw_title = &input[filename_start..title_end_abs];
    let boundary_offset = find_title_boundary(raw_title)?;

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
        return None;
    }

    let alt_raw = &raw_title[sep_end..];
    let alt_cleaned = clean_title(alt_raw);
    if alt_cleaned.is_empty() {
        return None;
    }

    Some(MatchSpan::new(
        filename_start + sep_end,
        title_end_abs,
        Property::AlternativeTitle,
        alt_cleaned,
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
