//! Screen size detection — now fully handled by `rules/screen_size.toml`.
//!
//! The TOML rule engine handles:
//! - Standard: 720p, 1080p, 2160p, 480i, etc.
//! - Shorthands: 4K, 8K, UHD, FHD, QHD
//! - WxH: 1920x1080, 1280*720
//! - fps-attached: 1080p25, 720p60
//! - Bare res + profile: [720.Hi10p] via requires_after
//! - 4K edition guard: skips "4K Restored/Remastered" via not_before

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn screen(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("screen_size")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_1080p() {
        assert_eq!(screen("Movie.1080p.mkv"), Some("1080p".into()));
    }

    #[test]
    fn test_720p() {
        assert_eq!(screen("Movie.720p.mkv"), Some("720p".into()));
    }

    #[test]
    fn test_4k() {
        assert_eq!(screen("Movie.4K.mkv"), Some("2160p".into()));
    }

    #[test]
    fn test_2160p() {
        assert_eq!(screen("Movie.2160p.mkv"), Some("2160p".into()));
    }

    #[test]
    fn test_1080i() {
        assert_eq!(screen("Movie.1080i.mkv"), Some("1080i".into()));
    }

    #[test]
    fn test_explicit_1920x1080() {
        assert_eq!(screen("Movie.1920x1080.mkv"), Some("1080p".into()));
    }
}

#[cfg(test)]
mod regression_tests {
    use crate::hunch;

    fn screen(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("screen_size")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn test_480p_in_brackets() {
        let input = "[Kaylith] Zankyou no Terror - 04 [480p][B4D4514E].mp4";
        assert_eq!(screen(input), Some("480p".into()));
    }
}
