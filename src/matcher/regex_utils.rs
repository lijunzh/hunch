//! Regex utilities — helpers for boundary-aware pattern matching.
//!
//! Patterns use standard `regex` for the core match plus post-match
//! boundary checks to replace lookaround assertions.

use regex::Regex;

/// Describes which characters must NOT appear at a boundary.
#[derive(Debug, Clone)]
pub struct BoundarySpec {
    /// Characters that must NOT appear immediately before the match.
    pub left: Option<CharClass>,
    /// Characters that must NOT appear immediately after the match.
    pub right: Option<CharClass>,
}

/// A character class for boundary checking.
#[derive(Debug, Clone)]
pub enum CharClass {
    /// `[a-z]`
    Lower,
    /// `[a-z0-9]`
    LowerDigit,
    /// `[a-zA-Z]`
    Alpha,
    /// `[a-zA-Z0-9]`
    AlphaDigit,
    /// `[0-9]`
    Digit,
    /// Custom byte predicate as a list of ranges.
    Custom(Vec<(u8, u8)>),
}

impl CharClass {
    fn matches_byte(&self, b: u8) -> bool {
        match self {
            Self::Lower => b.is_ascii_lowercase(),
            Self::LowerDigit => b.is_ascii_lowercase() || b.is_ascii_digit(),
            Self::Alpha => b.is_ascii_alphabetic(),
            Self::AlphaDigit => b.is_ascii_alphanumeric(),
            Self::Digit => b.is_ascii_digit(),
            Self::Custom(ranges) => ranges.iter().any(|(lo, hi)| b >= *lo && b <= *hi),
        }
    }
}

/// Check that the boundary conditions hold for a match at `[start..end]`.
pub fn check_boundary(input: &[u8], start: usize, end: usize, spec: &BoundarySpec) -> bool {
    if let Some(ref left) = spec.left
        && start > 0
        && left.matches_byte(input[start - 1])
    {
        return false;
    }
    if let Some(ref right) = spec.right
        && end < input.len()
        && right.matches_byte(input[end])
    {
        return false;
    }
    true
}

/// Strip leading `(?<![...])` and trailing `(?![...])` from a regex pattern,
/// returning the cleaned pattern and the boundary spec.
fn strip_boundaries(pattern: &str) -> (String, BoundarySpec) {
    let mut s = pattern.to_string();
    let mut left = None;
    let mut right = None;

    // Detect case-insensitive mode — affects boundary char classes.
    let case_insensitive = s.contains("(?i)");

    // Strip leading flags like `(?i)` or `(?-i)` to find the lookbehind.
    let work = skip_flags(&s);

    // Try to strip leading negative lookbehind: (?<![...])
    if let Some(rest) = work.strip_prefix("(?<!")
        && let Some(end) = rest.find(')')
    {
        let class_str = &rest[..end];
        // Strip surrounding brackets if present: [a-z0-9] -> a-z0-9
        let inner = class_str
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(class_str);
        if let Some(cc) = parse_char_class(inner, case_insensitive) {
            left = Some(cc);
            let lb_full = format!("(?<!{})", class_str);
            s = s.replacen(&lb_full, "", 1);
        }
    }

    // Try to strip trailing negative lookahead: (?![...])
    if let Some(pos) = find_trailing_lookahead(&s) {
        let la_str = &s[pos..];
        if let Some(class_str) = extract_lookahead_class(la_str)
            && let Some(cc) = parse_char_class(&class_str, case_insensitive)
        {
            right = Some(cc);
            s = s[..pos].to_string();
        }
    }

    (s, BoundarySpec { left, right })
}

/// Skip leading regex flags like `(?i)`, `(?-i)`, returning the rest.
fn skip_flags(s: &str) -> &str {
    let mut rest = s;
    while let Some(stripped) = rest.strip_prefix("(?i)") {
        rest = stripped;
    }
    while let Some(stripped) = rest.strip_prefix("(?-i)") {
        rest = stripped;
    }
    rest
}

/// Find the position of a trailing `(?![...])` at the end of the pattern.
fn find_trailing_lookahead(s: &str) -> Option<usize> {
    // Look for (?![...]) at the end, possibly preceded by other patterns.
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len < 5 {
        return None;
    }
    // Walk backwards from end to find (?!
    // The pattern ends with )
    if bytes[len - 1] != b')' {
        return None;
    }
    // Find the matching (?!
    let mut depth = 0;
    let mut i = len - 1;
    loop {
        if bytes[i] == b')' {
            depth += 1;
        } else if bytes[i] == b'(' {
            depth -= 1;
            if depth == 0 {
                // Check if this is (?!
                if i + 1 < len && bytes[i + 1] == b'?' && i + 2 < len && bytes[i + 2] == b'!' {
                    return Some(i);
                }
                return None;
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}

/// Extract the character class string from `(?![CLASS])` or `(?![CLASS])`.
fn extract_lookahead_class(s: &str) -> Option<String> {
    let rest = s.strip_prefix("(?!")?;
    // Handle both `(?![class])` and `(?![^class])` and bare `(?!chars)`
    if let Some(bracket_rest) = rest.strip_prefix('[') {
        let end = bracket_rest.find(']')?;
        Some(bracket_rest[..end].to_string())
    } else {
        None
    }
}

/// Parse a character class string like `a-z`, `a-z0-9`, `a-zA-Z0-9` into a `CharClass`.
///
/// When `case_insensitive` is true, `a-z` is upgraded to `a-zA-Z` (matching
/// the behavior of `(?i)` mode in regex engines).
fn parse_char_class(s: &str, case_insensitive: bool) -> Option<CharClass> {
    match s {
        "a-z" if case_insensitive => Some(CharClass::Alpha),
        "a-z" => Some(CharClass::Lower),
        "a-z0-9" if case_insensitive => Some(CharClass::AlphaDigit),
        "a-z0-9" => Some(CharClass::LowerDigit),
        "a-zA-Z" | "A-Za-z" => Some(CharClass::Alpha),
        "a-zA-Z0-9" | "A-Za-z0-9" => Some(CharClass::AlphaDigit),
        "0-9" => Some(CharClass::Digit),
        _ => {
            // Try to parse as custom ranges.
            let mut ranges = Vec::new();
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
                    let lo = bytes[i];
                    let hi = bytes[i + 2];
                    ranges.push((lo, hi));
                    // If case-insensitive, add the opposite case range.
                    if case_insensitive {
                        if lo.is_ascii_lowercase() && hi.is_ascii_lowercase() {
                            ranges.push((lo.to_ascii_uppercase(), hi.to_ascii_uppercase()));
                        } else if lo.is_ascii_uppercase() && hi.is_ascii_uppercase() {
                            ranges.push((lo.to_ascii_lowercase(), hi.to_ascii_lowercase()));
                        }
                    }
                    i += 3;
                } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    // Escaped char like \-
                    ranges.push((bytes[i + 1], bytes[i + 1]));
                    i += 2;
                } else {
                    // Single char
                    let b = bytes[i];
                    ranges.push((b, b));
                    if case_insensitive {
                        if b.is_ascii_lowercase() {
                            ranges.push((b.to_ascii_uppercase(), b.to_ascii_uppercase()));
                        } else if b.is_ascii_uppercase() {
                            ranges.push((b.to_ascii_lowercase(), b.to_ascii_lowercase()));
                        }
                    }
                    i += 1;
                }
            }
            if ranges.is_empty() {
                None
            } else {
                Some(CharClass::Custom(ranges))
            }
        }
    }
}

/// A standard `regex::Regex` paired with an auto-extracted boundary spec.
///
/// Construct from a pattern that may include lookaround assertions — they
/// are automatically stripped and enforced via post-match boundary checks.
///
/// # Panics
///
/// Panics if the core pattern (after stripping) is invalid or still contains
/// lookarounds.
pub struct BoundedRegex {
    re: Regex,
    /// The boundary constraints extracted from the original pattern's lookarounds.
    pub boundary: BoundarySpec,
}

impl BoundedRegex {
    /// Create from a pattern string, auto-stripping leading/trailing lookarounds.
    pub fn new(pattern: &str) -> Self {
        let (core, boundary) = strip_boundaries(pattern);
        let re = Regex::new(&core).unwrap_or_else(|e| {
            panic!("BoundedRegex: bad pattern `{core}` (from `{pattern}`): {e}")
        });
        Self { re, boundary }
    }

    /// Iterate all non-overlapping captures with boundary checking.
    pub fn captures_iter<'a>(&'a self, input: &'a str) -> Vec<regex::Captures<'a>> {
        captures_iter_bounded(&self.re, input, &self.boundary)
    }

    /// Return the first capture (with boundary checking).
    pub fn captures<'a>(&'a self, input: &'a str) -> Option<regex::Captures<'a>> {
        captures_iter_bounded(&self.re, input, &self.boundary)
            .into_iter()
            .next()
    }
}

/// Like `captures_iter` but uses standard `regex::Regex` + boundary checking.
pub fn captures_iter_bounded<'a>(
    re: &'a regex::Regex,
    input: &'a str,
    boundary: &BoundarySpec,
) -> Vec<regex::Captures<'a>> {
    let bytes = input.as_bytes();
    let mut results = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        let Some(cap) = re.captures_at(input, pos) else {
            break;
        };
        let Some(full) = cap.get(0) else {
            break;
        };
        if check_boundary(bytes, full.start(), full.end(), boundary) {
            results.push(cap);
            pos = full.end().max(pos + 1);
        } else {
            pos = full.start() + 1;
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_simple_boundaries() {
        let (core, spec) = strip_boundaries(r"(?i)(?<![a-z])HELLO(?![a-z])");
        assert_eq!(core, "(?i)HELLO");
        assert!(spec.left.is_some());
        assert!(spec.right.is_some());
    }

    #[test]
    fn test_strip_no_boundaries() {
        let (core, spec) = strip_boundaries(r"(?i)HELLO");
        assert_eq!(core, "(?i)HELLO");
        assert!(spec.left.is_none());
        assert!(spec.right.is_none());
    }

    #[test]
    fn test_strip_digit_boundaries() {
        let (core, spec) = strip_boundaries(r"(?<![0-9])\d{4}(?![0-9])");
        assert_eq!(core, r"\d{4}");
        assert!(spec.left.is_some());
        assert!(spec.right.is_some());
    }

    #[test]
    fn test_custom_char_class() {
        let cc = parse_char_class(r"a-z0-9\-", false).unwrap();
        if let CharClass::Custom(ranges) = cc {
            assert!(ranges.contains(&(b'a', b'z')));
            assert!(ranges.contains(&(b'0', b'9')));
            assert!(ranges.contains(&(b'-', b'-')));
        } else {
            panic!("Expected Custom");
        }
    }
}
