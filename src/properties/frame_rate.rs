//! Frame rate detection — now fully handled by `rules/frame_rate.toml`.
//!
//! The TOML rule engine handles:
//! - Explicit fps: `24fps`, `120fps`, `29.97fps`
//! - Resolution-attached: `1080p25`, `720p50`
//! - Standalone broadcast: `24p`, `50p`

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn fps(input: &str) -> Option<String> {
        let map = hunch(input).to_flat_map();
        map.get("frame_rate")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    #[test]
    fn fps_24() {
        assert_eq!(fps("(1440p_24fps_H264)"), Some("24fps".into()));
    }

    #[test]
    fn fps_120() {
        assert_eq!(fps("19.1mbits - 120fps.mkv"), Some("120fps".into()));
    }

    #[test]
    fn resolution_attached_25() {
        assert_eq!(
            fps("MotoGP.2016x03.USA.Race.BTSportHD.1080p25"),
            Some("25fps".into())
        );
    }

    #[test]
    fn no_false_positive_720p() {
        assert_eq!(fps("Movie.720p.mkv"), None);
    }
}
