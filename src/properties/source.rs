//! Source / origin detection (Blu-ray, WEB-DL, HDTV, DVD, etc.).
//!
//! When a source includes "Rip" (e.g., DVDRip), we emit BOTH
//! `Source` and `Other: "Rip"` to match guessit's behavior.

use fancy_regex::Regex;

use crate::matcher::regex_utils::ValuePattern;
use crate::matcher::span::{MatchSpan, Property};
use std::sync::LazyLock;

/// A source pattern that may also flag "Rip".
struct SourcePattern {
    vp: ValuePattern,
    /// Does the base (non-Rip) form exist as a separate match?
    has_rip_variant: bool,
}

impl SourcePattern {
    fn plain(pattern: &str, value: &'static str) -> Self {
        Self {
            vp: ValuePattern::new(pattern, value),
            has_rip_variant: false,
        }
    }
    fn with_rip(pattern: &str, value: &'static str) -> Self {
        Self {
            vp: ValuePattern::new(pattern, value),
            has_rip_variant: true,
        }
    }
}

/// Detects whether the matched text ends with "Rip" or "Cap" (capture = rip).
static REENCODED_RIP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^BR[-.]?Rip$").unwrap());

static RIP_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:Rip|Cap)$").unwrap());

/// BRRip/BDRip are re-encoded from Blu-ray (not direct rips like BluRay.Rip).
static SOURCE_PATTERNS: LazyLock<Vec<SourcePattern>> = LazyLock::new(|| {
    vec![
        // Ultra HD Blu-ray (must come before Blu-ray).
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:UHD|Ultra[-. ]?HD)[-. ]?(?:Blu[-.]?ray|BD|BR)(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:Blu[-.]?ray|BD|BR)[-. ]?(?:UHD|Ultra(?:[-. ]?HD)?)(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:4K|2160p)[-. ]?(?:Blu[-.]?ray|BD|BR)(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:Blu[-.]?ray|BD|BR)[-. ]?(?:4K|2160p)(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        // UHD with BRRip/BDRip (may have other tokens between).
        SourcePattern::plain(
            r"(?i)(?<![a-z])UHD(?:.{0,20})(?:BR|BD)(?:Rip)?(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])UHD(?:.{0,20})(?:BR|BD)[-.]?Rip(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:BR|BD)[-.]?Rip[^\n]{0,20}UHD(?![a-z])",
            "Ultra HD Blu-ray",
        ),
        // Blu-ray variants.
        SourcePattern::plain(
            r"(?i)(?<![a-z])(?:Blu[-.]?ray|BD[59R]?|BD25|BD50)(?![a-z])",
            "Blu-ray",
        ),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])(?:Blu[-.]?ray|BD)[-.]?Rip(?![a-z])",
            "Blu-ray",
        ),
        SourcePattern::with_rip(r"(?i)(?<![a-z])BR[-.]?Rip(?![a-z])", "Blu-ray"),
        SourcePattern::plain(r"(?i)(?<![a-z])(?:BD|BR)[-.]?Remux(?![a-z])", "Blu-ray"),
        SourcePattern::plain(r"(?i)(?<![a-z])BR[-.]?Scr(?:eener)?(?![a-z])", "Blu-ray"),
        // Web sources.
        SourcePattern::plain(r"(?i)(?<![a-z])WEB[-.]?DL(?![a-z])", "Web"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])WEB[-.]?(?:DL[-.]?)?Rip(?![a-z])", "Web"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])WEB[-.]?Cap(?:[-.]?Rip)?(?![a-z])", "Web"),
        SourcePattern::plain(r"(?i)(?<![a-z])WEB[-.]?(?:UHD|HD)(?![a-z])", "Web"),
        SourcePattern::plain(r"(?i)(?<![a-z])DL[-.]?WEB(?![a-z])", "Web"),
        SourcePattern::plain(r"(?i)(?<![a-z])WEB(?![a-z])", "Web"),
        SourcePattern::plain(r"(?i)(?<![a-z])DL[-.]?Mux(?![a-z])", "Web"),
        // HDTV.
        SourcePattern::plain(r"(?i)(?<![a-z])AHDTV(?![a-z])", "Analog HDTV"),
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-.]?TV(?![a-z])", "HDTV"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])HD[-.]?TV[-.]?Rip(?![a-z])", "HDTV"),
        SourcePattern::plain(r"(?i)(?<![a-z])UHD[-.]?TV(?![a-z])", "Ultra HDTV"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])UHD[-.]?(?:TV[-.]?)?Rip(?![a-z])",
            "Ultra HDTV",
        ),
        SourcePattern::plain(r"(?i)(?<![a-z])PD[-.]?TV(?![a-z])", "Digital TV"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])(?:PD[-.]?TV|DVB)[-.]?Rip(?![a-z])",
            "Digital TV",
        ),
        SourcePattern::plain(r"(?i)(?<![a-z])DVB(?![a-z])", "Digital TV"),
        // DVD.
        SourcePattern::plain(r"(?i)(?<![a-z])VIDEO[-._\s]?TS(?![a-z])", "DVD"),
        SourcePattern::plain(r"(?i)(?<![a-z])DVD(?:R|\s*[59])?(?![a-z])", "DVD"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])DVD[-.]?Rip(?![a-z])", "DVD"),
        // HD-DVD.
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-. ]?DVD(?![a-z])", "HD-DVD"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])HD[-. ]?DVD[-.]?Rip(?![a-z])", "HD-DVD"),
        // Satellite.
        SourcePattern::plain(r"(?i)(?<![a-z])(?:DSR|DTH)(?![a-z])", "Satellite"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])(?:DSR?|DTH|SAT)[-. ]?Rip(?![a-z])",
            "Satellite",
        ),
        // Telecine / Telesync.
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-. ]?TELECINE(?![a-z])", "HD Telecine"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])HD[-. ]?(?:TELECINE|TC)[-. ]?Rip(?![a-z])",
            "HD Telecine",
        ),
        SourcePattern::plain(r"(?i)(?<![a-z])HDTC(?![a-z])", "HD Telecine"),
        SourcePattern::plain(r"(?i)(?<![a-z])TELECINE(?![a-z])", "Telecine"),
        SourcePattern::plain(r"(?i)(?<![a-z])TC(?![a-z])", "Telecine"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])(?:TELECINE|TC)[-. ]?Rip(?![a-z])",
            "Telecine",
        ),
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-. ]?TELESYNC(?![a-z])", "HD Telesync"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])HD[-. ]?TS[-.]?Rip(?![a-z])", "HD Telesync"),
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-. ]TS(?![a-z0-9])", "HD Telesync"),
        SourcePattern::plain(r"(?i)(?<![a-z])TELESYNC(?![a-z])", "Telesync"),
        SourcePattern::with_rip(
            r"(?i)(?<![a-z])(?:TELESYNC|TS)[-.]?Rip(?![a-z])",
            "Telesync",
        ),
        SourcePattern::plain(r"(?i)(?<![a-z.])TS(?![a-z])", "Telesync"),
        // Camera.
        SourcePattern::plain(r"(?i)(?<![a-z])HD[-.]?CAM(?![a-z])", "HD Camera"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])HD[-.]?CAM[-.]?Rip(?![a-z])", "HD Camera"),
        SourcePattern::plain(r"(?i)(?<![a-z])CAM(?![a-z])", "Camera"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])CAM[-.]?Rip(?![a-z])", "Camera"),
        // Screener (with DVD prefix → maps to DVD source).
        SourcePattern::plain(r"(?i)(?<![a-z])DVD[-.]?SCR(?:eener)?(?![a-z])", "DVD"),
        // Generic screener (no prefix).
        SourcePattern::plain(r"(?i)(?<![a-z])SCR(?:eener)?(?![a-z])", "Screener"),
        // PPV / VOD.
        SourcePattern::plain(r"(?i)(?<![a-z])PPV(?![a-z])", "Pay-per-view"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])PPV[-.]?Rip(?![a-z])", "Pay-per-view"),
        SourcePattern::plain(r"(?i)(?<![a-z])VOD(?![a-z])", "Video on Demand"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])VOD[-.]?Rip(?![a-z])", "Video on Demand"),
        // VHS / Workprint.
        SourcePattern::plain(r"(?i)(?<![a-z])VHS(?![a-z])", "VHS"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])VHS[-.]?Rip(?![a-z])", "VHS"),
        SourcePattern::plain(r"(?i)(?<![a-z])(?:WORKPRINT|WP)(?![a-z])", "Workprint"),
        // Digital Master.
        SourcePattern::plain(r"(?i)(?<![a-z])DM(?![a-z])", "Digital Master"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])DM[-.]?Rip(?![a-z])", "Digital Master"),
        // SD TV (weak, must come last).
        SourcePattern::plain(r"(?i)(?<![a-z])SD[-.]?TV(?![a-z])", "TV"),
        SourcePattern::with_rip(r"(?i)(?<![a-z])(?:SD[-.]?)?TV[-.]?Rip(?![a-z])", "TV"),
        SourcePattern::plain(r"(?i)(?<![a-z])TV[-.]?Dub(?![a-z])", "TV"),
        // HD Rip (generic HD source).
        SourcePattern::with_rip(r"(?i)(?<![a-z])HD[-.]?Rip(?![a-z])", "HD"),
    ]
});

pub fn find_matches(input: &str) -> Vec<MatchSpan> {
    let mut matches = Vec::new();
    for sp in SOURCE_PATTERNS.iter() {
        for (start, end) in sp.vp.find_iter(input) {
            matches.push(MatchSpan::new(start, end, Property::Source, sp.vp.value));

            // If this pattern has a rip variant AND the text ends with "Rip",
            // also emit Other: "Rip".
            if sp.has_rip_variant {
                let matched_text = &input[start..end];
                if RIP_SUFFIX.is_match(matched_text).unwrap_or(false) {
                    matches.push(MatchSpan::new(start, end, Property::Other, "Rip"));
                    // BRRip is re-encoded from Blu-ray (BDRip is not).
                    if REENCODED_RIP.is_match(matched_text).unwrap_or(false) {
                        matches.push(MatchSpan::new(start, end, Property::Other, "Reencoded"));
                    }
                }
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluray() {
        let m = find_matches("Movie.BluRay.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "Blu-ray" && x.property == Property::Source)
        );
        // No Rip flag for plain Blu-ray.
        assert!(!m.iter().any(|x| x.value == "Rip"));
    }

    #[test]
    fn test_dvdrip_emits_rip() {
        let m = find_matches("Movie.DVDRip.mkv");
        assert!(
            m.iter()
                .any(|x| x.value == "DVD" && x.property == Property::Source)
        );
        assert!(
            m.iter()
                .any(|x| x.value == "Rip" && x.property == Property::Other)
        );
    }

    #[test]
    fn test_webdl() {
        let m = find_matches("Movie.WEB-DL.mkv");
        assert!(m.iter().any(|x| x.value == "Web"));
    }

    #[test]
    fn test_hdtv() {
        let m = find_matches("Movie.HDTV.mkv");
        assert!(m.iter().any(|x| x.value == "HDTV"));
    }

    #[test]
    fn test_webrip() {
        let m = find_matches("Movie.WEBRip.mkv");
        assert!(m.iter().any(|x| x.value == "Web"));
        assert!(
            m.iter()
                .any(|x| x.value == "Rip" && x.property == Property::Other)
        );
    }

    #[test]
    fn test_hd_dvd() {
        let m = find_matches("Movie.HDDVD.mkv");
        assert!(m.iter().any(|x| x.value == "HD-DVD"));
    }

    #[test]
    fn test_hd_camera() {
        let m = find_matches("Movie.HDCam.mkv");
        assert!(m.iter().any(|x| x.value == "HD Camera"));
    }

    #[test]
    fn test_satellite_rip() {
        let m = find_matches("Movie.SatRip.mkv");
        assert!(m.iter().any(|x| x.value == "Satellite"));
        assert!(m.iter().any(|x| x.value == "Rip"));
    }
}
