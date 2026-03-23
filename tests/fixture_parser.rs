mod helpers;

use helpers::load_test_cases;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn write_fixture(contents: &str) -> (TempDir, String) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("fixture.yml");
    fs::write(&path, contents).expect("write fixture");
    (dir, path.to_string_lossy().into_owned())
}

fn expected_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

#[test]
fn applies_defaults_and_negated_overrides() {
    let (_dir, path) = write_fixture(
        r#"
? __default__
: source: Blu-ray
  language: English
  other: HDR

? Movie.2024.mkv
: title: Movie
  -language:
  source: Web
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].filename, "Movie.2024.mkv");
    assert_eq!(
        cases[0].expected,
        expected_map(&[("title", "Movie"), ("source", "Web"), ("other", "HDR")])
    );
}

#[test]
fn expands_multi_key_groups_into_multiple_cases() {
    let (_dir, path) = write_fixture(
        r#"
? Show.S01E01.mkv
? Show.S01E02.mkv
: type: episode
  title: Show
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].filename, "Show.S01E01.mkv");
    assert_eq!(cases[1].filename, "Show.S01E02.mkv");
    assert_eq!(
        cases[0].expected,
        expected_map(&[("type", "episode"), ("title", "Show")])
    );
    assert_eq!(
        cases[1].expected,
        expected_map(&[("type", "episode"), ("title", "Show")])
    );
}

#[test]
fn preserves_duplicate_filenames_as_separate_cases() {
    let (_dir, path) = write_fixture(
        r#"
? Duplicate.mkv
: type: movie

? Duplicate.mkv
: title: Duplicate
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].filename, "Duplicate.mkv");
    assert_eq!(cases[1].filename, "Duplicate.mkv");
    assert_eq!(cases[0].expected, expected_map(&[("type", "movie")]));
    assert_eq!(cases[1].expected, expected_map(&[("title", "Duplicate")]));
}

#[test]
fn skips_options_and_prefixed_keys() {
    let (_dir, path) = write_fixture(
        r#"
? +name_only_case.mkv
: type: movie

? -negated_case.mkv
: type: movie

? Options.case.mkv
: options: ignore
  type: movie

? Real.case.mkv
: type: episode
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].filename, "Real.case.mkv");
    assert_eq!(cases[0].expected, expected_map(&[("type", "episode")]));
}

#[test]
fn parses_lists_and_preserves_commas_inside_quoted_values() {
    let (_dir, path) = write_fixture(
        r#"
? Episode.mkv
: language:
  - English
  - Japanese
  subtitle_language:
  episode_title: "Right Place, Wrong Time"
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 1);
    assert_eq!(
        cases[0].expected.get("language"),
        Some(&"[English, Japanese]".to_string())
    );
    assert_eq!(
        cases[0].expected.get("subtitle_language"),
        Some(&"".to_string())
    );
    assert_eq!(
        cases[0].expected.get("episode_title"),
        Some(&"Right Place, Wrong Time".to_string())
    );
}

#[test]
fn strips_inline_comments_but_not_inside_quotes() {
    let (_dir, path) = write_fixture(
        r#"
? Commented.mkv
: title: Movie # real comment
  note: "Keep # inside quotes"
"#,
    );

    let cases = load_test_cases(&path);
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].expected.get("title"), Some(&"Movie".to_string()));
    assert_eq!(
        cases[0].expected.get("note"),
        Some(&"Keep # inside quotes".to_string())
    );
}
