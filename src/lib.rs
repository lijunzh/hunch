//! # Hunch
//!
//! A Rust library for extracting media metadata from filenames,
//! inspired by Python's [guessit](https://github.com/guessit-io/guessit).
//!
//! Hunch parses messy media filenames and release names into structured
//! metadata: title, year, season, episode, video codec, audio codec,
//! resolution, and 40+ other properties.
//!
//! ## Architecture
//!
//! Hunch uses a span-based architecture with plain function pointers
//! (no trait objects):
//!
//! 1. **29 matcher functions** scan the input and produce `MatchSpan`s
//! 2. **Conflict resolution** keeps higher-priority / longer matches
//! 3. **Title extraction** claims the unclaimed leading region
//! 4. **Computed properties** (media type, proper count) are set directly
//!
//! ## Quick Start
//!
//! ```rust
//! use hunch::hunch;
//!
//! let result = hunch("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
//! assert_eq!(result.title(), Some("The Matrix"));
//! assert_eq!(result.year(), Some(1999));
//! assert_eq!(result.screen_size(), Some("1080p"));
//! assert_eq!(result.source(), Some("Blu-ray"));
//! assert_eq!(result.video_codec(), Some("H.264"));
//! assert_eq!(result.release_group(), Some("GROUP"));
//! assert_eq!(result.container(), Some("mkv"));
//! ```

pub mod matcher;
pub mod properties;
pub mod tokenizer;

mod hunch_result;
mod options;
mod pipeline;

pub use hunch_result::{HunchResult, MediaType};
pub use options::Options;
pub use pipeline::Pipeline;

/// Parse a media filename and return structured metadata.
///
/// This is the main entry point for the library.
///
/// ```rust
/// let result = hunch::hunch("Movie.2024.1080p.BluRay.x264-GROUP.mkv");
/// assert_eq!(result.title(), Some("Movie"));
/// assert_eq!(result.year(), Some(2024));
/// ```
pub fn hunch(input: &str) -> HunchResult {
    Pipeline::default().run(input)
}

/// Parse a media filename with custom options.
pub fn hunch_with(input: &str, options: Options) -> HunchResult {
    Pipeline::new(options).run(input)
}
