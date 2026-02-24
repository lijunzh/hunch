//! "Other" flags: HDR, Remux, Proper, Repack, 3D, region codes, etc.

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

static OTHER_PATTERNS: LazyLock<Vec<ValuePattern>> = LazyLock::new(|| {
    vec![
        // HDR variants (most specific first).
        ValuePattern::new(r"(?i)(?<![a-z])Dolby[-. ]?Vision(?![a-z])", "Dolby Vision"),
        ValuePattern::new(r"(?i)(?<![a-z])DV(?![a-z])", "Dolby Vision"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR10\+(?![a-z])", "HDR10+"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR10(?![a-z+])", "HDR10"),
        ValuePattern::new(r"(?i)(?<![a-z])HDR(?![a-z0-9])", "HDR10"),
        ValuePattern::new(r"(?i)(?<![a-z])SDR(?![a-z])", "Standard Dynamic Range"),
        ValuePattern::new(r"(?i)(?<![a-z])BT[-. ]?2020(?![0-9])", "BT.2020"),
        // Quality / resolution flags.
        ValuePattern::new(r"(?i)(?<![a-z])(?:Full[-. ]?HD|FHD)(?![a-z])", "Full HD"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:Ultra(?:[-. ]?HD)?|UHD)(?![a-z])", "Ultra HD"),
        ValuePattern::new(r"(?i)(?<![a-z])Netflix[-. ]?UHD(?![a-z])", "Ultra HD"),
        ValuePattern::new(r"(?i)(?<![a-z])iTunes[-. ]?HD(?![a-z])", "HD"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:mHD|HDLight)(?![a-z])", "Micro HD"),
        ValuePattern::new(r"(?i)(?<![a-z0-9\-])HD(?![a-zTV0-9\-])", "HD"),
        ValuePattern::new(r"(?i)(?<![a-z])HQ(?![a-z])", "High Quality"),
        ValuePattern::new(r"(?i)(?<![a-z])HR(?![a-z])", "High Resolution"),
        ValuePattern::new(r"(?i)(?<![a-z])LDTV(?![a-z])", "Low Definition"),
        ValuePattern::new(r"(?i)(?<![a-z])Upscale[d]?(?![a-z])", "Upscaled"),
        // Release quality flags.
        ValuePattern::new(r"(?i)(?<![a-z])Remux(?![a-z])", "Remux"),
        ValuePattern::new(r"(?i)(?<![a-z])PROPER(?![a-z])", "Proper"),
        ValuePattern::new(r"(?i)(?<![a-z])(?:REPACK|RERIP)\d*(?![a-z])", "Proper"),
        ValuePattern::new(r"(?i)(?<![a-z])REAL(?![a-z])", "Proper"),
        // Reencoded (explicit + BRRip implies re-encoding from Blu-ray).
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:re[-. ]?enc(?:oded)?|reencoded|BRRip)(?![a-z])",
            "Reencoded",
        ),
        // Converted.
        ValuePattern::new(r"(?i)(?<![a-z])CONVERT(?:ED)?(?![a-z])", "Converted"),
        // Fix variants.
        ValuePattern::new(
            r"(?i)(?<![a-z])Audio[-. ]?Fix(?:ed)?(?![a-z])",
            "Audio Fixed",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])Sync[-. ]?Fix(?:ed)?(?![a-z])", "Sync Fixed"),
        // Dub / Sub flags (require explicit markers, not bare words).
        ValuePattern::new(r"(?i)(?<![a-z])DUBBED(?![a-z])", "Dubbed"),
        ValuePattern::new(r"(?i)(?<![a-z])HC(?![a-z])(?=.*(?:SUB|sub|SUBS|subs))", "Hardcoded Subtitles"),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:HARDCODED|HC)[-. ]?SUBS?(?![a-z])",
            "Hardcoded Subtitles",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])Fan[-. ]?Sub(?:bed|titled|s)?(?![a-z])",
            "Fan Subtitled",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])Fast[-. ]?Sub(?:bed|titled|s)?(?![a-z])",
            "Fast Subtitled",
        ),
        // Widescreen.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Wide[-. ]?Screen|WS)(?![a-z])",
            "Widescreen",
        ),
        // Dual / Multi audio.
        ValuePattern::new(r"(?i)(?<![a-z])Dual[-. ]?Audio(?![a-z])", "Dual Audio"),
        ValuePattern::new(
            r"(?i)(?<![a-z])Dual(?=[-. ]?(?:DVD|BD|BR|WEB|BluRay|Sub))(?![a-z])",
            "Dual Audio",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])Multi[-. ]?Audio(?![a-z])", "Multi Audio"),
        ValuePattern::new(r"(?<![a-zA-Z])LiNE(?![a-zA-Z])", "Line Audio"),
        // Dubbing quality.
        ValuePattern::new(r"(?i)(?<![a-z])LD(?![a-z])", "Line Dubbed"),
        ValuePattern::new(r"(?i)(?<![a-z])MD(?![a-z])", "Mic Dubbed"),
        // 3D.
        ValuePattern::new(r"(?i)(?<![a-z])3D(?![a-z])", "3D"),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Half[-. ]?)?(?:SBS|Side[-. ]?by[-. ]?Side)(?![a-z])",
            "3D",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Half[-. ]?)?(?:OU|Over[-. ]?Under)(?![a-z])",
            "3D",
        ),
        // TV standards.
        ValuePattern::new(r"(?i)(?<![a-z])PAL(?![a-z])", "PAL"),
        ValuePattern::new(r"(?i)(?<![a-z])NTSC(?![a-z])", "NTSC"),
        ValuePattern::new(r"(?i)(?<![a-z])SECAM(?![a-z])", "SECAM"),
        // Region codes.
        ValuePattern::new(r"(?i)(?<![a-z])R5(?![a-z0-9])", "Region 5"),
        ValuePattern::new(r"(?i)(?<![a-z])RC(?![a-z0-9])", "Region C"),
        // Screener (standalone only — DVD/BD/BR/WEB prefixed forms are handled by source).
        ValuePattern::new(r"(?i)(?<![a-z])Screener(?![a-z])", "Screener"),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:Advance[-. ]?)?Screener(?![a-z])",
            "Screener",
        ),
        // Mux / encode.
        ValuePattern::new(r"(?i)(?<![a-z])Hybrid(?![a-z])", "Hybrid"),
        // Extras / bonus / complete.
        ValuePattern::new(r"(?i)(?<![a-z])PreAir(?![a-z])", "Preair"),
        ValuePattern::new(r"(?i)(?<![a-z])Pre[-. ]?Air(?![a-z])", "Preair"),
        // 2in1.
        ValuePattern::new(r"(?i)(?<![a-z])2in1(?![a-z])", "2in1"),
        // Internal / sample / NFO.
        ValuePattern::new(r"(?i)(?<![a-z])INTERNAL(?![a-z])", "Internal"),
        ValuePattern::new(r"(?i)(?<![a-z])READ\.?NFO(?![a-z])", "Read NFO"),
        ValuePattern::new(r"(?i)(?<![a-z])SAMPLE(?![a-z])", "Sample"),
        // Mux.
        ValuePattern::new(r"(?i)(?<![a-z])(?:DivX|XviD)?[-.]?Mux(?![a-z])", "Mux"),
        // Repost / Obfuscated.
        ValuePattern::new(r"(?i)(?<![a-z])REPOST(?![a-z])", "Repost"),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:OBFUSCATED|Obfuscation|scrambled)(?![a-z])",
            "Obfuscated",
        ),
        // Complete (broader patterns).
        ValuePattern::new(r"(?i)(?<![a-z])COMPLETE(?![a-z])", "Complete"),
        ValuePattern::new(r"(?i)(?<![a-z])COMPLET(?![a-z])", "Complete"),
        // Straight to Video.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:STV|Straight[-. ]?to[-. ]?Video)(?![a-z])",
            "Straight to Video",
        ),
        // Fix (generic and specific).
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:DIRFIX|NFOFIX|SAMPLEFIX|PROOFFIX)(?![a-z])",
            "Fix",
        ),
        ValuePattern::new(r"(?i)(?<![a-z])FIX(?![a-z])", "Fix"),
        // XXX.
        ValuePattern::new(r"(?i)(?<![a-z])XXX(?![a-z])", "XXX"),
        // Open Matte.
        ValuePattern::new(r"(?i)(?<![a-z])Open[-. ]?Matte(?![a-z])", "Open Matte"),
        // Extras / Bonus.
        ValuePattern::new(r"(?i)(?<![a-z])EXTRAS?(?![a-z])", "Extras"),
        // Documentary.
        ValuePattern::new(r"(?i)(?<![a-z])DOCU(?:MENTARY)?(?![a-z])", "Documentary"),
        // Original Video.
        ValuePattern::new(r"(?i)(?<![a-z])OVA(?![a-z])", "Original Video"),
        ValuePattern::new(r"(?i)(?<![a-z])OAD(?![a-z])", "Original Animation DVD"),
        // Original Aspect Ratio.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:OAR|Original[-. ]?Aspect[-. ]?Ratio)(?![a-z])",
            "Original Aspect Ratio",
        ),
        // Colorized.
        ValuePattern::new(r"(?i)(?<![a-z])Colorized(?![a-z])", "Colorized"),
        // East/West Coast Feed.
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:East|EST)[-. ]?(?:Coast[-. ]?)?Feed(?![a-z])",
            "East Coast Feed",
        ),
        ValuePattern::new(
            r"(?i)(?<![a-z])(?:West|WST)[-. ]?(?:Coast[-. ]?)?Feed(?![a-z])",
            "West Coast Feed",
        ),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for pattern in OTHER_PATTERNS.iter() {
        for (start, end) in pattern.find_iter(input) {
            matches.push(MatchSpan::new(start, end, Property::Other, pattern.value));
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdr10() {
        let m = find_matches("Movie.HDR10.mkv");
        assert!(m.iter().any(|x| x.value == "HDR10"));
    }

    #[test]
    fn test_hdr_maps_to_hdr10() {
        let m = find_matches("Movie.HDR.mkv");
        assert!(m.iter().any(|x| x.value == "HDR10"));
    }

    #[test]
    fn test_remux() {
        let m = find_matches("Movie.Remux.mkv");
        assert!(m.iter().any(|x| x.value == "Remux"));
    }

    #[test]
    fn test_proper() {
        let m = find_matches("Movie.PROPER.mkv");
        assert!(m.iter().any(|x| x.value == "Proper"));
    }

    #[test]
    fn test_repack_is_proper() {
        let m = find_matches("Movie.REPACK.mkv");
        assert!(m.iter().any(|x| x.value == "Proper"));
    }

    #[test]
    fn test_dual_audio() {
        let m = find_matches("Movie.Dual.Audio.mkv");
        assert!(m.iter().any(|x| x.value == "Dual Audio"));
    }

    #[test]
    fn test_dolby_vision() {
        let m = find_matches("Movie.Dolby.Vision.mkv");
        assert!(m.iter().any(|x| x.value == "Dolby Vision"));
    }

    #[test]
    fn test_region_5() {
        let m = find_matches("Movie.R5.mkv");
        assert!(m.iter().any(|x| x.value == "Region 5"));
    }

    #[test]
    fn test_widescreen() {
        let m = find_matches("Movie.WideScreen.mkv");
        assert!(m.iter().any(|x| x.value == "Widescreen"));
    }

    #[test]
    fn test_pal() {
        let m = find_matches("Movie.PAL.mkv");
        assert!(m.iter().any(|x| x.value == "PAL"));
    }

    #[test]
    fn test_sdr() {
        let m = find_matches("Movie.SDR.mkv");
        assert!(m.iter().any(|x| x.value == "Standard Dynamic Range"));
    }

    #[test]
    fn test_reencoded() {
        let m = find_matches("Movie.re-enc.mkv");
        assert!(m.iter().any(|x| x.value == "Reencoded"));
    }

    #[test]
    fn test_bdrip_no_reencoded() {
        let m = find_matches("Movie.BDRip.720p.mkv");
        let reenc: Vec<_> = m.iter().filter(|x| x.value == "Reencoded").collect();
        assert!(
            reenc.is_empty(),
            "BDRip should not trigger Reencoded, got: {:?}",
            reenc
        );
    }

    #[test]
    fn test_complete_season() {
        let m = find_matches("Movie.Season.Complete.mkv");
        assert!(m.iter().any(|x| x.value == "Complete"));
    }
}
