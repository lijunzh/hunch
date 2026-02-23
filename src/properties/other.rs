//! "Other" flags: HDR, Remux, Proper, Repack, 3D, etc.

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref OTHER_PATTERNS: Vec<ValuePattern> = vec![
        // HDR variants (most specific first).
        ValuePattern::new(r"(?i)(?<![a-z])Dolby[-.]?Vision(?![a-z])", "Dolby Vision"),
        ValuePattern::new(r"(?i)(?<![a-z])DV(?![a-z])", "Dolby Vision"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR10\+(?![a-z])", "HDR10+"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR10(?![a-z+])", "HDR10"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR(?![a-z0-9])", "HDR"),
        ValuePattern::new(r"(?i)(?<![a-z])SDR(?![a-z])", "SDR"),
        // Release quality flags.
        ValuePattern::new(r"(?i)(?<![a-z])Remux(?![a-z])", "Remux"),
        ValuePattern::new(r"(?i)(?<![a-z])PROPER(?![a-z])", "Proper"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:REPACK|RERIP)(?![a-z])", "Repack"),
        ValuePattern::new(r"(?i)(?<![a-z])REAL(?![a-z])", "Real"),
        ValuePattern::new(r"(?i)(?<![a-z])CONVERT(?![a-z])", "Convert"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:DUBBED|DUBS?)(?![a-z])", "Dubbed"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:SUBBED|SUBS?)(?![a-z])", "Subbed"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:HARDCODED|HC)[-.]?SUBS?(?![a-z])", "Hardcoded Subtitles"),
        // 3D.
        ValuePattern::new(r"(?i)(?<![a-z])3D(?![a-z])", "3D"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Half[-.]?)?(?:SBS|Side[-.]?by[-.]?Side)(?![a-z])", "3D"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Half[-.]?)?(?:OU|Over[-.]?Under)(?![a-z])", "3D"),
        // Mux / encode.
        ValuePattern::new(r"(?i)(?<![a-z])(?:Mux|Re[-]?Mux)(?![a-z])", "Mux"),
        ValuePattern::new(r"(?i)(?<![a-z])Hybrid(?![a-z])", "Hybrid"),
        // Extras / bonus.
        ValuePattern::new(r"(?i)(?<![a-z])Complete[-.]?Series(?![a-z])", "Complete Series"),
        ValuePattern::new(r"(?i)(?<![a-z])LiNE(?![a-z])", "Line Audio"),
        ValuePattern::new(r"(?i)(?<![a-z])Dual[-.]?Audio(?![a-z])", "Dual Audio"),
        ValuePattern::new(r"(?i)(?<![a-z])Multi(?![a-z])", "Multi Audio"),
        ValuePattern::new(r"(?i)(?<![a-z])INTERNAL(?![a-z])", "Internal"),
        ValuePattern::new(r"(?i)(?<![a-z])READ\.?NFO(?![a-z])", "Read NFO"),
        ValuePattern::new(r"(?i)(?<![a-z])SAMPLE(?![a-z])", "Sample"),
    ];
}

pub struct OtherMatcher;

impl PropertyMatcher for OtherMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in OTHER_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(start, end, Property::Other, pattern.value));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr10() {
        let m = OtherMatcher.find_matches("Movie.HDR10.mkv");
        assert!(m.iter().any(|x| x.value == "HDR10"));
    }

    #[test]
    fn test_remux() {
        let m = OtherMatcher.find_matches("Movie.Remux.mkv");
        assert!(m.iter().any(|x| x.value == "Remux"));
    }

    #[test]
    fn test_proper() {
        let m = OtherMatcher.find_matches("Movie.PROPER.mkv");
        assert!(m.iter().any(|x| x.value == "Proper"));
    }

    #[test]
    fn test_dual_audio() {
        let m = OtherMatcher.find_matches("Movie.Dual.Audio.mkv");
        assert!(m.iter().any(|x| x.value == "Dual Audio"));
    }

    #[test]
    fn test_dolby_vision() {
        let m = OtherMatcher.find_matches("Movie.Dolby.Vision.mkv");
        assert!(m.iter().any(|x| x.value == "Dolby Vision"));
    }
}
