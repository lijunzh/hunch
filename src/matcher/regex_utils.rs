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

/// Iterate non-overlapping captures from a `fancy_regex::Regex`.
///
/// The standard `fancy_regex` crate lacks a `captures_iter` method,
/// so we implement one via `captures_from_pos`.
pub fn captures_iter<'a>(re: &'a FancyRegex, input: &'a str) -> Vec<fancy_regex::Captures<'a>> {
    let mut results = Vec::new();
    let mut start = 0;
    while start < input.len() {
        match re.captures_from_pos(input, start) {
            Ok(Some(cap)) => {
                if let Some(full) = cap.get(0) {
                    results.push(cap);
                    start = full.end().max(start + 1);
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    results
}
