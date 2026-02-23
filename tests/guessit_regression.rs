//! Regression tests: validate hunch against guessit's YAML test vectors.
//!
//! Test vectors are copied from guessit (LGPL-3.0) — see tests/fixtures/ATTRIBUTION.md.
//! Each YAML file is parsed and every test case is run through `hunch()`.
//!
//! Each fixture file has a minimum pass-rate floor (ratchet pattern).
//! Tests assert we don't regress below that floor. As accuracy improves,
//! ratchet the floors up — never lower them.

mod helpers;

use helpers::{load_test_cases, TestCase};
use hunch::hunch;

/// Run a single test case: parse the filename and compare expected properties.
fn check(tc: &TestCase) -> Vec<String> {
    let result = hunch(&tc.filename);
    let got = result.to_flat_map();
    let mut failures = Vec::new();

    for (key, expected) in &tc.expected {
        let actual = got.get(key);
        let expected_str = normalize(expected);

        // !!null means the property should be absent.
        if expected_str == "!!null" || expected_str == "null" {
            if actual.is_some() {
                let actual_desc = format!("{:?}", actual.unwrap());
                failures.push(format!(
                    "{key}: expected absent (!!null), got {actual_desc}",
                ));
            }
            continue;
        }

        let actual_str = match actual {
            Some(serde_json::Value::String(s)) => s.to_lowercase(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Array(arr)) => {
                // For multi-value properties, check if expected is in the array.
                let values: Vec<String> = arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.to_lowercase(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string().to_lowercase(),
                    })
                    .collect();
                if values.iter().any(|v| *v == expected_str) {
                    continue; // Match found in array.
                }
                values.join(", ")
            }
            Some(other) => other.to_string().to_lowercase(),
            None => String::new(),
        };

        if actual_str != expected_str {
            failures.push(format!(
                "{key}: expected {expected_str:?}, got {actual_str:?}",
            ));
        }
    }
    failures
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

// ---- Generated test modules per fixture file ----

macro_rules! guessit_test_file {
    ($mod_name:ident, $path:expr) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn passing() {
                let cases = load_test_cases($path);
                assert!(!cases.is_empty(), "No test cases loaded from {}", $path);

                let mut passed = 0;
                let mut failed_cases = Vec::new();

                for tc in &cases {
                    let failures = check(tc);
                    if failures.is_empty() {
                        passed += 1;
                    } else {
                        failed_cases.push((&tc.filename, failures));
                    }
                }

                let total = cases.len();
                let fail_count = failed_cases.len();
                let rate = (passed as f64 / total as f64) * 100.0;

                // Print summary for visibility.
                eprintln!(
                    "[{}] {passed}/{total} passed ({rate:.1}%), {fail_count} failed",
                    $path
                );

                // We don't assert all pass — we assert we don't regress below
                // a threshold. This threshold should only go UP over time.
                let min_pass_rate = min_pass_rate($path);
                assert!(
                    rate >= min_pass_rate,
                    "Pass rate {rate:.1}% dropped below minimum {min_pass_rate}% for {}",
                    $path
                );
            }
        }
    };
}

/// Minimum pass rates per fixture file. These are floors — ratchet them up
/// as we improve. Never lower them.
///
/// Set to (actual_rate - 2%) to catch real regressions without false alarms.
/// Last calibrated: 2026-02-23.
fn min_pass_rate(path: &str) -> f64 {
    match path {
        // Rule files (isolated property tests).
        "tests/fixtures/rules/screen_size.yml" => 98.0, // actual: 100.0%
        "tests/fixtures/rules/size.yml" => 98.0,         // actual: 100.0%
        "tests/fixtures/rules/other.yml" => 93.0,        // actual: 95.7%
        "tests/fixtures/rules/common_words.yml" => 91.0, // actual: 93.6%
        "tests/fixtures/rules/video_codec.yml" => 84.0,  // actual: 86.7%
        "tests/fixtures/rules/edition.yml" => 79.0,      // actual: 81.8%
        "tests/fixtures/rules/audio_codec.yml" => 74.0,  // actual: 76.5%
        "tests/fixtures/rules/bonus.yml" => 64.0,        // actual: 66.7%
        "tests/fixtures/rules/date.yml" => 60.0,         // actual: 62.5%
        "tests/fixtures/rules/source.yml" => 58.0,       // actual: 60.9%
        "tests/fixtures/rules/release_group.yml" => 55.0,// actual: 57.9%
        "tests/fixtures/rules/part.yml" => 53.0,         // actual: 55.6%
        "tests/fixtures/rules/cd.yml" => 48.0,           // actual: 50.0%
        "tests/fixtures/rules/website.yml" => 48.0,      // actual: 50.0%
        "tests/fixtures/rules/title.yml" => 42.0,        // actual: 44.4%
        "tests/fixtures/rules/episodes.yml" => 42.0,     // actual: 44.4%
        "tests/fixtures/rules/country.yml" => 31.0,      // actual: 33.3%
        "tests/fixtures/rules/language.yml" => 20.0,     // actual: 22.2%
        "tests/fixtures/rules/film.yml" => 0.0,          // actual: 0.0%
        // Full-filename tests (all properties must match).
        "tests/fixtures/movies.yml" => 26.0,             // actual: 28.6%
        "tests/fixtures/various.yml" => 25.0,            // actual: 27.4%
        "tests/fixtures/episodes.yml" => 24.0,           // actual: 26.6%
        _ => 0.0,
    }
}

// Rule files (isolated property tests).
guessit_test_file!(rules_edition, "tests/fixtures/rules/edition.yml");
guessit_test_file!(rules_other, "tests/fixtures/rules/other.yml");
guessit_test_file!(rules_audio_codec, "tests/fixtures/rules/audio_codec.yml");
guessit_test_file!(rules_video_codec, "tests/fixtures/rules/video_codec.yml");
guessit_test_file!(rules_source, "tests/fixtures/rules/source.yml");
guessit_test_file!(rules_screen_size, "tests/fixtures/rules/screen_size.yml");
guessit_test_file!(rules_release_group, "tests/fixtures/rules/release_group.yml");
guessit_test_file!(rules_episodes, "tests/fixtures/rules/episodes.yml");
guessit_test_file!(rules_title, "tests/fixtures/rules/title.yml");
guessit_test_file!(rules_bonus, "tests/fixtures/rules/bonus.yml");
guessit_test_file!(rules_cd, "tests/fixtures/rules/cd.yml");
guessit_test_file!(rules_common_words, "tests/fixtures/rules/common_words.yml");
guessit_test_file!(rules_country, "tests/fixtures/rules/country.yml");
guessit_test_file!(rules_date, "tests/fixtures/rules/date.yml");
guessit_test_file!(rules_film, "tests/fixtures/rules/film.yml");
guessit_test_file!(rules_language, "tests/fixtures/rules/language.yml");
guessit_test_file!(rules_part, "tests/fixtures/rules/part.yml");
guessit_test_file!(rules_size, "tests/fixtures/rules/size.yml");
guessit_test_file!(rules_website, "tests/fixtures/rules/website.yml");

// Full-filename tests (all properties must match).
guessit_test_file!(movies, "tests/fixtures/movies.yml");
guessit_test_file!(episodes, "tests/fixtures/episodes.yml");
guessit_test_file!(various, "tests/fixtures/various.yml");
