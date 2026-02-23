//! End-to-end integration tests for `hunch()` — self-contained, no external deps.
//!
//! These test the full pipeline: input filename → structured metadata.

use hunch::hunch;

// ─── Movies ──────────────────────────────────────────────────────────

#[test]
fn movie_basic() {
    let r = hunch("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(r.title(), Some("The Matrix"));
    assert_eq!(r.year(), Some(1999));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.release_group(), Some("GROUP"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn movie_with_path() {
    let r = hunch("Movies/Fear and Loathing in Las Vegas (1998)/Fear.and.Loathing.in.Las.Vegas.720p.HDDVD.DTS.x264-ESiR.mkv");
    assert_eq!(r.title(), Some("Fear and Loathing in Las Vegas"));
    assert_eq!(r.year(), Some(1998));
    assert_eq!(r.screen_size(), Some("720p"));
    assert_eq!(r.audio_codec(), Some("DTS"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn movie_4k_hdr() {
    let r = hunch("Blade.Runner.2049.2017.2160p.UHD.BluRay.REMUX.HDR.HEVC.Atmos-EPSiLON.mkv");
    // Known limitation: title stops at "2049" because it looks like a year.
    assert_eq!(r.title(), Some("Blade Runner"));
    assert_eq!(r.year(), Some(2017));
    assert_eq!(r.screen_size(), Some("2160p"));
    assert_eq!(r.source(), Some("Ultra HD Blu-ray"));
    assert_eq!(r.video_codec(), Some("H.265"));
    assert_eq!(r.container(), Some("mkv"));
    assert_eq!(r.release_group(), Some("EPSiLON"));
}

#[test]
fn movie_dvdrip() {
    let r = hunch("Pulp.Fiction.1994.DVDRip.XviD-SAPHiRE.avi");
    assert_eq!(r.title(), Some("Pulp Fiction"));
    assert_eq!(r.year(), Some(1994));
    assert_eq!(r.source(), Some("DVD"));
    assert_eq!(r.video_codec(), Some("Xvid"));
    assert_eq!(r.release_group(), Some("SAPHiRE"));
    assert_eq!(r.container(), Some("avi"));
}

#[test]
fn movie_web_dl() {
    let r = hunch("Dune.Part.Two.2024.WEB-DL.1080p.DDP5.1.Atmos.H.264-FLUX.mkv");
    assert_eq!(r.title(), Some("Dune Part Two"));
    assert_eq!(r.year(), Some(2024));
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.container(), Some("mkv"));
    assert_eq!(r.release_group(), Some("FLUX"));
}

// ─── Episodes ────────────────────────────────────────────────────────

#[test]
fn episode_sxxexx() {
    let r = hunch("The.Walking.Dead.S05E03.720p.BluRay.x264-DEMAND.mkv");
    assert_eq!(r.title(), Some("The Walking Dead"));
    assert_eq!(r.season(), Some(5));
    assert_eq!(r.episode(), Some(3));
    assert_eq!(r.screen_size(), Some("720p"));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.release_group(), Some("DEMAND"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn episode_with_title() {
    let r = hunch("Californication.2x05.Vaginatown.HDTV.XviD-0TV.avi");
    assert_eq!(r.title(), Some("Californication"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(5));
    assert_eq!(r.episode_title(), Some("Vaginatown"));
    assert_eq!(r.source(), Some("HDTV"));
    assert_eq!(r.video_codec(), Some("Xvid"));
    assert_eq!(r.release_group(), Some("0TV"));
    assert_eq!(r.container(), Some("avi"));
}

#[test]
fn episode_hdtv() {
    let r = hunch("Breaking.Bad.S01E01.720p.HDTV.x264-NewbIES.mkv");
    assert_eq!(r.title(), Some("Breaking Bad"));
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.screen_size(), Some("720p"));
    assert_eq!(r.source(), Some("HDTV"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn episode_daily_show() {
    let r = hunch("The.Daily.Show.2024.03.15.720p.WEB.h264-EDITH.mkv");
    assert_eq!(r.title(), Some("The Daily Show"));
    assert_eq!(r.screen_size(), Some("720p"));
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn episode_full_season_pack() {
    let r = hunch("Game.of.Thrones.S08.1080p.BluRay.x264-ROVERS.mkv");
    assert_eq!(r.title(), Some("Game of Thrones"));
    assert_eq!(r.season(), Some(8));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.video_codec(), Some("H.264"));
}

// ─── Audio Codecs ────────────────────────────────────────────────────

#[test]
fn audio_aac() {
    let r = hunch("Movie.2024.1080p.WEBRip.AAC2.0.x264.mkv");
    assert_eq!(r.audio_codec(), Some("AAC"));
    assert_eq!(r.audio_channels(), Some("2.0"));
}

#[test]
fn audio_dts_hd_ma() {
    let r = hunch("Movie.2024.1080p.BluRay.DTS-HD.MA.5.1.x264.mkv");
    // hunch reports the codec family; MA lands in audio_profile.
    assert_eq!(r.audio_codec(), Some("DTS-HD"));
    assert_eq!(r.audio_channels(), Some("5.1"));
}

#[test]
fn audio_truehd_atmos() {
    let r = hunch("Movie.2024.2160p.BluRay.TrueHD.7.1.Atmos.x265.mkv");
    // Multi-value: hunch returns first codec in the array.
    assert_eq!(r.audio_codec(), Some("Dolby TrueHD"));
    assert_eq!(r.audio_channels(), Some("7.1"));
}

// ─── Video Codecs ────────────────────────────────────────────────────

#[test]
fn video_h264_variants() {
    assert_eq!(hunch("Movie.x264.mkv").video_codec(), Some("H.264"));
    assert_eq!(hunch("Movie.h264.mkv").video_codec(), Some("H.264"));
    assert_eq!(hunch("Movie.H.264.mkv").video_codec(), Some("H.264"));
}

#[test]
fn video_h265_variants() {
    assert_eq!(hunch("Movie.x265.mkv").video_codec(), Some("H.265"));
    assert_eq!(hunch("Movie.HEVC.mkv").video_codec(), Some("H.265"));
    assert_eq!(hunch("Movie.h265.mkv").video_codec(), Some("H.265"));
}

#[test]
fn video_av1() {
    assert_eq!(hunch("Movie.AV1.mkv").video_codec(), Some("AV1"));
}

// ─── Screen Sizes ────────────────────────────────────────────────────

#[test]
fn screen_sizes() {
    assert_eq!(hunch("Movie.480p.mkv").screen_size(), Some("480p"));
    assert_eq!(hunch("Movie.720p.mkv").screen_size(), Some("720p"));
    assert_eq!(hunch("Movie.1080p.mkv").screen_size(), Some("1080p"));
    assert_eq!(hunch("Movie.1080i.mkv").screen_size(), Some("1080i"));
    assert_eq!(hunch("Movie.2160p.mkv").screen_size(), Some("2160p"));
    assert_eq!(hunch("Movie.4K.mkv").screen_size(), Some("2160p"));
}

// ─── Sources ─────────────────────────────────────────────────────────

#[test]
fn sources() {
    assert_eq!(hunch("Movie.BluRay.mkv").source(), Some("Blu-ray"));
    assert_eq!(hunch("Movie.BDRip.mkv").source(), Some("Blu-ray"));
    assert_eq!(hunch("Movie.WEB-DL.mkv").source(), Some("Web"));
    assert_eq!(hunch("Movie.WEBRip.mkv").source(), Some("Web"));
    assert_eq!(hunch("Movie.HDTV.mkv").source(), Some("HDTV"));
    assert_eq!(hunch("Movie.DVDRip.mkv").source(), Some("DVD"));
}

// ─── Editions ────────────────────────────────────────────────────────

#[test]
fn editions() {
    let r = hunch("Movie.Directors.Cut.1080p.BluRay.mkv");
    assert_eq!(r.edition(), Some("Director's Cut"));

    let r = hunch("Movie.Unrated.1080p.BluRay.mkv");
    assert_eq!(r.edition(), Some("Unrated"));

    let r = hunch("Movie.Extended.Edition.1080p.BluRay.mkv");
    assert_eq!(r.edition(), Some("Extended"));

    // Verify no year leaks into edition.
    assert_eq!(r.year(), None);
}

// ─── Containers ──────────────────────────────────────────────────────

#[test]
fn containers() {
    assert_eq!(hunch("Movie.mkv").container(), Some("mkv"));
    assert_eq!(hunch("Movie.avi").container(), Some("avi"));
    assert_eq!(hunch("Movie.mp4").container(), Some("mp4"));
    assert_eq!(hunch("Movie.m4v").container(), Some("m4v"));
    assert_eq!(hunch("Subs.srt").container(), Some("srt"));
}

// ─── Edge Cases ──────────────────────────────────────────────────────

#[test]
fn no_year() {
    let r = hunch("Inception.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(r.title(), Some("Inception"));
    assert_eq!(r.year(), None);
}

#[test]
fn title_with_numbers() {
    let r = hunch("300.2006.1080p.BluRay.x264.mkv");
    assert_eq!(r.title(), Some("300"));
    assert_eq!(r.year(), Some(2006));
}

#[test]
fn minimal_input() {
    let r = hunch("movie.mkv");
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn proper_repack() {
    let r = hunch("Movie.2024.PROPER.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(r.proper_count(), Some(1));
}

#[test]
fn streaming_service() {
    let r = hunch("Movie.2024.NF.WEB-DL.1080p.x264-GROUP.mkv");
    assert_eq!(r.streaming_service(), Some("Netflix"));
    assert_eq!(r.source(), Some("Web"));
}

#[test]
fn color_depth_10bit() {
    let r = hunch("Movie.2024.10bit.1080p.BluRay.x265.mkv");
    assert_eq!(r.first(hunch::matcher::span::Property::ColorDepth), Some("10-bit"));
}

#[test]
fn crc32_in_brackets() {
    let r = hunch("[SubGroup] Anime Title - 01 [720p] [ABCD1234].mkv");
    assert_eq!(r.first(hunch::matcher::span::Property::Crc), Some("ABCD1234"));
}
