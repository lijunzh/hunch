//! # Hunch
//!
//! A media filename parser for Rust — spiritual descendant of Python's guessit.
//!
//! Extracts structured metadata (title, year, season, episode, video codec, etc.)
//! from media filenames and release names.
//!
//! ## Quick Start
//!
//! ```rust
//! use hunch::guess;
//!
//! let result = guess("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
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

mod guess;
mod options;
mod pipeline;

pub use guess::{Guess, GuessType, MediaType};
pub use options::Options;
pub use pipeline::Pipeline;

/// Parse a media filename and return structured metadata.
///
/// This is the main entry point for the library.
pub fn guess(input: &str) -> Guess {
    Pipeline::default().run(input)
}

/// Parse a media filename with custom options.
pub fn guess_with(input: &str, options: Options) -> Guess {
    Pipeline::new(options).run(input)
}
