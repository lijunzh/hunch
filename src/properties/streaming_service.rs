//! Streaming service detection.

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref STREAMING_PATTERNS: Vec<ValuePattern> = vec![
        ValuePattern::new(r"(?i)(?<![a-z])AMZN(?![a-z])", "Amazon Prime"),
        ValuePattern::new(r"(?i)(?<![a-z])Amazon(?:HD)?(?![a-z])", "Amazon Prime"),
        ValuePattern::new(r"(?i)(?<![a-z])NFLX(?![a-z])", "Netflix"),
        ValuePattern::new(r"(?i)(?<![a-z])Netflix(?:UHD)?(?:Rip)?(?![a-z])", "Netflix"),
        ValuePattern::new(r"(?i)(?<![a-z])NF(?![a-z])", "Netflix"),
        ValuePattern::new(r"(?i)(?<![a-z])ATVP(?![a-z])", "Apple TV+"),
        ValuePattern::new(r"(?i)(?<![a-z])DSNP(?![a-z])", "Disney+"),
        ValuePattern::new(r"(?i)(?<![a-z])Disney\+(?![a-z])", "Disney+"),
        ValuePattern::new(r"(?i)(?<![a-z])HMAX(?![a-z])", "HBO Max"),
        ValuePattern::new(r"(?i)(?<![a-z])HULU(?![a-z])", "Hulu"),
        ValuePattern::new(r"(?i)(?<![a-z])PCOK(?![a-z])", "Peacock"),
        ValuePattern::new(r"(?i)(?<![a-z])PMTP(?![a-z])", "Paramount+"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:HD)?iTunes(?:HD)?(?![a-z])", "iTunes"),
        ValuePattern::new(r"(?i)(?<![a-z])VUDU(?![a-z])", "Vudu"),
        ValuePattern::new(r"(?i)(?<![a-z])CRAV(?![a-z])", "Crave"),
        ValuePattern::new(r"(?i)(?<![a-z])DCU(?![a-z])", "DC Universe"),
        ValuePattern::new(r"(?i)(?<![a-z])DSCP(?![a-z])", "DramaFever"),
        ValuePattern::new(r"(?i)(?<![a-z])DramaFever(?![a-z])", "DramaFever"),
        ValuePattern::new(r"(?i)(?<![a-z])DF(?![a-z])", "DramaFever"),
        ValuePattern::new(r"(?i)(?<![a-z])VIKI(?![a-z])", "Viki"),
        ValuePattern::new(r"(?i)(?<![a-z])A&E(?![a-z])", "A&E"),
        ValuePattern::new(r"(?i)(?<![a-z])AE\.(?=WEB)", "A&E"),
        ValuePattern::new(r"(?i)(?<![a-z])MBCVOD(?![a-z])", "MBC"),
        ValuePattern::new(r"(?i)(?<![a-z])MBC(?![a-z])", "MBC"),
    ];
}

pub struct StreamingServiceMatcher;

impl PropertyMatcher for StreamingServiceMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in STREAMING_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(
                    MatchSpan::new(start, end, Property::StreamingService, pattern.value)
                        .with_priority(1),
                );
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amzn() {
        let m = StreamingServiceMatcher.find_matches("Movie.1080p.AMZN.WEB-DL.mkv");
        assert!(m.iter().any(|x| x.value == "Amazon Prime"));
    }

    #[test]
    fn test_netflix() {
        let m = StreamingServiceMatcher.find_matches("Show.S01E01.NFLX.WEB-DL.mkv");
        assert!(m.iter().any(|x| x.value == "Netflix"));
    }
}
