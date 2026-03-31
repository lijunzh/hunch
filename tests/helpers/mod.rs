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
/// Handles YAML list values (`- item`) by joining them as comma-separated.
fn parse_groups(content: &str) -> Vec<(Vec<String>, HashMap<String, String>)> {
    let mut groups: Vec<(Vec<String>, HashMap<String, String>)> = Vec::new();
    let mut current_keys: Vec<String> = Vec::new();
    let mut current_props: HashMap<String, String> = HashMap::new();
    let mut in_value = false;
    let mut current_list_key: Option<String> = None;
    let mut current_list_items: Vec<String> = Vec::new();

    let flush_list =
        |key: &Option<String>, items: &mut Vec<String>, props: &mut HashMap<String, String>| {
            if let Some(k) = key {
                if items.is_empty() {
                    // Key declared with no list items → preserve empty string.
                    props.entry(k.clone()).or_default();
                } else if items.len() == 1 {
                    props.insert(k.clone(), items[0].clone());
                } else {
                    // Wrap in brackets so parse_value_list can distinguish
                    // real multi-value lists from single values with commas
                    // (e.g., episode_title: "Right Place, Wrong Time").
                    props.insert(k.clone(), format!("[{}]", items.join(", ")));
                }
                items.clear();
            }
        };

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // New key: `? filename`
        if let Some(rest) = trimmed.strip_prefix("? ") {
            if in_value {
                // Flush any pending list.
                flush_list(
                    &current_list_key,
                    &mut current_list_items,
                    &mut current_props,
                );
                current_list_key = None;
                // Flush the previous group.
                groups.push((current_keys.clone(), current_props.clone()));
                current_keys.clear();
                current_props.clear();
                in_value = false;
            }
            let key = strip_yaml_quotes(rest.trim());
            current_keys.push(key);
            continue;
        }

        // Value block starts: `: prop: value`
        if let Some(rest) = trimmed.strip_prefix(": ") {
            in_value = true;
            // Flush any pending list.
            flush_list(
                &current_list_key,
                &mut current_list_items,
                &mut current_props,
            );
            current_list_key = None;
            parse_prop_line(rest, &mut current_props, &mut current_list_key);
            continue;
        }

        // Continuation of value block (indented).
        if in_value && (line.starts_with(' ') || line.starts_with('\t')) {
            // Check if this is a YAML list item: `  - value`
            if let Some(item) = trimmed.strip_prefix("- ") {
                current_list_items.push(strip_yaml_quotes(item.trim()));
            } else {
                // Flush any pending list, then parse as normal prop.
                flush_list(
                    &current_list_key,
                    &mut current_list_items,
                    &mut current_props,
                );
                current_list_key = None;
                parse_prop_line(trimmed, &mut current_props, &mut current_list_key);
            }
            continue;
        }
    }

    // Flush last pending list and group.
    flush_list(
        &current_list_key,
        &mut current_list_items,
        &mut current_props,
    );
    if !current_keys.is_empty() {
        groups.push((current_keys, current_props));
    }

    groups
}

fn parse_prop_line(line: &str, props: &mut HashMap<String, String>, list_key: &mut Option<String>) {
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim().to_string();
        let value = strip_yaml_quotes(value.trim());
        if !key.is_empty() {
            if value.is_empty() {
                // Empty value means a YAML list follows.
                *list_key = Some(key);
            } else {
                props.insert(key, value);
            }
        }
    }
}

/// Strip surrounding YAML quotes: `"5.1"` → `5.1`, `'7.1'` → `7.1`.
/// Also strips inline comments: `value # comment` → `value`.
fn strip_yaml_quotes(s: &str) -> String {
    // Strip inline comments (but not inside quoted strings).
    let s = if !s.starts_with('"') && !s.starts_with('\'') {
        s.split('#').next().unwrap_or(s).trim()
    } else {
        s
    };
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_yaml_quotes ────────────────────────────────────────────────────

    #[test]
    fn sqy_strips_double_quotes() {
        assert_eq!(strip_yaml_quotes(r#""hello""#), "hello");
    }

    #[test]
    fn sqy_strips_single_quotes() {
        assert_eq!(strip_yaml_quotes("'5.1'"), "5.1");
    }

    #[test]
    fn sqy_strips_inline_comment() {
        assert_eq!(strip_yaml_quotes("value # comment"), "value");
    }

    #[test]
    fn sqy_preserves_hash_inside_double_quotes() {
        assert_eq!(strip_yaml_quotes(r#""keep # this""#), "keep # this");
    }

    #[test]
    fn sqy_passthrough_plain_value() {
        assert_eq!(strip_yaml_quotes("plain"), "plain");
    }

    #[test]
    fn sqy_mismatched_quotes_not_stripped() {
        // Opening double-quote, closing single-quote: leave as-is.
        assert_eq!(strip_yaml_quotes("\"oops'"), "\"oops'");
    }

    #[test]
    fn sqy_empty_string() {
        assert_eq!(strip_yaml_quotes(""), "");
    }

    // ── parse_prop_line ──────────────────────────────────────────────────────

    #[test]
    fn ppl_basic_key_value() {
        let mut props = HashMap::new();
        let mut list_key = None;
        parse_prop_line("title: Movie", &mut props, &mut list_key);
        assert_eq!(props.get("title").map(String::as_str), Some("Movie"));
        assert!(list_key.is_none());
    }

    #[test]
    fn ppl_empty_value_sets_list_key() {
        let mut props = HashMap::new();
        let mut list_key = None;
        parse_prop_line("language:", &mut props, &mut list_key);
        assert!(props.is_empty(), "no value should be inserted yet");
        assert_eq!(list_key.as_deref(), Some("language"));
    }

    #[test]
    fn ppl_colon_in_value_uses_first_split() {
        // split_once(':') keeps the rest of the value intact.
        let mut props = HashMap::new();
        let mut list_key = None;
        parse_prop_line("episode_title: Part 1: The Beginning", &mut props, &mut list_key);
        assert_eq!(
            props.get("episode_title").map(String::as_str),
            Some("Part 1: The Beginning")
        );
    }

    #[test]
    fn ppl_strips_inline_comment_from_value() {
        let mut props = HashMap::new();
        let mut list_key = None;
        parse_prop_line("source: Blu-ray # best format", &mut props, &mut list_key);
        assert_eq!(props.get("source").map(String::as_str), Some("Blu-ray"));
    }

    #[test]
    fn ppl_ignores_line_without_colon() {
        let mut props = HashMap::new();
        let mut list_key = None;
        parse_prop_line("no colon here", &mut props, &mut list_key);
        assert!(props.is_empty());
        assert!(list_key.is_none());
    }

    // ── parse_groups ─────────────────────────────────────────────────────────

    #[test]
    fn pg_empty_content_yields_no_groups() {
        assert!(parse_groups("").is_empty());
    }

    #[test]
    fn pg_blank_lines_and_comments_ignored() {
        let content = "# top comment\n\n? file.mkv\n: type: movie\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, vec!["file.mkv"]);
        assert_eq!(groups[0].1.get("type").map(String::as_str), Some("movie"));
    }

    #[test]
    fn pg_single_group_key_and_props() {
        let content = "? Movie.mkv\n: title: Movie\n  type: movie\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, vec!["Movie.mkv"]);
        assert_eq!(groups[0].1.get("title").map(String::as_str), Some("Movie"));
        assert_eq!(groups[0].1.get("type").map(String::as_str), Some("movie"));
    }

    #[test]
    fn pg_multi_key_group_shares_same_props() {
        let content = "? A.mkv\n? B.mkv\n: type: episode\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, vec!["A.mkv", "B.mkv"]);
        assert_eq!(groups[0].1.get("type").map(String::as_str), Some("episode"));
    }

    #[test]
    fn pg_two_consecutive_groups() {
        let content = "? First.mkv\n: type: movie\n\n? Second.mkv\n: type: episode\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, vec!["First.mkv"]);
        assert_eq!(groups[1].0, vec!["Second.mkv"]);
    }

    #[test]
    fn pg_list_property_joined_with_brackets() {
        let content = "? Episode.mkv\n: language:\n  - English\n  - Japanese\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].1.get("language").map(String::as_str),
            Some("[English, Japanese]")
        );
    }

    #[test]
    fn pg_single_list_item_not_wrapped_in_brackets() {
        let content = "? Episode.mkv\n: language:\n  - English\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].1.get("language").map(String::as_str),
            Some("English")
        );
    }

    #[test]
    fn pg_empty_list_value_stored_as_empty_string() {
        // A key declared with no list items (e.g., `subtitle_language:` with
        // no `- …` lines) should be present with an empty-string value.
        let content = "? Movie.mkv\n: subtitle_language:\n  type: movie\n";
        let groups = parse_groups(content);
        assert_eq!(groups.len(), 1);
        assert_eq!(
            groups[0].1.get("subtitle_language").map(String::as_str),
            Some("")
        );
    }
}
