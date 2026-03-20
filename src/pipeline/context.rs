//! Cross-file context: invariance detection for title extraction.
//!
//! When sibling filenames are available, the title is the **invariant text**
//! across files — the text that doesn't change between episodes. Episode
//! numbers, episode titles, and per-file metadata are the **variant** text.
//!
//! This module finds unclaimed text gaps (regions not matched by Pass 1)
//! and identifies the longest common text across all files.

use crate::matcher::span::MatchSpan;

/// Separators used in media filenames for normalization.
const SEPS: &[char] = &['.', ' ', '_', '-', '+'];

/// Brackets to strip from gap boundaries.
const TRIM_CHARS: &[char] = &[
    '(', ')', '[', ']', '{', '}', '.', ' ', '_', '-', '+', '\u{3000}',
];

/// A gap (unclaimed region) in a parsed filename.
#[derive(Debug, Clone)]
pub(crate) struct UnclaimedGap {
    /// Byte offset of the gap start in the original input.
    pub start: usize,
    /// The normalized text content of the gap (separators → spaces, trimmed).
    pub text: String,
}

/// Find unclaimed text gaps between resolved matches in an input string.
///
/// Gaps are regions of the input that no `MatchSpan` covers. These are the
/// candidate regions for title text. The extension (last `.xxx`) is excluded.
pub(crate) fn find_unclaimed_gaps(input: &str, matches: &[MatchSpan]) -> Vec<UnclaimedGap> {
    // Sort matches by start position.
    let mut sorted: Vec<(usize, usize)> = matches.iter().map(|m| (m.start, m.end)).collect();
    sorted.sort_by_key(|&(s, _)| s);

    // Find the input region to scan (exclude file extension).
    let scan_end = strip_extension_pos(input);

    let mut gaps = Vec::new();
    let mut cursor = 0;

    for (start, end) in &sorted {
        let start = *start;
        let end = (*end).min(scan_end);
        if start <= cursor {
            cursor = cursor.max(end);
            continue;
        }
        // There's a gap from cursor..start.
        if start > cursor {
            let gap_start = cursor;
            let gap_text = &input[cursor..start];
            let normalized = normalize_gap(gap_text);
            if !normalized.is_empty() {
                gaps.push(UnclaimedGap {
                    start: gap_start,
                    text: normalized,
                });
            }
        }
        cursor = cursor.max(end);
    }

    // Trailing gap after last match.
    if cursor < scan_end {
        let gap_start = cursor;
        let gap_text = &input[cursor..scan_end];
        let normalized = normalize_gap(gap_text);
        if !normalized.is_empty() {
            gaps.push(UnclaimedGap {
                start: gap_start,
                text: normalized,
            });
        }
    }

    gaps
}

/// Find the longest invariant text across multiple files' unclaimed gaps.
///
/// For each gap in the target file, finds the corresponding gap in each
/// sibling (by position) and computes the **common prefix** — the text
/// before the first divergence. Titles are always prefixes: episode numbers
/// and episode-specific text follow the title and diverge between files.
///
/// Returns `None` if no common prefix of length >= 2 chars is found.
pub(crate) fn find_invariant_text(all_gaps: &[Vec<UnclaimedGap>]) -> Option<String> {
    if all_gaps.len() < 2 {
        return None;
    }

    let target_gaps = &all_gaps[0];
    let sibling_gaps = &all_gaps[1..];

    // For each target gap, find the common prefix with the best-matching
    // gap in each sibling. "Best-matching" = same gap index (positional).
    let mut best: Option<(usize, String)> = None; // (start_pos, text)

    for (gap_idx, target_gap) in target_gaps.iter().enumerate() {
        let mut prefix = target_gap.text.clone();

        for sibling_file_gaps in sibling_gaps {
            // Find the corresponding gap in the sibling.
            // Primary: same index. Fallback: best common prefix by content.
            let sib_text = if gap_idx < sibling_file_gaps.len() {
                &sibling_file_gaps[gap_idx].text
            } else {
                // No corresponding gap — try to find the best match.
                let best_match = sibling_file_gaps
                    .iter()
                    .max_by_key(|g| common_prefix_len(&prefix, &g.text));
                match best_match {
                    Some(g) => &g.text,
                    None => {
                        prefix.clear();
                        break;
                    }
                }
            };

            // Intersect: keep only the common prefix.
            prefix = common_prefix_chars(&prefix, sib_text);
            if prefix.chars().count() < 2 {
                prefix.clear();
                break;
            }
        }

        // Trim trailing separators and CJK ordinal markers from the common prefix.
        let trimmed = trim_title_suffix(&prefix);
        if trimmed.chars().count() < 2 {
            continue;
        }

        let dominated = best.as_ref().is_some_and(|(best_start, best_text)| {
            target_gap.start > *best_start
                || (target_gap.start == *best_start && trimmed.len() <= best_text.len())
        });
        if !dominated {
            best = Some((target_gap.start, trimmed));
        }
    }

    best.map(|(_, text)| text)
}

/// Compute the common prefix of two strings (by chars).
fn common_prefix_chars(a: &str, b: &str) -> String {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .map(|(c, _)| c)
        .collect()
}

/// Compute the length (in chars) of the common prefix of two strings.
fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count()
}

/// CJK ordinal/structural characters that commonly precede episode numbers.
/// These should be trimmed from the end of a title when they're the last
/// character of a common prefix (they belong to the episode identifier,
/// not the title).
const CJK_ORDINAL_CHARS: &[char] = &[
    '第', // ordinal marker (di/dai) — 第01話
    '巻', // volume (kan/maki)
    '集', // episode/collection (shū)
    '話', // episode (wa)
    '回', // episode/round (kai)
    '編', // arc/chapter (hen)
    '章', // chapter (shō)
    '期', // season/period (ki)
    '部', // part (bu)
];

/// Trim trailing separators and CJK ordinal markers from a title prefix.
fn trim_title_suffix(text: &str) -> String {
    let mut s = text.trim_end_matches(SEPS).trim();
    loop {
        let trimmed = s.trim_end_matches(CJK_ORDINAL_CHARS);
        let trimmed = trimmed.trim_end_matches(SEPS).trim();
        if trimmed.len() == s.len() {
            break;
        }
        s = trimmed;
    }
    s.to_string()
}

/// Normalize gap text: replace separators with spaces, strip bracket-enclosed
/// regions (sub-group tags, CRC checksums, etc.), trim boundaries.
fn normalize_gap(text: &str) -> String {
    // Step 1: strip bracket-enclosed regions [xxx], (xxx), {xxx}.
    let stripped = strip_bracket_regions(text);
    // Step 2: replace separators with spaces.
    let normalized: String = stripped
        .chars()
        .map(|c| if SEPS.contains(&c) { ' ' } else { c })
        .collect();
    // Step 3: trim boundary chars and collapse whitespace.
    let trimmed = normalized.trim_matches(TRIM_CHARS);
    // Step 4: handle orphaned closing brackets from boundary splits.
    let trimmed = trim_orphaned_brackets(trimmed);
    trimmed.to_string()
}

/// Strip bracket-enclosed regions: `[...]`, `(...)`, `{...}`.
fn strip_bracket_regions(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut depth: [u32; 3] = [0; 3]; // [round, square, curly]
    for c in text.chars() {
        match c {
            '(' => depth[0] += 1,
            '[' => depth[1] += 1,
            '{' => depth[2] += 1,
            ')' if depth[0] > 0 => {
                depth[0] -= 1;
                continue;
            }
            ']' if depth[1] > 0 => {
                depth[1] -= 1;
                continue;
            }
            '}' if depth[2] > 0 => {
                depth[2] -= 1;
                continue;
            }
            _ => {}
        }
        if depth.iter().all(|&d| d == 0) {
            result.push(c);
        }
    }
    result
}

/// Trim orphaned closing brackets left over from boundary splits.
/// e.g. "SubGroup] Naruto" → "Naruto" when `[` was outside this gap.
fn trim_orphaned_brackets(text: &str) -> &str {
    const CLOSE_BRACKETS: &[char] = &[')', ']', '}'];
    let mut s = text;
    // If text starts with content then an orphaned `]`, `)`, or `}`,
    // skip past it.
    for close in CLOSE_BRACKETS {
        if let Some(pos) = s.find(*close) {
            // Only trim if the closing bracket has no matching opener before it.
            let before = &s[..pos];
            let opener = match close {
                ')' => '(',
                ']' => '[',
                '}' => '{',
                _ => unreachable!(),
            };
            if !before.contains(opener) {
                s = s[pos + close.len_utf8()..].trim_start_matches(TRIM_CHARS);
            }
        }
    }
    s
}

/// Find the byte position where the file extension starts (the last `.xxx`).
/// Returns `input.len()` if no extension is found.
fn strip_extension_pos(input: &str) -> usize {
    // Only consider extensions ≤10 chars after the last dot.
    if let Some(dot_pos) = input.rfind('.') {
        let ext = &input[dot_pos + 1..];
        if ext.len() <= 10 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
            return dot_pos;
        }
    }
    input.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::span::Property;

    fn make_match(start: usize, end: usize) -> MatchSpan {
        MatchSpan::new(start, end, Property::VideoCodec, "test")
    }

    #[test]
    fn unclaimed_gaps_basic() {
        // Input: "Hello.World.x264.mkv"
        //         0123456789...
        let input = "Hello.World.x264.mkv";
        let matches = vec![make_match(12, 16)]; // "x264" at 12..16
        let gaps = find_unclaimed_gaps(input, &matches);
        // Should find "Hello World" (before x264), nothing after (extension stripped)
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].text, "Hello World");
    }

    #[test]
    fn unclaimed_gaps_no_matches() {
        let input = "Just.A.Title.mkv";
        let gaps = find_unclaimed_gaps(input, &[]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].text, "Just A Title");
    }

    #[test]
    fn invariant_text_basic() {
        let gaps1 = vec![
            UnclaimedGap {
                start: 0,
                text: "Breaking Bad".to_string(),
            },
            UnclaimedGap {
                start: 20,
                text: "S05E16".to_string(),
            },
        ];
        let gaps2 = vec![
            UnclaimedGap {
                start: 0,
                text: "Breaking Bad".to_string(),
            },
            UnclaimedGap {
                start: 20,
                text: "S05E14".to_string(),
            },
        ];
        let result = find_invariant_text(&[gaps1, gaps2]);
        assert_eq!(result, Some("Breaking Bad".to_string()));
    }

    #[test]
    fn invariant_text_cjk() {
        let gaps1 = vec![
            UnclaimedGap {
                start: 4,
                text: "十二国記".to_string(),
            },
            UnclaimedGap {
                start: 20,
                text: "第13話".to_string(),
            },
        ];
        let gaps2 = vec![
            UnclaimedGap {
                start: 4,
                text: "十二国記".to_string(),
            },
            UnclaimedGap {
                start: 20,
                text: "第01話".to_string(),
            },
        ];
        let result = find_invariant_text(&[gaps1, gaps2]);
        assert_eq!(result, Some("十二国記".to_string()));
    }

    #[test]
    fn invariant_text_single_set_returns_none() {
        let gaps = vec![UnclaimedGap {
            start: 0,
            text: "Title".to_string(),
        }];
        assert_eq!(find_invariant_text(&[gaps]), None);
    }

    #[test]
    fn invariant_text_no_common_returns_none() {
        let gaps1 = vec![UnclaimedGap {
            start: 0,
            text: "Alpha".to_string(),
        }];
        let gaps2 = vec![UnclaimedGap {
            start: 0,
            text: "Bravo".to_string(),
        }];
        assert_eq!(find_invariant_text(&[gaps1, gaps2]), None);
    }

    #[test]
    fn invariant_prefers_earliest_gap() {
        // "Title" at position 0 should win over "GROUP" at position 30,
        // even though GROUP is longer.
        let gaps1 = vec![
            UnclaimedGap {
                start: 0,
                text: "Show".to_string(),
            },
            UnclaimedGap {
                start: 30,
                text: "GROUP".to_string(),
            },
        ];
        let gaps2 = vec![
            UnclaimedGap {
                start: 0,
                text: "Show".to_string(),
            },
            UnclaimedGap {
                start: 30,
                text: "GROUP".to_string(),
            },
        ];
        let result = find_invariant_text(&[gaps1, gaps2]);
        assert_eq!(result, Some("Show".to_string()));
    }
}
