//! Property matchers and TOML integration tests.
//!
//! Modules fall into three categories:
//! - **Algorithmic matchers**: export `find_matches(input) -> Vec<MatchSpan>`
//!   (episodes, title, release_group, date, year, etc.)
//! - **Cooperative legacy**: export `find_matches` for patterns TOML can't
//!   express (language bracket codes, subtitle_language extensions)
//! - **TOML-only test shells**: no `find_matches`, just `#[cfg(test)]`
//!   integration tests for their TOML rule files (video_codec, source, etc.)

pub mod aspect_ratio;
pub mod audio_codec;
pub mod audio_profile;
pub mod bit_rate;
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
