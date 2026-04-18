//! Anime release shape: `[Group] Title - Ep [tags].mkv`.
//!
//! After one or more leading `[...]` groups, the title runs up to the
//! next structural match (typically the Episode number in the form
//! `Title - 04`). Internal `Part N` is preserved as title content when an
//! Episode match exists later in the same filename (#124 / #127).

use crate::FILENAME_SEPS as SEPS;
use crate::matcher::span::{MatchSpan, Property};

use super::super::clean::{clean_title, clean_title_preserve_dashes};
use super::super::find_title_boundary;
use super::{StrategyContext, TitleStrategy};

pub(crate) struct AfterBracketGroup;

impl TitleStrategy for AfterBracketGroup {
    fn name(&self) -> &'static str {
        "after_bracket_group"
    }

    fn try_extract(&self, ctx: &StrategyContext<'_>) -> Option<MatchSpan> {
        let StrategyContext {
            input,
            matches,
            filename_start,
        } = *ctx;
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

        // Anime bracket releases follow `[Group] <Title> - <epnum> [tags]`. When an
        // Episode match exists later in the filename, the " - <epnum>" pair is the
        // structural boundary; any `Part N` *inside* the title (e.g.
        // "San no Shou Part 2") must not pre-empt it. See issue #124.
        let has_episode_after = matches.iter().any(|m| {
            m.property == Property::Episode && m.start >= title_start_abs && m.start < filename_end
        });

        let next_match = matches
            .iter()
            .filter(|m| m.start >= title_start_abs && m.start < filename_end && !m.is_extension)
            .filter(|m| !(has_episode_after && m.property == Property::Part))
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

        // When the next match is the Episode (anime `Title - Ep` pattern),
        // the structural boundary is the " - " right before the episode number.
        // Trim trailing separators rather than letting find_title_boundary chop at
        // the *first* in-title " - " — that would lose multi-segment titles like
        // "Enen no Shouboutai - San no Shou Part 2".
        let is_anime_episode_boundary = next_match.map(|m| m.property) == Some(Property::Episode);
        let title_end_abs = if is_anime_episode_boundary {
            let trimmed = raw.trim_end_matches([' ', '.', '_', '-']);
            title_start_abs + trimmed.len()
        } else {
            find_title_boundary(raw)
                .map(|offset| title_start_abs + offset)
                .unwrap_or(title_end_abs)
        };
        let raw = &input[title_start_abs..title_end_abs];

        let cleaned = if is_anime_episode_boundary {
            clean_title_preserve_dashes(raw)
        } else {
            clean_title(raw)
        };
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
}
