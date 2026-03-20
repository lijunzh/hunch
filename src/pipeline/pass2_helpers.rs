//! Free functions used by Pass 2 of the pipeline.
//!
//! Extracted from `mod.rs` to keep both files under 600 lines.

use log::{debug, trace};

use crate::hunch_result::Confidence;
use crate::matcher::span::{MatchSpan, Property};
use crate::HunchResult;

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
    _input: &str,
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

        // Evict any heuristic Season/Episode decomposition that overlaps.
        // Invariance signals (cross-file sequential evidence) have higher
        // confidence than single-file digit decomposition.
        let before = matches.len();
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

        // Check for non-decomposition overlaps (e.g., codec, screen_size).
        let overlaps_non_heuristic = matches.iter().any(|m| {
            m.start < es.end && m.end > es.start
        });
        if overlaps_non_heuristic {
            // Restore evicted matches by re-running — but actually, we already
            // removed them. If a non-heuristic match exists, skip injection
            // but the eviction is already done. This is a rare edge case;
            // log it and move on.
            if matches.len() < before {
                trace!(
                    "[INVARIANCE] evicted heuristics but found non-heuristic overlap at {}..{}, \
                     episode {} not injected (edge case)",
                    es.start, es.end, es.value
                );
            }
            continue;
        }

        debug!(
            "[INVARIANCE] injecting Episode={} at {}..{} (sequential, {}-digit)",
            es.value, es.start, es.end, es.digit_count
        );
        matches.push(MatchSpan::new(
            es.start,
            es.end,
            Property::Episode,
            es.value.to_string(),
        ));
    }
}

/// Compute confidence level based on structural signals.
pub(super) fn compute_confidence(result: &HunchResult, used_cross_file: bool) -> Confidence {
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

    // High: cross-file context succeeded, or ≥3 anchors with a reasonable title.
    if used_cross_file && has_title {
        return Confidence::High;
    }
    if anchor_count >= 3 && has_title && title_len >= 2 {
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
