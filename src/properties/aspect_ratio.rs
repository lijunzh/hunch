//! Aspect ratio detection.
//!
//! Aspect ratio is derived from explicit WxH resolution patterns.
//! When we see "1920x1080", we compute 1920/1080 = 1.778.

use lazy_static::lazy_static;
use regex::Regex;

use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    /// Matches WxH resolution: 1920x1080, 640x480, etc.
    static ref RESOLUTION_WXH: Regex = Regex::new(
        r"(?i)(?P<w>[0-9]{3,4})\s*[xX×]\s*(?P<h>[0-9]{3,4})(?:i|p)?"
    ).unwrap();
}

pub struct AspectRatioMatcher;

impl PropertyMatcher for AspectRatioMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        for cap in RESOLUTION_WXH.captures_iter(input) {
            if let (Some(w), Some(h)) = (cap.name("w"), cap.name("h")) {
                let width: f64 = w.as_str().parse().unwrap_or(0.0);
                let height: f64 = h.as_str().parse().unwrap_or(0.0);
                if height > 0.0 {
                    let ratio = width / height;
                    let formatted = format!("{:.3}", ratio);
                    let full = cap.get(0).unwrap();
                    matches.push(
                        MatchSpan::new(full.start(), full.end(), Property::AspectRatio, formatted)
                            .with_priority(-1),
                    );
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
    fn test_1920x1080() {
        let m = AspectRatioMatcher.find_matches("Movie.1920x1080.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1.778");
    }

    #[test]
    fn test_640x480() {
        let m = AspectRatioMatcher.find_matches("Movie.640x480.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "1.333");
    }
}
