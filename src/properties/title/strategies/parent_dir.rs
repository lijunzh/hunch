//! Last-resort fallback: walk up the parent directory tree.
//!
//! Tries each non-generic parent directory deepest-first. The title is
//! the bytes from the start of the directory name up to the first match
//! that lives inside that directory, or the whole directory if no matches
//! land there.

use crate::matcher::span::{MatchSpan, Property};

use super::super::clean::{clean_title, is_generic_dir, strip_extension};
use super::{StrategyContext, TitleStrategy};

pub(crate) struct ParentDir;

impl TitleStrategy for ParentDir {
    fn name(&self) -> &'static str {
        "parent_dir"
    }

    fn try_extract(&self, ctx: &StrategyContext<'_>) -> Option<MatchSpan> {
        let StrategyContext { input, matches, .. } = *ctx;
        let parts: Vec<&str> = input.split(['/', '\\']).collect();
        if parts.len() < 2 {
            // No path separators: treat the bare input as the title only when
            // no matches exist (otherwise the main extractor handles it).
            if matches.is_empty() {
                let stripped = strip_extension(input);
                let cleaned = clean_title(stripped);
                if !cleaned.is_empty() {
                    return Some(MatchSpan::new(0, stripped.len(), Property::Title, cleaned));
                }
            }
            return None;
        }

        // Build (start, end, name) spans for each parent directory.
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
                            return Some(MatchSpan::new(
                                after,
                                after_end,
                                Property::Title,
                                cleaned,
                            ));
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
}
