//! Regression tests: validate hunch against guessit's YAML test vectors.
//!
//! Test vectors are copied from guessit (LGPL-3.0) — see tests/fixtures/ATTRIBUTION.md.
//! Each YAML file is parsed and every test case is run through `hunch()`.
//!
//! Each fixture file has a minimum pass-rate floor (ratchet pattern).
//! Tests assert we don't regress below that floor. As accuracy improves,
//! ratchet the floors up — never lower them.
//!
//! Run the full compatibility report:
//!     cargo test compatibility_report -- --ignored --nocapture

mod helpers;

use helpers::{TestCase, load_test_cases};
use hunch::hunch;
use std::collections::HashMap;

// ── Language normalization ──────────────────────────────────────────

/// Normalize language strings so that "English", "en", "eng" all compare equal.
fn normalize_language(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "en" | "eng" | "english" => "en".into(),
        "fr" | "fre" | "fra" | "french" => "fr".into(),
        "es" | "spa" | "spanish" => "es".into(),
        "de" | "ger" | "deu" | "german" => "de".into(),
        "it" | "ita" | "italian" => "it".into(),
        "pt" | "por" | "portuguese" => "pt".into(),
        "ja" | "jpn" | "japanese" => "ja".into(),
        "ko" | "kor" | "korean" => "ko".into(),
        "zh" | "chi" | "zho" | "chinese" => "zh".into(),
        "ru" | "rus" | "russian" => "ru".into(),
        "ar" | "ara" | "arabic" => "ar".into(),
        "hi" | "hin" | "hindi" => "hi".into(),
        "nl" | "dut" | "nld" | "dutch" => "nl".into(),
        "pl" | "pol" | "polish" => "pl".into(),
        "sv" | "swe" | "swedish" => "sv".into(),
        "no" | "nor" | "norwegian" => "no".into(),
        "da" | "dan" | "danish" => "da".into(),
        "fi" | "fin" | "finnish" => "fi".into(),
        "hu" | "hun" | "hungarian" => "hu".into(),
        "cs" | "cze" | "ces" | "czech" => "cs".into(),
        "ro" | "rum" | "ron" | "romanian" => "ro".into(),
        "el" | "gre" | "ell" | "greek" => "el".into(),
        "tr" | "tur" | "turkish" => "tr".into(),
        "he" | "heb" | "hebrew" => "he".into(),
        "uk" | "ukr" | "ukrainian" => "uk".into(),
        "bg" | "bul" | "bulgarian" => "bg".into(),
        "hr" | "hrv" | "croatian" => "hr".into(),
        "sr" | "srp" | "serbian" => "sr".into(),
        "sk" | "slo" | "slk" | "slovak" => "sk".into(),
        "sl" | "slv" | "slovenian" => "sl".into(),
        "et" | "est" | "estonian" => "et".into(),
        "lv" | "lav" | "latvian" => "lv".into(),
        "lt" | "lit" | "lithuanian" => "lt".into(),
        "ca" | "cat" | "catalan" => "ca".into(),
        "mul" | "multi" | "multiple languages" => "mul".into(),
        "und" | "undetermined" => "und".into(),
        other => other.to_string(),
    }
}

const LANG_PROPS: &[&str] = &["language", "subtitle_language"];

// ── Core comparison logic ───────────────────────────────────────────

/// Per-property result from checking a single test case.
struct PropResult {
    property: String,
    passed: bool,
}

/// Check a test case, returning per-property pass/fail and overall failures.
fn check(tc: &TestCase) -> (Vec<PropResult>, Vec<String>) {
    let result = hunch(&tc.filename);
    let got = result.to_flat_map();
    let mut prop_results = Vec::new();
    let mut failures = Vec::new();

    for (key, expected) in &tc.expected {
        let actual = got.get(key);
        let expected_str = expected.trim().to_lowercase();

        // !!null means the property should be absent.
        if expected_str == "!!null" || expected_str == "null" {
            let ok = actual.is_none();
            prop_results.push(PropResult {
                property: key.clone(),
                passed: ok,
            });
            if !ok {
                failures.push(format!("{key}: expected absent, got {:?}", actual.unwrap()));
            }
            continue;
        }

        let is_lang = LANG_PROPS.contains(&key.as_str());

        // Normalize a single value (language-aware).
        let norm = |s: &str| -> String {
            let v = s.trim().to_lowercase();
            if is_lang { normalize_language(&v) } else { v }
        };

        // Parse expected: could be a single value or `[ a, b, c ]` list.
        let expected_values = parse_value_list(&expected_str);
        let mut expected_set: Vec<String> = expected_values.iter().map(|s| norm(s)).collect();
        expected_set.sort();

        // Parse actual from JSON.
        let actual_values: Vec<String> = match actual {
            Some(serde_json::Value::String(s)) => vec![norm(s)],
            Some(serde_json::Value::Number(n)) => vec![n.to_string()],
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => norm(s),
                    serde_json::Value::Number(n) => n.to_string(),
                    other => other.to_string().to_lowercase(),
                })
                .collect(),
            Some(other) => vec![other.to_string().to_lowercase()],
            None => vec![],
        };
        let mut actual_set: Vec<String> = actual_values.clone();
        actual_set.sort();

        let ok = expected_set == actual_set;
        prop_results.push(PropResult {
            property: key.clone(),
            passed: ok,
        });
        if !ok {
            let exp_display = if expected_set.len() == 1 {
                expected_set[0].clone()
            } else {
                format!("[ {} ]", expected_set.join(", "))
            };
            let act_display = if actual_values.is_empty() {
                String::new()
            } else if actual_values.len() == 1 {
                actual_values[0].clone()
            } else {
                format!("[ {} ]", actual_values.join(", "))
            };
            failures.push(format!(
                "{key}: expected {exp_display:?}, got {act_display:?}"
            ));
        }
    }
    (prop_results, failures)
}

/// Parse a YAML-style value that may be a list: `[ a, b, c ]` or `[a, b]`.
/// Returns a vec of individual values. Single values return a 1-element vec.
fn parse_value_list(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    let strip_quotes = |v: &str| -> String {
        let v = v.trim();
        if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
            v[1..v.len() - 1].to_string()
        } else {
            v.to_string()
        }
    };
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner
            .split(',')
            .map(strip_quotes)
            .filter(|v| !v.is_empty())
            .collect()
    } else {
        vec![strip_quotes(trimmed)]
    }
}

// ── Per-file regression tests (ratchet pattern) ─────────────────────

macro_rules! guessit_test_file {
    ($mod_name:ident, $path:expr) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn passing() {
                let cases = load_test_cases($path);
                assert!(!cases.is_empty(), "No test cases loaded from {}", $path);

                let mut passed = 0;
                let mut failed_cases: Vec<(&str, Vec<String>)> = Vec::new();
                let mut prop_fail_counts: HashMap<String, usize> = HashMap::new();

                for tc in &cases {
                    let (prop_results, failures) = check(tc);
                    if failures.is_empty() {
                        passed += 1;
                    } else {
                        // Count per-property failures.
                        for pr in &prop_results {
                            if !pr.passed {
                                *prop_fail_counts.entry(pr.property.clone()).or_insert(0) += 1;
                            }
                        }
                        failed_cases.push((&tc.filename, failures));
                    }
                }

                let total = cases.len();
                let fail_count = failed_cases.len();
                let rate = (passed as f64 / total as f64) * 100.0;

                eprintln!(
                    "[{}] {passed}/{total} passed ({rate:.1}%), {fail_count} failed",
                    $path
                );

                // Always show property-level breakdown (compact, top 10).
                if !prop_fail_counts.is_empty() {
                    let mut sorted: Vec<_> = prop_fail_counts.iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(a.1));
                    let top: Vec<String> = sorted
                        .iter()
                        .take(10)
                        .map(|(k, v)| format!("{}:{}", k, v))
                        .collect();
                    let suffix = if sorted.len() > 10 {
                        format!(" (+{} more)", sorted.len() - 10)
                    } else {
                        String::new()
                    };
                    eprintln!("  failing props: {}{}", top.join(", "), suffix);
                }

                // Show individual failures when HUNCH_DUMP_FAILURES=1.
                let dump_limit = std::env::var("HUNCH_DUMP_FAILURES")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                if dump_limit > 0 {
                    for (name, fails) in failed_cases.iter().take(dump_limit) {
                        eprintln!("  FAIL: {}", name);
                        for f in fails {
                            eprintln!("    {}", f);
                        }
                    }
                }

                let min = min_pass_rate($path);
                assert!(
                    rate >= min,
                    "Pass rate {rate:.1}% dropped below minimum {min}% for {}",
                    $path
                );
            }
        }
    };
}

/// Minimum pass rates per fixture file — ratchet up, never down.
/// Set to (actual - 2%). Last calibrated: 2026-02-26 (v0.2.1).
fn min_pass_rate(path: &str) -> f64 {
    match path {
        "tests/fixtures/rules/screen_size.yml" => 98.0,
        "tests/fixtures/rules/size.yml" => 98.0,
        "tests/fixtures/rules/edition.yml" => 98.0,
        "tests/fixtures/rules/source.yml" => 98.0,
        "tests/fixtures/rules/audio_codec.yml" => 98.0,
        "tests/fixtures/rules/video_codec.yml" => 98.0,
        "tests/fixtures/rules/part.yml" => 98.0,
        "tests/fixtures/rules/common_words.yml" => 97.0,
        "tests/fixtures/rules/other.yml" => 94.0,
        "tests/fixtures/rules/episodes.yml" => 92.0,
        "tests/fixtures/rules/release_group.yml" => 76.0,
        "tests/fixtures/rules/title.yml" => 75.0,
        "tests/fixtures/rules/language.yml" => 98.0,
        "tests/fixtures/rules/date.yml" => 73.0,
        "tests/fixtures/rules/bonus.yml" => 64.0,
        "tests/fixtures/rules/country.yml" => 64.0,
        "tests/fixtures/rules/film.yml" => 64.0,
        "tests/fixtures/rules/cd.yml" => 48.0,
        "tests/fixtures/rules/website.yml" => 48.0,
        "tests/fixtures/movies.yml" => 66.0,
        "tests/fixtures/episodes.yml" => 62.0,
        "tests/fixtures/various.yml" => 63.0,
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
guessit_test_file!(
    rules_release_group,
    "tests/fixtures/rules/release_group.yml"
);
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

// Community-reported cases (beyond guessit's test suite).
// Gracefully skips when no cases are present yet.
mod community {
    use super::*;

    #[test]
    fn passing() {
        let cases = load_test_cases("tests/fixtures/community.yml");
        if cases.is_empty() {
            eprintln!("[community.yml] No test cases yet — skipping");
            return;
        }

        let mut passed = 0;
        let mut failed_cases: Vec<(&str, Vec<String>)> = Vec::new();

        for tc in &cases {
            let (_prop_results, failures) = check(tc);
            if failures.is_empty() {
                passed += 1;
            } else {
                failed_cases.push((&tc.filename, failures));
            }
        }

        let total = cases.len();
        let rate = (passed as f64 / total as f64) * 100.0;
        eprintln!("[community.yml] {passed}/{total} passed ({rate:.1}%)");

        // Community cases are our targets — all must pass.
        assert!(
            failed_cases.is_empty(),
            "{} community test case(s) failed:\n{}",
            failed_cases.len(),
            failed_cases
                .iter()
                .map(|(name, fails)| format!("  {}\n    {}", name, fails.join("\n    ")))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

// ── Full compatibility report ───────────────────────────────────────

const ALL_FIXTURES: &[(&str, &str)] = &[
    ("movies.yml", "tests/fixtures/movies.yml"),
    ("episodes.yml", "tests/fixtures/episodes.yml"),
    ("various.yml", "tests/fixtures/various.yml"),
    ("community.yml", "tests/fixtures/community.yml"),
    (
        "rules/audio_codec.yml",
        "tests/fixtures/rules/audio_codec.yml",
    ),
    ("rules/bonus.yml", "tests/fixtures/rules/bonus.yml"),
    ("rules/cd.yml", "tests/fixtures/rules/cd.yml"),
    (
        "rules/common_words.yml",
        "tests/fixtures/rules/common_words.yml",
    ),
    ("rules/country.yml", "tests/fixtures/rules/country.yml"),
    ("rules/date.yml", "tests/fixtures/rules/date.yml"),
    ("rules/edition.yml", "tests/fixtures/rules/edition.yml"),
    ("rules/episodes.yml", "tests/fixtures/rules/episodes.yml"),
    ("rules/film.yml", "tests/fixtures/rules/film.yml"),
    ("rules/language.yml", "tests/fixtures/rules/language.yml"),
    ("rules/other.yml", "tests/fixtures/rules/other.yml"),
    ("rules/part.yml", "tests/fixtures/rules/part.yml"),
    (
        "rules/release_group.yml",
        "tests/fixtures/rules/release_group.yml",
    ),
    (
        "rules/screen_size.yml",
        "tests/fixtures/rules/screen_size.yml",
    ),
    ("rules/size.yml", "tests/fixtures/rules/size.yml"),
    ("rules/source.yml", "tests/fixtures/rules/source.yml"),
    ("rules/title.yml", "tests/fixtures/rules/title.yml"),
    (
        "rules/video_codec.yml",
        "tests/fixtures/rules/video_codec.yml",
    ),
    ("rules/website.yml", "tests/fixtures/rules/website.yml"),
];

/// Full compatibility report — run with:
///     cargo test compatibility_report -- --ignored --nocapture
#[test]
#[ignore]
fn compatibility_report() {
    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut total_cases = 0usize;
    // property -> (passed, failed)
    let mut prop_stats: HashMap<String, (usize, usize)> = HashMap::new();
    let mut sample_failures: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut single_prop_failures: HashMap<String, usize> = HashMap::new();
    let mut single_prop_details: Vec<(String, String)> = Vec::new();

    eprintln!("\n{}", "=".repeat(70));
    eprintln!("HUNCH COMPATIBILITY REPORT");
    eprintln!("{}", "=".repeat(70));
    eprintln!("\nPASS RATE BY TEST FILE:");
    eprintln!(
        "  {:<35} {:>7} {:>7} {:>7}",
        "File", "Passed", "Total", "Rate"
    );
    eprintln!(
        "  {:<35} {:>7} {:>7} {:>7}",
        "-".repeat(35),
        "-".repeat(7),
        "-".repeat(7),
        "-".repeat(7)
    );

    for (label, path) in ALL_FIXTURES {
        let cases = load_test_cases(path);
        let mut file_passed = 0usize;

        for tc in &cases {
            total_cases += 1;
            let (prop_results, failures) = check(tc);

            for pr in &prop_results {
                let entry = prop_stats.entry(pr.property.clone()).or_insert((0, 0));
                if pr.passed {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
            }

            if failures.is_empty() {
                file_passed += 1;
                total_passed += 1;
            } else {
                total_failed += 1;
                // Track single-property failures for prioritization.
                if failures.len() == 1 {
                    let prop_name = failures[0].split(':').next().unwrap_or("").trim();
                    *single_prop_failures
                        .entry(prop_name.to_string())
                        .or_insert(0) += 1;
                    // Collect single-prop failure details for targeted debugging.
                    single_prop_details.push((tc.filename.clone(), failures[0].clone()));
                }
                if sample_failures.len() < 30 {
                    sample_failures.push((label.to_string(), tc.filename.clone(), failures));
                }
            }
        }

        let total = cases.len();
        let rate = if total > 0 {
            (file_passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!(
            "  {:<35} {:>7} {:>7} {:>6.1}%",
            label, file_passed, total, rate
        );
    }

    // Overall summary.
    let overall_rate = if total_cases > 0 {
        (total_passed as f64 / total_cases as f64) * 100.0
    } else {
        0.0
    };
    eprintln!(
        "\nOVERALL: {total_passed}/{total_cases} passed ({overall_rate:.1}%), {total_failed} failed"
    );

    // Per-property breakdown.
    let mut props: Vec<_> = prop_stats.iter().collect();
    props.sort_by(|a, b| {
        let rate_a = a.1.0 as f64 / (a.1.0 + a.1.1) as f64;
        let rate_b = b.1.0 as f64 / (b.1.0 + b.1.1) as f64;
        rate_b.partial_cmp(&rate_a).unwrap()
    });

    eprintln!("\nPER-PROPERTY ACCURACY:");
    eprintln!(
        "  {:<25} {:>7} {:>7} {:>7}",
        "Property", "Passed", "Failed", "Rate"
    );
    eprintln!(
        "  {:<25} {:>7} {:>7} {:>7}",
        "-".repeat(25),
        "-".repeat(7),
        "-".repeat(7),
        "-".repeat(7)
    );
    for (prop, (p, f)) in &props {
        let total = p + f;
        let rate = if total > 0 {
            (*p as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let emoji = if rate >= 95.0 {
            "✅"
        } else if rate >= 80.0 {
            "🟡"
        } else if rate >= 50.0 {
            "⚠️ "
        } else {
            "❌"
        };
        eprintln!("  {emoji} {:<23} {:>7} {:>7} {:>6.1}%", prop, p, f, rate);
    }

    // Sample failures.
    if !sample_failures.is_empty() {
        // Single-property failure analysis (highest-ROI fixes).
        let mut spf: Vec<_> = single_prop_failures.iter().collect();
        spf.sort_by(|a, b| b.1.cmp(a.1));
        let total_single: usize = spf.iter().map(|(_, c)| **c).sum();
        eprintln!("\nSINGLE-PROPERTY FAILURES ({total_single} test cases fail on exactly 1 prop):");
        eprintln!("  Fixing any of these would directly increase the overall pass rate.");
        eprintln!("  {:<25} {:>7}", "Property", "Cases");
        eprintln!("  {:<25} {:>7}", "-".repeat(25), "-".repeat(7));
        for (prop, count) in &spf {
            eprintln!("  {:<25} {:>7}", prop, count);
        }

        // Show single-property failure details for targeted debugging.
        let target_props = [
            "type",
            "screen_size",
            "proper_count",
            "container",
            "video_profile",
            "disc",
            "audio_channels",
            "edition",
            "video_codec",
            "audio_codec",
            "year",
            "title",
            "episode",
            "season",
            "source",
            "release_group",
            "episode_title",
            "other",
            "language",
        ];
        eprintln!("\nSINGLE-PROPERTY FAILURE DETAILS (targeted):");
        for (filename, failure) in &single_prop_details {
            let prop = failure.split(':').next().unwrap_or("").trim();
            if target_props.contains(&prop) {
                let short_fn = if filename.len() > 70 {
                    format!("{}...", &filename[..67])
                } else {
                    filename.clone()
                };
                eprintln!("  [{prop}] {short_fn}");
                eprintln!("    {failure}");
            }
        }

        eprintln!("\nSAMPLE FAILURES (first 30):");
        for (file, filename, fails) in &sample_failures {
            let short = if filename.len() > 70 {
                format!("{}...", &filename[..70])
            } else {
                filename.clone()
            };
            eprintln!("\n  [{file}] {short}");
            for f in fails {
                eprintln!("    {f}");
            }
        }
    }

    eprintln!("\n{}", "=".repeat(70));
}
