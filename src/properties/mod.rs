//! Property matchers — each module knows how to find one type of property.

pub mod aspect_ratio;
pub mod audio_codec;
pub mod audio_profile;
pub mod bonus;
pub mod color_depth;
pub mod container;
pub mod country;
pub mod crc32;
pub mod date;
pub mod edition;
pub mod episode_count;
pub mod episode_details;
pub mod episodes;
pub mod frame_rate;
pub mod language;
pub mod other;
pub mod part;
pub mod release_group;
pub mod screen_size;
pub mod size;
pub mod source;
pub mod streaming_service;
pub mod subtitle_language;
pub mod title;
pub mod uuid;
pub mod version;
pub mod video_codec;
pub mod video_profile;
pub mod website;
pub mod year;

use crate::matcher::span::MatchSpan;

/// Trait for a property matcher.
pub trait PropertyMatcher: Send + Sync {
    /// Find all matches of this property in the input string.
    fn find_matches(&self, input: &str) -> Vec<MatchSpan>;
}
