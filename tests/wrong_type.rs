//! Integration tests for #46: WRONG_TYPE audit fixes.
//!
//! Tests three layers of fix:
//! 1. Structural: CJK episode markers (第N話/第N集) — pattern recognition
//! 2. Vocabulary: Anime bonus tokens (NCOP/NCED/PV/CM) → EpisodeDetails
//! 3. Architectural: Path-based type inference (tv/ → episode)

use hunch::{MediaType, hunch};

// ── Layer 1: CJK episode markers (structural pattern) ───────────────────

#[test]
fn cjk_dai_wa_episode_marker() {
    // 第13話 = Episode 13 (Japanese)
    let r = hunch(
        "(BD)\u{5341}\u{4e8c}\u{56fd}\u{8a18} \u{7b2c}13\u{8a71}\u{300c}\u{6708}\u{306e}\u{5f71}\u{300d}(1440x1080 x264-10bpp flac).mkv",
    );
    assert_eq!(r.episode(), Some(13), "should detect 第13話 as episode 13");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

#[test]
fn cjk_dai_shu_episode_marker() {
    // 第1集 = Episode 1 (Chinese)
    let r = hunch(
        "01 - \u{7687}\u{592a}\u{5b50}\u{79d8}\u{53f2} \u{7b2c}1\u{96c6}\u{ff08}...\u{ff09}.mkv",
    );
    assert_eq!(r.episode(), Some(1), "should detect 第1集 as episode 1");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

#[test]
fn cjk_dai_wa_large_episode() {
    let r = hunch("(BD)Show \u{7b2c}45\u{8a71}\u{300c}Title\u{300d}(1080p).mkv");
    assert_eq!(r.episode(), Some(45));
}

// ── Layer 2: Anime bonus vocabulary (EpisodeDetails) ────────────────────

#[test]
fn nced_is_episode_details() {
    let r = hunch("[DBD-Raws][Saki][NCED1][1080P][BDRip][HEVC-10bit][FLAC].mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "NCED → EpisodeDetails → episode"
    );
}

#[test]
fn pv_is_episode_details() {
    let r = hunch("[DBD-Raws][Natsume Yuujinchou Shichi][PV][1080P][BDRip][HEVC-10bit][FLAC].mkv");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

#[test]
fn cm_is_episode_details() {
    let r = hunch(
        "[TxxZ&POPGO&MGRT][Cowboy_Bebop][BDrip][BDBOX_SP02][CM][1920x1080_x264Hi10P_flac][31C5B7B3].mkv",
    );
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

// ── Layer 3: Path-based type inference (architectural fix) ──────────────

#[test]
fn tv_directory_overrides_movie_default() {
    // SP without episode markers → would be "movie" by filename alone.
    // But tv/ path → "episode".
    let r = hunch("tv/Japanese/Legal.High.SP.2013.BluRay.1080p.x265.10bit.FRDS.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "tv/ directory should force episode type"
    );
}

#[test]
fn tv_shows_directory() {
    let r = hunch("TV Shows/Power Rangers/Power Rangers Special - Alpha's Magical Christmas.avi");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

#[test]
fn anime_directory() {
    let r = hunch("Anime/Saki/[DBD-Raws][Saki][SP][1080P][BDRip][HEVC-10bit][FLAC].mkv");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

#[test]
fn bare_numeric_in_tv_directory() {
    // Category 4: bare filename, all context from path.
    let r = hunch("tv/Chinese/西游记/01.mp4");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "bare numeric in tv/ should be episode"
    );
}

#[test]
fn season_directory() {
    let r = hunch("Series/Breaking Bad/Season 3/bonus_feature.mkv");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

// ── Regression: movies without path context stay movies ─────────────────

#[test]
fn standalone_movie_still_movie() {
    let r = hunch("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(r.media_type(), Some(MediaType::Movie));
}

#[test]
fn movie_in_movies_dir_still_movie() {
    let r = hunch("movies/The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
    assert_eq!(r.media_type(), Some(MediaType::Movie));
}

#[test]
fn regular_episode_still_episode() {
    let r = hunch("Show.S01E03.720p.mkv");
    assert_eq!(r.media_type(), Some(MediaType::Episode));
}

// ── P0: CJK edge cases ─────────────────────────────────────────────────

#[test]
fn cjk_fullwidth_digits() {
    // ０３ (full-width) should normalize to 3.
    let r = hunch("(BD)Show \u{7b2c}\u{ff10}\u{ff13}\u{8a71}(1080p).mkv");
    assert_eq!(
        r.episode(),
        Some(3),
        "full-width 第０３話 should be episode 3"
    );
}

#[test]
fn cjk_simplified_hua() {
    // 第5话 (simplified Chinese 话)
    let r = hunch("Show \u{7b2c}5\u{8bdd}.mkv");
    assert_eq!(r.episode(), Some(5), "第5话 should detect episode 5");
}

#[test]
fn cjk_kai_episode() {
    // 第12回 (回 = round/episode)
    let r = hunch("Show \u{7b2c}12\u{56de}.mkv");
    assert_eq!(r.episode(), Some(12), "第12回 should detect episode 12");
}

#[test]
fn cjk_episode_zero_rejected() {
    // 第0話 should NOT produce an episode (ep_num > 0 guard).
    let r = hunch("Show \u{7b2c}0\u{8a71}.mkv");
    assert_ne!(r.episode(), Some(0), "第0話 should not produce episode 0");
}

// ── P0: Panic safety ────────────────────────────────────────────────────

#[test]
fn empty_input_no_panic() {
    let r = hunch("");
    assert!(r.title().is_none());
}

#[test]
fn extension_only_no_panic() {
    let r = hunch(".mkv");
    // Should not panic. Title may or may not be detected.
    let _ = r.title();
}

// ── P0: Untested path directories ───────────────────────────────────────

#[test]
fn donghua_directory() {
    let r = hunch("donghua/Chinese Animation/01.mp4");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "donghua/ directory should force episode type"
    );
}

#[test]
fn s01_directory_shorthand() {
    let r = hunch("Show/s1/episode.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "s1/ directory shorthand should force episode type"
    );
}

// ── P0: SP regression guard ─────────────────────────────────────────────

#[test]
fn sp_without_path_context_is_movie() {
    // SP without tv/ path should not force episode type.
    let r = hunch("Legal.High.SP.2013.BluRay.1080p.x265.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Movie),
        "SP without episode context should remain movie"
    );
}

// ── P1: False positive guards ───────────────────────────────────────────

#[test]
fn atv_not_matched_as_tv_directory() {
    // "atv/" should NOT match — only exact "tv" component.
    let r = hunch("atv/show.2024.1080p.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Movie),
        "atv/ should not match tv directory pattern"
    );
}

#[test]
fn path_traversal_still_works() {
    // ../../../tv/ should still detect the tv component.
    let r = hunch("../../../tv/show.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "tv/ in traversal path should still be detected"
    );
}

#[test]
fn backslash_path_tv_directory() {
    // Windows-style path separators.
    let r = hunch("tv\\Japanese\\show.S01E01.mkv");
    assert_eq!(
        r.media_type(),
        Some(MediaType::Episode),
        "tv\\ with backslash should be detected"
    );
}
