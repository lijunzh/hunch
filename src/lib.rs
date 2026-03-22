//! # Hunch
//!
//! A fast, offline Rust library for extracting structured media metadata
//! from filenames, inspired by Python's [guessit](https://github.com/guessit-io/guessit).
//!
//! Hunch parses messy media filenames and release names into structured
//! metadata: title, year, season, episode, video codec, audio codec,
//! resolution, and **49 properties** in total.
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
//!
//! ## Accessing Properties
//!
//! [`HunchResult`] provides typed convenience accessors for common
//! properties, plus generic [`first`](HunchResult::first) and
//! [`all`](HunchResult::all) methods for any [`Property`](matcher::span::Property):
//!
//! ```rust
//! use hunch::hunch;
//! use hunch::matcher::span::Property;
//!
//! let r = hunch("Movie.2024.FRENCH.1080p.BluRay.DTS.x264-GROUP.mkv");
//!
//! // Typed accessors (return the first value):
//! assert_eq!(r.title(), Some("Movie"));
//! assert_eq!(r.year(), Some(2024));
//!
//! // Generic accessor for any property:
//! assert_eq!(r.first(Property::Language), Some("French"));
//! ```
//!
//! ## Multi-Valued Properties
//!
//! Some properties (languages, episodes, "other" flags) can have multiple
//! values. Use [`all`](HunchResult::all) to retrieve them:
//!
//! ```rust
//! use hunch::hunch;
//!
//! let r = hunch("Movie.2024.2160p.UHD.BluRay.Remux.HDR.HEVC.DTS-HD.MA-GROUP.mkv");
//! let flags = r.other();
//! assert!(flags.contains(&"HDR10"));
//! assert!(flags.contains(&"Remux"));
//! ```
//!
//! ## JSON Output
//!
//! Convert results to a JSON-friendly map with [`to_flat_map`](HunchResult::to_flat_map):
//!
//! ```rust
//! use hunch::hunch;
//!
//! let r = hunch("Movie.2024.1080p.BluRay.x264-GROUP.mkv");
//! let map = r.to_flat_map();
//! assert_eq!(map["title"], "Movie");
//! assert_eq!(map["year"], 2024);
//! ```
//!
//! ## Logging / Debugging
//!
//! Hunch uses the [`log`] crate for diagnostic output. Enable it to
//! see how each pipeline stage processes a filename:
//!
//! ```bash
//! # CLI: use --verbose for debug, RUST_LOG for trace
//! hunch -v "Movie.2024.1080p.mkv"
//! RUST_LOG=hunch=trace hunch "Movie.2024.1080p.mkv"
//! ```
//!
//! In library usage, attach any [`log`]-compatible subscriber
//! (e.g., `env_logger`, `tracing-log`).
//!
//! ## Architecture
//!
//! Hunch uses a span-based, two-pass architecture with plain function
//! pointers (no trait objects, no `unsafe`):
//!
//! 1. **Tokenize** — split on separators, extract extension, detect brackets
//! 2. **Zone map** — detect anchors (SxxExx, 720p, x264) to establish
//!    title-zone vs tech-zone boundaries
//! 3. **Pass 1: Match & Resolve** — 20 TOML rule files + algorithmic
//!    matchers produce [`MatchSpan`](matcher::span::MatchSpan)s; conflict
//!    resolution keeps higher-priority / longer matches
//! 4. **Pass 2: Extract** — release group, title, episode title run with
//!    access to resolved match positions from Pass 1
//! 5. **Result** — [`HunchResult`] with 49 typed property accessors
//!
//! All regex patterns use the [`regex`] crate only (linear-time, ReDoS-immune).
//! TOML rule files are embedded at compile time via `include_str!` — no
//! runtime file I/O.

#![warn(missing_docs)]

/// Codec-like bare numbers to skip during number extraction.
///
/// These values appear in filenames as part of codec identifiers
/// (x264, x265, AES-128) and should not be treated as years or episodes.
pub(crate) const CODEC_NUMBERS: &[u32] = &[264, 265, 128];

/// Common separators used in media filenames.
///
/// These characters are treated as word boundaries when normalizing
/// filenames for title extraction and gap analysis. Path separators
/// (`/`, `\`) are intentionally excluded — they are only relevant
/// in the cross-file context module where full paths are analyzed.
pub(crate) const FILENAME_SEPS: &[char] = &['.', ' ', '_', '-', '+'];

pub mod matcher;
pub mod properties;
pub mod tokenizer;
pub mod zone_map;

mod hunch_result;
mod pipeline;

pub use hunch_result::{Confidence, HunchResult, MediaType};
pub use pipeline::Pipeline;

/// Parse a media filename and return structured metadata.
///
/// This is the main entry point for the library. It creates a default
/// [`Pipeline`] and runs it against the input string.
///
/// # Example
///
/// ```rust
/// let result = hunch::hunch("Movie.2024.1080p.BluRay.x264-GROUP.mkv");
/// assert_eq!(result.title(), Some("Movie"));
/// assert_eq!(result.year(), Some(2024));
/// assert_eq!(result.source(), Some("Blu-ray"));
/// assert_eq!(result.video_codec(), Some("H.264"));
/// assert_eq!(result.container(), Some("mkv"));
/// ```
pub fn hunch(input: &str) -> HunchResult {
    Pipeline::default().run(input)
}

/// Parse a media filename using sibling filenames for improved title detection.
///
/// When you have sibling files from the same directory, pass them here to
/// enable cross-file invariance detection. The **invariant text** across all
/// files is identified as the title — no language translation needed.
///
/// Falls back to standard [`hunch`] behavior when no invariant is found
/// or when `siblings` is empty.
///
/// # Example
///
/// ```rust
/// let result = hunch::hunch_with_context(
///     "Show.S01E03.720p.mkv",
///     &["Show.S01E01.720p.mkv", "Show.S01E02.720p.mkv"],
/// );
/// assert_eq!(result.title(), Some("Show"));
/// ```
pub fn hunch_with_context(input: &str, siblings: &[&str]) -> HunchResult {
    Pipeline::default().run_with_context(input, siblings)
}
