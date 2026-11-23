//! Regression tests for real-world filenames that guessit v4.5.x covers and
//! hunch used to get wrong. Each test names the shape it pins down.

use hunch::hunch;

// ─── Title words that are also vocabulary ───────────────────────────────

#[test]
fn article_plus_edition_word_is_the_title() {
    let r = hunch("The.Collector.2009.1080p.mkv");
    assert_eq!(r.title(), Some("The Collector"));
    assert_eq!(r.year(), Some(2009));
    assert!(r.first(hunch::Property::Edition).is_none());
}

#[test]
fn article_plus_other_word_is_the_title() {
    let r = hunch("The.Convert.2024.1080p.mkv");
    assert_eq!(r.title(), Some("The Convert"));
    assert!(r.first(hunch::Property::Other).is_none());
}

#[test]
fn bare_edition_or_other_word_keeps_its_property() {
    // Without a leading article the word is metadata, as in guessit.
    let r = hunch("Extended.2019.1080p.mkv");
    assert_eq!(r.first(hunch::Property::Edition), Some("Extended"));
    let r = hunch("3D.2019");
    assert_eq!(r.first(hunch::Property::Other), Some("3D"));
}

#[test]
fn leading_film_number_is_the_title() {
    let r = hunch("F1.2024.1080p.mkv");
    assert_eq!(r.title(), Some("F1"));
    assert_eq!(r.first(hunch::Property::Film), None);

    let r = hunch("F1.The.Movie.2025.1080p.AMZN.WEB-DL.DDP5.1.Atmos.H.264-BYNDR.mkv");
    assert_eq!(r.title(), Some("F1 The Movie"));
}

#[test]
fn film_number_after_a_title_stays_a_film_number() {
    let r = hunch("Fast and Furious F9 2021 BluRay 1080p.H264 Ita Eng AC3 5.1 Sub Ita Eng.mkv");
    assert_eq!(r.first(hunch::Property::Film), Some("9"));
}

#[test]
fn mixed_case_xxx_is_a_title() {
    let r = hunch("xXx.2002.1080p.mkv");
    assert_eq!(r.title(), Some("xXx"));
    assert!(r.first(hunch::Property::Other).is_none());

    let r = hunch("Bjyukujyo Kyoushi Kan XXX 720P WEBRIP MP4-GUSH");
    assert!(r.all(hunch::Property::Other).contains(&"XXX"));
}

#[test]
fn opus_before_the_year_is_a_title_word() {
    let r = hunch("opus.2025.1080p.web-dl.ddp5.1.atmos.h265-aoc.mkv");
    assert_eq!(r.title(), Some("opus"));
    assert!(!r.all(hunch::Property::AudioCodec).contains(&"Opus"));

    let r = hunch("Something.Opus.2025.1080p.WEB-DL.DDP5.1-AOC.mkv");
    assert_eq!(r.title(), Some("Something Opus"));
}

#[test]
fn opus_after_the_year_is_still_a_codec() {
    let r = hunch("How To Steal A Dog.2014.BluRay.1080p.12bit.HEVC.OPUS 5.1-Hn1Dr2.mkv");
    assert_eq!(r.title(), Some("How To Steal A Dog"));
    assert_eq!(r.audio_codec(), Some("Opus"));
    assert_eq!(r.release_group(), Some("Hn1Dr2"));
}

// ─── Episode numbers in both the folder and the filename ────────────────

#[test]
fn file_episode_beats_a_pack_folder_range() {
    let r = hunch("Show.S01E01-E10.Complete.720p.WEB-DL/Show.S01E05.720p.WEB-DL.mkv");
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(5));
    assert_ne!(r.release_group(), Some("DL"));
}

#[test]
fn file_episode_beats_a_different_folder_episode() {
    let r = hunch("Show.S02E03/Show.S01E09.mkv");
    assert_eq!(r.season(), Some(1));
    assert_eq!(r.episode(), Some(9));
}

#[test]
fn season_folder_still_combines_with_file_episode() {
    let r = hunch("Friends S03/Friends - 07 - The One.mkv");
    assert_eq!(r.season(), Some(3));
    assert_eq!(r.episode(), Some(7));
}

#[test]
fn sample_in_single_episode_folder_takes_the_folder_episode() {
    let r = hunch("Westworld.S02E03.720p.WEB-DL/sample/Westworld.S01E01.sample.mkv");
    assert_eq!(r.title(), Some("Westworld"));
    assert_eq!(r.season(), Some(2));
    assert_eq!(r.episode(), Some(3));
}

#[test]
fn sample_in_pack_folder_keeps_its_own_episode() {
    let r = hunch("Show.S01E01-E10.Complete.720p/Sample/Show.S01E05.sample.mkv");
    assert_eq!(r.episode(), Some(5));
}
