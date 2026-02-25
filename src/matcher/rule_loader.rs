//! TOML rule loader: generic engine for data-driven property matching.
//!
//! Loads property definitions from embedded TOML files and provides
//! both exact (HashMap) and regex-based matching against isolated tokens.
//!
//! All regex patterns use the `regex` crate only (linear-time, ReDoS-immune).
//! Word boundary assertions are unnecessary because matching happens
//! against tokens isolated by the tokenizer.
//!
//! ## Capture-group value templates
//!
//! Pattern values can contain `{N}` placeholders that are replaced with
//! regex capture group contents at match time:
//!
//! ```toml
//! [[patterns]]
//! match = '(?i)^(\d{3,4})x(\d{3,4})$'
//! value = "{2}p"  # Uses group 2 (height) → "1080p"
//! ```
//!
//! A value without `{N}` is returned as-is (static value).

use regex::Regex;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

/// A compiled pattern rule with optional capture-group templates.
#[derive(Debug)]
struct PatternRule {
    regex: Regex,
    /// The raw value template (may contain `{1}`, `{2}`, etc.).
    template: String,
    /// True if template contains at least one `{N}` placeholder.
    is_dynamic: bool,
}

/// A parsed rule file loaded from TOML.
#[derive(Debug)]
pub struct RuleSet {
    /// The property this rule set matches (e.g., "video_codec").
    pub property: String,
    /// Case-insensitive exact token lookups.
    exact: HashMap<String, String>,
    /// Case-sensitive exact token lookups (for short ambiguous tokens like country codes).
    exact_sensitive: HashMap<String, String>,
    /// Compiled regex patterns with their output values.
    patterns: Vec<PatternRule>,
}

/// Raw TOML structure for deserialization.
#[derive(Deserialize)]
struct RawRuleFile {
    property: String,
    #[serde(default)]
    exact: HashMap<String, String>,
    #[serde(default)]
    exact_sensitive: HashMap<String, String>,
    #[serde(default)]
    patterns: Vec<RawPattern>,
}

#[derive(Deserialize)]
struct RawPattern {
    #[serde(rename = "match")]
    pattern: String,
    value: String,
}

impl RuleSet {
    /// Parse a TOML string into a RuleSet.
    ///
    /// # Panics
    /// Panics if the TOML is malformed or any regex pattern is invalid.
    pub fn from_toml(toml_str: &str) -> Self {
        let raw: RawRuleFile =
            toml::from_str(toml_str).unwrap_or_else(|e| panic!("Bad TOML rule file: {e}"));

        // Build case-insensitive exact lookup.
        let exact: HashMap<String, String> = raw
            .exact
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        // Compile regex patterns.
        let patterns: Vec<PatternRule> = raw
            .patterns
            .into_iter()
            .map(|p| {
                let regex = Regex::new(&p.pattern).unwrap_or_else(|e| {
                    panic!("Bad regex in {} rules: `{}`: {e}", raw.property, p.pattern)
                });
                let is_dynamic = p.value.contains('{');
                PatternRule {
                    regex,
                    template: p.value,
                    is_dynamic,
                }
            })
            .collect();

        Self {
            property: raw.property,
            exact,
            exact_sensitive: raw.exact_sensitive,
            patterns,
        }
    }

    /// Try to match a single token against this rule set.
    ///
    /// Returns the canonical value if the token matches, or `None`.
    /// Case-sensitive exact is checked first, then case-insensitive, then regex.
    ///
    /// For regex patterns with `{N}` templates, capture groups are substituted
    /// into the template to produce the final value.
    pub fn match_token(&self, token: &str) -> Option<Cow<'_, str>> {
        // Case-sensitive exact lookup (for ambiguous short tokens).
        if let Some(value) = self.exact_sensitive.get(token) {
            return Some(Cow::Borrowed(value.as_str()));
        }

        // Case-insensitive exact lookup.
        let lower = token.to_lowercase();
        if let Some(value) = self.exact.get(&lower) {
            return Some(Cow::Borrowed(value.as_str()));
        }

        // Regex patterns.
        for rule in &self.patterns {
            if !rule.is_dynamic {
                // Static value — no capture groups needed.
                if rule.regex.is_match(token) {
                    return Some(Cow::Borrowed(rule.template.as_str()));
                }
            } else {
                // Dynamic value — substitute capture groups into template.
                if let Some(caps) = rule.regex.captures(token) {
                    let value = substitute_captures(&rule.template, &caps);
                    return Some(Cow::Owned(value));
                }
            }
        }

        None
    }

    /// Number of exact entries.
    pub fn exact_count(&self) -> usize {
        self.exact.len()
    }

    /// Number of regex patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

/// Substitute `{N}` placeholders in a template with capture group values.
///
/// `{0}` = entire match, `{1}` = first group, `{2}` = second, etc.
/// Missing groups are replaced with empty string.
fn substitute_captures(template: &str, caps: &regex::Captures<'_>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            // Parse the group index.
            let mut digits = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    digits.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            // Consume the closing '}'.
            if chars.peek() == Some(&'}') {
                chars.next();
            }
            if let Ok(idx) = digits.parse::<usize>() {
                if let Some(m) = caps.get(idx) {
                    result.push_str(m.as_str());
                }
                // Missing group → empty string (nothing pushed).
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOML: &str = r#"
property = "video_codec"

[exact]
x264 = "H.264"
h264 = "H.264"
hevc = "H.265"
xvid = "Xvid"

[[patterns]]
match = '(?i)^[xh][.-]?265$'
value = "H.265"

[[patterns]]
match = '(?i)^rv\d{2}$'
value = "RealVideo"
"#;

    #[test]
    fn test_parse_rule_file() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.property, "video_codec");
        assert_eq!(rules.exact_count(), 4);
        assert_eq!(rules.pattern_count(), 2);
    }

    #[test]
    fn test_exact_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("x264").as_deref(), Some("H.264"));
        assert_eq!(rules.match_token("X264").as_deref(), Some("H.264"));
        assert_eq!(rules.match_token("HEVC").as_deref(), Some("H.265"));
        assert_eq!(rules.match_token("XviD").as_deref(), Some("Xvid"));
    }

    #[test]
    fn test_regex_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("x.265").as_deref(), Some("H.265"));
        assert_eq!(rules.match_token("H-265").as_deref(), Some("H.265"));
        assert_eq!(rules.match_token("Rv20").as_deref(), Some("RealVideo"));
    }

    #[test]
    fn test_no_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("Movie").as_deref(), None);
        assert_eq!(rules.match_token("720p").as_deref(), None);
    }

    #[test]
    fn test_exact_preferred_over_regex() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("hevc").as_deref(), Some("H.265"));
    }

    #[test]
    fn test_load_video_codec_toml() {
        let toml_str = include_str!("../../rules/video_codec.toml");
        let rules = RuleSet::from_toml(toml_str);
        assert_eq!(rules.property, "video_codec");
        assert!(rules.exact_count() >= 10);
        assert!(rules.pattern_count() >= 5);

        assert_eq!(rules.match_token("x264").as_deref(), Some("H.264"));
        assert_eq!(rules.match_token("HEVC").as_deref(), Some("H.265"));
        assert_eq!(rules.match_token("h.265").as_deref(), Some("H.265"));
        assert_eq!(rules.match_token("XviD").as_deref(), Some("Xvid"));
        assert_eq!(rules.match_token("AV1").as_deref(), Some("AV1"));
        assert_eq!(rules.match_token("Rv10").as_deref(), Some("RealVideo"));
    }

    #[test]
    fn test_capture_group_template() {
        let toml = r#"
property = "screen_size"

[exact]

[[patterns]]
match = '(?i)^(\d{3,4})x(\d{3,4})$'
value = "{2}p"

[[patterns]]
match = '(?i)^(\d{3,4})p(\d{2,3})$'
value = "{1}p"
"#;
        let rules = RuleSet::from_toml(toml);
        // WxH: extract height.
        assert_eq!(rules.match_token("1920x1080").as_deref(), Some("1080p"));
        assert_eq!(rules.match_token("1280x720").as_deref(), Some("720p"));
        // Resolution + frame rate: extract resolution.
        assert_eq!(rules.match_token("720p60").as_deref(), Some("720p"));
        assert_eq!(rules.match_token("1080p25").as_deref(), Some("1080p"));
    }
}
