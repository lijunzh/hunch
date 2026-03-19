//! Core types for match spans and properties.
//!
//! This module defines [`Property`] — the enum of all 49 extractable metadata
//! fields — and [`MatchSpan`] — a positioned, valued match within an input
//! string. These are the fundamental types that flow through the entire
//! pipeline: matchers produce `MatchSpan`s, conflict resolution filters them,
//! and [`HunchResult`](crate::HunchResult) aggregates the survivors.

use std::fmt;

/// A named property that can be extracted from a media filename.
///
/// Each variant corresponds to one metadata field in the final
/// [`HunchResult`](crate::HunchResult). Use [`Property::from_name`] to parse
/// from a string, or [`fmt::Display`] to convert back.
///
/// # Example values
///
/// ```rust
/// use hunch::hunch;
/// use hunch::matcher::span::Property;
///
/// let r = hunch("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
/// assert_eq!(r.first(Property::Title), Some("The Matrix"));
/// assert_eq!(r.first(Property::Source), Some("Blu-ray"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Property {
    /// Movie name or series name (e.g., `"The Matrix"`, `"Breaking Bad"`).
    Title,
    /// Secondary title after a structural separator like `" - "` (e.g., `"Rogue Nation"`).
    AlternativeTitle,
    /// Release year as a 4-digit number (e.g., `"1999"`, `"2024"`).
    Year,
    /// Season number (e.g., `"1"` from `S01E02`).
    Season,
    /// Episode number (e.g., `"2"` from `S01E02`). Multi-episode spans produce multiple values.
    Episode,
    /// Absolute episode number without season context (e.g., `"147"` in anime releases).
    AbsoluteEpisode,
    /// Episode title — the subtitle after the SxxExx marker (e.g., `"Ozymandias"`).
    EpisodeTitle,
    /// Video codec (e.g., `"H.264"`, `"H.265"`, `"Xvid"`, `"AV1"`).
    VideoCodec,
    /// Video encoding profile (e.g., `"High"`, `"High 10"`, `"AVCHD"`).
    VideoProfile,
    /// Audio codec (e.g., `"AAC"`, `"DTS"`, `"TrueHD"`, `"Atmos"`, `"FLAC"`).
    AudioCodec,
    /// Audio codec profile (e.g., `"HD"`, `"HD MA"`, `"HE"`).
    AudioProfile,
    /// Audio channel layout (e.g., `"5.1"`, `"7.1"`, `"2.0"`).
    AudioChannels,
    /// Media source (e.g., `"Blu-ray"`, `"Web"`, `"HDTV"`, `"DVD"`, `"Ultra HD Blu-ray"`).
    Source,
    /// Resolution / screen size (e.g., `"1080p"`, `"720p"`, `"2160p"`, `"480i"`).
    ScreenSize,
    /// Video frame rate (e.g., `"24fps"`, `"60fps"`, `"25fps"`).
    FrameRate,
    /// Color bit depth (e.g., `"10-bit"`, `"8-bit"`).
    ColorDepth,
    /// File container / extension (e.g., `"mkv"`, `"mp4"`, `"avi"`).
    Container,
    /// Scene release group (e.g., `"GROUP"`, `"SPARKS"`, `"YIFY"`).
    ReleaseGroup,
    /// Streaming service origin (e.g., `"Netflix"`, `"Amazon Prime"`, `"Disney+"`).
    StreamingService,
    /// Audio language (e.g., `"English"`, `"French"`, `"Japanese"`). May have multiple values.
    Language,
    /// Subtitle language (e.g., `"English"`, `"French"`). May have multiple values.
    SubtitleLanguage,
    /// Country of origin (e.g., `"US"`, `"UK"`, `"FR"`).
    Country,
    /// Special edition label (e.g., `"Director's Cut"`, `"Extended"`, `"Remastered"`, `"IMAX"`).
    Edition,
    /// Release or air date (e.g., `"2024-01-15"`).
    Date,
    /// Catch-all flags (e.g., `"HDR10"`, `"Remux"`, `"Proper"`, `"Dual Audio"`, `"Widescreen"`).
    /// A single filename can produce multiple `Other` values.
    Other,
    /// File size (e.g., `"1.4 GB"`, `"700 MB"`).
    Size,
    /// Audio or video bit rate (e.g., `"320Kbps"`, `"1.5Mbps"`).
    BitRate,
    /// CD / disc number within a multi-disc set (e.g., `"1"` from `CD1`).
    Cd,
    /// Bonus content number (e.g., `"2"` from `-x02`).
    Bonus,
    /// Title of a bonus feature.
    BonusTitle,
    /// Film number in a franchise set (e.g., `"3"` from `-f03`).
    Film,
    /// Title associated with a film number marker.
    FilmTitle,
    /// Part number (e.g., `"2"` from `Part 2` or `Pt.II`).
    Part,
    /// CRC32 checksum in brackets (e.g., `"ABCD1234"` from `[ABCD1234]`).
    Crc,
    /// UUID identifier (e.g., `"12345678-1234-1234-1234-123456789abc"`).
    Uuid,
    /// Total number of CDs / discs (e.g., `"3"` from `3CDs`).
    CdCount,
    /// Disc number in multi-disc releases (e.g., `"1"` from `Disc 1`).
    Disc,
    /// Website / distribution site embedded in the filename (e.g., `"example.com"`).
    Website,
    /// Episode detail tag (e.g., `"Special"`, `"Pilot"`, `"Unaired"`).
    EpisodeDetails,
    /// Episode format (e.g., `"Minisode"`).
    EpisodeFormat,
    /// Calendar week number (e.g., `"52"` from `Week 52`).
    Week,
    /// Display aspect ratio (e.g., `"16:9"`, `"4:3"`, `"2.35:1"`).
    AspectRatio,
    /// Number of PROPER / REPACK re-releases (e.g., `"1"`, `"2"`).
    ProperCount,
    /// Inferred media type: `"movie"` or `"episode"`.
    MediaType,
    /// Release version number (e.g., `"2"` from `v2`).
    Version,
    /// Total episode count in a batch (e.g., `"24"` from `1 of 24`).
    EpisodeCount,
    /// Total season count in a batch (e.g., `"5"` from `2 of 5 Seasons`).
    SeasonCount,
    /// Video API / framework (e.g., `"DXVA"`).
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
///
/// Represents a span of bytes in the original input that was recognized as
/// a specific [`Property`] with a normalized value. Matchers produce these,
/// the conflict resolver filters overlapping ones, and the pipeline collects
/// survivors into a [`HunchResult`](crate::HunchResult).
///
/// # Example
///
/// ```rust
/// use hunch::matcher::span::{MatchSpan, Property};
///
/// let span = MatchSpan::new(11, 15, Property::Year, "1999")
///     .with_priority(1);
/// assert_eq!(span.len(), 4);
/// assert_eq!(span.value, "1999");
/// ```
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
    /// Create a new match span with default priority (0) and no flags.
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

    /// Mark this match as originating from a file extension.
    ///
    /// Extension-based matches (e.g., `.mkv` → `Container`) are excluded
    /// from title boundary calculations.
    #[must_use]
    pub fn as_extension(mut self) -> Self {
        self.is_extension = true;
        self
    }

    /// Mark this match as originating from a path component (directory name).
    #[must_use]
    pub fn as_path_based(mut self) -> Self {
        self.is_path_based = true;
        self
    }

    /// Set the conflict-resolution priority (higher values win).
    ///
    /// Default is `0`. Extension-derived container matches use `10`.
    /// Directory-segment matches receive a `-5` penalty.
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

    /// Whether this match span is zero-length.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Property {
    /// Parse a property name string into a `Property`.
    ///
    /// Accepts the same snake_case names used in JSON output and TOML
    /// `side_effects` declarations. Returns `None` for unrecognized names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use hunch::matcher::span::Property;
    ///
    /// assert_eq!(Property::from_name("video_codec"), Some(Property::VideoCodec));
    /// assert_eq!(Property::from_name("unknown"), None);
    /// ```
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

    /// Whether this property is semantically numeric and should be coerced
    /// to a JSON number in [`HunchResult::to_flat_map`](crate::HunchResult::to_flat_map).
    ///
    /// Name-like properties (`Title`, `ReleaseGroup`, `EpisodeTitle`, etc.)
    /// always serialize as strings, even when the value is all digits.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Year
                | Self::Season
                | Self::Episode
                | Self::AbsoluteEpisode
                | Self::Part
                | Self::Bonus
                | Self::Film
                | Self::Cd
                | Self::CdCount
                | Self::Disc
                | Self::Week
                | Self::EpisodeCount
                | Self::SeasonCount
                | Self::ProperCount
                | Self::Version
        )
    }
}
