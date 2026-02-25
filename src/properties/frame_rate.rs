//! Frame rate detection.
//!
//! Detects frame rates like `24fps`, `25fps`, `120fps`, `29.97fps`,
//! or resolution-attached rates like `1080p25`.

use crate::matcher::regex_utils::{BoundarySpec, CharClass, check_boundary};
use crate::matcher::span::{MatchSpan, Property};
use regex::Regex;
use std::sync::LazyLock;

static ALPHADIGIT_BOUNDARY: BoundarySpec = BoundarySpec {
    left: Some(CharClass::AlphaDigit),
    right: Some(CharClass::AlphaDigit),
};

static RIGHT_ONLY_BOUNDARY: BoundarySpec = BoundarySpec {
    left: None,
    right: Some(CharClass::AlphaDigit),
};

/// (regex, boundary, is_explicit)
static FPS_PATTERNS: LazyLock<Vec<(Regex, &'static BoundarySpec, bool)>> = LazyLock::new(|| {
    vec![
        // Explicit: `24fps`, `120fps`
        (
            Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*fps").unwrap(),
            &ALPHADIGIT_BOUNDARY,
            true,
        ),
        // Resolution-attached: `1080p25`, `720p50`
        (
            Regex::new(r"(?i)(?:1080|720|1440|2160)[pi](\d{2,3})").unwrap(),
            &RIGHT_ONLY_BOUNDARY,
            false,
        ),
        // Standalone broadcast: `24p`
        (
            Regex::new(r"(?i)(\d{2,3})p").unwrap(),
            &ALPHADIGIT_BOUNDARY,
            false,
        ),
    ]
});

/// Known frame rates for validation of ambiguous patterns.
const VALID_FRAME_RATES: &[&str] = &["23", "24", "25", "29", "30", "48", "50", "59", "60", "120"];

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let bytes = input.as_bytes();
    let mut matches = Vec::new();

    for (regex, boundary, is_explicit) in FPS_PATTERNS.iter() {
        let mut pos = 0;
        while pos < input.len() {
            let Some(cap) = regex.captures_at(input, pos) else {
                break;
            };
            let full = cap.get(0).unwrap();
            if !check_boundary(bytes, full.start(), full.end(), boundary) {
                pos = full.start() + 1;
                continue;
            }
            pos = full.end();

            if let Some(m) = cap.get(1) {
                let fps_val = &input[m.start()..m.end()];

                if !is_explicit && !VALID_FRAME_RATES.contains(&fps_val) {
                    continue;
                }

                if !is_explicit
                    && matches!(fps_val, "720" | "1080" | "1440" | "2160" | "480" | "576")
                {
                    continue;
                }

                matches.push(
                    MatchSpan::new(
                        full.start(),
                        full.end(),
                        Property::FrameRate,
                        format!("{fps_val}fps"),
                    )
                    .with_priority(if *is_explicit { 0 } else { -1 }),
                );
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_24() {
        let m = find_matches("(1440p_24fps_H264)");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "24fps");
    }

    #[test]
    fn fps_120() {
        let m = find_matches("19.1mbits - 120fps.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "120fps");
    }

    #[test]
    fn resolution_attached_25() {
        let m = find_matches("MotoGP.2016x03.USA.Race.BTSportHD.1080p25");
        assert!(m.iter().any(|s| s.value == "25fps"));
    }

    #[test]
    fn no_false_positive_720p() {
        let m = find_matches("Movie.720p.mkv");
        assert!(m.is_empty());
    }
}
