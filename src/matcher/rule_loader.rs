//! TOML rule loader: generic engine for data-driven property matching.
//!
//! Loads property definitions from embedded TOML files and provides
//! both exact (HashMap) and regex-based matching against isolated tokens.
//!
//! All regex patterns use the `regex` crate only (linear-time, ReDoS-immune).
//! Word boundary assertions are unnecessary because matching happens
//! against tokens isolated by the tokenizer.

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

/// A parsed rule file loaded from TOML.
#[derive(Debug)]
pub struct RuleSet {
    /// The property this rule set matches (e.g., "video_codec").
    pub property: String,
    /// Case-insensitive exact token lookups.
    exact: HashMap<String, String>,
    /// Compiled regex patterns with their output values.
    patterns: Vec<(Regex, String)>,
}

/// Raw TOML structure for deserialization.
#[derive(Deserialize)]
struct RawRuleFile {
    property: String,
    #[serde(default)]
    exact: HashMap<String, String>,
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
        let patterns: Vec<(Regex, String)> = raw
            .patterns
            .into_iter()
            .map(|p| {
                let re = Regex::new(&p.pattern).unwrap_or_else(|e| {
                    panic!("Bad regex in {} rules: `{}`: {e}", raw.property, p.pattern)
                });
                (re, p.value)
            })
            .collect();

        Self {
            property: raw.property,
            exact,
            patterns,
        }
    }

    /// Try to match a single token against this rule set.
    ///
    /// Returns the canonical value if the token matches, or `None`.
    /// Exact lookup is tried first (O(1)), then regex patterns (linear scan).
    pub fn match_token(&self, token: &str) -> Option<&str> {
        // Exact lookup (case-insensitive).
        let lower = token.to_lowercase();
        if let Some(value) = self.exact.get(&lower) {
            return Some(value.as_str());
        }

        // Regex patterns.
        for (re, value) in &self.patterns {
            if re.is_match(token) {
                return Some(value.as_str());
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
        assert_eq!(rules.match_token("x264"), Some("H.264"));
        assert_eq!(rules.match_token("X264"), Some("H.264"));
        assert_eq!(rules.match_token("HEVC"), Some("H.265"));
        assert_eq!(rules.match_token("XviD"), Some("Xvid"));
    }

    #[test]
    fn test_regex_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("x.265"), Some("H.265"));
        assert_eq!(rules.match_token("H-265"), Some("H.265"));
        assert_eq!(rules.match_token("Rv20"), Some("RealVideo"));
    }

    #[test]
    fn test_no_match() {
        let rules = RuleSet::from_toml(TEST_TOML);
        assert_eq!(rules.match_token("Movie"), None);
        assert_eq!(rules.match_token("720p"), None);
    }

    #[test]
    fn test_exact_preferred_over_regex() {
        let rules = RuleSet::from_toml(TEST_TOML);
        // "hevc" matches exact lookup (no regex needed).
        assert_eq!(rules.match_token("hevc"), Some("H.265"));
    }

    #[test]
    fn test_load_video_codec_toml() {
        // Validate the actual rules/video_codec.toml file compiles.
        let toml_str = include_str!("../../rules/video_codec.toml");
        let rules = RuleSet::from_toml(toml_str);
        assert_eq!(rules.property, "video_codec");
        assert!(rules.exact_count() >= 10);
        assert!(rules.pattern_count() >= 5);

        // Spot-check matches.
        assert_eq!(rules.match_token("x264"), Some("H.264"));
        assert_eq!(rules.match_token("HEVC"), Some("H.265"));
        assert_eq!(rules.match_token("h.265"), Some("H.265"));
        assert_eq!(rules.match_token("XviD"), Some("Xvid"));
        assert_eq!(rules.match_token("AV1"), Some("AV1"));
        assert_eq!(rules.match_token("Rv10"), Some("RealVideo"));
    }
}
