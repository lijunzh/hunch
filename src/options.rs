//! Configuration / options for parsing.

/// Options to customize the parsing behavior.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Hint the parser that the input is a specific media type.
    pub media_type: Option<String>,
    /// Parse as name-only (no path separators).
    pub name_only: bool,
    /// Expected title(s) to help with ambiguous names.
    pub expected_title: Vec<String>,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_type(mut self, media_type: &str) -> Self {
        self.media_type = Some(media_type.to_string());
        self
    }

    pub fn name_only(mut self) -> Self {
        self.name_only = true;
        self
    }
}
