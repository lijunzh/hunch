//! Token context classification for structure-aware disambiguation.
//!
//! Instead of using fragile positional heuristics ("first half of title zone",
//! "before the anchor"), this module classifies each token's role based on
//! its actual context:
//!
//! 1. **Neighbor roles**: Are surrounding tokens claimed by confident tech
//!    properties (codec, resolution, source) or unclaimed (title words)?
//! 2. **Structural separators**: Is this token after a metadata boundary
//!    like " - " or in brackets?
//! 3. **Duplicate detection**: Does the same value appear in a clearly
//!    technical position elsewhere in the path?
//!
//! This replaces the zone_rules heuristics that used position as a proxy
//! for semantic role.

use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::{Separator, TokenStream};

/// Properties that are NEVER title words — their presence signals tech context.
///
/// If a neighboring token is claimed by one of these, the region is
/// technical/metadata, not title content.
const CONFIDENT_TECH: &[Property] = &[
    Property::VideoCodec,
    Property::AudioCodec,
    Property::ScreenSize,
    Property::Source,
    Property::Year,
    Property::Season,
    Property::Episode,
    Property::Date,
    Property::Container,
    Property::FrameRate,
    Property::ColorDepth,
    Property::VideoProfile,
    Property::AudioChannels,
    Property::AudioProfile,
    Property::Edition,
    Property::StreamingService,
    Property::EpisodeCount,
    Property::SeasonCount,
    Property::Part,
    Property::Bonus,
    Property::Other,
    Property::VideoApi,
    Property::ReleaseGroup,
];

/// Check whether a match is in "title context" — surrounded by title words
/// rather than technical tokens.
///
/// Returns `true` if the match's neighbors suggest it's part of a title
/// phrase (should be dropped as metadata), `false` if it's in a technical
/// region (should be kept as a real property match).
///
/// ## Algorithm
///
/// 1. Find the token(s) immediately before and after this match
///    (within the same path segment).
/// 2. Check if each neighbor is "claimed" by a confident tech property.
/// 3. Score: unclaimed neighbors = title context, claimed = tech context.
///
/// Special cases:
/// - **After a ` - ` separator**: Metadata boundary — NOT title context.
///   Patterns like `"Movie - FR"` use " - " to delimit metadata.
/// - **In brackets**: Brackets signal metadata — NOT title context.
/// - **No neighbors on one side** (start/end of segment): That side
///   is neutral (doesn't count either way).
pub fn is_in_title_context(
    m: &MatchSpan,
    matches: &[MatchSpan],
    token_stream: &TokenStream,
) -> bool {
    // Find the segment this match belongs to.
    let segment = token_stream
        .segments
        .iter()
        .find(|s| s.start <= m.start && m.end <= s.end);
    let segment = match segment {
        Some(s) => s,
        None => return false, // Can't determine context → keep the match.
    };

    // Find the token(s) that correspond to this match.
    let match_token_indices: Vec<usize> = segment
        .tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.start >= m.start && t.end <= m.end)
        .map(|(i, _)| i)
        .collect();

    if match_token_indices.is_empty() {
        return false;
    }

    let first_idx = match_token_indices[0];
    let last_idx = *match_token_indices.last().unwrap();

    // Check structural signals that override neighbor analysis.
    // A token after " - " is in a metadata slot, not title context.
    if let Some(token) = segment.tokens.get(first_idx) {
        if is_after_metadata_separator(token, first_idx, &segment.tokens) {
            return false; // Metadata position → keep the match.
        }
        if token.in_brackets {
            return false; // Brackets = metadata → keep the match.
        }
    }

    // Score neighbors: +1 for title (unclaimed), -1 for tech (claimed).
    // A neighbor counts as "tech" if it's claimed by a confident tech
    // property OR if it's claimed by the SAME property type (peer
    // reinforcement: FRENCH next to ENGLISH = language cluster = metadata).
    let mut score: i32 = 0;
    let mut sides_checked = 0;

    // Left neighbor(s): look at up to 2 tokens before the match.
    for offset in 1..=2 {
        if first_idx >= offset {
            let neighbor = &segment.tokens[first_idx - offset];
            if is_tech_or_peer(neighbor, matches, m.property) {
                score -= 1;
            } else {
                score += 1;
            }
            sides_checked += 1;
            break; // Use the nearest neighbor only.
        }
    }

    // Right neighbor(s): look at up to 2 tokens after the match.
    for offset in 1..=2 {
        let right_idx = last_idx + offset;
        if right_idx < segment.tokens.len() {
            let neighbor = &segment.tokens[right_idx];
            if is_tech_or_peer(neighbor, matches, m.property) {
                score -= 1;
            } else {
                score += 1;
            }
            sides_checked += 1;
            break;
        }
    }

    // No neighbors or only one side checked → insufficient neighbor evidence.
    // Fall back to structural position: is this before the first tech anchor
    // in the segment? If so, it's in title territory.
    if sides_checked < 2 {
        return is_before_first_anchor_in_segment(m, matches, segment);
    }

    // Title context when score > 0 (more title neighbors than tech).
    // Score == 0 (mixed — one title neighbor, one tech neighbor) →
    // use structural position as tiebreaker. Tokens before the first
    // anchor with mixed context are more likely title words than metadata.
    if score == 0 {
        return is_before_first_anchor_in_segment(m, matches, segment);
    }
    score > 0
}

/// Structural fallback for edge-of-segment tokens.
///
/// When a token is at the start or end of a segment and we can't check
/// both neighbors, use the position relative to the first tech anchor:
/// - Before the first anchor → title territory (drop ambiguous matches)
/// - After the first anchor → tech territory (keep ambiguous matches)
/// - No anchors in segment → ambiguous (keep, be conservative)
fn is_before_first_anchor_in_segment(
    m: &MatchSpan,
    matches: &[MatchSpan],
    segment: &crate::tokenizer::PathSegment,
) -> bool {
    // Find the first tech anchor in this segment.
    // Anchors are strong structural markers: Year, Season, Episode, Date.
    let first_anchor = matches
        .iter()
        .filter(|other| {
            other.start >= segment.start
                && other.end <= segment.end
                && matches!(
                    other.property,
                    Property::Year | Property::Season | Property::Episode | Property::Date
                )
        })
        .map(|other| other.start)
        .min();

    match first_anchor {
        Some(anchor_pos) => m.start < anchor_pos, // Before anchor → title zone.
        None => false, // No anchors → ambiguous → conservative (keep match).
    }
}

/// Check whether a match is in "tech context" — surrounded by confident
/// tech tokens on both sides.
///
/// Stricter than `!is_in_title_context()`: returns true only when
/// neighbors are predominantly technical. Used by `has_duplicate_in_tech_context`
/// to avoid circular drops (both instances claiming the other is tech).
pub fn is_in_tech_context(
    m: &MatchSpan,
    matches: &[MatchSpan],
    token_stream: &TokenStream,
) -> bool {
    let segment = token_stream
        .segments
        .iter()
        .find(|s| s.start <= m.start && m.end <= s.end);
    let segment = match segment {
        Some(s) => s,
        None => return false,
    };

    let match_token_indices: Vec<usize> = segment
        .tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.start >= m.start && t.end <= m.end)
        .map(|(i, _)| i)
        .collect();

    if match_token_indices.is_empty() {
        return false;
    }

    let first_idx = match_token_indices[0];
    let last_idx = *match_token_indices.last().unwrap();

    // Need at least one tech neighbor. Both sides must be tech or absent.
    let mut has_tech = false;

    // Left neighbor.
    if first_idx > 0 {
        let left = &segment.tokens[first_idx - 1];
        if !is_claimed_by_tech(left, matches) {
            return false; // Left is a title word → not firmly tech context.
        }
        has_tech = true;
    }

    // Right neighbor.
    if last_idx + 1 < segment.tokens.len() {
        let right = &segment.tokens[last_idx + 1];
        if !is_claimed_by_tech(right, matches) {
            return false; // Right is a title word → not firmly tech context.
        }
        has_tech = true;
    }

    has_tech
}

/// Check whether a token is claimed by a confident tech property.
fn is_claimed_by_tech(token: &crate::tokenizer::Token, matches: &[MatchSpan]) -> bool {
    matches.iter().any(|m| {
        CONFIDENT_TECH.contains(&m.property) && m.start <= token.start && m.end >= token.end
    })
}

/// Check whether a token is claimed by a confident tech property
/// OR by the same property type as the match being evaluated.
///
/// Peer reinforcement: FRENCH next to ENGLISH (both Language) signals
/// a metadata cluster, not title content. Two adjacent tokens of the
/// same ambiguous property type reinforce each other as metadata.
fn is_tech_or_peer(
    token: &crate::tokenizer::Token,
    matches: &[MatchSpan],
    current_property: Property,
) -> bool {
    matches.iter().any(|m| {
        m.start <= token.start
            && m.end >= token.end
            && (CONFIDENT_TECH.contains(&m.property) || m.property == current_property)
    })
}

/// Check whether a token is in a metadata position after a " - " separator.
///
/// The " - " pattern (space-dash-space) is a strong metadata delimiter in
/// media filenames: `"Movie Title - FR"`, `"Show - S01E02 - Episode Title"`.
///
/// A token preceded by " - " where the OTHER side of the dash also has a
/// space is in a metadata slot.
fn is_after_metadata_separator(
    _token: &crate::tokenizer::Token,
    token_idx: usize,
    tokens: &[crate::tokenizer::Token],
) -> bool {
    // The " - " pattern produces a token with Separator::Dash where the
    // previous token also ended with a space (or Separator::Space before dash).
    // In practice, we detect this by checking: this token has a Dash separator,
    // and the previous token also had Space or the token before that had Space.
    if token_idx == 0 {
        return false;
    }

    let prev = &tokens[token_idx - 1];

    // Check: current token is after a dash, and the dash was after a space.
    // This catches " - " patterns (the tokenizer splits "A - B" into
    // [A(sep=None), -(sep=Space), B(sep=Space)] or similar).
    // Since the tokenizer treats " - " as just separators between tokens,
    // we look for the pattern: prev_token then dash-space before current.
    // The simplest check: did the original input have " - " before this token?
    if let Some(token) = tokens.get(token_idx) {
        if token.start >= 3 {
            // We need access to the raw input here, but we only have tokens.
            // Instead, check the separator chain: if this token's separator
            // is Dash and the gap between prev.end and token.start includes
            // spaces, it's a " - " pattern.
            if token.separator == Separator::Dash || token.separator == Separator::Space {
                // Check if there's a dash in the gap between prev and current.
                let gap_start = prev.end;
                let gap_end = token.start;
                if gap_end > gap_start + 1 {
                    // Multi-character gap → likely " - "
                    return true;
                }
            }
        }
    }

    false
}

/// Check whether the same property+value appears in a clearly technical
/// position elsewhere in the path.
///
/// When "French" appears both in the title zone and the tech zone,
/// the title-zone instance is a title word (redundant with the tech one).
pub fn has_duplicate_in_tech_context(
    m: &MatchSpan,
    matches: &[MatchSpan],
    token_stream: &TokenStream,
) -> bool {
    matches.iter().any(|other| {
        other.property == m.property
            && other.value.to_lowercase() == m.value.to_lowercase()
            && other.start != m.start
            && is_in_tech_context(other, matches, token_stream)
    })
}
