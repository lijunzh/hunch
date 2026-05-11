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
        
        // Strip trailing season marker (e.g., " S2", " Season 2") from the title.
        // This handles cases like "[Show Name S2]" where the season is part
        // of the title bracket but should be extracted separately.
        let title_content = strip_trailing_season_marker(content);
        
        if title_content.is_empty() {
            return None;
        }

        let abs_end = filename_start + second_open + 1 + title_content.len();

        // Check if this bracket content is already claimed by a tech match.
        // Season matches inside the title bracket (e.g., "S2" in "[Show S2]")
        // should not prevent title extraction - they're part of the title.
        let is_claimed = matches.iter().any(|m| {
            !matches!(
                m.property,
                Property::ReleaseGroup | Property::Title | Property::Episode | Property::Season
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
            title_content.to_string(),
        ))
    }
}

/// Strip trailing season marker like " S2", " Season 2", etc. from title content.
/// Returns the cleaned title without the season suffix.
fn strip_trailing_season_marker(content: &str) -> &str {
    let trimmed = content.trim_end();
    
    // Try to match patterns ending with "S" + digits (e.g., "Show S2")
    if let Some(pos) = trimmed.rfind(' ') {
        let suffix = &trimmed[pos + 1..];
        
        // Match patterns like "S2", "S3", etc. (single or double digit season)
        if suffix.len() >= 2 && (suffix.starts_with('S') || suffix.starts_with('s')) {
            let num_part = &suffix[1..];
            if num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty() {
                // Found a season marker like " S2", strip it.
                return &trimmed[..pos].trim_end();
            }
        }
    }
    
    // Try to match " Season N" pattern (case-insensitive)
    // Look for "Season " followed by digits at the end
    let trimmed_lower = trimmed.to_lowercase();
    if let Some(season_pos) = trimmed_lower.rfind("season ") {
        let after_season = &trimmed[season_pos + 7..]; // skip "Season " (7 chars)
        if !after_season.is_empty() && after_season.chars().all(|c| c.is_ascii_digit()) {
            // Found " Season N" pattern, strip it including the space before "Season"
            return &trimmed[..season_pos].trim_end();
        }
    }
    
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_trailing_season_marker_s2() {
        assert_eq!(strip_trailing_season_marker("Re Zero kara Hajimeru Isekai Seikatsu S2"), "Re Zero kara Hajimeru Isekai Seikatsu");
    }

    #[test]
    fn test_strip_trailing_season_marker_no_suffix() {
        assert_eq!(strip_trailing_season_marker("Show Name"), "Show Name");
    }

    #[test]
    fn test_strip_trailing_season_marker_season_word() {
        assert_eq!(strip_trailing_season_marker("Show Name Season 2"), "Show Name");
    }

    #[test]
    fn test_strip_trailing_season_marker_s10() {
        assert_eq!(strip_trailing_season_marker("Show S10"), "Show");
    }
}
