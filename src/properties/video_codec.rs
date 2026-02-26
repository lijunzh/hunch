//! Video codec detection — now fully handled by `rules/video_codec.toml`.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn codec(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("video_codec").and_then(|v| v.as_str()).map(String::from)
    }

    #[test]
    fn test_h264() { assert_eq!(codec("Movie.x264.mkv"), Some("H.264".into())); }
    #[test]
    fn test_hevc() { assert_eq!(codec("Movie.HEVC.mkv"), Some("H.265".into())); }
    #[test]
    fn test_x265() { assert_eq!(codec("Movie.x265.mkv"), Some("H.265".into())); }
    #[test]
    fn test_av1() { assert_eq!(codec("Movie.AV1.mkv"), Some("AV1".into())); }
}
