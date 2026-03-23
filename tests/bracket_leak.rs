//! Issue #91 regression: bracket content leaks into episode_title and release_group.
//!
//! CJK fansub bracket format: `[Group] Title - NN [metadata][subtitle_tag].ext`
//! "Bilibili" (a streaming platform inside metadata brackets) was leaking as
//! episode_title, and the release_group was picking up bracket fragments
//! instead of the actual group name.

use hunch::hunch;

// ── Episode title must not leak bracket metadata ──────────────────────

#[test]
fn issue_91_no_episode_title_from_bracket() {
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 Kimetsu no Yaiba - 01 [Bilibili WEB-DL 1080P AVC 8bit AAC MP4][简体内嵌].mp4",
    );
    assert_eq!(r.title(), Some("鬼灭之刃 Kimetsu no Yaiba"));
    assert_eq!(r.episode(), Some(1));
    assert_eq!(
        r.episode_title(),
        None,
        "bracket content 'Bilibili' must not leak into episode_title"
    );
    assert_eq!(r.source(), Some("Web"));
}

#[test]
fn issue_91_no_episode_title_different_episode() {
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 Kimetsu no Yaiba - 12 [Bilibili WEB-DL 1080P AVC 8bit AAC MKV][简繁内封].mkv",
    );
    assert_eq!(r.title(), Some("鬼灭之刃 Kimetsu no Yaiba"));
    assert_eq!(r.episode(), Some(12));
    assert_eq!(
        r.episode_title(),
        None,
        "bracket content must not leak regardless of episode number"
    );
}

#[test]
fn issue_91_real_episode_title_preserved() {
    // Part1 is real episode title content (between ep number and bracket).
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 游郭篇 Kimetsu no Yaiba Yuukaku-hen - 01 Part1 [Bilibili WEB-DL 1080P AVC 8bit AAC MP4][简体内嵌].mp4",
    );
    assert_eq!(
        r.title(),
        Some("鬼灭之刃 游郭篇 Kimetsu no Yaiba Yuukaku-hen")
    );
    assert_eq!(r.episode(), Some(1));
    // Part1 appears between the episode number and the bracket — it's real content.
    assert!(r.episode_title().is_some(), "Part1 should be episode_title");
}

// ── Release group must be the first bracket, not bracket fragments ────

#[test]
fn issue_91_release_group_from_first_bracket() {
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 Kimetsu no Yaiba - 01 [Bilibili WEB-DL 1080P AVC 8bit AAC MP4][简体内嵌].mp4",
    );
    assert_eq!(
        r.release_group(),
        Some("Prejudice-Studio"),
        "release_group must come from [Prejudice-Studio], not bracket fragments"
    );
}

#[test]
fn issue_91_release_group_consistent_across_episodes() {
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 Kimetsu no Yaiba - 12 [Bilibili WEB-DL 1080P AVC 8bit AAC MKV][简繁内封].mkv",
    );
    assert_eq!(r.release_group(), Some("Prejudice-Studio"));
}

#[test]
fn issue_91_release_group_with_part_episode() {
    let r = hunch(
        "[Prejudice-Studio] 鬼灭之刃 游郭篇 Kimetsu no Yaiba Yuukaku-hen - 01 Part1 [Bilibili WEB-DL 1080P AVC 8bit AAC MP4][简体内嵌].mp4",
    );
    assert_eq!(r.release_group(), Some("Prejudice-Studio"));
}
