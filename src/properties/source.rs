//! Source / origin detection (Blu-ray, WEB-DL, HDTV, DVD, etc.).
//!
//! Now fully handled by TOML rules (`rules/source.toml`) with side_effects
//! for Rip, Screener, and Reencoded flags. Ultra HD Blu-ray promotion for
//! wide-gap patterns (UHD...Bluray with 4+ tokens between) is handled by
//! zone_rules `subtitle_source_conflict`.

#[cfg(test)]
mod tests {
    use crate::hunch;
    use crate::matcher::span::Property;

    fn source(input: &str) -> Option<String> {
        hunch(input).first(Property::Source).map(String::from)
    }

    fn other(input: &str) -> Vec<String> {
        hunch(input).other().iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_bluray() {
        assert_eq!(source("Movie.BluRay.mkv"), Some("Blu-ray".into()));
        assert!(!other("Movie.BluRay.mkv").contains(&"Rip".to_string()));
    }

    #[test]
    fn test_dvdrip_emits_rip() {
        assert_eq!(source("Movie.DVDRip.mkv"), Some("DVD".into()));
        assert!(other("Movie.DVDRip.mkv").contains(&"Rip".to_string()));
    }

    #[test]
    fn test_brrip_emits_reencoded() {
        assert_eq!(source("Movie.BRRip.mkv"), Some("Blu-ray".into()));
        let o = other("Movie.BRRip.mkv");
        assert!(o.contains(&"Rip".to_string()));
        assert!(o.contains(&"Reencoded".to_string()));
    }

    #[test]
    fn test_webdl() {
        assert_eq!(source("Movie.WEB-DL.mkv"), Some("Web".into()));
    }

    #[test]
    fn test_webrip_emits_rip() {
        assert_eq!(source("Movie.WEBRip.mkv"), Some("Web".into()));
        assert!(other("Movie.WEBRip.mkv").contains(&"Rip".to_string()));
    }

    #[test]
    fn test_hdtv() {
        assert_eq!(source("Movie.HDTV.mkv"), Some("HDTV".into()));
    }

    #[test]
    fn test_hd_dvd() {
        assert_eq!(source("Movie.HDDVD.mkv"), Some("HD-DVD".into()));
    }

    #[test]
    fn test_dvdscr_emits_screener() {
        assert_eq!(source("Movie.DVDSCR.mkv"), Some("DVD".into()));
        assert!(other("Movie.DVDSCR.mkv").contains(&"Screener".to_string()));
    }

    #[test]
    fn test_uhd_bluray_promotion() {
        assert_eq!(
            source("Movie.UHD.2160p.BluRay.mkv"),
            Some("Ultra HD Blu-ray".into())
        );
    }
}
