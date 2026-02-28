//! Configuration / options for parsing.
//!
//! Use [`Options`] with [`hunch_with`](crate::hunch_with) to customize
//! parsing behaviour — hint the media type, disable path handling, or
//! supply expected titles for ambiguous filenames.
//!
//! ```rust
//! use hunch::{Options, hunch_with};
//!
//! let opts = Options::new().with_type("episode").name_only();
//! let result = hunch_with("Show.S01E01.720p.mkv", opts);
//! assert_eq!(result.season(), Some(1));
//! ```

/// Options to customize the parsing behavior.
///
/// Constructed via [`Options::new`] and refined with builder methods.
/// Pass to [`hunch_with`](crate::hunch_with) to apply.
///
/// # Example
///
/// ```rust
/// use hunch::{Options, hunch_with};
///
/// let result = hunch_with(
///     "Movie.2024.1080p.BluRay.mkv",
///     Options::new().with_type("movie"),
/// );
/// assert_eq!(result.title(), Some("Movie"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Hint the parser that the input is a specific media type.
    ///
    /// Accepts `"movie"` or `"episode"`. When set, the inferred
    /// [`MediaType`](crate::MediaType) is overridden.
    pub media_type: Option<String>,
    /// When `true`, treat the entire input as a bare name (no path separators).
    ///
    /// Disables directory-segment extraction and parent-directory title
    /// fallback. Useful when the input is a release name, not a file path.
    pub name_only: bool,
    /// Expected title(s) to help with ambiguous names.
    ///
    /// Reserved for future use. Not yet wired into the pipeline.
    pub expected_title: Vec<String>,
}

impl Options {
    /// Create a new `Options` with all defaults (no hints, path handling enabled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Hint the expected media type: `"movie"` or `"episode"`.
    ///
    /// This helps the parser resolve ambiguous filenames where the media
    /// type cannot be reliably inferred from structure alone.
    ///
    /// ```rust
    /// use hunch::Options;
    ///
    /// let opts = Options::new().with_type("episode");
    /// assert_eq!(opts.media_type.as_deref(), Some("episode"));
    /// ```
    #[must_use]
    pub fn with_type(mut self, media_type: &str) -> Self {
        self.media_type = Some(media_type.to_string());
        self
    }

    /// Treat the input as a name only (disable path separator handling).
    ///
    /// When enabled, `/` and `\` are treated as literal characters rather
    /// than path separators. Useful for parsing release names from APIs
    /// or databases where the input is not a filesystem path.
    ///
    /// ```rust
    /// use hunch::Options;
    ///
    /// let opts = Options::new().name_only();
    /// assert!(opts.name_only);
    /// ```
    #[must_use]
    pub fn name_only(mut self) -> Self {
        self.name_only = true;
        self
    }
}
