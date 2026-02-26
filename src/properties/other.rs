//! "Other" flags: HDR, Remux, Proper, Repack, 3D, region codes, etc.
//!
//! Now fully handled by TOML rules:
//! - `rules/other.toml`: unambiguous tokens (Remux, HDR10, Proper, etc.)
//! - `rules/other_positional.toml`: position-dependent tokens (HD, 3D, Proof, DV)
//!   with `zone_scope = "tech_only"` to suppress in title zones.

#[cfg(test)]
mod tests {
    use crate::hunch;

    fn other(input: &str) -> Vec<String> {
        hunch(input).other().iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_hdr10() {
        assert!(other("Movie.HDR10.mkv").contains(&"HDR10".to_string()));
    }

    #[test]
    fn test_hdr_maps_to_hdr10() {
        assert!(other("Movie.HDR.mkv").contains(&"HDR10".to_string()));
    }

    #[test]
    fn test_remux() {
        assert!(other("Movie.Remux.mkv").contains(&"Remux".to_string()));
    }

    #[test]
    fn test_proper() {
        assert!(other("Movie.PROPER.mkv").contains(&"Proper".to_string()));
    }

    #[test]
    fn test_repack_is_proper() {
        assert!(other("Movie.REPACK.mkv").contains(&"Proper".to_string()));
    }

    #[test]
    fn test_dual_audio() {
        assert!(other("Movie.Dual.Audio.mkv").contains(&"Dual Audio".to_string()));
    }

    #[test]
    fn test_dolby_vision() {
        assert!(other("Movie.Dolby.Vision.mkv").contains(&"Dolby Vision".to_string()));
    }

    #[test]
    fn test_region_5() {
        assert!(other("Movie.R5.mkv").contains(&"Region 5".to_string()));
    }

    #[test]
    fn test_widescreen() {
        assert!(other("Movie.WideScreen.mkv").contains(&"Widescreen".to_string()));
    }

    #[test]
    fn test_pal() {
        assert!(other("Movie.PAL.mkv").contains(&"PAL".to_string()));
    }

    #[test]
    fn test_sdr() {
        assert!(other("Movie.SDR.mkv").contains(&"Standard Dynamic Range".to_string()));
    }

    #[test]
    fn test_complete_season() {
        assert!(other("Movie.Season.Complete.mkv").contains(&"Complete".to_string()));
    }
}
