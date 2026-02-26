//! Video profile detection — now fully handled by `rules/video_profile.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn profile(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("video_profile")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_high_10() {
        assert_eq!(profile("Movie.Hi10P.mkv"), Some("High 10".into()));
    }
    #[test]
    fn test_avchd() {
        assert_eq!(
            profile("Movie.AVCHD.mkv"),
            Some("Advanced Video Codec High Definition".into())
        );
    }
}
