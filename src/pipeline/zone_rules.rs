//! Zone-based disambiguation rules.
//!
//! Post-matching disambiguation that handles cross-property semantics.
//! Rules 1 and 2 use neighbor-based context analysis (token_context module)
//! instead of fragile positional heuristics.
//!
//! ## Rule inventory (7 active)
//!
//! | # | Name | Context signal |
//! |---|------|----------------|
//! | 1 | Language disambiguation | Neighbor roles + duplicate detection |
//! | 2 | Source disambiguation | Neighbor roles (title words vs tech) |
//! | 3 | UHD Blu-ray (atomic) | Co-occurrence (semantic) |
//! | 5 | Other ↔ ReleaseGroup | Adjacency to release group |
//! | 6 | Source subsumption | Subsumption table (semantic) |
//! | 7 | Language inside tech span | Byte-range containment |
//! | 8 | Language inside subtitle span | Byte-range containment |

use log::trace;

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;

use super::token_context;

/// Structure-aware disambiguation using neighbor context.
///
/// Rules 1 and 2 use the `token_context` module to classify ambiguous
/// matches based on their neighbors' roles (title word vs tech token),
/// replacing the old positional heuristics ("first half of title zone",
/// "before the anchor").
pub fn apply_zone_rules(
    input: &str,
    _zone_map: &ZoneMap,
    token_stream: &TokenStream,
    matches: &mut Vec<MatchSpan>,
) {
    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let initial_count = matches.len();

    // ── Rule 1: Language disambiguation (context-aware) ────────────────
    // Instead of positional heuristics ("first half of title zone"),
    // check each language match's actual context:
    //   - Is it surrounded by title words? → drop (it's a title word)
    //   - Is it surrounded by tech tokens? → keep (it's a language label)
    //   - Does a duplicate exist in tech context? → drop (redundant)
    //   - Is it after a " - " separator or in brackets? → keep (metadata slot)
    {
        let drop_positions: Vec<usize> = matches
            .iter()
            .filter(|m| m.property == Property::Language)
            .filter(|m| {
                token_context::is_in_title_context(m, matches, token_stream)
                    || token_context::has_duplicate_in_tech_context(m, matches, token_stream)
            })
            .map(|m| m.start)
            .collect();
        matches.retain(|m| m.property != Property::Language || !drop_positions.contains(&m.start));
    }

    trace!(
        "zone rules: {} match(es) after language filtering (was {})",
        matches.len(),
        initial_count
    );

    // ── Rule 2: Source in title context → title word ──────────────────
    // When multiple sources exist, drop the one(s) surrounded by title words.
    // This replaces the fragile "before anchor = title word" heuristic.
    let source_count = matches
        .iter()
        .filter(|m| m.property == Property::Source && m.start >= fn_start)
        .count();
    if source_count > 1 {
        let drop_positions: Vec<usize> = matches
            .iter()
            .filter(|m| m.property == Property::Source && m.start >= fn_start)
            .filter(|m| token_context::is_in_title_context(m, matches, token_stream))
            .map(|m| m.start)
            .collect();
        matches.retain(|m| m.property != Property::Source || !drop_positions.contains(&m.start));
    }

    // ── Rule 3+4: UHD Blu-ray promotion + redundant Ultra HD cleanup ──
    // When UHD/4K/2160p appears alongside Blu-ray, promote the source
    // to "Ultra HD Blu-ray" and drop the redundant Other:"Ultra HD".
    // Combined into a single atomic rule (no ordering dependency).
    let has_uhd_signal = matches.iter().any(|m| {
        m.start >= fn_start
            && ((m.property == Property::Other && m.value == "Ultra HD")
                || (m.property == Property::ScreenSize && m.value == "2160p"))
    });
    if has_uhd_signal {
        for m in matches.iter_mut() {
            if m.start >= fn_start && m.property == Property::Source && m.value == "Blu-ray" {
                m.value = "Ultra HD Blu-ray".into();
            }
        }
    }
    // Drop redundant Ultra HD Other when the source already carries UHD
    // (either via promotion above or from a direct TOML pattern match).
    let source_has_uhd = matches
        .iter()
        .any(|m| m.property == Property::Source && m.value.contains("Ultra HD"));
    if source_has_uhd {
        matches.retain(|m| !(m.property == Property::Other && m.value == "Ultra HD"));
    }

    // ── Rule 5: MOVED to apply_post_release_group_rules() ─────────────────
    // HQ/HR/FanSub adjacency check depends on release group positions,
    // which are now extracted in Pass 2 (post-resolution).

    // ── Rule 7: Language/SubtitleLanguage contained within a tech span ───

    // ── Rule 8: Language contained within a SubtitleLanguage span ────────
    // When a short language token (e.g., "FR", "SWE") falls inside a wider
    // subtitle_language span (e.g., "FR Sub", "SWE Sub"), drop the language
    // match — the token is part of a subtitle marker, not an audio language.
    {
        let sub_spans: Vec<(usize, usize)> = matches
            .iter()
            .filter(|m| m.property == Property::SubtitleLanguage)
            .map(|m| (m.start, m.end))
            .collect();

        if !sub_spans.is_empty() {
            matches.retain(|m| {
                if m.property != Property::Language {
                    return true;
                }
                !sub_spans
                    .iter()
                    .any(|(ss, se)| m.start >= *ss && m.end <= *se)
            });
        }
    }

    // ── Rule 6: Deduplicate subsumed Source values ──────────────────────────
    // When both a generic source (TV, HD) and a specific source (HDTV, HD-DVD)
    // exist, drop the generic one since the specific subsumes it.
    {
        let source_values: Vec<(usize, String)> = matches
            .iter()
            .filter(|m| m.property == Property::Source)
            .map(|m| (m.start, m.value.to_string()))
            .collect();
        if source_values.len() > 1 {
            // Subsumption pairs: if specific exists, drop generic.
            const SUBSUMPTIONS: &[(&str, &str)] = &[
                ("TV", "HDTV"),
                ("TV", "Ultra HDTV"),
                ("TV", "Digital TV"),
                ("HD", "HD-DVD"),
                ("HD", "HD Camera"),
            ];
            let values: Vec<&str> = source_values.iter().map(|(_, v)| v.as_str()).collect();
            let to_drop: Vec<&str> = SUBSUMPTIONS
                .iter()
                .filter(|(_, specific)| values.contains(specific))
                .map(|(generic, _)| *generic)
                .collect();
            if !to_drop.is_empty() {
                matches.retain(|m| {
                    !(m.property == Property::Source && to_drop.contains(&m.value.as_ref()))
                });
            }
        }
    }
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

/// Post-release-group zone rules.
///
/// These rules depend on release group positions, which are only available
/// after Pass 2 extraction. Called from the pipeline after release_group
/// has been extracted.
pub fn apply_post_release_group_rules(matches: &mut Vec<MatchSpan>) {
    // ── Rule 5: Other overlapping or adjacent to ReleaseGroup → drop ambiguous Other ───
    let rg_spans: Vec<(usize, usize)> = matches
        .iter()
        .filter(|m| m.property == Property::ReleaseGroup)
        .map(|m| (m.start, m.end))
        .collect();

    if !rg_spans.is_empty() {
        const AMBIGUOUS_OTHER: &[&str] = &["High Quality", "High Resolution", "Fan Subtitled"];
        // Max gap (in bytes) to consider "adjacent" — covers separator chars.
        const ADJACENCY_GAP: usize = 2;
        matches.retain(|m| {
            if m.property != Property::Other || !AMBIGUOUS_OTHER.contains(&m.value.as_ref()) {
                return true;
            }
            // Drop if overlapping or immediately adjacent to any release group span.
            !rg_spans.iter().any(|(rs, re)| {
                m.start < re.saturating_add(ADJACENCY_GAP)
                    && m.end.saturating_add(ADJACENCY_GAP) > *rs
            })
        });
    }
}
