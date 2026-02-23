//! Audio codec detection (AAC, DTS, Dolby, FLAC, etc.).

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

lazy_static! {
    static ref AUDIO_CODEC_PATTERNS: Vec<ValuePattern> = vec![
        // Order matters: more specific patterns first.
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-:]?X(?![a-z])", "DTS:X"),
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-]?HD(?:[-]?MA)?(?![a-z])", "DTS-HD"),
        ValuePattern::new(r"(?i)(?<![a-z])DTS(?![a-z:-])", "DTS"),
        ValuePattern::new(r"(?i)(?<![a-z])True[-]?HD(?![a-z])", "Dolby TrueHD"),
        ValuePattern::new(r"(?i)(?<![a-z])Dolby[-]?Atmos(?![a-z])", "Dolby Atmos"),
        ValuePattern::new(r"(?i)(?<![a-z])Atmos(?![a-z])", "Dolby Atmos"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:E[-]?AC[-]?3|DDP|DD\+)(?![a-z])", "Dolby Digital Plus"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Dolby(?:[-]?Digital)?|DD|AC[-]?3D?)(?![a-z+])", "Dolby Digital"),
        ValuePattern::new(r"(?i)(?<![a-z])AAC(?![a-z])", "AAC"),
        ValuePattern::new(r"(?i)(?<![a-z])FLAC(?![a-z])", "FLAC"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:MP3|LAME(?:\d+-?\d+)?)(?![a-z])", "MP3"),
        ValuePattern::new(r"(?i)(?<![a-z])MP2(?![a-z])", "MP2"),
        ValuePattern::new(r"(?i)(?<![a-z])Opus(?![a-z])", "Opus"),
        ValuePattern::new(r"(?i)(?<![a-z])Vorbis(?![a-z])", "Vorbis"),
        ValuePattern::new(r"(?i)(?<![a-z])PCM(?![a-z])", "PCM"),
        ValuePattern::new(r"(?i)(?<![a-z])LPCM(?![a-z])", "LPCM"),
    ];

    static ref AUDIO_CHANNELS_PATTERNS: Vec<ValuePattern> = vec![
        ValuePattern::new(r"(?i)(?<![a-z0-9])7[\W_]1(?:ch)?(?=[^\d]|$)", "7.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])5[\W_]1(?:ch)?(?=[^\d]|$)", "5.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])2[\W_]0(?:ch)?(?=[^\d]|$)", "2.0"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:stereo)(?![a-z])", "2.0"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:mono|1ch|1[\W_]0(?:ch)?)(?=[^\d]|$)", "1.0"),
    ];
}

pub struct AudioCodecMatcher;

impl PropertyMatcher for AudioCodecMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();
        for pattern in AUDIO_CODEC_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(start, end, Property::AudioCodec, pattern.value));
            }
        }
        for pattern in AUDIO_CHANNELS_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(start, end, Property::AudioChannels, pattern.value));
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aac() {
        let m = AudioCodecMatcher.find_matches("Movie.AAC.mkv");
        assert!(m.iter().any(|x| x.value == "AAC"));
    }

    #[test]
    fn test_dts_hd_ma() {
        let m = AudioCodecMatcher.find_matches("Movie.DTS-HD.MA.mkv");
        assert!(m.iter().any(|x| x.value == "DTS-HD"));
    }

    #[test]
    fn test_atmos() {
        let m = AudioCodecMatcher.find_matches("Movie.Atmos.mkv");
        assert!(m.iter().any(|x| x.value == "Dolby Atmos"));
    }

    #[test]
    fn test_channels_51() {
        let m = AudioCodecMatcher.find_matches("Movie.5.1.mkv");
        assert!(m.iter().any(|x| x.value == "5.1" && x.property == Property::AudioChannels));
    }

    #[test]
    fn test_eac3() {
        let m = AudioCodecMatcher.find_matches("Movie.EAC3.mkv");
        assert!(m.iter().any(|x| x.value == "Dolby Digital Plus"));
    }
}
