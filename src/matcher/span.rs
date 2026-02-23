//! Core types for match spans and properties.

use std::fmt;

/// A named property that can be extracted from a filename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Property {
    Title,
    AlternativeTitle,
    Year,
    Season,
    Episode,
    EpisodeTitle,
    VideoCodec,
    VideoProfile,
    AudioCodec,
    AudioProfile,
    AudioChannels,
    Source,
    ScreenSize,
    FrameRate,
    ColorDepth,
    Container,
    ReleaseGroup,
    StreamingService,
    Language,
    SubtitleLanguage,
    Country,
    Edition,
    Date,
    Other,
    Size,
    BitRate,
    Cd,
    Bonus,
    Film,
    Part,
    Crc,
    MediaType,
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Title => "title",
            Self::AlternativeTitle => "alternative_title",
            Self::Year => "year",
            Self::Season => "season",
            Self::Episode => "episode",
            Self::EpisodeTitle => "episode_title",
            Self::VideoCodec => "video_codec",
            Self::VideoProfile => "video_profile",
            Self::AudioCodec => "audio_codec",
            Self::AudioProfile => "audio_profile",
            Self::AudioChannels => "audio_channels",
            Self::Source => "source",
            Self::ScreenSize => "screen_size",
            Self::FrameRate => "frame_rate",
            Self::ColorDepth => "color_depth",
            Self::Container => "container",
            Self::ReleaseGroup => "release_group",
            Self::StreamingService => "streaming_service",
            Self::Language => "language",
            Self::SubtitleLanguage => "subtitle_language",
            Self::Country => "country",
            Self::Edition => "edition",
            Self::Date => "date",
            Self::Other => "other",
            Self::Size => "size",
            Self::BitRate => "bit_rate",
            Self::Cd => "cd",
            Self::Bonus => "bonus",
            Self::Film => "film",
            Self::Part => "part",
            Self::Crc => "crc",
            Self::MediaType => "type",
        };
        write!(f, "{s}")
    }
}

/// A single match found in the input string.
#[derive(Debug, Clone)]
pub struct MatchSpan {
    /// Byte offset start (inclusive).
    pub start: usize,
    /// Byte offset end (exclusive).
    pub end: usize,
    /// Which property this match represents.
    pub property: Property,
    /// The normalized/canonical value.
    pub value: String,
    /// Tags for rule processing (e.g., "extension", "weak").
    pub tags: Vec<String>,
    /// Priority for conflict resolution (higher wins).
    pub priority: i32,
}

impl MatchSpan {
    pub fn new(start: usize, end: usize, property: Property, value: impl Into<String>) -> Self {
        Self {
            start,
            end,
            property,
            value: value.into(),
            tags: Vec::new(),
            priority: 0,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if two spans overlap.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// The raw length of this match in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
