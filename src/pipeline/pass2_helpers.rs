//! Free functions used by Pass 2 of the pipeline.
//!
//! Extracted from `mod.rs` to keep both files under 600 lines.

use log::{debug, trace};

use crate::HunchResult;
use crate::hunch_result::Confidence;
use crate::matcher::span::{MatchSpan, Property, Source};

use super::invariance;

/// Apply cross-file invariance signals to disambiguate year and episode matches.
///
/// **Year signals**: If a year-like number is *invariant* across siblings
/// (same value in every file at the same gap position), it's title content,
/// not a release year. Remove any existing `Year` match at that position.
///
/// **Episode signals**: If a bare number *varies* across siblings and forms
/// a sequential pattern, inject an `Episode` match (if none already exists
/// at that position). Non-sequential variant numbers are logged but not
/// injected — they may represent something else.
pub(super) fn apply_invariance_signals(
    matches: &mut Vec<MatchSpan>,
    report: &invariance::InvarianceReport,
) {
    // Year signals: suppress Year matches for invariant year-like numbers.
    for ys in &report.year_signals {
        if ys.is_invariant {
            let before = matches.len();
            matches.retain(|m| {
                if m.property != Property::Year {
                    return true;
                }
                !(m.start <= ys.start && m.end >= ys.end)
            });
            if matches.len() < before {
                debug!(
                    "[INVARIANCE] suppressed Year match for invariant \"{}\" at {}..{}",
                    ys.value, ys.start, ys.end
                );
            }
        }
    }

    // Episode signals: inject Episode match for sequential variant numbers.
    for es in &report.episode_signals {
        if !es.is_sequential {
            trace!(
                "[INVARIANCE] variant number {} at {}..{} is non-sequential, skipping",
                es.value, es.start, es.end
            );
            continue;
        }

        // Check if there's already an Episode match at this position with the
        // correct value (e.g., from SxxExx pattern). Don't clobber it.
        let correctly_claimed = matches.iter().any(|m| {
            m.property == Property::Episode
                && m.start <= es.start
                && m.end >= es.end
                && m.value == es.value.to_string()
        });
        if correctly_claimed {
            trace!(
                "[INVARIANCE] episode {} at {}..{} already correctly claimed, skipping",
                es.value, es.start, es.end
            );
            continue;
        }

        // Check for non-decomposition overlaps BEFORE evicting heuristics.
        // If a non-heuristic match exists at this position, skip entirely
        // to avoid evicting heuristic decomposition with no replacement.
        let overlaps_non_heuristic = matches.iter().any(|m| {
            let overlaps = m.start < es.end && m.end > es.start;
            let is_decomposed = overlaps
                && (m.property == Property::Season || m.property == Property::Episode)
                && m.priority <= 0;
            overlaps && !is_decomposed
        });
        if overlaps_non_heuristic {
            trace!(
                "[INVARIANCE] non-heuristic overlap at {}..{}, skipping episode {} injection",
                es.start, es.end, es.value
            );
            continue;
        }

        // Evict any heuristic Season/Episode decomposition that overlaps.
        // Invariance signals (cross-file sequential evidence) have higher
        // confidence than single-file digit decomposition.
        matches.retain(|m| {
            let overlaps = m.start < es.end && m.end > es.start;
            let is_decomposed = overlaps
                && (m.property == Property::Season || m.property == Property::Episode)
                && m.priority <= 0;
            if is_decomposed {
                debug!(
                    "[INVARIANCE] evicting heuristic {:?}={} at {}..{} (pri={})",
                    m.property, m.value, m.start, m.end, m.priority
                );
            }
            !is_decomposed
        });

        debug!(
            "[CONTEXT] injecting Episode={} at {}..{} (sequential, {}-digit)",
            es.value, es.start, es.end, es.digit_count
        );
        matches.push(
            MatchSpan::new(es.start, es.end, Property::Episode, es.value.to_string())
                .with_source(Source::Context),
        );
    }
}

/// Compute confidence level based on structural signals and match sources.
pub(super) fn compute_confidence(
    result: &HunchResult,
    used_cross_file: bool,
    matches: &[MatchSpan],
) -> Confidence {
    let tech_properties = [
        Property::VideoCodec,
        Property::AudioCodec,
        Property::ScreenSize,
        Property::Source,
        Property::Season,
        Property::Episode,
    ];
    let anchor_count = tech_properties
        .iter()
        .filter(|p| result.first(**p).is_some())
        .count();

    let has_title = result.title().is_some();
    let title_len = result.title().map(|t| t.chars().count()).unwrap_or(0);

    // Check if any result property is backed only by heuristic sources
    // with no context confirmation.
    let has_heuristic_only = matches.iter().any(|m| m.source == Source::Heuristic)
        && !matches.iter().any(|m| m.source == Source::Context);

    // High: cross-file context succeeded, or ≥3 anchors with a reasonable title.
    if used_cross_file && has_title {
        return Confidence::High;
    }
    if anchor_count >= 3 && has_title && title_len >= 2 {
        // Cap at Medium if heuristic-only matches are present.
        if has_heuristic_only {
            return Confidence::Medium;
        }
        return Confidence::High;
    }

    // Low: no title, or title is suspiciously short.
    if !has_title || title_len <= 1 {
        return Confidence::Low;
    }

    // Medium: everything else.
    if anchor_count >= 1 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// Subtitle container extensions.
const SUBTITLE_CONTAINERS: &[&str] = &["srt", "sub", "ass", "ssa", "idx", "sup", "vtt", "smi"];

/// Properties that are meaningless for subtitle files.
const SUBTITLE_STRIP_PROPERTIES: &[Property] = &[
    Property::VideoCodec,
    Property::ColorDepth,
    Property::VideoProfile,
    Property::Source,
    Property::AudioCodec,
    Property::AudioChannels,
    Property::AudioProfile,
    Property::FrameRate,
];

/// Strip video/audio tech properties from subtitle containers.
pub(super) fn strip_tech_from_subtitle_containers(matches: &mut Vec<MatchSpan>) {
    let is_subtitle = matches.iter().any(|m| {
        m.property == Property::Container
            && SUBTITLE_CONTAINERS
                .iter()
                .any(|ext| m.value.eq_ignore_ascii_case(ext))
    });
    if is_subtitle {
        matches.retain(|m| !SUBTITLE_STRIP_PROPERTIES.contains(&m.property));
    }
}

/// Compute the (start, end) byte range for an override-title span,
/// clamping `end` to `input_len` to avoid out-of-bounds when the title
/// text is normalized to a different length than the raw substring.
///
/// Extracted from Pass 2 step 5b for testability — the inline arithmetic
/// (`start + title_len`) was a surviving mutant target (#146).
pub(super) fn compute_override_title_span(
    start: usize,
    title_len: usize,
    input_len: usize,
) -> (usize, usize) {
    let end = start.saturating_add(title_len).min(input_len);
    (start, end)
}

/// Decide whether a release_group span should be **dropped** because it
/// substantially overlaps an episode title span. Returns `true` if at
/// least 50% of the release_group span is inside the episode title.
///
/// Extracted from Pass 2 step 5c for testability — the inline boundary
/// check (`overlap * 2 < rg_len`) was a surviving mutant target (#146).
/// The exact 50% boundary is the interesting case: the release_group is
/// dropped when overlap is *strictly more than* half (`overlap * 2 > rg_len`)
/// OR equal to half (`overlap * 2 == rg_len`).
pub(super) fn release_group_overlaps_episode_title(
    rg_start: usize,
    rg_end: usize,
    ep_start: usize,
    ep_end: usize,
) -> bool {
    let overlap_start = rg_start.max(ep_start);
    let overlap_end = rg_end.min(ep_end);
    let overlap = overlap_end.saturating_sub(overlap_start);
    let rg_len = rg_end.saturating_sub(rg_start).max(1);
    // "Drop" semantics: true means the RG should be removed.
    // Equivalent to the original retain closure's `overlap * 2 < rg_len`
    // negated (retain keeps when expression is true).
    overlap * 2 >= rg_len
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_override_title_span (#146 mutant kills) ─────────────────

    #[test]
    fn compute_override_title_span_basic_addition() {
        // Pins `+ -> *` mutant on `start + title_len`.
        //   - With `+`: 5 + 10 = 15 (correct)
        //   - With `*`: 5 * 10 = 50 (clearly wrong)
        // The .min(100) clamp does NOT mask either: 15.min(100)=15, 50.min(100)=50.
        assert_eq!(compute_override_title_span(5, 10, 100), (5, 15));
    }

    #[test]
    fn compute_override_title_span_clamps_to_input_len() {
        // Pin the .min(input_len) clamp: when the title length pushes past
        // input_len, end must clamp to input_len.
        // 90 + 20 = 110, clamp to 100 → (90, 100).
        assert_eq!(compute_override_title_span(90, 20, 100), (90, 100));
    }

    #[test]
    fn compute_override_title_span_zero_length_title() {
        // Edge: empty override title → zero-width span at start.
        assert_eq!(compute_override_title_span(7, 0, 100), (7, 7));
    }

    #[test]
    fn compute_override_title_span_at_input_boundary() {
        // Edge: span ends exactly at input_len (no clamp triggered).
        assert_eq!(compute_override_title_span(0, 100, 100), (0, 100));
    }

    #[test]
    fn compute_override_title_span_does_not_underflow() {
        // Belt: saturating_add prevents overflow panic on absurd inputs.
        // Without saturating_add, `usize::MAX + 1` would panic in debug.
        let (s, e) = compute_override_title_span(usize::MAX, 1, usize::MAX);
        assert_eq!(s, usize::MAX);
        assert_eq!(e, usize::MAX); // saturated then clamped
    }

    // ── release_group_overlaps_episode_title (#146 mutant kills) ────────────

    #[test]
    fn rg_overlaps_ep_title_no_overlap_keeps() {
        // RG [0..5], EP [10..20]: zero overlap → keep (return false).
        assert!(!release_group_overlaps_episode_title(0, 5, 10, 20));
    }

    #[test]
    fn rg_overlaps_ep_title_fully_inside_drops() {
        // RG [12..16] fully inside EP [10..20]: 100% overlap → drop.
        assert!(release_group_overlaps_episode_title(12, 16, 10, 20));
    }

    #[test]
    fn rg_overlaps_ep_title_exactly_50pct_drops() {
        // CRITICAL boundary: RG is 10 wide [10..20], overlap is 5 [10..15].
        //   - overlap * 2 = 10, rg_len = 10 → 10 >= 10 → DROP (correct)
        //   - With `< -> <=` mutant on the original (`overlap * 2 < rg_len`):
        //     5*2 < 10 false (orig) vs 5*2 <= 10 true (mutant) → mutant
        //     KEEPS at the boundary, original DROPS.
        // Asserting drop at exactly 50% kills the `< -> <=` mutant.
        assert!(release_group_overlaps_episode_title(10, 20, 5, 15));
    }

    #[test]
    fn rg_overlaps_ep_title_just_under_50pct_keeps() {
        // RG is 10 wide [10..20], overlap is 4 [10..14].
        //   - overlap * 2 = 8, rg_len = 10 → 8 >= 10 → false → KEEP
        // Confirms the 50% threshold is strict enough that 49% keeps.
        assert!(!release_group_overlaps_episode_title(10, 20, 5, 14));
    }

    #[test]
    fn rg_overlaps_ep_title_just_over_50pct_drops() {
        // RG is 10 wide [10..20], overlap is 6 [10..16].
        //   - overlap * 2 = 12, rg_len = 10 → 12 >= 10 → true → DROP
        assert!(release_group_overlaps_episode_title(10, 20, 5, 16));
    }

    #[test]
    fn rg_overlaps_ep_title_zero_width_rg_is_handled() {
        // Edge: degenerate zero-width RG. .max(1) prevents divide-by-zero
        // semantics; overlap is 0, rg_len floored to 1, 0 >= 1 false → KEEP.
        assert!(!release_group_overlaps_episode_title(10, 10, 5, 20));
    }
}
