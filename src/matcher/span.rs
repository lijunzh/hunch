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
    AbsoluteEpisode,
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
    BonusTitle,
    Film,
    FilmTitle,
    Part,
    Crc,
    Uuid,
    CdCount,
    Disc,
    Website,
    EpisodeDetails,
    EpisodeFormat,
    Week,
    AspectRatio,
    ProperCount,
    MediaType,
    Version,
    EpisodeCount,
    SeasonCount,
    VideoApi,
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Title => "title",
            Self::AlternativeTitle => "alternative_title",
            Self::Year => "year",
            Self::Season => "season",
            Self::Episode => "episode",
            Self::AbsoluteEpisode => "absolute_episode",
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
            Self::BonusTitle => "bonus_title",
            Self::Film => "film",
            Self::FilmTitle => "film_title",
            Self::Part => "part",
            Self::Crc => "crc32",
            Self::Uuid => "uuid",
            Self::CdCount => "cd_count",
            Self::Disc => "disc",
            Self::Website => "website",
            Self::EpisodeDetails => "episode_details",
            Self::EpisodeFormat => "episode_format",
            Self::Week => "week",
            Self::AspectRatio => "aspect_ratio",
            Self::ProperCount => "proper_count",
            Self::MediaType => "type",
            Self::Version => "version",
            Self::EpisodeCount => "episode_count",
            Self::SeasonCount => "season_count",
            Self::VideoApi => "video_api",
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
    /// True if this match came from a file extension (e.g., `.mkv`).
    pub is_extension: bool,
    /// True if this match came from a path component (e.g., `/Season 1/`).
    pub is_path_based: bool,
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
            is_extension: false,
            is_path_based: false,
            priority: 0,
        }
    }

    #[must_use]
    pub fn as_extension(mut self) -> Self {
        self.is_extension = true;
        self
    }

    #[must_use]
    pub fn as_path_based(mut self) -> Self {
        self.is_path_based = true;
        self
    }

    #[must_use]
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

impl Property {
    /// Parse a property name string (as used in TOML side_effects) into a Property.
    ///
    /// Returns `None` for unrecognized names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "title" => Some(Self::Title),
            "alternative_title" => Some(Self::AlternativeTitle),
            "year" => Some(Self::Year),
            "season" => Some(Self::Season),
            "episode" => Some(Self::Episode),
            "absolute_episode" => Some(Self::AbsoluteEpisode),
            "episode_title" => Some(Self::EpisodeTitle),
            "video_codec" => Some(Self::VideoCodec),
            "video_profile" => Some(Self::VideoProfile),
            "audio_codec" => Some(Self::AudioCodec),
            "audio_profile" => Some(Self::AudioProfile),
            "audio_channels" => Some(Self::AudioChannels),
            "source" => Some(Self::Source),
            "screen_size" => Some(Self::ScreenSize),
            "frame_rate" => Some(Self::FrameRate),
            "color_depth" => Some(Self::ColorDepth),
            "container" => Some(Self::Container),
            "release_group" => Some(Self::ReleaseGroup),
            "streaming_service" => Some(Self::StreamingService),
            "language" => Some(Self::Language),
            "subtitle_language" => Some(Self::SubtitleLanguage),
            "country" => Some(Self::Country),
            "edition" => Some(Self::Edition),
            "date" => Some(Self::Date),
            "other" => Some(Self::Other),
            "size" => Some(Self::Size),
            "bit_rate" => Some(Self::BitRate),
            "cd" => Some(Self::Cd),
            "bonus" => Some(Self::Bonus),
            "bonus_title" => Some(Self::BonusTitle),
            "film" => Some(Self::Film),
            "film_title" => Some(Self::FilmTitle),
            "part" => Some(Self::Part),
            "crc32" => Some(Self::Crc),
            "uuid" => Some(Self::Uuid),
            "cd_count" => Some(Self::CdCount),
            "disc" => Some(Self::Disc),
            "website" => Some(Self::Website),
            "episode_details" => Some(Self::EpisodeDetails),
            "episode_format" => Some(Self::EpisodeFormat),
            "week" => Some(Self::Week),
            "aspect_ratio" => Some(Self::AspectRatio),
            "proper_count" => Some(Self::ProperCount),
            "type" => Some(Self::MediaType),
            "version" => Some(Self::Version),
            "episode_count" => Some(Self::EpisodeCount),
            "season_count" => Some(Self::SeasonCount),
            "video_api" => Some(Self::VideoApi),
            _ => None,
        }
    }
}
