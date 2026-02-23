//! Audio codec and channel detection (AAC, DTS, Dolby, FLAC, etc.).
//!
//! Handles combined patterns like `DD5.1`, `AAC2.0`, `TrueHD51` that
//! encode both codec + channel info in a single token.

use lazy_static::lazy_static;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use crate::properties::PropertyMatcher;

/// A combined codec+channel pattern that emits two MatchSpans.
struct CombinedPattern {
    vp: ValuePattern,
    codec: &'static str,
    channels: &'static str,
}

impl CombinedPattern {
    fn new(pattern: &str, codec: &'static str, channels: &'static str) -> Self {
        Self {
            vp: ValuePattern::new(pattern, codec),
            codec,
            channels,
        }
    }
}

lazy_static! {
    static ref AUDIO_CODEC_PATTERNS: Vec<ValuePattern> = vec![
        // Order matters: more specific patterns first.
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-:]?X(?![a-z])", "DTS:X"),
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-]?HD(?:[-. ]?(?:MA|Master(?:[-. ]?Audio)?))?(?![a-z])", "DTS-HD"),
        ValuePattern::new(r"(?i)(?<![a-z])DTS[-]?ES(?![a-z])", "DTS"),
        ValuePattern::new(r"(?i)(?<![a-z])DTS(?![a-z:-])", "DTS"),
        ValuePattern::new(r"(?i)(?<![a-z])True[-]?HD(?![a-z0-9])", "Dolby TrueHD"),
        ValuePattern::new(r"(?i)(?<![a-z])Dolby[-. ]?Atmos(?![a-z])", "Dolby Atmos"),
        ValuePattern::new(r"(?i)(?<![a-z])Atmos(?![a-z])", "Dolby Atmos"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:E[-]?AC[-]?3|DDP|DD\+)(?![a-z0-9])", "Dolby Digital Plus"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Dolby(?:[-. ]?Digital)?|DD|AC[-]?3D?)(?![a-z0-9+])", "Dolby Digital"),
        ValuePattern::new(r"(?i)(?<![a-z])AAC(?![a-z0-9])", "AAC"),
        ValuePattern::new(r"(?i)(?<![a-z])FLAC(?![a-z])", "FLAC"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:MP3|LAME(?:\d+-?\d+)?)(?![a-z])", "MP3"),
        ValuePattern::new(r"(?i)(?<![a-z])MP2(?![a-z])", "MP2"),
        ValuePattern::new(r"(?i)(?<![a-z])Opus(?![a-z])", "Opus"),
        ValuePattern::new(r"(?i)(?<![a-z])Vorbis(?![a-z])", "Vorbis"),
        ValuePattern::new(r"(?i)(?<![a-z])PCM(?![a-z])", "PCM"),
        ValuePattern::new(r"(?i)(?<![a-z])LPCM(?![a-z])", "LPCM"),
    ];

    /// Combined codec+channel patterns (checked BEFORE standalone channels).
    static ref COMBINED_PATTERNS: Vec<CombinedPattern> = vec![
        CombinedPattern::new(r"(?i)(?<![a-z])DD[-.]?5[\W_]?1(?![a-z0-9])", "Dolby Digital", "5.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])DD[-.]?51(?![a-z0-9])", "Dolby Digital", "5.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])DD[-.]?7[\W_]?1(?![a-z0-9])", "Dolby Digital", "7.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])True[-]?HD[-.]?51(?![a-z0-9])", "Dolby TrueHD", "5.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])True[-]?HD[-.]?5[\W_]1(?![a-z0-9])", "Dolby TrueHD", "5.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])AAC[-.]?2[\W_]?0(?![a-z0-9])", "AAC", "2.0"),
        CombinedPattern::new(r"(?i)(?<![a-z])AAC[-.]?20(?![a-z0-9])", "AAC", "2.0"),
        CombinedPattern::new(r"(?i)(?<![a-z])DDP[-.]?5[\W_]?1(?![a-z0-9])", "Dolby Digital Plus", "5.1"),
        CombinedPattern::new(r"(?i)(?<![a-z])DDP[-.]?51(?![a-z0-9])", "Dolby Digital Plus", "5.1"),
    ];

    static ref AUDIO_CHANNELS_PATTERNS: Vec<ValuePattern> = vec![
        // Explicit channel counts.
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:8ch|7[\W_]1(?:ch)?)(?=[^\d]|$)", "7.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])7ch(?=[^\d]|$)", "7.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:6ch|5[\W_]1(?:ch)?)(?=[^\d]|$)", "5.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])5ch(?=[^\d]|$)", "5.1"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:2ch|2[\W_]0(?:ch)?|stereo)(?=[^\d]|$)", "2.0"),
        ValuePattern::new(r"(?i)(?<![a-z0-9])(?:mono|1ch|1[\W_]0(?:ch)?)(?=[^\d]|$)", "1.0"),
    ];
}

pub struct AudioCodecMatcher;

impl PropertyMatcher for AudioCodecMatcher {
    fn find_matches(&self, input: &str) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // Combined codec+channels patterns first (higher priority).
        for cp in COMBINED_PATTERNS.iter() {
            for (start, end) in cp.vp.find_iter(input) {
                matches.push(
                    MatchSpan::new(start, end, Property::AudioCodec, cp.codec).with_priority(2),
                );
                matches.push(
                    MatchSpan::new(start, end, Property::AudioChannels, cp.channels)
                        .with_priority(2),
                );
            }
        }

        // Standalone codec patterns.
        for pattern in AUDIO_CODEC_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(
                    start,
                    end,
                    Property::AudioCodec,
                    pattern.value,
                ));
            }
        }

        // Standalone channel patterns.
        for pattern in AUDIO_CHANNELS_PATTERNS.iter() {
            for (start, end) in pattern.find_iter(input) {
                matches.push(MatchSpan::new(
                    start,
                    end,
                    Property::AudioChannels,
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
        assert!(
            m.iter()
                .any(|x| x.value == "5.1" && x.property == Property::AudioChannels)
        );
    }

    #[test]
    fn test_6ch_is_51() {
        let m = AudioCodecMatcher.find_matches("Movie.6ch.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "5.1" && x.property == Property::AudioChannels)
        );
    }

    #[test]
    fn test_eac3() {
        let m = AudioCodecMatcher.find_matches("Movie.EAC3.mkv");
        assert!(m.iter().any(|x| x.value == "Dolby Digital Plus"));
    }

    #[test]
    fn test_dd51_combined() {
        let m = AudioCodecMatcher.find_matches("Movie.DD5.1.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "Dolby Digital" && x.property == Property::AudioCodec)
        );
        assert!(
            m.iter()
                .any(|x| x.value == "5.1" && x.property == Property::AudioChannels)
        );
    }

    #[test]
    fn test_aac20_combined() {
        let m = AudioCodecMatcher.find_matches("Movie.AAC2.0.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "AAC" && x.property == Property::AudioCodec)
        );
        assert!(
            m.iter()
                .any(|x| x.value == "2.0" && x.property == Property::AudioChannels)
        );
    }

    #[test]
    fn test_truehd51_combined() {
        let m = AudioCodecMatcher.find_matches("Movie.TrueHD51.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "Dolby TrueHD" && x.property == Property::AudioCodec)
        );
        assert!(
            m.iter()
                .any(|x| x.value == "5.1" && x.property == Property::AudioChannels)
        );
    }
}
