//! Regression tests: validate hunch against guessit's YAML test vectors.
//!
//! Test vectors are copied from guessit (LGPL-3.0) — see tests/fixtures/ATTRIBUTION.md.
//! Each YAML file is parsed and every test case is run through `hunch()`.
//!
//! Tests that hunch currently fails are marked `#[ignore]` so CI stays green,
//! but you can run them with `cargo test -- --ignored` to track progress.

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
fn min_pass_rate(path: &str) -> f64 {
    match path {
        // Ratchet these up as we improve. Never lower them.
        // Rule files (isolated property tests).
        "tests/fixtures/rules/screen_size.yml" => 100.0,
        "tests/fixtures/rules/other.yml" => 93.0,
        "tests/fixtures/rules/video_codec.yml" => 84.0,
        "tests/fixtures/rules/edition.yml" => 79.0,
        "tests/fixtures/rules/audio_codec.yml" => 74.0,
        "tests/fixtures/rules/source.yml" => 58.0,
        "tests/fixtures/rules/release_group.yml" => 55.0,
        "tests/fixtures/rules/title.yml" => 42.0,
        "tests/fixtures/rules/episodes.yml" => 42.0,
        // Full-filename tests (all properties must match).
        "tests/fixtures/movies.yml" => 26.0,
        "tests/fixtures/various.yml" => 25.0,
        "tests/fixtures/episodes.yml" => 24.0,
        _ => 0.0,
    }
}

guessit_test_file!(rules_edition, "tests/fixtures/rules/edition.yml");
guessit_test_file!(rules_other, "tests/fixtures/rules/other.yml");
guessit_test_file!(rules_audio_codec, "tests/fixtures/rules/audio_codec.yml");
guessit_test_file!(rules_video_codec, "tests/fixtures/rules/video_codec.yml");
guessit_test_file!(rules_source, "tests/fixtures/rules/source.yml");
guessit_test_file!(rules_screen_size, "tests/fixtures/rules/screen_size.yml");
guessit_test_file!(rules_release_group, "tests/fixtures/rules/release_group.yml");
guessit_test_file!(rules_episodes, "tests/fixtures/rules/episodes.yml");
guessit_test_file!(rules_title, "tests/fixtures/rules/title.yml");
guessit_test_file!(movies, "tests/fixtures/movies.yml");
guessit_test_file!(episodes, "tests/fixtures/episodes.yml");
guessit_test_file!(various, "tests/fixtures/various.yml");
