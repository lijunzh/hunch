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

// ── Pass 1 match_all helpers (#146 mutant kills) ─────────────────────
//
// Two tiny pure helpers extracted from `Pipeline::match_all` so the
// boundary arithmetic and lookup predicates can be unit-tested.

/// Compute the effective priority for a rule firing within a segment.
/// Directory segments get a [`crate::priority::DIR_PENALTY`] adjustment
/// so filename matches win in conflicts.
///
/// Extracted from match_all to pin two surviving mutants on the inline
/// `rule.priority + priority::DIR_PENALTY` arithmetic (#146):
///   - `+ -> -` would flip the sign of the penalty
///   - `+ -> *` would multiply instead of add
pub(super) fn effective_priority_for_segment(rule_priority: i32, is_dir: bool) -> i32 {
    if is_dir {
        rule_priority + crate::priority::DIR_PENALTY
    } else {
        rule_priority
    }
}

/// Look up the per-directory zone map for a given segment index.
/// Returns the first `SegmentZone` whose `segment_idx` matches.
///
/// Extracted from match_all to pin one surviving mutant on the inline
/// `dz.segment_idx == seg_idx` predicate (#146):
///   - `== -> !=` would return the first NON-matching zone, which has
///     a different segment_idx by construction.
pub(super) fn find_dir_zone_for_segment(
    dir_zones: &[zone_map::SegmentZone],
    seg_idx: usize,
) -> Option<&zone_map::SegmentZone> {
    dir_zones.iter().find(|dz| dz.segment_idx == seg_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::priority;
    use crate::zone_map::SegmentZone;

    // ── effective_priority_for_segment ───────────────────────────────

    #[test]
    fn effective_priority_filename_unchanged() {
        // Pin: filename segments do NOT receive the penalty. With the
        // original `if is_dir { ... }` guard inverted (e.g., the if-arm
        // taken for !is_dir), the filename would get a penalty applied.
        assert_eq!(effective_priority_for_segment(10, false), 10);
        assert_eq!(effective_priority_for_segment(0, false), 0);
        assert_eq!(effective_priority_for_segment(-3, false), -3);
    }

    #[test]
    fn effective_priority_directory_adds_dir_penalty() {
        // Pin `+ -> -` and `+ -> *` mutants on the directory branch.
        //
        // DIR_PENALTY is currently -5 (see src/priority.rs).
        //
        // For rule_priority=10, is_dir=true:
        //   - With `+`: 10 + (-5) = 5  (correct)
        //   - With `-`: 10 - (-5) = 15 (mutant: penalty becomes a bonus!)
        //   - With `*`: 10 * (-5) = -50 (mutant: clearly wrong magnitude)
        // Asserting the result is exactly 5 kills both mutants.
        assert_eq!(
            effective_priority_for_segment(10, true),
            10 + priority::DIR_PENALTY
        );
        // Concrete value sanity check (depends on DIR_PENALTY=-5):
        assert_eq!(effective_priority_for_segment(10, true), 5);
    }

    #[test]
    fn effective_priority_dir_with_negative_rule_priority() {
        // Belt: ensure the addition handles negative rule priorities
        // (HEURISTIC=-1, POSITIONAL=-2). With the mutation `+ -> *`,
        // a negative rule priority * negative DIR_PENALTY would give a
        // POSITIVE result — opposite of intent.
        assert_eq!(
            effective_priority_for_segment(priority::HEURISTIC, true),
            priority::HEURISTIC + priority::DIR_PENALTY
        );
        // Concrete value: -1 + (-5) = -6
        assert_eq!(
            effective_priority_for_segment(priority::HEURISTIC, true),
            -6
        );
    }

    // ── find_dir_zone_for_segment ────────────────────────────────────

    fn make_zone(segment_idx: usize) -> SegmentZone {
        SegmentZone {
            segment_idx,
            title_zone: 0..0,
            tech_zone: 0..0,
            has_anchors: false,
        }
    }

    #[test]
    fn find_dir_zone_empty_list_returns_none() {
        assert!(find_dir_zone_for_segment(&[], 0).is_none());
        assert!(find_dir_zone_for_segment(&[], 42).is_none());
    }

    #[test]
    fn find_dir_zone_no_match_returns_none() {
        let zones = vec![make_zone(1), make_zone(3)];
        assert!(find_dir_zone_for_segment(&zones, 2).is_none());
    }

    #[test]
    fn find_dir_zone_single_match_returns_it() {
        let zones = vec![make_zone(2)];
        let found = find_dir_zone_for_segment(&zones, 2).expect("should find");
        assert_eq!(found.segment_idx, 2);
    }

    #[test]
    fn find_dir_zone_picks_correct_zone_among_many() {
        // CRITICAL: pins `== -> !=` mutant on `dz.segment_idx == seg_idx`.
        //
        // Three zones with distinct segment_idx values. Looking up idx=1
        // must return the zone with segment_idx==1, not 0 or 2.
        //   - With `==`: returns zone where segment_idx == 1 (correct)
        //   - With `!=`: returns first zone where segment_idx != 1, which
        //     would be the zone at idx=0 (wrong segment_idx).
        // Asserting the returned zone has segment_idx==1 kills it.
        let zones = vec![make_zone(0), make_zone(1), make_zone(2)];
        let found = find_dir_zone_for_segment(&zones, 1).expect("should find");
        assert_eq!(
            found.segment_idx, 1,
            "must return the zone whose idx matches the query"
        );
    }

    #[test]
    fn find_dir_zone_first_match_wins_on_duplicates() {
        // Documents: with duplicate segment_idx values, the first one in
        // iteration order is returned. Belt for any future Iterator change.
        let zones = vec![make_zone(5), make_zone(5)];
        let found = find_dir_zone_for_segment(&zones, 5).expect("should find");
        assert_eq!(found.segment_idx, 5);
    }
}
