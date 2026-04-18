//! All-bracket filenames: `[a][b][title][c][d].mkv`.
//!
//! When a filename is composed entirely of bracket groups, the bracket
//! whose content isn't claimed by any property matcher is likely the title.
//!
//! Skips the first bracket group (typically release group) and bracket
//! groups that contain only digits (likely episode numbers).

use crate::matcher::span::{MatchSpan, Property};

use super::super::clean::clean_title;
use super::{StrategyContext, TitleStrategy};

pub(crate) struct UnclaimedBracket;

impl TitleStrategy for UnclaimedBracket {
    fn name(&self) -> &'static str {
        "unclaimed_bracket"
    }

    fn try_extract(&self, ctx: &StrategyContext<'_>) -> Option<MatchSpan> {
        let StrategyContext {
            input,
            matches,
            filename_start,
        } = *ctx;
        let filename = &input[filename_start..];

        // Only applies to all-bracket filenames.
        if !filename.starts_with('[') {
            return None;
        }

        // Collect all bracket groups: (content_start_abs, content_end_abs, content).
        let mut brackets: Vec<(usize, usize, &str)> = Vec::new();
        let mut pos = 0;
        while pos < filename.len() {
            if filename[pos..].starts_with('[') {
                if let Some(close) = filename[pos..].find(']') {
                    let content = &filename[pos + 1..pos + close];
                    let abs_start = filename_start + pos + 1;
                    let abs_end = filename_start + pos + close;
                    brackets.push((abs_start, abs_end, content));
                    pos += close + 1;
                } else {
                    break;
                }
            } else {
                // Non-bracket content means this isn't an all-bracket filename.
                // Allow separators and extension at the end.
                let rest = &filename[pos..];
                if rest.starts_with(['.', ' ', '-', '_']) {
                    break; // extension area
                }
                return None;
            }
        }

        // Need at least 2 bracket groups (first is typically release group).
        if brackets.len() < 2 {
            return None;
        }

        // Find the first unclaimed bracket group. Prefer skipping the first bracket
        // (typically a release group), but allow it when no release group was
        // detected and the first bracket is the only plausible title (#100).
        let start_index = usize::from(matches.iter().any(|m| m.property == Property::ReleaseGroup));
        for &(abs_start, abs_end, content) in &brackets[start_index..] {
            if content.is_empty() || content.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            // Check if this bracket's content overlaps with any existing match.
            let is_claimed = matches.iter().any(|m| {
                !matches!(m.property, Property::ReleaseGroup | Property::Title)
                    && m.start < abs_end
                    && m.end > abs_start
            });

            if !is_claimed {
                let cleaned = clean_title(content);
                if !cleaned.is_empty() {
                    return Some(MatchSpan::new(abs_start, abs_end, Property::Title, cleaned));
                }
            }
        }

        None
    }
}
