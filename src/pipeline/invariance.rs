//! Cross-file invariance analysis for year and episode disambiguation.
//!
//! Extends the basic title invariance detection in `context.rs` with
//! number classification: year-like numbers that stay the same across
//! siblings are title content, numbers that vary are metadata (episodes,
//! years). Sequential variant numbers provide episode evidence.
//!
//! See `InvarianceReport` for the unified result, and docs/plan-52-53.md
//! for the full design.

use std::sync::LazyLock;

use crate::matcher::span::MatchSpan;
use super::context::{find_invariant_text, find_unclaimed_gaps, strip_extension_pos, UnclaimedGap, SEPS, TRIM_CHARS};

// ── InvarianceReport: unified cross-file analysis ─────────────────────────
// Phase 1 of #52/#53: structs + analysis logic.
// Wired into the pipeline in Phase 2.

/// Unified cross-file analysis result.
///
/// Produced by [`analyze_invariance`] from a target file + siblings.
/// Each field is independently optional — partial results are fine.
#[derive(Debug, Clone, Default)]
pub(crate) struct InvarianceReport {
    /// Invariant title text (existing functionality from `find_invariant_text`).
    pub title: Option<String>,
    /// Year signals: which year-like numbers are invariant (title) vs variant.
    pub year_signals: Vec<YearSignal>,
    /// Episode signals: which bare numbers form sequences across siblings.
    pub episode_signals: Vec<EpisodeSignal>,
}

/// A year-like number classified by cross-file analysis.
#[derive(Debug, Clone)]
pub(crate) struct YearSignal {
    /// Byte offset in the target file.
    pub start: usize,
    /// Byte offset end (exclusive) in the target file.
    pub end: usize,
    /// The 4-digit value.
    pub value: u32,
    /// `true` if this number is the same across all siblings at this position.
    /// Invariant → title content. Variant → release year or other metadata.
    pub is_invariant: bool,
}

/// A bare number classified by cross-file analysis.
#[derive(Debug, Clone)]
pub(crate) struct EpisodeSignal {
    /// Byte offset in the target file.
    pub start: usize,
    /// Byte offset end (exclusive) in the target file.
    pub end: usize,
    /// The numeric value in the target file.
    pub value: u32,
    /// Whether sibling values at this position form a sequence (e.g., 3,4,5).
    pub is_sequential: bool,
    /// Number of digits (helps distinguish 3-digit decomposition from absolute).
    pub digit_count: usize,
}

/// A bare number found in an unclaimed gap.
#[derive(Debug, Clone)]
struct NumberInGap {
    /// Byte offset in the original input string.
    start: usize,
    /// Byte offset end (exclusive).
    end: usize,
    /// The numeric value.
    value: u32,
    /// Number of digits in the original string.
    digit_count: usize,
    /// Which gap index this number was found in.
    gap_idx: usize,
    /// Position within the gap (for alignment across siblings).
    idx_within_gap: usize,
}

/// Input bundle for a single file's Pass 1 results.
pub(crate) struct FileAnalysis<'a> {
    /// The raw input string.
    pub input: &'a str,
    /// Resolved matches from Pass 1.
    pub matches: &'a [MatchSpan],
}

/// Regex for finding bare numbers in gap text.
static GAP_NUMBER: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\d+").unwrap());

/// Perform unified cross-file invariance analysis.
///
/// Analyzes the target file against siblings to produce:
/// - Title (invariant text)
/// - Year signals (invariant vs variant year-like numbers)
/// - Episode signals (sequential variant numbers)
///
/// Falls back gracefully: if no siblings, returns an empty report.
pub(crate) fn analyze_invariance(
    target: &FileAnalysis<'_>,
    siblings: &[FileAnalysis<'_>],
) -> InvarianceReport {
    if siblings.is_empty() {
        return InvarianceReport::default();
    }

    // 1. Compute unclaimed gaps for all files.
    let target_gaps = find_unclaimed_gaps(target.input, target.matches);
    let sibling_gaps: Vec<Vec<UnclaimedGap>> = siblings
        .iter()
        .map(|s| find_unclaimed_gaps(s.input, s.matches))
        .collect();

    // 2. Title: invariant text (existing algorithm).
    let mut all_gaps = vec![target_gaps.clone()];
    all_gaps.extend(sibling_gaps.clone());
    let title = find_invariant_text(&all_gaps);

    // 3. Extract bare numbers from unclaimed gaps for each file.
    let target_numbers = extract_numbers_from_gaps(target.input, &target_gaps);
    let sibling_numbers: Vec<Vec<NumberInGap>> = siblings
        .iter()
        .zip(&sibling_gaps)
        .map(|(s, gaps)| extract_numbers_from_gaps(s.input, gaps))
        .collect();

    // 4. Classify numbers by cross-file comparison.
    let year_signals = classify_year_signals(&target_numbers, &sibling_numbers);
    let episode_signals = classify_episode_signals(&target_numbers, &sibling_numbers);

    InvarianceReport {
        title,
        year_signals,
        episode_signals,
    }
}

/// Extract bare numbers from unclaimed gaps in an input string.
fn extract_numbers_from_gaps(input: &str, gaps: &[UnclaimedGap]) -> Vec<NumberInGap> {
    let mut numbers = Vec::new();

    for (gap_idx, gap) in gaps.iter().enumerate() {
        let gap_end = find_gap_end_in_input(input, gap);
        let gap_slice = &input[gap.start..gap_end];

        let mut idx_within_gap = 0;
        for m in GAP_NUMBER.find_iter(gap_slice) {
            let abs_start = gap.start + m.start();
            let abs_end = gap.start + m.end();
            let digit_str = m.as_str();

            // Skip codec-like numbers.
            if digit_str == "264" || digit_str == "265" || digit_str == "128" {
                continue;
            }

            if let Ok(value) = digit_str.parse::<u32>() {
                numbers.push(NumberInGap {
                    start: abs_start,
                    end: abs_end,
                    value,
                    digit_count: digit_str.len(),
                    gap_idx,
                    idx_within_gap,
                });
                idx_within_gap += 1;
            }
        }
    }

    numbers
}

/// Find the end byte offset of a gap in the original input.
fn find_gap_end_in_input(input: &str, gap: &UnclaimedGap) -> usize {
    let scan_end = strip_extension_pos(input);
    let mut pos = gap.start;
    let mut content_chars = 0;
    let target_chars = gap.text.chars().filter(|c| !c.is_whitespace()).count();

    for ch in input[gap.start..scan_end].chars() {
        pos += ch.len_utf8();
        if !SEPS.contains(&ch) && !TRIM_CHARS.contains(&ch) {
            content_chars += 1;
        }
        if content_chars >= target_chars {
            break;
        }
    }
    pos
}

/// Classify year-like numbers as invariant (title) or variant (metadata).
fn classify_year_signals(
    target_numbers: &[NumberInGap],
    sibling_numbers: &[Vec<NumberInGap>],
) -> Vec<YearSignal> {
    let mut signals = Vec::new();

    for tn in target_numbers {
        if tn.digit_count != 4 || !(1920..=2039).contains(&tn.value) {
            continue;
        }

        let mut all_same = true;
        let mut found_in_all = true;

        for sib_nums in sibling_numbers {
            let aligned = sib_nums.iter().find(|sn| {
                sn.gap_idx == tn.gap_idx && sn.idx_within_gap == tn.idx_within_gap
            });
            match aligned {
                Some(sn) => {
                    if sn.value != tn.value {
                        all_same = false;
                    }
                }
                None => {
                    found_in_all = false;
                    break;
                }
            }
        }

        if found_in_all {
            signals.push(YearSignal {
                start: tn.start,
                end: tn.end,
                value: tn.value,
                is_invariant: all_same,
            });
        }
    }

    signals
}

/// Classify bare numbers as episode signals based on cross-file patterns.
fn classify_episode_signals(
    target_numbers: &[NumberInGap],
    sibling_numbers: &[Vec<NumberInGap>],
) -> Vec<EpisodeSignal> {
    let mut signals = Vec::new();

    for tn in target_numbers {
        if tn.digit_count == 4 && (1920..=2039).contains(&tn.value) {
            continue;
        }

        let mut values: Vec<u32> = vec![tn.value];
        let mut found_in_all = true;

        for sib_nums in sibling_numbers {
            let aligned = sib_nums.iter().find(|sn| {
                sn.gap_idx == tn.gap_idx && sn.idx_within_gap == tn.idx_within_gap
            });
            match aligned {
                Some(sn) => values.push(sn.value),
                None => {
                    found_in_all = false;
                    break;
                }
            }
        }

        if !found_in_all {
            continue;
        }

        let all_same = values.iter().all(|v| *v == values[0]);
        if all_same {
            continue;
        }

        let is_sequential = is_sequential_set(&values);

        signals.push(EpisodeSignal {
            start: tn.start,
            end: tn.end,
            value: tn.value,
            is_sequential,
            digit_count: tn.digit_count,
        });
    }

    signals
}

/// Check if a set of values forms a sequential pattern.
fn is_sequential_set(values: &[u32]) -> bool {
    if values.len() < 2 {
        return false;
    }
    let mut sorted: Vec<u32> = values.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.len() < 2 {
        return false;
    }
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    (max - min + 1) as usize == sorted.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::span::Property;

    fn make_match_at(start: usize, end: usize, property: Property, value: &str) -> MatchSpan {
        MatchSpan::new(start, end, property, value)
    }

    #[test]
    fn is_sequential_basic() {
        assert!(is_sequential_set(&[3, 4, 5]));
        assert!(is_sequential_set(&[1, 2]));
        assert!(is_sequential_set(&[501, 502, 503]));
    }

    #[test]
    fn is_sequential_out_of_order() {
        assert!(is_sequential_set(&[5, 3, 4]));
    }

    #[test]
    fn is_sequential_gaps_not_sequential() {
        assert!(!is_sequential_set(&[1, 3, 5]));
        assert!(!is_sequential_set(&[1, 10]));
    }

    #[test]
    fn is_sequential_single_value() {
        assert!(!is_sequential_set(&[5]));
    }

    #[test]
    fn is_sequential_all_same() {
        assert!(!is_sequential_set(&[5, 5, 5]));
    }

    #[test]
    fn year_invariant_detected() {
        let target_input = "2001.A.Space.Odyssey.1080p.mkv";
        let sib_input = "2001.A.Space.Odyssey.720p.mkv";

        let target_matches = vec![
            make_match_at(21, 26, Property::ScreenSize, "1080p"),
        ];
        let sib_matches = vec![
            make_match_at(21, 25, Property::ScreenSize, "720p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target_input, matches: &target_matches },
            &[FileAnalysis { input: sib_input, matches: &sib_matches }],
        );

        let year_2001: Vec<_> = report.year_signals.iter()
            .filter(|y| y.value == 2001)
            .collect();
        assert!(!year_2001.is_empty(), "should detect 2001 as year signal");
        assert!(year_2001[0].is_invariant, "2001 should be invariant (title content)");
    }

    #[test]
    fn year_variant_detected() {
        let target_input = "Movie.2023.1080p.mkv";
        let sib_input = "Movie.2024.1080p.mkv";

        let target_matches = vec![
            make_match_at(11, 16, Property::ScreenSize, "1080p"),
        ];
        let sib_matches = vec![
            make_match_at(11, 16, Property::ScreenSize, "1080p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target_input, matches: &target_matches },
            &[FileAnalysis { input: sib_input, matches: &sib_matches }],
        );

        let year_signals: Vec<_> = report.year_signals.iter()
            .filter(|y| (2023..=2024).contains(&y.value))
            .collect();
        assert!(!year_signals.is_empty(), "should detect year signal");
        assert!(!year_signals[0].is_invariant, "year should be variant (metadata)");
    }

    #[test]
    fn episode_sequential_detected() {
        let target = "Show.03.720p.mkv";
        let sib = "Show.04.720p.mkv";

        let target_matches = vec![
            make_match_at(9, 13, Property::ScreenSize, "720p"),
        ];
        let sib_matches = vec![
            make_match_at(9, 13, Property::ScreenSize, "720p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target, matches: &target_matches },
            &[FileAnalysis { input: sib, matches: &sib_matches }],
        );

        assert!(!report.episode_signals.is_empty(), "should detect episode signal");
        let ep = &report.episode_signals[0];
        assert_eq!(ep.value, 3);
        assert!(ep.is_sequential, "episodes should be sequential");
        assert_eq!(ep.digit_count, 2);
    }

    #[test]
    fn episode_three_digit_sequential() {
        let target = "Show.501.720p.mkv";
        let sib1 = "Show.502.720p.mkv";
        let sib2 = "Show.503.720p.mkv";

        let target_matches = vec![
            make_match_at(9, 13, Property::ScreenSize, "720p"),
        ];
        let sib1_matches = vec![
            make_match_at(9, 13, Property::ScreenSize, "720p"),
        ];
        let sib2_matches = vec![
            make_match_at(9, 13, Property::ScreenSize, "720p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target, matches: &target_matches },
            &[
                FileAnalysis { input: sib1, matches: &sib1_matches },
                FileAnalysis { input: sib2, matches: &sib2_matches },
            ],
        );

        assert!(!report.episode_signals.is_empty(), "should detect 3-digit episode");
        let ep = &report.episode_signals[0];
        assert_eq!(ep.value, 501);
        assert!(ep.is_sequential);
        assert_eq!(ep.digit_count, 3);
    }

    #[test]
    fn no_siblings_empty_report() {
        let target = "Movie.2024.1080p.mkv";
        let matches = vec![
            make_match_at(11, 16, Property::ScreenSize, "1080p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target, matches: &matches },
            &[],
        );

        assert!(report.title.is_none());
        assert!(report.year_signals.is_empty());
        assert!(report.episode_signals.is_empty());
    }

    #[test]
    fn invariant_number_not_episode() {
        let target = "Show.42.720p.mkv";
        let sib = "Show.42.1080p.mkv";

        let target_matches = vec![
            make_match_at(8, 12, Property::ScreenSize, "720p"),
        ];
        let sib_matches = vec![
            make_match_at(8, 13, Property::ScreenSize, "1080p"),
        ];

        let report = analyze_invariance(
            &FileAnalysis { input: target, matches: &target_matches },
            &[FileAnalysis { input: sib, matches: &sib_matches }],
        );

        assert!(report.episode_signals.is_empty(),
            "invariant number should not produce episode signal, got: {:?}",
            report.episode_signals);
    }
}
