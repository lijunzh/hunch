//! Core types for match spans and properties.
//!
//! This module defines [`Property`] — the enum of all 49 extractable metadata
//! fields — and [`MatchSpan`] — a positioned, valued match within an input
//! string. These are the fundamental types that flow through the entire
//! pipeline: matchers produce `MatchSpan`s, conflict resolution filters them,
//! and [`HunchResult`](crate::HunchResult) aggregates the survivors.

use std::fmt;

/// How a match was produced — structural, context-confirmed, or heuristic.
///
/// This tag is purely informational: it feeds logging (`[CONTEXT]`,
/// `[HEURISTIC]` prefixes) and confidence scoring. It does **not** affect
/// conflict resolution — priority is still the authority there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Source {
    /// Matched by a deterministic structural rule (SxxExx, parenthesized year,
    /// codec keyword, etc.). This is the default and "happy path".
    #[default]
    Structural,
    /// Confirmed or injected by cross-file invariance analysis.
    Context,
    /// Produced by a single-file heuristic (digit decomposition, positional
    /// year guess). Valid but lower confidence than structural/context.
    Heuristic,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Structural => write!(f, "structural"),
            Source::Context => write!(f, "context"),
            Source::Heuristic => write!(f, "heuristic"),
        }
    }
}

/// Declares the [`Property`] enum, its `Display` impl, and `from_name`
/// constructor from a single source of truth.
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
macro_rules! define_properties {
    ($( $(#[$meta:meta])* $variant:ident => $name:expr ),* $(,)?) => {
        /// A named property that can be extracted from a media filename.
        ///
        /// Each variant corresponds to one metadata field in the final
        /// [`HunchResult`](crate::HunchResult). Use [`Property::from_name`] to parse
        /// from a string, or [`fmt::Display`] to convert back.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Property {
            $( $(#[$meta])* $variant ),*
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
                    $( $name => Some(Self::$variant), )*
                    _ => None,
                }
            }
        }

        impl fmt::Display for Property {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let s = match self {
                    $( Self::$variant => $name, )*
                };
                write!(f, "{s}")
            }
        }
    }
}

define_properties! {
    /// Movie name or series name (e.g., `"The Matrix"`, `"Breaking Bad"`).
    Title => "title",
    /// Secondary title after a structural separator like `" - "` (e.g., `"Rogue Nation"`).
    AlternativeTitle => "alternative_title",
    /// Release year as a 4-digit number (e.g., `"1999"`, `"2024"`).
    Year => "year",
    /// Season number (e.g., `"1"` from `S01E02`).
    Season => "season",
    /// Episode number (e.g., `"2"` from `S01E02`). Multi-episode spans produce multiple values.
    Episode => "episode",
    /// Absolute episode number without season context (e.g., `"147"` in anime releases).
    AbsoluteEpisode => "absolute_episode",
    /// Episode title — the subtitle after the SxxExx marker (e.g., `"Ozymandias"`).
    EpisodeTitle => "episode_title",
    /// Video codec (e.g., `"H.264"`, `"H.265"`, `"Xvid"`, `"AV1"`).
    VideoCodec => "video_codec",
    /// Video encoding profile (e.g., `"High"`, `"High 10"`, `"AVCHD"`).
    VideoProfile => "video_profile",
    /// Audio codec (e.g., `"AAC"`, `"DTS"`, `"TrueHD"`, `"Atmos"`, `"FLAC"`).
    AudioCodec => "audio_codec",
    /// Audio codec profile (e.g., `"HD"`, `"HD MA"`, `"HE"`).
    AudioProfile => "audio_profile",
    /// Audio channel layout (e.g., `"5.1"`, `"7.1"`, `"2.0"`).
    AudioChannels => "audio_channels",
    /// Media source (e.g., `"Blu-ray"`, `"Web"`, `"HDTV"`, `"DVD"`, `"Ultra HD Blu-ray"`).
    Source => "source",
    /// Resolution / screen size (e.g., `"1080p"`, `"720p"`, `"2160p"`, `"480i"`).
    ScreenSize => "screen_size",
    /// Video frame rate (e.g., `"24fps"`, `"60fps"`, `"25fps"`).
    FrameRate => "frame_rate",
    /// Color bit depth (e.g., `"10-bit"`, `"8-bit"`).
    ColorDepth => "color_depth",
    /// File container / extension (e.g., `"mkv"`, `"mp4"`, `"avi"`).
    Container => "container",
    /// Scene release group (e.g., `"GROUP"`, `"SPARKS"`, `"YIFY"`).
    ReleaseGroup => "release_group",
    /// Streaming service origin (e.g., `"Netflix"`, `"Amazon Prime"`, `"Disney+"`).
    StreamingService => "streaming_service",
    /// Audio language (e.g., `"English"`, `"French"`, `"Japanese"`). May have multiple values.
    Language => "language",
    /// Subtitle language (e.g., `"English"`, `"French"`). May have multiple values.
    SubtitleLanguage => "subtitle_language",
    /// Country of origin (e.g., `"US"`, `"UK"`, `"FR"`).
    Country => "country",
    /// Special edition label (e.g., `"Director's Cut"`, `"Extended"`, `"Remastered"`, `"IMAX"`).
    Edition => "edition",
    /// Release or air date (e.g., `"2024-01-15"`).
    Date => "date",
    /// Catch-all flags (e.g., `"HDR10"`, `"Remux"`, `"Proper"`, `"Dual Audio"`, `"Widescreen"`).
    /// A single filename can produce multiple `Other` values.
    Other => "other",
    /// File size (e.g., `"1.4 GB"`, `"700 MB"`).
    Size => "size",
    /// Audio or video bit rate (e.g., `"320Kbps"`, `"1.5Mbps"`).
    BitRate => "bit_rate",
    /// CD / disc number within a multi-disc set (e.g., `"1"` from `CD1`).
    Cd => "cd",
    /// Bonus content number (e.g., `"2"` from `-x02`).
    Bonus => "bonus",
    /// Title of a bonus feature.
    BonusTitle => "bonus_title",
    /// Film number in a franchise set (e.g., `"3"` from `-f03`).
    Film => "film",
    /// Title associated with a film number marker.
    FilmTitle => "film_title",
    /// Part number (e.g., `"2"` from `Part 2` or `Pt.II`).
    Part => "part",
    /// CRC32 checksum in brackets (e.g., `"ABCD1234"` from `[ABCD1234]`).
    Crc => "crc32",
    /// UUID identifier (e.g., `"12345678-1234-1234-1234-123456789abc"`).
    Uuid => "uuid",
    /// Total number of CDs / discs (e.g., `"3"` from `3CDs`).
    CdCount => "cd_count",
    /// Disc number in multi-disc releases (e.g., `"1"` from `Disc 1`).
    Disc => "disc",
    /// Website / distribution site embedded in the filename (e.g., `"example.com"`).
    Website => "website",
    /// Episode detail tag (e.g., `"Special"`, `"Pilot"`, `"Unaired"`).
    EpisodeDetails => "episode_details",
    /// Episode format (e.g., `"Minisode"`).
    EpisodeFormat => "episode_format",
    /// Calendar week number (e.g., `"52"` from `Week 52`).
    Week => "week",
    /// Display aspect ratio (e.g., `"16:9"`, `"4:3"`, `"2.35:1"`).
    AspectRatio => "aspect_ratio",
    /// Number of PROPER / REPACK re-releases (e.g., `"1"`, `"2"`).
    ProperCount => "proper_count",
    /// Inferred media type: `"movie"` or `"episode"`.
    MediaType => "type",
    /// Release version number (e.g., `"2"` from `v2`).
    Version => "version",
    /// Total episode count in a batch (e.g., `"24"` from `1 of 24`).
    EpisodeCount => "episode_count",
    /// Total season count in a batch (e.g., `"5"` from `2 of 5 Seasons`).
    SeasonCount => "season_count",
    /// Video API / framework (e.g., `"DXVA"`).
    VideoApi => "video_api",
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
    /// If true, the title extractor may absorb this match when it
    /// appears to be title content rather than metadata.
    pub reclaimable: bool,
    /// How this match was produced (structural, context, heuristic).
    pub source: Source,
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
            reclaimable: false,
            source: Source::default(),
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

    /// Mark this match as reclaimable by the title extractor.
    #[must_use]
    pub fn as_reclaimable(mut self) -> Self {
        self.reclaimable = true;
        self
    }

    /// Tag this match with a [`Source`] classification.
    #[must_use]
    pub fn with_source(mut self, source: Source) -> Self {
        self.source = source;
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
