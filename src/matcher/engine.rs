//! Conflict resolution for overlapping match spans.

use super::span::{MatchSpan, Property};

/// Resolve conflicts: when two matches overlap, keep the one with higher
/// priority; if tied, keep the longer (more specific) match.
pub fn resolve_conflicts(matches: &mut Vec<MatchSpan>) {
    if matches.len() < 2 {
        return;
    }

    // Sort by start position, then by priority descending, then by length descending.
    matches.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then(b.priority.cmp(&a.priority))
            .then(b.len().cmp(&a.len()))
    });

    let mut keep = vec![true; matches.len()];

    for i in 0..matches.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..matches.len() {
            if !keep[j] {
                continue;
            }
            // Allow different properties to coexist on the same or overlapping spans
            // (e.g., Season + Episode from "S01E02", Source + Other:Rip from "DVDRip").
            if matches[i].property != matches[j].property {
                continue;
            }
            // Allow same-span Other with different values (e.g., Other:Rip + Other:Reencoded).
            // Also allow same-span Episode/Season with different values (multi-episode/season).
            if (matches[i].property == Property::Other
                || matches[i].property == Property::Episode
                || matches[i].property == Property::Season)
                && matches[i].value != matches[j].value
                && matches[i].start == matches[j].start
                && matches[i].end == matches[j].end
            {
                continue;
            }
            if matches[i].overlaps(&matches[j]) {
                // Higher priority wins; if tied, longer match wins.
                if matches[j].priority > matches[i].priority {
                    keep[i] = false;
                    break;
                }
                keep[j] = false;
            }
        }
    }

    let mut idx = 0;
    matches.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::span::Property;

    #[test]
    fn test_no_conflict() {
        let mut matches = vec![
            MatchSpan::new(0, 5, Property::Title, "Hello"),
            MatchSpan::new(6, 10, Property::Year, "2020"),
        ];
        resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_overlap_keeps_higher_priority() {
        let mut matches = vec![
            MatchSpan::new(0, 5, Property::Source, "web").with_priority(0),
            MatchSpan::new(2, 7, Property::Source, "other").with_priority(-1),
        ];
        resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].value, "web");
    }

    #[test]
    fn test_overlap_keeps_longer_at_same_priority() {
        let mut matches = vec![
            MatchSpan::new(0, 3, Property::VideoCodec, "264"),
            MatchSpan::new(0, 5, Property::VideoCodec, "x264"),
        ];
        resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].value, "x264");
    }
}
