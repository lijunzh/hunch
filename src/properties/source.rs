//! Source / origin detection (Blu-ray, WEB-DL, HDTV, DVD, etc.).

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref SOURCE_PATTERNS: Vec<ValuePattern> = vec![
        // Ultra HD Blu-ray (must come before Blu-ray).
        ValuePattern::new(r"(?i)(?<![a-z])Ultra[-.]?Blu[-.]?ray(?![a-z])", "Ultra HD Blu-ray"),
        ValuePattern::new(r"(?i)(?<![a-z])Blu[-.]?ray[-.]?Ultra(?![a-z])", "Ultra HD Blu-ray"),
        // Blu-ray variants.
        ValuePattern::new(r"(?i)(?<![a-z])(?:Blu[-.]?ray|BD|BD[59]|BD25|BD50)(?:[-.]?Rip)?(?![a-z])", "Blu-ray"),
        ValuePattern::new(r"(?i)(?<![a-z])BR[-.]?Rip(?![a-z])", "Blu-ray"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:BD|BR)[-.]?Remux(?![a-z])", "Blu-ray"),
        // Web sources.
        ValuePattern::new(r"(?i)(?<![a-z])WEB[-.]?DL(?![a-z])", "Web"),
        ValuePattern::new(r"(?i)(?<![a-z])WEB[-.]?Rip(?![a-z])", "Web"),
        ValuePattern::new(r"(?i)(?<![a-z])WEB[-.]?Cap(?![a-z])", "Web"),
        ValuePattern::new(r"(?i)(?<![a-z])DL[-.]?WEB(?![a-z])", "Web"),
        ValuePattern::new(r"(?i)(?<![a-z])WEB[-.]?UHD(?![a-z])", "Web"),
        ValuePattern::new(r"(?i)(?<![a-z])WEB(?![a-z])", "Web"),
        // HDTV.
        ValuePattern::new(r"(?i)(?<![a-z])UHD[-.]?TV(?:[-.]?Rip)?(?![a-z])", "Ultra HDTV"),
        ValuePattern::new(r"(?i)(?<![a-z])HD[-.]?TV(?:[-.]?Rip)?(?![a-z])", "HDTV"),
        ValuePattern::new(r"(?i)(?<![a-z])PD[-.]?TV(?:[-.]?Rip)?(?![a-z])", "Digital TV"),
        ValuePattern::new(r"(?i)(?<![a-z])DVB(?:[-.]?Rip)?(?![a-z])", "Digital TV"),
        // DVD.
        ValuePattern::new(r"(?i)(?<![a-z])DVD(?:[-.]?Rip)?(?![a-z])", "DVD"),
        ValuePattern::new(r"(?i)(?<![a-z])VIDEO[-.]?TS(?![a-z])", "DVD"),
        // HD-DVD.
        ValuePattern::new(r"(?i)(?<![a-z])HD[-.]?DVD(?:[-.]?Rip)?(?![a-z])", "HD-DVD"),
        // Satellite.
        ValuePattern::new(r"(?i)(?<![a-z])(?:DSR|DTH|SAT)[-.]?Rip(?![a-z])", "Satellite"),
        // Telecine / Telesync.
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD[-.]?)?TELECINE(?![a-z])", "Telecine"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD[-.]?)?TC(?:[-.]?Rip)?(?![a-z])", "Telecine"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD[-.]?)?TELESYNC(?![a-z])", "Telesync"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD[-.]?)?TS(?:[-.]?Rip)?(?![a-z])", "Telesync"),
        // Camera.
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD[-.]?)?CAM(?:[-.]?Rip)?(?![a-z])", "Camera"),
        // Screener.
        ValuePattern::new(r"(?i)(?<![a-z])(?:DVD|BD|BR)?[-.]?SCR(?:eener)?(?![a-z])", "Screener"),
        // PPV / VOD.
        ValuePattern::new(r"(?i)(?<![a-z])PPV(?:[-.]?Rip)?(?![a-z])", "Pay-per-view"),
        ValuePattern::new(r"(?i)(?<![a-z])VOD(?:[-.]?Rip)?(?![a-z])", "Video on Demand"),
        // VHS / Workprint.
        ValuePattern::new(r"(?i)(?<![a-z])VHS(?:[-.]?Rip)?(?![a-z])", "VHS"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:WORKPRINT|WP)(?![a-z])", "Workprint"),
        // SD TV (weak, must come last).
        ValuePattern::new(r"(?i)(?<![a-z])SD[-.]?TV(?:[-.]?Rip)?(?![a-z])", "TV"),
        ValuePattern::new(r"(?i)(?<![a-z])TV[-.]?Rip(?![a-z])", "TV"),
    ];
}

pub struct SourceMatcher;

impl PropertyMatcher for SourceMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in SOURCE_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(start, end, Property::Source, pattern.value));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluray() {
        let m = SourceMatcher.find_matches("Movie.BluRay.mkv");
        assert!(m.iter().any(|x| x.value == "Blu-ray"));
    }

    #[test]
    fn test_webdl() {
        let m = SourceMatcher.find_matches("Movie.WEB-DL.mkv");
        assert!(m.iter().any(|x| x.value == "Web"));
    }

    #[test]
    fn test_hdtv() {
        let m = SourceMatcher.find_matches("Movie.HDTV.mkv");
        assert!(m.iter().any(|x| x.value == "HDTV"));
    }

    #[test]
    fn test_dvd() {
        let m = SourceMatcher.find_matches("Movie.DVDRip.mkv");
        assert!(m.iter().any(|x| x.value == "DVD"));
    }

    #[test]
    fn test_webrip() {
        let m = SourceMatcher.find_matches("Movie.WEBRip.mkv");
        assert!(m.iter().any(|x| x.value == "Web"));
    }
}
