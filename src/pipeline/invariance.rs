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
    let mut year_signals = classify_year_signals(&target_numbers, &sibling_numbers);
    let mut episode_signals = classify_episode_signals(&target_numbers, &sibling_numbers);

    // 5. Also check Year matches claimed by Pass 1 — they're not in unclaimed
    //    gaps, so classify_year_signals misses them. If a Year match has the
    //    same value across all siblings, it's invariant (title content).
    let claimed_year_signals = classify_claimed_year_signals(target, siblings);
    year_signals.extend(claimed_year_signals);

    // 5b. Also check Season+Episode from digit decomposition.
    //     Pass 1 may decompose "501" → S5E01, claiming the span.
    //     If raw values (501, 502, 503) form a sequence across siblings,
    //     create an episode signal with the raw (undecomposed) value.
    let claimed_ep_signals = classify_claimed_decomposed_episodes(target, siblings);
    episode_signals.extend(claimed_ep_signals);

    // 6. Expand title to include adjacent invariant years.
    //    E.g., "2001.A.Space.Odyssey" → title="A Space Odyssey", but "2001" is
    //    an invariant year at position 0..4. If it's immediately before the
    //    title position in the input, prepend it.
    let title = expand_title_with_invariant_years(target.input, title, &year_signals);

    InvarianceReport {
        title,
        year_signals,
        episode_signals,
    }
}

/// Expand the title to include adjacent invariant year-like numbers.
///
/// When a year like "2001" is invariant (title content), it may have been
/// claimed by Pass 1's year matcher and thus excluded from unclaimed gaps.
/// This function checks if any invariant year is immediately before or after
/// the title text in the input, and expands the title to include it.
fn expand_title_with_invariant_years(
    input: &str,
    title: Option<String>,
    year_signals: &[YearSignal],
) -> Option<String> {
    let title_text = title.as_deref()?;

    // Find the title's position in the input.
    let title_start = input.find(title_text)?;
    let title_end = title_start + title_text.len();

    let mut expanded_start = title_start;
    let mut expanded_end = title_end;

    for ys in year_signals {
        if !ys.is_invariant {
            continue;
        }

        // Check if this invariant year is immediately before the title
        // (separated only by separators/whitespace).
        if ys.end <= expanded_start {
            let between = &input[ys.end..expanded_start];
            if between.chars().all(|c| SEPS.contains(&c)) {
                expanded_start = ys.start;
            }
        }

        // Check if this invariant year is immediately after the title.
        if ys.start >= expanded_end {
            let between = &input[expanded_end..ys.start];
            if between.chars().all(|c| SEPS.contains(&c)) {
                expanded_end = ys.end;
            }
        }
    }

    if expanded_start == title_start && expanded_end == title_end {
        return Some(title_text.to_string());
    }

    // Rebuild the title from the expanded range, normalizing separators.
    let raw = &input[expanded_start..expanded_end];
    let normalized: String = raw
        .chars()
        .map(|c| if SEPS.contains(&c) { ' ' } else { c })
        .collect();
    Some(normalized.trim().to_string())
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

/// Classify Year matches from Pass 1 that are invariant across siblings.
///
/// This catches year-like numbers that Pass 1 already claimed (e.g., "2001"
/// matched as Year by the year matcher). If every sibling has a Year match
/// with the same value, this year is title content, not release metadata.
fn classify_claimed_year_signals(
    target: &FileAnalysis<'_>,
    siblings: &[FileAnalysis<'_>],
) -> Vec<YearSignal> {
    use crate::matcher::span::Property;

    let mut signals = Vec::new();

    // Find all Year matches in the target.
    for tm in target.matches {
        if tm.property != Property::Year {
            continue;
        }
        let target_value: u32 = match tm.value.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Check if every sibling has a Year match with the same value.
        let mut all_same = true;
        let mut found_in_all = true;

        for sib in siblings {
            let sib_year = sib.matches.iter().find(|m| m.property == Property::Year);
            match sib_year {
                Some(sy) => {
                    if let Ok(sv) = sy.value.parse::<u32>() {
                        if sv != target_value {
                            all_same = false;
                        }
                    } else {
                        found_in_all = false;
                        break;
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
                start: tm.start,
                end: tm.end,
                value: target_value,
                is_invariant: all_same,
            });
        }
    }

    signals
}

/// Detect 3-digit digit decomposition matches that should be absolute episodes.
///
/// Pass 1 decomposes "501" → Season=5+Episode=01 at the same span. If the raw
/// numbers (501, 502, 503) form a sequence across siblings, this is really an
/// absolute episode, not a season+episode decomposition.
fn classify_claimed_decomposed_episodes(
    target: &FileAnalysis<'_>,
    siblings: &[FileAnalysis<'_>],
) -> Vec<EpisodeSignal> {
    use crate::matcher::span::Property;

    let mut signals = Vec::new();

    // Find Season+Episode pairs from decomposition (same span, priority ≤0).
    for tm in target.matches {
        if tm.property != Property::Season || tm.priority > 0 {
            continue;
        }
        // Find the corresponding Episode match at the same span.
        let ep_match = target.matches.iter().find(|m| {
            m.property == Property::Episode
                && m.start == tm.start
                && m.end == tm.end
                && m.priority <= 0
        });
        let ep_match = match ep_match {
            Some(m) => m,
            None => continue,
        };

        // Reconstruct the raw number: season * 100 + episode.
        let season: u32 = match tm.value.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let episode: u32 = match ep_match.value.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let raw_value = season * 100 + episode;

        // Collect raw values from siblings at similar decomposition positions.
        let mut values: Vec<u32> = vec![raw_value];
        let mut found_in_all = true;

        for sib in siblings {
            // Find a Season+Episode decomposition pair in the sibling.
            let sib_season = sib.matches.iter().find(|m| {
                m.property == Property::Season && m.priority <= 0
            });
            let sib_season = match sib_season {
                Some(s) => s,
                None => { found_in_all = false; break; }
            };
            let sib_ep = sib.matches.iter().find(|m| {
                m.property == Property::Episode
                    && m.start == sib_season.start
                    && m.end == sib_season.end
                    && m.priority <= 0
            });
            let sib_ep = match sib_ep {
                Some(e) => e,
                None => { found_in_all = false; break; }
            };

            let ss: u32 = match sib_season.value.parse() {
                Ok(v) => v,
                Err(_) => { found_in_all = false; break; }
            };
            let se: u32 = match sib_ep.value.parse() {
                Ok(v) => v,
                Err(_) => { found_in_all = false; break; }
            };
            values.push(ss * 100 + se);
        }

        if !found_in_all {
            continue;
        }

        // Check if values vary and form a sequence.
        let all_same = values.iter().all(|v| *v == values[0]);
        if all_same {
            continue;
        }

        let is_sequential = is_sequential_set(&values);
        if is_sequential {
            signals.push(EpisodeSignal {
                start: tm.start,
                end: tm.end,
                value: raw_value,
                is_sequential: true,
                digit_count: 3,
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
