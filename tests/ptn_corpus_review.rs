//! Cherry-picked regression pins from the parse-torrent-name JSON corpus.
//!
//! ## Background
//!
//! As part of #157, hunch's parsing was triaged against the
//! [parse-torrent-name](https://github.com/divijbindlish/parse-torrent-name/)
//! 76-fixture corpus (a 2017-era Python port of a 2015 JS library used by
//! several downstream projects).
//!
//! ## Triage finding
//!
//! Of the 76 fixtures, **only 9 produce identical output**. The remaining 67
//! "diffs" fall into three buckets:
//!
//! | Bucket | Count | Hunch's behavior |
//! |---|---|---|
//! | Hunch normalizes raw tokens to canonical names | ~50 | x264 → "H.264", DD5.1 → "Dolby Digital", BluRay → "Blu-ray" |
//! | PTN preserves nonsensical multi-token compounds | ~10 | "WEBDL DVDRip" treated as one source value |
//! | Hunch genuinely under-parses or has a bug | ~7 | Cherry-picked into the tests below |
//!
//! **Conclusion: bulk-importing the corpus would assert the wrong values for
//! ~60 cases (PTN's denormalized output is semantically inferior).** Instead,
//! this module pins the small set of genuinely-novel cases that surface
//! either positive regression risks or actual bugs.
//!
//! ## What's pinned here
//!
//! - **Multi-language detection in dotted episode filenames** (positive
//!   regression — proves hunch correctly handles `S02E20.rus.eng.720p`
//!   pattern, which PTN can only emit as a single concatenated string)
//! - **Hindi language + DesiSCR-Rip source** (positive — proves hunch
//!   handles the South-Asian release-naming convention even when it
//!   doesn't recognize the raw "DesiSCR-Rip" source token verbatim)
//! - **CamRip in parens-comment notation** (positive — `(CamRip / 2014)`
//!   correctly yields `source: "Camera"` even though the format is exotic)
//!
//! ## Known bug filed separately
//!
//! `Community.s02e20.rus.eng.720p.Kybik.v.Kybe` triggers a website
//! false-positive (`website: "s02e20.ru"`) because the website matcher
//! mistakes the Russian-language abbreviation prefix `.ru` for a Russia TLD.
//! Filed as a separate tracking issue rather than pinned here \u2014 a
//! pin-test would force us to keep emitting the bug.

use hunch::hunch;

#[test]
fn ptn_multi_language_detection_in_dotted_episode_filename() {
    // PTN expected: language = "rus.eng" (single concatenated string \u2014
    // semantic nonsense). Hunch correctly emits a typed list with each
    // language identified separately. This pin asserts the LIST shape.
    let r = hunch("Community.s02e20.rus.eng.720p.Kybik.v.Kybe");
    assert_eq!(r.title(), Some("Community"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(20));
    assert_eq!(r.screen_size(), Some("720p"));
    let langs = r.languages();
    assert!(
        langs.contains(&"Russian") && langs.contains(&"English"),
        "expected Russian + English in language list, got: {langs:?}"
    );
}

#[test]
fn ptn_hindi_audio_with_desi_scr_rip_source() {
    // Hindi-language theatrical-recording (DesiSCR-Rip) is a common
    // South-Asian release pattern. PTN expected the literal "Hindi" and
    // ignored the codec/quality info. Hunch correctly extracts:
    //   - title = "Akira"
    //   - language = "Hindi"
    //   - video_codec = H.264 (from x264)
    //   - audio_codec = Dolby Digital (from AC3)
    //   - other = "Upscaled" (from UpScaled)
    // Pin all of it so a future change doesn't silently drop any one piece.
    let r = hunch("Akira (2016) - UpScaled - 720p - DesiSCR-Rip - Hindi - x264 - AC3");
    assert_eq!(r.title(), Some("Akira"));
    assert_eq!(r.year(), Some(2016));
    assert_eq!(r.screen_size(), Some("720p"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.audio_codec(), Some("Dolby Digital"));
    assert!(r.languages().contains(&"Hindi"));
    assert!(r.other().contains(&"Upscaled"));
}

#[test]
fn ptn_cam_rip_in_parens_comment_notation() {
    // PTN expected: title = "Guardians of the Galaxy", quality = "CamRip".
    // Hunch correctly extracts source = "Camera" (canonical normalization
    // of CamRip) plus other = "Rip". The exotic `(CamRip / 2014)` format
    // \u2014 release-type bracketed with year \u2014 is correctly parsed despite
    // the unusual structure.
    let r = hunch("Guardians of the Galaxy (CamRip / 2014)");
    assert_eq!(r.title(), Some("Guardians of the Galaxy"));
    assert_eq!(r.year(), Some(2014));
    assert_eq!(r.source(), Some("Camera"));
    assert!(r.other().contains(&"Rip"));
}
