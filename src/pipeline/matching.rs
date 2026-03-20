//! Token-level TOML rule matching within pipeline segments.
//!
//! Uses a sliding window of 1–3 tokens (longest first) to handle compound
//! patterns like "WEB-DL" or "HD-DVD". Emits primary matches and any
//! side-effect spans declared in the TOML pattern.

use crate::matcher::rule_loader::{RuleSet, ZoneScope};
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer;
use crate::zone_map::{self, ZoneMap};

/// Context for matching tokens in a single segment against a TOML rule set.
///
/// Bundles the segment data, rule set, and zone context that
/// `match_tokens_in_segment` needs, replacing 7 positional arguments
/// with a single struct.
pub(crate) struct MatchContext<'a> {
    /// The full input string (for extracting compound token text).
    pub input: &'a str,
    /// Tokens within this segment.
    pub tokens: &'a [tokenizer::Token],
    /// The TOML rule set to match against.
    pub rule_set: &'a RuleSet,
    /// Which property this rule set targets.
    pub property: Property,
    /// Priority for emitted matches (directory segments get a penalty).
    pub priority: i32,
    /// Filename-level zone map (anchors, title zone, tech zone).
    pub zone_map: &'a ZoneMap,
    /// Per-directory zone map, if this is a directory segment.
    pub dir_zone: Option<&'a zone_map::SegmentZone>,
}

/// Match tokens within a single segment against a TOML rule set.
///
/// Uses a sliding window of 1–3 tokens (longest first) to handle compound
/// patterns like "WEB-DL" or "HD-DVD". Emits primary matches and any
/// side-effect spans declared in the TOML pattern.
pub(crate) fn match_tokens_in_segment(ctx: &MatchContext, matches: &mut Vec<MatchSpan>) {
    let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

    for window_size in (1..=3).rev() {
        for i in 0..ctx.tokens.len() {
            if i + window_size > ctx.tokens.len() {
                break;
            }

            let win_start = ctx.tokens[i].start;
            let win_end = ctx.tokens[i + window_size - 1].end;

            // ── Zone scope filtering ─────────────────────────────
            // Use per-directory zone when available, otherwise filename zone.
            let (effective_has_anchors, effective_title_zone) = if let Some(dz) = ctx.dir_zone {
                (dz.has_anchors, &dz.title_zone)
            } else {
                (ctx.zone_map.has_anchors, &ctx.zone_map.title_zone)
            };

            if effective_has_anchors {
                let in_title_zone = effective_title_zone.contains(&win_start);
                match ctx.rule_set.zone_scope {
                    ZoneScope::TechOnly if in_title_zone => continue,
                    ZoneScope::AfterAnchor if in_title_zone => continue,
                    _ => {}
                }
            }
            if matched_ranges
                .iter()
                .any(|(s, e)| win_start < *e && win_end > *s)
            {
                continue;
            }

            let compound = if window_size == 1 {
                ctx.tokens[i].text.clone()
            } else {
                ctx.input[win_start..win_end].to_string()
            };

            if let Some(token_match) = ctx.rule_set.match_token(&compound) {
                // ── Neighbor constraint checks ──────────────────
                let last_idx = i + window_size - 1;
                if let Some(ref blocked) = token_match.not_before
                    && last_idx + 1 < ctx.tokens.len()
                    && blocked
                        .iter()
                        .any(|b| b.as_str() == ctx.tokens[last_idx + 1].lower())
                {
                    continue;
                }
                if let Some(ref blocked) = token_match.not_after
                    && i > 0
                    && blocked
                        .iter()
                        .any(|b| b.as_str() == ctx.tokens[i - 1].lower())
                {
                    continue;
                }
                if let Some(ref required) = token_match.requires_after {
                    let ok = last_idx + 1 < ctx.tokens.len()
                        && required
                            .iter()
                            .any(|r| r.as_str() == ctx.tokens[last_idx + 1].lower());
                    if !ok {
                        continue;
                    }
                }
                // requires_context: only match when tech anchors exist
                // OR when requires_before matches (fallback for context words).
                if token_match.requires_context && !ctx.zone_map.has_anchors {
                    if let Some(ref required) = token_match.requires_before {
                        let ok = i > 0
                            && required
                                .iter()
                                .any(|r| r.as_str() == ctx.tokens[i - 1].lower());
                        if !ok {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else if !token_match.requires_context {
                    if let Some(ref required) = token_match.requires_before {
                        let ok = i > 0
                            && required
                                .iter()
                                .any(|r| r.as_str() == ctx.tokens[i - 1].lower());
                        if !ok {
                            continue;
                        }
                    }
                }

                // ── Primary match ─────────────────────────────
                let mut reclaimable = token_match.reclaimable;
                if let Some(ref nearby) = token_match.requires_nearby {
                    let nearby_found = ctx
                        .tokens
                        .iter()
                        .any(|t| nearby.iter().any(|n| n.as_str() == t.lower()));
                    if !nearby_found {
                        reclaimable = true;
                    }
                }

                let span = MatchSpan::new(win_start, win_end, ctx.property, token_match.value)
                    .with_priority(ctx.priority);
                let span = if reclaimable {
                    span.as_reclaimable()
                } else {
                    span
                };
                matches.push(span);
                matched_ranges.push((win_start, win_end));

                // ── Side effects ──────────────────────────────
                for se in &token_match.side_effects {
                    if let Some(se_prop) = Property::from_name(&se.property) {
                        matches.push(
                            MatchSpan::new(win_start, win_end, se_prop, &se.value)
                                .with_priority(ctx.priority),
                        );
                    }
                }
            }
        }
    }
}
