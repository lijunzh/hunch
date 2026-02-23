//! Match engine: collects matches from all property matchers, resolves conflicts.

use super::span::MatchSpan;

/// Collects all matches and resolves conflicts between overlapping spans.
pub struct MatchEngine;

impl MatchEngine {
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
                // Allow different properties to coexist on the same span
                // (e.g., Season + Episode from "S01E02").
                if matches[i].property != matches[j].property
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
        MatchEngine::resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_overlap_keeps_higher_priority() {
        let mut matches = vec![
            MatchSpan::new(0, 5, Property::Source, "web").with_priority(0),
            MatchSpan::new(2, 7, Property::Source, "other").with_priority(-1),
        ];
        MatchEngine::resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].value, "web");
    }

    #[test]
    fn test_overlap_keeps_longer_at_same_priority() {
        let mut matches = vec![
            MatchSpan::new(0, 3, Property::VideoCodec, "264"),
            MatchSpan::new(0, 5, Property::VideoCodec, "x264"),
        ];
        MatchEngine::resolve_conflicts(&mut matches);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].value, "x264");
    }
}
