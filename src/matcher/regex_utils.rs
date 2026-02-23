//! Regex utilities — helpers for boundary-aware pattern matching.
//!
//! We use `fancy_regex` for patterns that need look-around assertions
//! (which Rust's standard `regex` crate doesn't support).

use fancy_regex::Regex as FancyRegex;

/// A compiled pattern with a canonical output value.
pub struct ValuePattern {
    pub regex: FancyRegex,
    pub value: &'static str,
}

impl ValuePattern {
    pub fn new(pattern: &str, value: &'static str) -> Self {
        Self {
            regex: FancyRegex::new(pattern)
                .unwrap_or_else(|e| panic!("Bad regex `{pattern}`: {e}")),
            value,
        }
    }

    /// Find all non-overlapping matches, returning (start, end) byte offsets.
    pub fn find_iter<'a>(&'a self, input: &'a str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let mut start = 0;
        while start < input.len() {
            match self.regex.find_from_pos(input, start) {
                Ok(Some(m)) => {
                    results.push((m.start(), m.end()));
                    start = m.end().max(start + 1);
                }
                _ => break,
            }
        }
        results
    }
}
