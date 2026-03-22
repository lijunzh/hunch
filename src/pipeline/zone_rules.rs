//! Zone-based disambiguation rules.
//!
//! Post-matching disambiguation that handles cross-property semantics.
//! `language_disambiguation` and `source_in_title_context` use
//! neighbor-based context analysis (token_context module) instead of
//! fragile positional heuristics.
//!
//! ## Rule inventory (7 active)
//!
//! | Rule | Phase | Context signal |
//! |------|-------|----------------|
//! | `language_disambiguation` | Pass 1 | Neighbor roles + duplicate detection |
//! | `source_in_title_context` | Pass 1 | Neighbor roles (title words vs tech) |
//! | `uhd_bluray_promotion` | Pass 1 | Co-occurrence (semantic) |
//! | `subtitle_source_conflict` | Pass 1 | Same-span priority comparison |
//! | `language_inside_tech_span` | Pass 1 | Byte-range containment |
//! | `language_inside_subtitle_span` | Pass 1 | Byte-range containment |
//! | `source_subsumption` | Pass 1 | Subsumption table (semantic) |
//! | `other_release_group_adjacency` | Pass 2 | Adjacency to release group |

use log::trace;

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::TokenStream;
use crate::zone_map::ZoneMap;

use super::token_context;

/// Structure-aware disambiguation using neighbor context.
///
/// `language_disambiguation` and `source_in_title_context` use the
/// `token_context` module to classify ambiguous matches based on their
/// neighbors' roles (title word vs tech token), replacing the old
/// positional heuristics ("first half of title zone", "before the anchor").
pub fn apply_zone_rules(
    input: &str,
    _zone_map: &ZoneMap,
    token_stream: &TokenStream,
    matches: &mut Vec<MatchSpan>,
) {
    let fn_start = crate::filename_start(input);
    let initial_count = matches.len();

    // ── language_disambiguation ────────────────────────────────────────
    // Check each language match's actual neighbor context:
    //   - Surrounded by title words? → drop (it's a title word)
    //   - Surrounded by tech tokens? → keep (it's a language label)
    //   - Duplicate exists in tech context? → drop (redundant)
    //   - After a " - " separator or in brackets? → keep (metadata slot)
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

    // ── source_in_title_context ───────────────────────────────────────
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

    // ── uhd_bluray_promotion ──────────────────────────────────────────
    // When UHD/4K/2160p appears alongside Blu-ray, promote the source
    // to "Ultra HD Blu-ray" and drop the redundant Other:"Ultra HD".
    // Promotion and cleanup are atomic (no ordering dependency).
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

    // ── other_release_group_adjacency → see apply_post_release_group_rules()
    // HQ/HR/FanSub adjacency check depends on release group positions,
    // which are only available after Pass 2 extraction.

    // ── subtitle_source_conflict ──────────────────────────────────────
    // When SubtitleLanguage and Source occupy the exact same span (e.g.,
    // `tc` matching both Telecine and Traditional Chinese), keep the
    // higher-priority SubtitleLanguage and drop the Source.
    // Also: "TC" for Telecine is extremely rare and almost always a CJK
    // subtitle indicator. If SubtitleLanguage is already detected (e.g.,
    // from BIG5 or .tc.ass), drop any Source=Telecine match.
    {
        let has_sub_lang = matches
            .iter()
            .any(|m| m.property == Property::SubtitleLanguage);
        let sub_spans: Vec<(usize, usize, i32)> = matches
            .iter()
            .filter(|m| m.property == Property::SubtitleLanguage)
            .map(|m| (m.start, m.end, m.priority))
            .collect();
        matches.retain(|m| {
            if m.property != Property::Source {
                return true;
            }
            // Drop Telecine when subtitle language is already known.
            if m.value == "Telecine" && has_sub_lang {
                return false;
            }
            // Drop Source when SubtitleLanguage occupies the exact same span.
            !sub_spans
                .iter()
                .any(|(ss, se, sp)| m.start == *ss && m.end == *se && *sp >= m.priority)
        });
    }

    // ── language_inside_tech_span ─────────────────────────────────────

    // ── language_inside_subtitle_span ─────────────────────────────────
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

    // ── source_subsumption ────────────────────────────────────────────
    // When both a generic source (TV, HD) and a specific source (HDTV,
    // HD-DVD) exist, drop the generic one since the specific subsumes it.
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
    // ── other_release_group_adjacency ─────────────────────────────────
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
