//! CJK fansub format: `[Group][Title][Episode][Resolution]...`
//!
//! When the filename starts with consecutive bracket groups and we have an
//! Episode match, the second bracket group contains the title.

use crate::matcher::span::{MatchSpan, Property};

use super::super::clean::clean_title;
use super::{StrategyContext, TitleConfidence, TitleStrategy};

pub(crate) struct CjkBracket;

impl TitleStrategy for CjkBracket {
    fn name(&self) -> &'static str {
        "cjk_bracket"
    }

    /// `[Group][Title][Episode]` is a deliberate, structurally-marked
    /// title in the dominant fansub convention. The author placed the
    /// title inside a bracket bounded by an episode anchor — about as
    /// explicit a self-description as a filename can be.
    fn confidence(&self) -> TitleConfidence {
        TitleConfidence::Strong
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
        //
        // Season/Episode matches inside the title bracket (e.g., `S2` in
        // `[Re Zero ... Seikatsu S2]`) do NOT disqualify the bracket from
        // being the title — they're inline metadata that lives ALONGSIDE
        // the title, not a competing claim on it. Trailing `S2`/`Season N`
        // is then stripped by `clean_title` via `RE_TRAILING_SEASON`. (#244)
        let is_claimed = matches.iter().any(|m| {
            !matches!(
                m.property,
                Property::ReleaseGroup | Property::Title | Property::Season | Property::Episode
            ) && m.start < abs_end
                && m.end > abs_start
        });
        if is_claimed {
            return None;
        }

        // Run the bracket content through the standard title-cleaning
        // pipeline. This strips trailing `S2` / `Season N` / `Part N` /
        // bonus markers using the same regexes used by every other title
        // path — no parallel implementation. (#244)
        let cleaned = clean_title(content);
        if cleaned.is_empty() {
            return None;
        }

        Some(MatchSpan::new(abs_start, abs_end, Property::Title, cleaned))
    }
}
