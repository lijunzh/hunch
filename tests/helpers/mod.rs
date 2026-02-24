//! Custom parser for guessit YAML test fixture files.
//!
//! guessit test files use YAML features serde_yaml can't handle:
//! - Duplicate keys (same filename tested with different expected values)
//! - Multi-key syntax (`? key1 \n ? key2 \n : value`)
//! - Special prefixes: `+` (name_only), `-` (negated)

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub filename: String,
    pub expected: HashMap<String, String>,
}

pub fn load_test_cases(path: &str) -> Vec<TestCase> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));

    let mut defaults: HashMap<String, String> = HashMap::new();
    let mut cases: Vec<TestCase> = Vec::new();

    // Parse into (keys, props) groups.
    let groups = parse_groups(&content);

    for (keys, props) in &groups {
        // Handle __default__.
        if keys.len() == 1 && keys[0] == "__default__" {
            defaults = props.clone();
            continue;
        }

        for key in keys {
            // Skip special-prefixed keys.
            if key.starts_with('+') || key.starts_with('-') {
                continue;
            }

            // Merge defaults with this entry's props.
            let mut expected = defaults.clone();
            for (k, v) in props {
                if let Some(stripped) = k.strip_prefix('-') {
                    expected.remove(stripped);
                } else {
                    expected.insert(k.clone(), v.clone());
                }
            }

            // Remove negated keys and skip entries with "options".
            expected.retain(|k, _| !k.starts_with('-'));
            if expected.contains_key("options") {
                continue;
            }

            cases.push(TestCase {
                filename: key.clone(),
                expected,
            });
        }
    }

    cases
}

/// Parse the YAML-ish file into groups of (keys, properties).
///
/// Each group is one or more `? key` lines followed by `: prop: value` lines.
fn parse_groups(content: &str) -> Vec<(Vec<String>, HashMap<String, String>)> {
    let mut groups: Vec<(Vec<String>, HashMap<String, String>)> = Vec::new();
    let mut current_keys: Vec<String> = Vec::new();
    let mut current_props: HashMap<String, String> = HashMap::new();
    let mut in_value = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // New key: `? filename`
        if let Some(rest) = trimmed.strip_prefix("? ") {
            if in_value {
                // We were reading props — flush the previous group.
                groups.push((current_keys.clone(), current_props.clone()));
                current_keys.clear();
                current_props.clear();
                in_value = false;
            }
            let key = rest.trim().to_string();
            current_keys.push(key);
            continue;
        }

        // Value block starts: `: prop: value`
        if let Some(rest) = trimmed.strip_prefix(": ") {
            in_value = true;
            parse_prop_line(rest, &mut current_props);
            continue;
        }

        // Continuation of value block (indented).
        if in_value && (line.starts_with(' ') || line.starts_with('\t')) {
            parse_prop_line(trimmed, &mut current_props);
            continue;
        }
    }

    // Flush last group.
    if !current_keys.is_empty() {
        groups.push((current_keys, current_props));
    }

    groups
}

fn parse_prop_line(line: &str, props: &mut HashMap<String, String>) {
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim().to_string();
        let value = value.trim().to_string();
        if !key.is_empty() {
            props.insert(key, value);
        }
    }
}
