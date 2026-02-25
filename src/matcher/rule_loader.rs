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

/// Result of a successful token match from the TOML rule engine.
#[derive(Debug, Clone)]
pub struct TokenMatch<'a> {
    /// The canonical value for the primary property.
    pub value: Cow<'a, str>,
    /// Additional property:value pairs to emit alongside the primary match.
    pub side_effects: Vec<SideEffect>,
    /// If set, the match should be rejected if the NEXT token (lowercased) is in this list.
    pub not_before: Option<Vec<String>>,
    /// If set, the match should be rejected if the PREVIOUS token (lowercased) is in this list.
    pub not_after: Option<Vec<String>>,
    /// If set, the match should be rejected UNLESS the NEXT token (lowercased) is in this list.
    pub requires_after: Option<Vec<String>>,
}

/// An additional property:value pair emitted as a side effect of a pattern match.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SideEffect {
    pub property: String,
    pub value: String,
}

/// A compiled pattern rule with optional capture-group templates.
#[derive(Debug)]
struct PatternRule {
    regex: Regex,
    /// The raw value template (may contain `{1}`, `{2}`, etc.).
    template: String,
    /// True if template contains at least one `{N}` placeholder.
    is_dynamic: bool,
    /// Additional property:value pairs emitted on match.
    side_effects: Vec<SideEffect>,
    /// Reject if next token (lowercased) is in this list.
    not_before: Option<Vec<String>>,
    /// Reject if previous token (lowercased) is in this list.
    not_after: Option<Vec<String>>,
    /// Reject unless next token (lowercased) is in this list.
    requires_after: Option<Vec<String>>,
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
    #[serde(default)]
    side_effects: Vec<RawSideEffect>,
    #[serde(default)]
    not_before: Option<Vec<String>>,
    #[serde(default)]
    not_after: Option<Vec<String>>,
    #[serde(default)]
    requires_after: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawSideEffect {
    property: String,
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
                let side_effects = p
                    .side_effects
                    .into_iter()
                    .map(|s| SideEffect {
                        property: s.property,
                        value: s.value,
                    })
                    .collect();
                PatternRule {
                    regex,
                    template: p.value,
                    is_dynamic,
                    side_effects,
                    not_before: p.not_before,
                    not_after: p.not_after,
                    requires_after: p.requires_after,
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
    pub fn match_token(&self, token: &str) -> Option<TokenMatch<'_>> {
        // Case-sensitive exact lookup (for ambiguous short tokens).
        if let Some(value) = self.exact_sensitive.get(token) {
            return Some(TokenMatch::exact(Cow::Borrowed(value.as_str())));
        }

        // Case-insensitive exact lookup.
        let lower = token.to_lowercase();
        if let Some(value) = self.exact.get(&lower) {
            return Some(TokenMatch::exact(Cow::Borrowed(value.as_str())));
        }

        // Regex patterns.
        for rule in &self.patterns {
            if !rule.is_dynamic {
                // Static value — no capture groups needed.
                if rule.regex.is_match(token) {
                    return Some(TokenMatch::from_pattern(
                        Cow::Borrowed(rule.template.as_str()),
                        rule,
                    ));
                }
            } else {
                // Dynamic value — substitute capture groups into template.
                if let Some(caps) = rule.regex.captures(token) {
                    let value = substitute_captures(&rule.template, &caps);
                    return Some(TokenMatch::from_pattern(
                        Cow::Owned(value),
                        rule,
                    ));
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

impl<'a> TokenMatch<'a> {
    /// Create a TokenMatch from an exact (non-pattern) hit — no side effects or constraints.
    fn exact(value: Cow<'a, str>) -> Self {
        Self {
            value,
            side_effects: Vec::new(),
            not_before: None,
            not_after: None,
            requires_after: None,
        }
    }

    /// Create a TokenMatch from a pattern rule, carrying over side effects and constraints.
    fn from_pattern(value: Cow<'a, str>, rule: &PatternRule) -> Self {
        Self {
            value,
            side_effects: rule.side_effects.clone(),
            not_before: rule.not_before.clone(),
            not_after: rule.not_after.clone(),
            requires_after: rule.requires_after.clone(),
        }
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

    /// Helper: extract just the value string from a match result.
    fn val(m: Option<TokenMatch<'_>>) -> Option<String> {
        m.map(|t| t.value.into_owned())
    }

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
        assert_eq!(val(rules.match_token("x264")), Some("H.264".into()));
        assert_eq!(val(rules.match_token("X264")), Some("H.264".into()));
        assert_eq!(val(rules.match_token("HEVC")), Some("H.265".into()));
        assert_eq!(val(rules.match_token("XviD")), Some("Xvid".into()));
    }

    #[test]
    fn test_regex_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(val(rules.match_token("x.265")), Some("H.265".into()));
        assert_eq!(val(rules.match_token("H-265")), Some("H.265".into()));
        assert_eq!(val(rules.match_token("Rv20")), Some("RealVideo".into()));
    }

    #[test]
    fn test_no_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert!(rules.match_token("Movie").is_none());
        assert!(rules.match_token("720p").is_none());
    }

    #[test]
    fn test_exact_preferred_over_regex() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(val(rules.match_token("hevc")), Some("H.265".into()));
    }

    #[test]
    fn test_load_video_codec_toml() {
        let toml_str = include_str!("../../rules/video_codec.toml");
        let rules = RuleSet::from_toml(toml_str);
        assert_eq!(rules.property, "video_codec");
        assert!(rules.exact_count() >= 10);
        assert!(rules.pattern_count() >= 5);

        assert_eq!(val(rules.match_token("x264")), Some("H.264".into()));
        assert_eq!(val(rules.match_token("HEVC")), Some("H.265".into()));
        assert_eq!(val(rules.match_token("h.265")), Some("H.265".into()));
        assert_eq!(val(rules.match_token("XviD")), Some("Xvid".into()));
        assert_eq!(val(rules.match_token("AV1")), Some("AV1".into()));
        assert_eq!(val(rules.match_token("Rv10")), Some("RealVideo".into()));
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
        assert_eq!(val(rules.match_token("1920x1080")), Some("1080p".into()));
        assert_eq!(val(rules.match_token("1280x720")), Some("720p".into()));
        assert_eq!(val(rules.match_token("720p60")), Some("720p".into()));
        assert_eq!(val(rules.match_token("1080p25")), Some("1080p".into()));
    }

    #[test]
    fn test_exact_match_has_no_side_effects_or_constraints() {
        let rules = RuleSet::from_toml(TEST_TOML);
        let m = rules.match_token("x264").expect("should match");
        assert!(m.side_effects.is_empty());
        assert!(m.not_before.is_none());
        assert!(m.not_after.is_none());
        assert!(m.requires_after.is_none());
    }

    #[test]
    fn test_side_effects_from_toml() {
        let toml = r#"
property = "source"

[exact]

[[patterns]]
match = '(?i)^dvd[-. ]?rip$'
value = "DVD"
side_effects = [
    { property = "other", value = "Rip" }
]
"#;
        let rules = RuleSet::from_toml(toml);
        let m = rules.match_token("DVDRip").expect("should match");
        assert_eq!(m.value, "DVD");
        assert_eq!(m.side_effects.len(), 1);
        assert_eq!(m.side_effects[0].property, "other");
        assert_eq!(m.side_effects[0].value, "Rip");
    }

    #[test]
    fn test_neighbor_constraints_from_toml() {
        let toml = r#"
property = "streaming_service"

[exact]

[[patterns]]
match = '(?i)^hd$'
value = "HD"
not_before = ["tv", "dvd"]

[[patterns]]
match = '(?i)^ae$'
value = "A&E"
requires_after = ["web"]

[[patterns]]
match = '(?i)^cam$'
value = "Camera"
not_after = ["web"]
"#;
        let rules = RuleSet::from_toml(toml);

        let hd = rules.match_token("HD").expect("should match");
        assert_eq!(hd.value, "HD");
        assert_eq!(hd.not_before.as_deref(), Some(&["tv".to_string(), "dvd".to_string()][..]));
        assert!(hd.not_after.is_none());
        assert!(hd.requires_after.is_none());

        let ae = rules.match_token("AE").expect("should match");
        assert_eq!(ae.value, "A&E");
        assert_eq!(ae.requires_after.as_deref(), Some(&["web".to_string()][..]));
        assert!(ae.not_before.is_none());

        let cam = rules.match_token("cam").expect("should match");
        assert_eq!(cam.value, "Camera");
        assert_eq!(cam.not_after.as_deref(), Some(&["web".to_string()][..]));
    }

    #[test]
    fn test_pattern_without_side_effects_has_empty_vec() {
        let rules = RuleSet::from_toml(TEST_TOML);
        let m = rules.match_token("x.265").expect("should match regex");
        assert_eq!(m.value, "H.265");
        assert!(m.side_effects.is_empty());
        assert!(m.not_before.is_none());
    }
}
