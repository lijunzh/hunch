//! Issue #93 regression: generic category dirs pollute path-hint titles.
//!
//! Directory names like "Anime", "Chinese", "English" are library organization
//! labels, not show titles. They must be filtered by `is_generic_dir()` so
//! `extract_title_from_parent` skips them.

use hunch::hunch;

#[test]
fn issue_93_anime_dir_not_title() {
    // "Anime" is a category dir, not the show title.
    let r = hunch("Anime/Natsume Yuujinchou/S01E01.mkv");
    assert_ne!(
        r.title(),
        Some("Anime"),
        "category dir 'Anime' must not become the title"
    );
}

#[test]
fn issue_93_chinese_dir_not_title() {
    let r = hunch("Chinese/SomeShow/S01E01.mkv");
    assert_ne!(
        r.title(),
        Some("Chinese"),
        "language category 'Chinese' must not become the title"
    );
}

#[test]
fn issue_93_english_dir_not_title() {
    let r = hunch("English/SomeShow/S01E01.mkv");
    assert_ne!(
        r.title(),
        Some("English"),
        "language category 'English' must not become the title"
    );
}

#[test]
fn issue_93_japanese_dir_not_title() {
    let r = hunch("Japanese/SomeShow/S01E01.mkv");
    assert_ne!(
        r.title(),
        Some("Japanese"),
        "language category 'Japanese' must not become the title"
    );
}

#[test]
fn issue_93_documentary_dir_not_title() {
    let r = hunch("Documentary/Planet Earth/S01E01.mkv");
    assert_ne!(
        r.title(),
        Some("Documentary"),
        "category dir 'Documentary' must not become the title"
    );
}

#[test]
fn issue_93_kids_dir_not_title() {
    let r = hunch("Kids/Paw Patrol/S01E10.mkv");
    assert_ne!(
        r.title(),
        Some("Kids"),
        "category dir 'Kids' must not become the title"
    );
}

#[test]
fn issue_93_tokuten_eizou_dir_not_title() {
    let r = hunch("特典映像/[Group][ShowTitle][Concert][1080P].mkv");
    assert_ne!(
        r.title(),
        Some("特典映像"),
        "CJK bonus dir '特典映像' must not become the title"
    );
}

#[test]
fn issue_93_tokuten_dir_not_title() {
    let r = hunch("特典/bonus.mkv");
    assert_ne!(
        r.title(),
        Some("特典"),
        "CJK bonus dir '特典' must not become the title"
    );
}

#[test]
fn issue_93_sp_dir_not_title() {
    let r = hunch("SP/Special.720p.mkv");
    assert_ne!(
        r.title(),
        Some("SP"),
        "bonus dir 'SP' must not become the title"
    );
}
