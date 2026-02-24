//! Video codec detection (H.264, H.265, HEVC, XviD, etc.).

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;
use std::sync::LazyLock;

static CODEC_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        ValuePattern::new(r"(?i)(?<![a-z])(?:x|h)[-.]?265(?![a-z])", "H.265"),
        ValuePattern::new(r"(?i)(?<![a-z])HEVC(?![a-z])", "H.265"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:x|h)[-.]?264(?![a-z])", "H.264"),
        // Handle `WEB-DLx264` where codec is glued to source suffix.
        ValuePattern::new(r"(?i)(?:DL|Rip|HD)(?:x|h)[-.]?264(?![a-z])", "H.264"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:MPEG-?4)?AVC(?:HD)?(?![a-z])", "H.264"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:x|h)[-.]?263(?![a-z])", "H.263"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:x|h)[-.]?262(?![a-z])", "MPEG-2"),
        ValuePattern::new(r"(?i)(?<![a-z])Mpe?g[-.]?2(?![a-z])", "MPEG-2"),
        ValuePattern::new(r"(?i)(?<![a-z])XviD(?![a-z])", "Xvid"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:DVD)?DivX(?![a-z])", "DivX"),
        ValuePattern::new(r"(?i)(?<![a-z])DVDivX(?![a-z])", "DivX"),
        ValuePattern::new(r"(?i)(?<![a-z])VC[-.]?1(?![a-z])", "VC-1"),
        ValuePattern::new(r"(?i)(?<![a-z])VP9(?![a-z])", "VP9"),
        ValuePattern::new(r"(?i)(?<![a-z])VP8(?:0)?(?![a-z])", "VP8"),
        ValuePattern::new(r"(?i)(?<![a-z])VP7(?![a-z])", "VP7"),
        ValuePattern::new(r"(?i)(?<![a-z])AV1(?![a-z])", "AV1"),
        ValuePattern::new(r"(?i)(?<![a-z])Rv\d{2}(?![a-z])", "RealVideo"),
    ]
});

pub struct VideoCodecMatcher;

impl PropertyMatcher for VideoCodecMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in CODEC_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(
                    start,
                    end,
                    Property::VideoCodec,
                    pattern.value,
                ));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h264() {
        let m = VideoCodecMatcher.find_matches("Movie.x264.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "H.264");
    }

    #[test]
    fn test_hevc() {
        let m = VideoCodecMatcher.find_matches("Movie.HEVC.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "H.265");
    }

    #[test]
    fn test_x265() {
        let m = VideoCodecMatcher.find_matches("Movie.x265.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "H.265");
    }

    #[test]
    fn test_av1() {
        let m = VideoCodecMatcher.find_matches("Movie.AV1.mkv");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, "AV1");
    }
}
