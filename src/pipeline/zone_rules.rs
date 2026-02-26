//! Zone-based disambiguation rules.
//!
//! Post-matching disambiguation that handles cross-property semantics
//! not expressible as TOML zone_scope declarations.

use crate::matcher::span::{MatchSpan, Property};
use crate::zone_map::ZoneMap;

/// Zone-based disambiguation using the pre-computed ZoneMap.
///
/// v0.2.1: Uses ZoneMap boundaries directly instead of re-deriving zones
/// from match positions. Rules handled by TOML zone_scope filtering
/// (EpisodeDetails) have been retired.
///
/// Remaining rules handle cross-property semantics:
///   - Language in title zone (needs unmatched-byte heuristic for anchor-less cases)
///   - Duplicate source across zones
///   - Redundant UHD tags
///   - Ambiguous Other overlapping ReleaseGroup
///   - Language nested inside tech spans
pub fn apply_zone_rules(
    input: &str,
    zone_map: &ZoneMap,
    matches: &mut Vec<MatchSpan>,
) {
    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

    // ── Rule 1: Language in title zone → likely a title word ─────────
    if zone_map.has_anchors {
        let title_zone_mid = zone_map.title_zone.start
            + (zone_map.title_zone.end - zone_map.title_zone.start) / 2;
        matches.retain(|m| {
            !(m.property == Property::Language
                && m.start >= fn_start
                && m.start < title_zone_mid)
        });
    } else {
        // No anchors → prune language when substantial unmatched content exists.
        let lang_matches: Vec<&MatchSpan> = matches
            .iter()
            .filter(|m| m.start >= fn_start && m.property == Property::Language)
            .collect();

        if !lang_matches.is_empty() {
            let fn_end = input.len();
            let matched_bytes: usize = matches
                .iter()
                .filter(|m| m.start >= fn_start)
                .map(|m| m.end.saturating_sub(m.start))
                .sum();
            let unmatched = (fn_end - fn_start).saturating_sub(matched_bytes);
            let lang_bytes: usize = lang_matches
                .iter()
                .map(|m| m.end.saturating_sub(m.start))
                .sum();
            if unmatched > lang_bytes {
                matches.retain(|m| !(m.property == Property::Language && m.start >= fn_start));
            }
        }
    }

    // ── Rule 2: Duplicate source in title zone → title word ─────────
    let source_anchor_pos = matches
        .iter()
        .filter(|m| {
            m.start >= fn_start
                && matches!(
                    m.property,
                    Property::Year | Property::Season | Property::Episode
                )
        })
        .map(|m| m.start)
        .min();

    if let Some(anchor) = source_anchor_pos {
        let has_early_source = matches
            .iter()
            .any(|m| m.property == Property::Source && m.start >= fn_start && m.start < anchor);
        let has_late_source = matches
            .iter()
            .any(|m| m.property == Property::Source && m.start >= anchor);

        if has_early_source && has_late_source {
            matches.retain(|m| {
                !(m.property == Property::Source && m.start >= fn_start && m.start < anchor)
            });
        }
    }

    // ── Rule 7: Promote Blu-ray → Ultra HD Blu-ray when UHD signals exist ──
    // When UHD/4K/2160p appears in the filename alongside a Blu-ray source,
    // the source should be "Ultra HD Blu-ray". This handles cases where the
    // UHD marker and Blu-ray marker are too far apart for TOML's 3-token
    // window (e.g., "UHD.10bit.HDR.Bluray").
    let has_uhd_signal = matches.iter().any(|m| {
        m.start >= fn_start
            && ((m.property == Property::Other && m.value == "Ultra HD")
                || (m.property == Property::ScreenSize && m.value == "2160p"))
    });
    if has_uhd_signal {
        for m in matches.iter_mut() {
            if m.start >= fn_start
                && m.property == Property::Source
                && m.value == "Blu-ray"
            {
                m.value = "Ultra HD Blu-ray".into();
            }
        }
    }

    // ── Rule 3: Redundant HD tags when source has UHD ────────────────
    // Must run AFTER Rule 7 (promotion) so the promoted source is detected.
    let source_has_uhd = matches
        .iter()
        .any(|m| m.property == Property::Source && m.value.contains("Ultra HD"));
    if source_has_uhd {
        matches.retain(|m| !(m.property == Property::Other && m.value == "Ultra HD"));
    }

    // ── Rule 4: RETIRED ──────────────────────────────────────────────
    // EpisodeDetails before episode marker → now handled by
    // episode_details.toml zone_scope = "tech_only".

    // ── Rule 5: Other overlapping ReleaseGroup → drop ambiguous Other ───
    let rg_spans: Vec<(usize, usize)> = matches
        .iter()
        .filter(|m| m.property == Property::ReleaseGroup)
        .map(|m| (m.start, m.end))
        .collect();

    if !rg_spans.is_empty() {
        const AMBIGUOUS_OTHER: &[&str] = &["High Quality", "High Resolution"];
        matches.retain(|m| {
            if m.property != Property::Other || !AMBIGUOUS_OTHER.contains(&m.value.as_ref()) {
                return true;
            }
            !rg_spans.iter().any(|(rs, re)| m.start < *re && m.end > *rs)
        });
    }

    // ── Rule 6: Language/SubtitleLanguage contained within a tech span ───
    let tech_spans: Vec<(usize, usize)> = matches
        .iter()
        .filter(|m| {
            matches!(
                m.property,
                Property::Source
                    | Property::VideoCodec
                    | Property::AudioCodec
                    | Property::ScreenSize
                    | Property::StreamingService
            )
        })
        .map(|m| (m.start, m.end))
        .collect();

    if !tech_spans.is_empty() {
        matches.retain(|m| {
            if !matches!(m.property, Property::Language | Property::SubtitleLanguage) {
                return true;
            }
            !tech_spans
                .iter()
                .any(|(ts, te)| m.start >= *ts && m.end <= *te)
        });
    }
}

