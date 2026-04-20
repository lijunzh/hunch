//! Audio profile detection — now fully handled by `src/rules/audio_profile.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn profile(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("audio_profile")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_atmos() {
        assert_eq!(profile("Movie.Atmos.mkv"), Some("Atmos".into()));
    }
    #[test]
    fn test_truehd() {
        assert_eq!(profile("Movie.TrueHD.mkv"), Some("TrueHD".into()));
    }
    #[test]
    fn test_hd_ma() {
        assert_eq!(profile("Movie.DTS-HD.MA.mkv"), Some("Master Audio".into()));
    }
}
