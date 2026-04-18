//! CJK fansub format: `[Group][Title][Episode][Resolution]...`
//!
//! When the filename starts with consecutive bracket groups and we have an
//! Episode match, the second bracket group contains the title.

use crate::matcher::span::{MatchSpan, Property};

use super::{StrategyContext, TitleStrategy};

pub(crate) struct CjkBracket;

impl TitleStrategy for CjkBracket {
    fn name(&self) -> &'static str {
        "cjk_bracket"
    }

    fn try_extract(&self, ctx: &StrategyContext<'_>) -> Option<MatchSpan> {
        let StrategyContext {
            input,
            matches,
            filename_start,
        } = *ctx;
        let filename = &input[filename_start..];

        // Must start with a bracket group.
        if !filename.starts_with('[') {
            return None;
        }

        // Must have an episode match (CJK bracket episodes have been detected).
        let has_episode = matches.iter().any(|m| m.property == Property::Episode);
        if !has_episode {
            return None;
        }

        // Find the first bracket group (release group).
        let first_close = filename.find(']')?;

        // The second bracket group should immediately follow.
        let rest = &filename[first_close + 1..];
        if !rest.starts_with('[') {
            return None;
        }

        let second_open = first_close + 1;
        let second_close = rest.find(']')?;
        let content = &rest[1..second_close];

        // The content should not be a pure number (that's an episode)
        // and should not be a known tech token.
        if content.is_empty() || content.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        let abs_start = filename_start + second_open + 1;
        let abs_end = filename_start + second_open + 1 + content.len();

        // Check if this bracket content is already claimed by a tech match.
        let is_claimed = matches.iter().any(|m| {
            !matches!(
                m.property,
                Property::ReleaseGroup | Property::Title | Property::Episode
            ) && m.start < abs_end
                && m.end > abs_start
        });
        if is_claimed {
            return None;
        }

        Some(MatchSpan::new(
            abs_start,
            abs_end,
            Property::Title,
            content.to_string(),
        ))
    }
}
