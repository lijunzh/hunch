//! Frame rate detection.
//!
//! Detects frame rates like `24fps`, `25fps`, `120fps`, `29.97fps`,
//! or resolution-attached rates like `1080p25`.

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;
use std::sync::LazyLock;

/// Explicit fps patterns: `24fps`, `29.97fps`, `120fps`.
static FPS_PATTERNS: LazyLock<Vec<(fancy_regex::Regex, bool)>> = LazyLock::new(|| {
    vec![
        // Explicit: `24fps`, `120fps`
        (
            fancy_regex::Regex::new(r"(?i)(?<![a-z0-9])(\d+(?:\.\d+)?)\s*fps(?![a-z0-9])").unwrap(),
            true,
        ),
        // Resolution-attached: `1080p25`, `720p50`
        (
            fancy_regex::Regex::new(r"(?i)(?:1080|720|1440|2160)[pi](\d{2,3})(?![a-z0-9])")
                .unwrap(),
            false,
        ),
        // Standalone broadcast: `24p` at end or with separator
        (
            fancy_regex::Regex::new(r"(?i)(?<![a-z0-9])(\d{2,3})p(?![a-z0-9])").unwrap(),
            false,
        ),
    ]
});

/// Known frame rates for validation of ambiguous patterns.
const VALID_FRAME_RATES: &[&str] = &["23", "24", "25", "29", "30", "48", "50", "59", "60", "120"];

pub struct FrameRateMatcher;

impl PropertyMatcher for FrameRateMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        for (regex, is_explicit) in FPS_PATTERNS.iter() {
            let mut search_start = 0;
            while search_start < input.len() {
                let Some(cap) = regex.captures_from_pos(input, search_start).ok().flatten() else {
                    break;
                };
                let full = cap.get(0).unwrap();
                search_start = full.end();

                if let Some(m) = cap.get(1) {
                    let fps_val = &input[m.start()..m.end()];

                    // For ambiguous patterns, validate against known rates.
                    if !is_explicit && !VALID_FRAME_RATES.contains(&fps_val) {
                        continue;
                    }

                    // Skip values that look like screen sizes (720, 1080, etc.)
                    if !is_explicit
                        && matches!(fps_val, "720" | "1080" | "1440" | "2160" | "480" | "576")
                    {
                        continue;
                    }

                    matches.push(MatchSpan {
                        start: full.start(),
                        end: full.end(),
                        property: Property::FrameRate,
                        value: format!("{fps_val}fps"),
                        tags: vec![],
                        priority: if *is_explicit { 0 } else { -1 },
                    });
                }
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_24() {
        let m = FrameRateMatcher.find_matches("(1440p_24fps_H264)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "24fps");
    }

    #[test]
    fn fps_120() {
        let m = FrameRateMatcher.find_matches("19.1mbits - 120fps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "120fps");
    }

    #[test]
    fn resolution_attached_25() {
        let m = FrameRateMatcher.find_matches("MotoGP.2016x03.USA.Race.BTSportHD.1080p25");
        assert!(m.iter().any(|s| s.value == "25fps"));
    }

    #[test]
    fn no_false_positive_720p() {
        let m = FrameRateMatcher.find_matches("Movie.720p.mkv");
        assert!(m.is_empty());
    }
}
