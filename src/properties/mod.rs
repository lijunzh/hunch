//! Property matchers — each module knows how to find one type of property.

pub mod audio_codec;
pub mod container;
pub mod edition;
pub mod episodes;
pub mod language;
pub mod other;
pub mod release_group;
pub mod screen_size;
pub mod source;
pub mod streaming_service;
pub mod title;
pub mod video_codec;
pub mod year;

use crate::matcher::span::MatchSpan;

/// Trait for a property matcher.
pub trait PropertyMatcher: Send + Sync {
    /// Find all matches of this property in the input string.
    fn find_matches(&self, input: &str) -> Vec<MatchSpan>;
}
