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
    let r = hunch(
        "Movies/Fear and Loathing in Las Vegas (1998)/Fear.and.Loathing.in.Las.Vegas.720p.HDDVD.DTS.x264-ESiR.mkv",
    );
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
    assert_eq!(r.title(), Some("Dune"));
    assert_eq!(r.part(), Some(2));
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
    assert_eq!(
        r.first(hunch::matcher::span::Property::ColorDepth),
        Some("10-bit")
    );
}

#[test]
fn crc32_in_brackets() {
    let r = hunch("[SubGroup] Anime Title - 01 [720p] [ABCD1234].mkv");
    assert_eq!(
        r.first(hunch::matcher::span::Property::Crc),
        Some("ABCD1234")
    );
}

// ─── Bit Rate ─────────────────────────────────────────────────────────────

#[test]
fn bit_rate_kbps() {
    let r = hunch("Chuck Berry The Very Best Of Chuck Berry(2010)[320 Kbps]");
    // Kbps must be classified as audio (#158): the unit is the disambiguator.
    assert_eq!(r.audio_bit_rate(), Some("320Kbps"));
    assert_eq!(r.video_bit_rate(), None);
    assert_eq!(r.year(), Some(2010));
}

#[test]
fn bit_rate_mbps() {
    let r = hunch("Title Name [480p][1.5Mbps][.mp4]");
    // Mbps must be classified as video (#158).
    assert_eq!(r.video_bit_rate(), Some("1.5Mbps"));
    assert_eq!(r.audio_bit_rate(), None);
    assert_eq!(r.screen_size(), Some("480p"));
}

#[test]
fn bit_rate_after_codec() {
    let r = hunch("Show.Name.S01E01.H264.384Kbps.mkv");
    // 384Kbps is audio (Kbps unit), regardless of being adjacent to H264.
    assert_eq!(r.audio_bit_rate(), Some("384Kbps"));
    assert_eq!(r.video_codec(), Some("H.264"));
}

// ─── Episode Format ────────────────────────────────────────────────────────

#[test]
fn episode_format_minisode() {
    let r = hunch(
        "Series/Breaking Bad/Minisodes/Breaking.Bad.(Minisodes).01.Good.Cop.Bad.Cop.WEBRip.XviD.avi",
    );
    assert_eq!(
        r.first(hunch::matcher::span::Property::EpisodeFormat),
        Some("Minisode")
    );
    assert_eq!(r.title(), Some("Breaking Bad"));
    assert_eq!(r.episode(), Some(1));
}

// ─── Week ──────────────────────────────────────────────────────────────────

#[test]
fn week_in_episode_context() {
    let r = hunch("Show Name - S32-Week 45-Ep 6478");
    assert_eq!(r.first(hunch::matcher::span::Property::Week), Some("45"));
    assert_eq!(r.season(), Some(32));
    assert_eq!(r.episode(), Some(6478));
}

// ─── CJK Fansub (Issue #34) ────────────────────────────────────────────────
// Regression tests for CJK fansub bracket format:
//   [Group][Title][Episode][Resolution][Codec]...

use hunch::matcher::span::Property;

#[test]
fn cjk_yamada_full_path() {
    let r = hunch(
        "[Comicat&KissSub] Yamada-kun to Lv999 no Koi wo Suru \
         (01-13Fin WEBRip 1080p AVC AAC TC)/\
         [Comicat&KissSub][Yamada-kun to Lv999 no Koi wo Suru]\
         [01][1080P][BIG5][MP4].mp4",
    );
    assert_eq!(r.title(), Some("Yamada-kun to Lv999 no Koi wo Suru"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("Comicat&KissSub"));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(r.audio_codec(), Some("AAC"));
    assert_eq!(r.video_codec(), Some("H.264"));
    assert_eq!(r.container(), Some("mp4"));
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Traditional Chinese")
    );
    // TC in parent dir must NOT be parsed as Telecine.
    let sources: Vec<_> = r.all(Property::Source);
    assert!(
        !sources.contains(&"Telecine"),
        "TC should not match as Telecine when subtitle language is present"
    );
}

#[test]
fn cjk_saki_zenkoku_mkv() {
    let r = hunch(
        "[DBD-Raws][天才麻将少女 全国篇][01-13TV全集+特典映像]\
         [1080P][BDRip][HEVC-10bit][简繁外挂][FLAC][MKV]/\
         [DBD-Raws][Saki Zenkoku Hen][01][1080P][BDRip]\
         [HEVC-10bit][FLACx2].mkv",
    );
    assert_eq!(r.title(), Some("Saki Zenkoku Hen"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
    assert_eq!(r.audio_codec(), Some("FLAC"));
    assert_eq!(r.video_codec(), Some("H.265"));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.first(Property::ColorDepth), Some("10-bit"));
}

#[test]
fn cjk_saki_zenkoku_sc_ass() {
    let r = hunch(
        "[DBD-Raws][Saki Zenkoku Hen][01][1080P][BDRip]\
         [HEVC-10bit][FLACx2].sc.ass",
    );
    assert_eq!(r.title(), Some("Saki Zenkoku Hen"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
    assert_eq!(r.container(), Some("ass"));
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Simplified Chinese")
    );
    // Subtitle containers must NOT carry video/audio tech.
    assert_eq!(r.video_codec(), None);
    assert_eq!(r.audio_codec(), None);
    assert_eq!(r.first(Property::ColorDepth), None);
    assert_eq!(r.source(), None);
}

#[test]
fn cjk_saki_zenkoku_tc_ass() {
    let r = hunch(
        "[DBD-Raws][Saki Zenkoku Hen][01][1080P][BDRip]\
         [HEVC-10bit][FLACx2].tc.ass",
    );
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Traditional Chinese")
    );
    assert_eq!(r.container(), Some("ass"));
}

#[test]
fn cjk_saki_rev_sc_ass() {
    let r = hunch(
        "[Rev][DBD-Raws][天才麻将少女][01-25TV全集+SP][1080P][BDRip]\
         [HEVC-10bit][简繁外挂][FLAC][MKV]/\
         [DBD-Raws][Saki][01][1080P][BDRip][HEVC-10bit][FLAC][Rev].sc.ass",
    );
    assert_eq!(r.title(), Some("Saki"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
    assert_eq!(r.container(), Some("ass"));
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Simplified Chinese")
    );
    let others: Vec<_> = r.all(Property::Other);
    assert!(
        others.contains(&"Revised"),
        "[Rev] should be parsed as Revised"
    );
}

#[test]
fn cjk_natsume_scjp_ass() {
    let r = hunch(
        "[DBD-Raws][夏目友人帐 柒][01-12TV全集+SP+特典映像]\
         [1080P][BDRip][HEVC-10bit][简繁日双语外挂][FLAC][MKV]/\
         [DBD-Raws][Natsume Yuujinchou Shichi][01][1080P][BDRip]\
         [HEVC-10bit][FLAC].scjp.ass",
    );
    assert_eq!(r.title(), Some("Natsume Yuujinchou Shichi"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("DBD-Raws"));
    assert_eq!(r.container(), Some("ass"));
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Simplified Chinese")
    );
    // .scjp.ass = subtitle container → no video tech.
    assert_eq!(r.video_codec(), None);
    assert_eq!(r.source(), None);
}

#[test]
fn cjk_natsume_tcjp_ass() {
    let r = hunch(
        "[DBD-Raws][Natsume Yuujinchou Shichi][01][1080P][BDRip]\
         [HEVC-10bit][FLAC].tcjp.ass",
    );
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Traditional Chinese")
    );
}

#[test]
fn cjk_natsume_sp_episode() {
    let r = hunch(
        "[DBD-Raws][Natsume Yuujinchou Shichi][13(SP)][1080P]\
         [BDRip][HEVC-10bit][FLAC].mkv",
    );
    assert_eq!(r.title(), Some("Natsume Yuujinchou Shichi"));
    assert_eq!(r.episode(), Some(13));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn cjk_saki_achiga_flacx2() {
    let r = hunch(
        "[DBD-Raws][Saki Achiga Hen Episode of Side-A][01][1080P]\
         [BDRip][HEVC-10bit][FLACx2].mkv",
    );
    assert_eq!(r.title(), Some("Saki Achiga Hen Episode of Side-A"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.audio_codec(), Some("FLAC"));
}

#[test]
fn cjk_saki_achiga_nc_ver() {
    let r = hunch(
        "[DBD-Raws][Saki Achiga Hen Episode of Side-A][14][NC.Ver]\
         [1080P][BDRip][HEVC-10bit][FLAC].mkv",
    );
    assert_eq!(r.episode(), Some(14));
}

#[test]
fn cjk_solo_leveling_sxxexx_in_bracket_dir() {
    let r = hunch("[H-Enc] Solo Leveling Season 2 (BDRip 1080p HEVC FLAC)/S01E13.mkv");
    assert_eq!(r.title(), Some("Solo Leveling"));
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(13));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.video_codec(), Some("H.265"));
    assert_eq!(r.audio_codec(), Some("FLAC"));
}

#[test]
fn cjk_lolihouse_dash_episode() {
    let r = hunch(
        "[LoliHouse] Kage no Jitsuryokusha ni Naritakute! [01-20]\
         [WebRip 1080p HEVC-10bit AAC]/\
         [LoliHouse] Kage no Jitsuryokusha ni Naritakute! - 01 \
         [WebRip 1080p HEVC-10bit AAC SRTx2].mkv",
    );
    assert_eq!(r.title(), Some("Kage no Jitsuryokusha ni Naritakute!"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("LoliHouse"));
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(r.audio_codec(), Some("AAC"));
}

#[test]
fn cjk_lolihouse_season2() {
    let r = hunch(
        "[LoliHouse] Kage no Jitsuryokusha ni Naritakute! S2 - 03 \
         [WebRip 1080p HEVC-10bit AAC SRTx2].mkv",
    );
    assert_eq!(r.title(), Some("Kage no Jitsuryokusha ni Naritakute!"));
    assert_eq!(r.episode(), Some(3));
    assert_eq!(r.season(), Some(2));
}

#[test]
fn cjk_cowboy_bebop_from_parent() {
    let r = hunch("Cowboy_Bebop[BDrip][1080p]/Cowboy.Bebop.E01.mkv");
    assert_eq!(r.title(), Some("Cowboy Bebop"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.screen_size(), Some("1080p"));
}

#[test]
fn cjk_frieren_with_episode_title() {
    let r = hunch(
        "Frieren - Beyond Journey's End S01 1080p Dual Audio BDRip \
         10 bits DD+ x265-EMBER/\
         S01E01-The Journey's End [18D1CE8D].mkv",
    );
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode_title(), Some("The Journey's End"));
    assert_eq!(r.release_group(), Some("EMBER"));
    assert_eq!(r.first(Property::Crc), Some("18D1CE8D"));
}

#[test]
fn cjk_prejudice_studio_mixed_title() {
    let r = hunch(
        "[Prejudice-Studio] 我独自升级 Ore dake Level Up na Ken - 01 \
         [Bilibili WEB-DL 1080P AVC 8bit AAC MP4][简日内嵌].mp4",
    );
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.source(), Some("Web"));
    assert_eq!(r.container(), Some("mp4"));
    // Was previously TODO(issue #34) — the compound bracket merger
    // mistakenly absorbed Prejudice-Studio. Fixed by the v1.1.x bracket
    // strategy refactor; pinned here so the regression can't sneak back.
    assert_eq!(r.release_group(), Some("Prejudice-Studio"));
}

// ── Issue #38 regression: episode title last word ≠ release_group ─────────

#[test]
fn issue_38_plex_dash_long_episode_title() {
    let r = hunch("LEGO Ninjago Dragons Rising - S02E03 - The Temple of the Dragon Cores.mkv");
    assert_eq!(r.title(), Some("LEGO Ninjago Dragons Rising"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(3));
    assert_eq!(r.episode_title(), Some("The Temple of the Dragon Cores"));
    assert_eq!(r.release_group(), None);
}

#[test]
fn issue_38_plex_dash_simple_episode_title() {
    let r = hunch("Bluey - S02E01 - Dance Mode.mkv");
    assert_eq!(r.title(), Some("Bluey"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.episode_title(), Some("Dance Mode"));
    assert_eq!(r.release_group(), None);
}

// ── Issue #39 regression: CJK bracket episode detection ───────────────────

#[test]
fn issue_39_cjk_bracket_big5() {
    let r = hunch(
        "[Comicat&KissSub][Yamada-kun to Lv999 no Koi wo Suru]\
         [01][1080P][BIG5][MP4].mp4",
    );
    assert_eq!(r.title(), Some("Yamada-kun to Lv999 no Koi wo Suru"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(r.release_group(), Some("Comicat&KissSub"));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(
        r.first(Property::SubtitleLanguage),
        Some("Traditional Chinese")
    );
}

#[test]
fn issue_39_cjk_bracket_sp_prefix() {
    let r = hunch("[DBD-Raws][Saki][SP][01][1080P][BDRip][HEVC-10bit][FLAC].mkv");
    assert_eq!(r.title(), Some("Saki"));
    // SP + 01 — episode should be detected.
    assert!(r.episode().is_some(), "episode should be detected");
    // Was previously TODO(v1.1.4) — the bracket merger absorbed [Saki]
    // into the group name. Resolved by the format-aware bracket extractor;
    // pin the corrected behavior so the regression can't sneak back.
    assert_eq!(r.release_group(), Some("DBD-Raws"));
    assert_eq!(r.episode_details(), Some("Special"));
}

#[test]
fn issue_39_cjk_cowboy_bebop_sp02() {
    let r = hunch(
        "[TxxZ&POPGO&MGRT][Cowboy_Bebop][BDrip]\
         [BDBOX_SP02][CM][1920x1080_x264Hi10P_flac][31C5B7B3].mkv",
    );
    assert_eq!(r.release_group(), Some("TxxZ&POPGO&MGRT"));
    assert_eq!(r.source(), Some("Blu-ray"));
    assert_eq!(r.first(Property::Crc), Some("31C5B7B3"));
}

#[test]
fn issue_100_first_bracket_is_title_when_natural_language() {
    let r = hunch(
        "[Kimetsu no Yaiba Mugen Ressha Hen][JPN+ENG][BDRIP][1080P][H264_FLACx3_DTS-HDMA].mkv",
    );
    assert_eq!(r.title(), Some("Kimetsu no Yaiba Mugen Ressha Hen"));
    assert_eq!(r.release_group(), None);
}

#[test]
fn issue_100_real_release_group_still_detected() {
    let r =
        hunch("[Prejudice-Studio][Kimetsu no Yaiba Mugen Ressha Hen][JPN+ENG][BDRIP][1080P].mkv");
    assert_eq!(r.release_group(), Some("Prejudice-Studio"));
    assert_eq!(r.title(), Some("Kimetsu no Yaiba Mugen Ressha Hen"));
}

// ── Issue #35 regression: subtitle containers strip video tech ──────────────────

#[test]
fn issue_35_western_srt_no_video_props() {
    let r = hunch("Arcane.S01E01.1080p.NF.WEB-DL.DDP5.1.H.265-npuer.srt");
    assert_eq!(r.container(), Some("srt"));
    assert_eq!(r.title(), Some("Arcane"));
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(1));
    // Subtitle file → no video/audio tech.
    assert_eq!(r.video_codec(), None);
    assert_eq!(r.audio_codec(), None);
    assert_eq!(r.source(), None);
}

// ── Issue #124 regression: anime multi-segment title with " - " and "Part N" ──
//
// The bug (pre-#127): "[Group] Show - Sub Part 2 - 13 [tags].mkv" had its
// title truncated at the first " - ", losing the "Sub Part 2" segment
// (the title became "Show" instead of "Show - Sub Part 2").
//
// The fix is in `clean_title_preserve_dashes` (src/properties/title/clean.rs)
// + the anime-bracket title-boundary detector. This integration test
// pins the *cross-module contract* end-to-end — the unit tests in
// `clean.rs` only exercise the cleaner in isolation, and the YAML
// fixture in `tests/fixtures/episodes.yml` is the catch-all harness.
// A focused integration test makes the regression discoverable when
// anyone changes either the cleaner OR the title-boundary detector,
// and prints a sharper diagnostic than the harness does.

#[test]
fn issue_124_anime_multi_segment_title_preserved() {
    let r = hunch(
        "[Erai-raws] Enen no Shouboutai - San no Shou Part 2 - 13 \
         [1080p CR WEB-DL AVC AAC][MultiSub][7FF9B816].mkv",
    );
    assert_eq!(
        r.title(),
        Some("Enen no Shouboutai - San no Shou Part 2"),
        "the multi-segment title with embedded \" - \" and \"Part 2\" \
         must be preserved verbatim (not truncated at the first dash)"
    );
    assert_eq!(r.episode(), Some(13));
    assert_eq!(r.release_group(), Some("Erai-raws"));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.container(), Some("mkv"));
}

#[test]
fn issue_124_anime_dash_only_no_part_keyword() {
    // Sibling case without "Part N": still must keep the second segment
    // as part of the title (the boundary detector should treat the
    // trailing " - 11" as the episode marker, not the inner " - ").
    //
    // Note: the YAML fixture in tests/fixtures/episodes.yml asserts
    // `title: "Show Name"` + `alternative_title: "Still Name"` — that's
    // the *guessit-compat aspirational* target (the compatibility report
    // tracks how close we are). Current behavior keeps both segments in
    // the title; this test pins that behavior so we don't accidentally
    // change it via an unrelated cleaner refactor.
    let r = hunch("[SuperGroup].Show.Name.-.Still.Name.-.11.[1080p].mkv");
    assert_eq!(r.release_group(), Some("SuperGroup"));
    assert_eq!(r.episode(), Some(11));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(
        r.title(),
        Some("Show Name - Still Name"),
        "the inner \" - \" must be preserved as part of the title (the \
         trailing \" - 11\" is the episode marker)"
    );
}

// ── Competitor-borrowed regression pins (April 2026 review of go-ptn / parse-torrent-name) ──
//
// These tests pin behaviors discovered during a comparative review against
// `razsteinmetz/go-ptn` and `divijbindlish/parse-torrent-name`. Two of the
// behaviors below (SBS/OU stereoscopic detection) were already correctly
// implemented in hunch but lacked focused tests; the third (R0–R6 DVD region
// codes) extended coverage from a single value (R5) to the standard set.

#[test]
fn stereoscopic_half_sbs_emits_3d_other() {
    // Half-SBS (Side-by-Side) is a 3D delivery format — it implies the
    // content is stereoscopic 3D even when the literal "3D" token is absent.
    // hunch's TOML rule (rules/other_positional.toml) maps Half-SBS → "3D",
    // which is more semantically correct than emitting a separate "SBS" tag
    // (the approach taken by go-ptn / parse-torrent-name).
    let r = hunch("TEST.2015.1080p.3D.BluRay.Half-SBS.x264.DTS-HD.MA.7.1-ABC");
    assert!(
        r.other().contains(&"3D"),
        "Half-SBS must contribute a \"3D\" Other value (stereoscopic delivery format)"
    );
    assert_eq!(r.title(), Some("TEST"));
    assert_eq!(r.year(), Some(2015));
    assert_eq!(r.screen_size(), Some("1080p"));
    assert_eq!(r.source(), Some("Blu-ray"));
}

#[test]
fn stereoscopic_half_ou_emits_3d_other() {
    // Half-OU (Over-Under) is the vertical-stack stereoscopic counterpart to
    // Half-SBS. Same semantic mapping: implies 3D delivery.
    let r = hunch("TEST.2015.1080p.3D.BluRay.Half-OU.x264.DTS-HD.MA.7.1-ABC");
    assert!(
        r.other().contains(&"3D"),
        "Half-OU must contribute a \"3D\" Other value (stereoscopic delivery format)"
    );
    assert_eq!(r.source(), Some("Blu-ray"));
}

#[test]
fn dvd_region_codes_r0_through_r6() {
    // DVD region codes R0 (worldwide) through R6 (China) are the standard
    // MPAA region set. R7–R9 are reserved/non-theatrical and intentionally
    // omitted to limit false positives on niche release-group tokens.
    //
    // Pre-review (April 2026) only R5 was supported. This test pins the
    // extension to R0–R6 from rules/other.toml.
    let pairs = [
        ("R0", "Region 0"),
        ("R1", "Region 1"),
        ("R2", "Region 2"),
        ("R3", "Region 3"),
        ("R4", "Region 4"),
        ("R5", "Region 5"),
        ("R6", "Region 6"),
    ];
    for (token, expected) in pairs {
        let filename = format!("Movie.2024.{token}.DVDRip.x264-GROUP.mkv");
        let r = hunch(&filename);
        assert!(
            r.other().contains(&expected),
            "{filename} should yield Other = \"{expected}\", got {:?}",
            r.other()
        );
        assert_eq!(r.source(), Some("DVD"), "{filename} source");
    }
}

#[test]
fn dvd_region_r5_does_not_break_classic_brave_fixture() {
    // Regression: the canonical go-ptn / parse-torrent-name R5 fixture.
    // Pinning this guards against any future change to the region-code
    // exact-match table that might over-generalize and corrupt the classic
    // case both libraries use as their R5 reference example.
    let r = hunch("Brave.2012.R5.DVDRip.XViD.LiNE-UNiQUE");
    assert_eq!(r.title(), Some("Brave"));
    assert_eq!(r.year(), Some(2012));
    assert_eq!(r.source(), Some("DVD"));
    assert_eq!(r.video_codec(), Some("Xvid"));
    assert_eq!(r.release_group(), Some("UNiQUE"));
    assert!(r.other().contains(&"Region 5"));
}

// ── #158 regression pins: bit_rate split + mimetype derivation ────────────

#[test]
fn mimetype_derived_from_mp4_container() {
    // mp4 → video/mp4 — the universal video container.
    let r = hunch("Movie.2024.1080p.WEB-DL.x264.mp4");
    assert_eq!(r.container(), Some("mp4"));
    assert_eq!(r.mimetype(), Some("video/mp4"));
}

#[test]
fn mimetype_derived_from_mkv_container() {
    // mkv → video/x-matroska — de facto MIME for Matroska.
    let r = hunch("Movie.2024.1080p.BluRay.x265.mkv");
    assert_eq!(r.container(), Some("mkv"));
    assert_eq!(r.mimetype(), Some("video/x-matroska"));
}

#[test]
fn mimetype_none_when_container_unknown() {
    // Unknown container → None (never fabricate a fallback MIME).
    let r = hunch("Movie.2024.1080p.WEB-DL.x264.weirdext");
    assert_eq!(r.container(), None);
    assert_eq!(r.mimetype(), None);
}

#[test]
fn bit_rate_kbps_lowercase_with_channel_collision() {
    // Pin the regression where "DD5.1.448kbps" was being parsed as
    // bit_rate "1.448Kbps" because the regex greedily absorbed the
    // fractional channel digit. Now bounded decimals (\d{1,2}) force
    // the regex to backtrack to the next valid match.
    let r = hunch("Hotel.Hell.S01E01.720p.DD5.1.448kbps-ALANiS");
    assert_eq!(r.audio_bit_rate(), Some("448Kbps"));
    assert_eq!(r.audio_channels(), Some("5.1"));
    assert_eq!(r.audio_codec(), Some("Dolby Digital"));
}

#[test]
fn bit_rate_kbit_short_suffix() {
    // Pin the `bit` short-suffix support added in #158. Older filenames
    // (especially anime) use `kbit` instead of `kbps` — both must yield
    // the same canonical "Kbps" output.
    let r = hunch("Show.Name.S01E01.H264-384kbit_AAC.mp4");
    assert_eq!(r.audio_bit_rate(), Some("384Kbps"));
}

#[test]
fn bit_rate_mbits_plural_suffix() {
    // Pin the `bits` plural-short-suffix support added in #158. Some
    // anime-release notations (e.g. "19.1mbits") use this form.
    let r = hunch("[HorribleSubs] Overlord II - 01 [1080p] 19.1mbits - 120fps.mkv");
    assert_eq!(r.video_bit_rate(), Some("19.1Mbps"));
    assert_eq!(r.frame_rate(), Some("120fps"));
}

// ─── Pass 2 boundary mutants (#146) ──────────────────────────────────
//
// These tests pin two boundary checks in Pass 2 of the pipeline that
// survived as mutants in the 2026-04-19 nightly run:
//   - Step 5e: drop heuristic episode matches in movie context
//   - Step 6:  only emit ProperCount when count > 0
//
// The other two #146 Pass 2 mutants (lines 454, 515) are killed by
// the unit tests on the hoisted `compute_override_title_span` and
// `release_group_overlaps_episode_title` helpers in
// `src/pipeline/pass2_helpers.rs`.

#[test]
fn movie_context_drops_heuristic_episode_match() {
    // "Movie.10.mkv" — bare number "10" in movie context is a franchise
    // number ("Toy Story 10"), not an episode. Step 5e drops episode
    // matches with priority <= HEURISTIC when media_type == "movie".
    //
    // Pins the `<= -> >` mutant on `m.priority <= priority::HEURISTIC`
    // in src/pipeline/mod.rs. With the mutation, retain would drop only
    // matches with priority STRICTLY GREATER than HEURISTIC (i.e., the
    // strong matches would be dropped while weak ones survive — exactly
    // the wrong direction). The bare "10" would then surface as an
    // episode in movie context.
    let r = hunch("Movie.10.mkv");
    assert_eq!(r.media_type(), Some(hunch::MediaType::Movie));
    assert_eq!(
        r.episode(),
        None,
        "bare number in movie context must not surface as episode"
    );
}

#[test]
fn movie_without_proper_omits_proper_count_field() {
    // For a clean movie filename (no PROPER/REPACK), proper_count is 0
    // and the field must NOT be emitted. Pins the `> -> >=` mutant on
    // `if proper_count > 0` in step 6 of Pass 2.
    //
    // With `>=`, ProperCount would be emitted as "0" for every clean
    // file, polluting downstream consumers.
    let r = hunch("The.Movie.2024.1080p.BluRay.x264.mkv");
    assert_eq!(
        r.proper_count(),
        None,
        "ProperCount must be absent when count is 0"
    );
}

#[test]
fn episode_with_proper_emits_proper_count_one() {
    // Happy-path companion to the previous test — ensures we don't
    // accidentally regress the emit path while pinning the > vs >=
    // boundary. With either operator, count=1 produces Some(1), so this
    // doesn't kill the mutant on its own; it documents the positive case.
    let r = hunch("Show.S01E03.PROPER.mkv");
    assert_eq!(r.proper_count(), Some(1));
}

#[test]
fn episode_with_repack_and_proper_emits_proper_count_two() {
    // Stronger positive case — exercises the to_string() path with a
    // multi-digit count and confirms the field is set correctly.
    let r = hunch("Show.S01E03.REPACK.PROPER.mkv");
    assert_eq!(r.proper_count(), Some(2));
}
