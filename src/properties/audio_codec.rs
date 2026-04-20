//! Audio codec and channel detection — now handled by TOML rules.
//!
//! - `src/rules/audio_codec.toml`: codec patterns + combined codec+channel side_effects
//! - `src/rules/audio_channels.toml`: standalone channel count patterns
//! - `src/rules/audio_profile.toml`: codec profile patterns (MA, Atmos, etc.)

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn codec(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("audio_codec")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    fn channels(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("audio_channels")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_channels_51() {
        assert_eq!(channels("Movie.5.1.mkv"), Some("5.1".into()));
    }

    #[test]
    fn test_6ch_is_51() {
        assert_eq!(channels("Movie.6ch.mkv"), Some("5.1".into()));
    }

    #[test]
    fn test_dd51_combined() {
        assert_eq!(codec("Movie.DD5.1.mkv"), Some("Dolby Digital".into()));
        assert_eq!(channels("Movie.DD5.1.mkv"), Some("5.1".into()));
    }

    #[test]
    fn test_aac20_combined() {
        assert_eq!(codec("Movie.AAC2.0.mkv"), Some("AAC".into()));
        assert_eq!(channels("Movie.AAC2.0.mkv"), Some("2.0".into()));
    }

    #[test]
    fn test_truehd51_combined() {
        assert_eq!(codec("Movie.TrueHD51.mkv"), Some("Dolby TrueHD".into()));
        assert_eq!(channels("Movie.TrueHD51.mkv"), Some("5.1".into()));
    }
}
