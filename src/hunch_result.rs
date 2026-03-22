//! The [`HunchResult`] type — a structured bag of extracted metadata.
//!
//! This is the return type of [`hunch`](crate::hunch) and
//! [`hunch_with_context`](crate::hunch_with_context). It holds all properties extracted
//! from a media filename, with typed accessors for common fields and
//! generic [`first`](HunchResult::first) / [`all`](HunchResult::all)
//! methods for the full 49-property set.
//!
//! # Display
//!
//! `HunchResult` implements [`Display`](std::fmt::Display) as
//! pretty-printed JSON, and [`to_flat_map`](HunchResult::to_flat_map)
//! provides a `BTreeMap` suitable for serialization.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::matcher::span::{MatchSpan, Property};

/// How confident hunch is in the extracted result.
///
/// Computed from structural signals like the number of tech anchors found,
/// whether a title was extracted, and whether cross-file context was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Few or no properties extracted; title may be wrong.
    Low,
    /// Reasonable extraction but some ambiguity remains.
    Medium,
    /// Strong anchors found; high certainty in title and properties.
    High,
}

/// The type of media detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// A standalone movie / film.
    Movie,
    /// A TV series episode (has season/episode markers).
    Episode,
    /// Supplementary content: bonus features, openings, endings, previews,
    /// specials (SP/OVA/OAD/ONA), commercials, menus, tokuten.
    /// The specific marker is available via [`episode_details`](HunchResult::episode_details).
    Extra,
}

/// The result of parsing a media filename.
///
/// Provides typed convenience accessors for common properties (e.g.,
/// [`title`](Self::title), [`year`](Self::year), [`season`](Self::season))
/// and generic [`first`](Self::first) / [`all`](Self::all) methods that
/// accept any [`Property`](crate::matcher::span::Property) variant.
///
/// # Example
///
/// ```rust
/// use hunch::hunch;
///
/// let r = hunch("Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv");
/// assert_eq!(r.title(), Some("Breaking Bad"));
/// assert_eq!(r.season(), Some(5));
/// assert_eq!(r.episode(), Some(16));
/// assert_eq!(r.release_group(), Some("DEMAND"));
///
/// // Pretty-print as JSON:
/// println!("{r}");
/// ```
#[derive(Debug, Clone)]
pub struct HunchResult {
    /// All properties extracted, keyed by property.
    props: BTreeMap<Property, Vec<String>>,
    /// Confidence level of the extraction.
    confidence: Confidence,
}

impl HunchResult {
    /// Build a `HunchResult` from resolved match spans, deduplicating values.
    /// Language/SubtitleLanguage values use case-insensitive dedup to prevent
    /// duplicates from TOML captures ("NL") and legacy normalizers ("nl").
    pub(crate) fn from_matches(matches: &[MatchSpan]) -> Self {
        let mut props: BTreeMap<Property, Vec<String>> = BTreeMap::new();
        for m in matches {
            let values = props.entry(m.property).or_default();
            let is_lang = matches!(m.property, Property::Language | Property::SubtitleLanguage);
            let already_present = if is_lang {
                // Case-insensitive dedup for language values.
                values.iter().any(|v| v.eq_ignore_ascii_case(&m.value))
            } else {
                values.contains(&m.value)
            };
            if !already_present {
                values.push(m.value.clone());
            }
        }
        Self {
            props,
            confidence: Confidence::Medium, // default; pipeline sets the real value
        }
    }

    /// Set a computed property value directly (not from a match span).
    pub(crate) fn set(&mut self, property: Property, value: impl Into<String>) {
        let values = self.props.entry(property).or_default();
        let v = value.into();
        if !values.contains(&v) {
            values.push(v);
        }
    }

    /// Set the confidence level.
    pub(crate) fn set_confidence(&mut self, confidence: Confidence) {
        self.confidence = confidence;
    }

    // ── Typed accessors (return first value) ──

    /// How confident hunch is in this result.
    ///
    /// Based on structural signals: number of tech anchors, title quality,
    /// and whether cross-file context was used.
    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// The main title (movie name or series name).
    pub fn title(&self) -> Option<&str> {
        self.first(Property::Title)
    }

    /// Release year.
    pub fn year(&self) -> Option<i32> {
        self.first(Property::Year).and_then(|v| v.parse().ok())
    }

    /// Season number.
    pub fn season(&self) -> Option<i32> {
        self.first(Property::Season).and_then(|v| v.parse().ok())
    }

    /// Episode number.
    pub fn episode(&self) -> Option<i32> {
        self.first(Property::Episode).and_then(|v| v.parse().ok())
    }

    /// Episode title.
    pub fn episode_title(&self) -> Option<&str> {
        self.first(Property::EpisodeTitle)
    }

    /// Video codec (e.g., "H.264", "H.265").
    pub fn video_codec(&self) -> Option<&str> {
        self.first(Property::VideoCodec)
    }

    /// Audio codec (e.g., "AAC", "DTS").
    pub fn audio_codec(&self) -> Option<&str> {
        self.first(Property::AudioCodec)
    }

    /// Audio channels (e.g., "5.1", "7.1").
    pub fn audio_channels(&self) -> Option<&str> {
        self.first(Property::AudioChannels)
    }

    /// Source (e.g., "Blu-ray", "Web", "HDTV").
    pub fn source(&self) -> Option<&str> {
        self.first(Property::Source)
    }

    /// Screen size (e.g., "1080p", "720p", "2160p").
    pub fn screen_size(&self) -> Option<&str> {
        self.first(Property::ScreenSize)
    }

    /// Container / file extension (e.g., "mkv", "mp4").
    pub fn container(&self) -> Option<&str> {
        self.first(Property::Container)
    }

    /// Release group.
    pub fn release_group(&self) -> Option<&str> {
        self.first(Property::ReleaseGroup)
    }

    /// Edition (e.g., "Director's Cut", "Extended").
    pub fn edition(&self) -> Option<&str> {
        self.first(Property::Edition)
    }

    /// Streaming service (e.g., "Netflix", "Amazon Prime").
    pub fn streaming_service(&self) -> Option<&str> {
        self.first(Property::StreamingService)
    }

    /// Color depth (e.g., "10-bit", "8-bit").
    pub fn color_depth(&self) -> Option<&str> {
        self.first(Property::ColorDepth)
    }

    /// Video profile (e.g., "High", "High 10").
    pub fn video_profile(&self) -> Option<&str> {
        self.first(Property::VideoProfile)
    }

    /// Part number.
    pub fn part(&self) -> Option<i32> {
        self.first(Property::Part).and_then(|s| s.parse().ok())
    }

    /// Proper count (number of PROPER/REPACK occurrences).
    pub fn proper_count(&self) -> Option<u32> {
        self.first(Property::ProperCount)
            .and_then(|s| s.parse().ok())
    }

    /// Detected media type.
    pub fn media_type(&self) -> Option<MediaType> {
        match self.first(Property::MediaType)?.to_lowercase().as_str() {
            "movie" => Some(MediaType::Movie),
            "episode" => Some(MediaType::Episode),
            "extra" => Some(MediaType::Extra),
            _ => None,
        }
    }

    /// All "other" flags (e.g., "HDR", "Remux", "Proper").
    pub fn other(&self) -> Vec<&str> {
        self.all(Property::Other)
    }

    /// Episode detail tag (e.g., "Special", "OVA", "NCED", "OP").
    pub fn episode_details(&self) -> Option<&str> {
        self.first(Property::EpisodeDetails)
    }

    /// Audio language (e.g., "English", "French", "Japanese").
    pub fn language(&self) -> Option<&str> {
        self.first(Property::Language)
    }

    /// All audio languages (when multiple are present).
    pub fn languages(&self) -> Vec<&str> {
        self.all(Property::Language)
    }

    /// Subtitle language (e.g., "English", "French").
    pub fn subtitle_language(&self) -> Option<&str> {
        self.first(Property::SubtitleLanguage)
    }

    /// All subtitle languages (when multiple are present).
    pub fn subtitle_languages(&self) -> Vec<&str> {
        self.all(Property::SubtitleLanguage)
    }

    /// Bonus content number (e.g., 2 from `-x02`).
    pub fn bonus(&self) -> Option<i32> {
        self.first(Property::Bonus).and_then(|s| s.parse().ok())
    }

    /// Release or air date (e.g., "2024-01-15").
    pub fn date(&self) -> Option<&str> {
        self.first(Property::Date)
    }

    /// Film number in a franchise set (e.g., 3 from `-f03`).
    pub fn film(&self) -> Option<i32> {
        self.first(Property::Film).and_then(|s| s.parse().ok())
    }

    /// Disc number (e.g., 1 from `Disc 1`).
    pub fn disc(&self) -> Option<i32> {
        self.first(Property::Disc).and_then(|s| s.parse().ok())
    }

    /// Video frame rate (e.g., "24fps", "60fps").
    pub fn frame_rate(&self) -> Option<&str> {
        self.first(Property::FrameRate)
    }

    // ── Generic accessors ──

    /// Get the first value for a property.
    pub fn first(&self, property: Property) -> Option<&str> {
        self.props
            .get(&property)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Get all values for a property.
    pub fn all(&self, property: Property) -> Vec<&str> {
        self.props
            .get(&property)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the full properties map.
    pub fn properties(&self) -> &BTreeMap<Property, Vec<String>> {
        &self.props
    }

    /// Convert to a flat map (first value per property), useful for JSON output.
    ///
    /// Only semantically numeric properties (year, season, episode, etc.) are
    /// coerced to JSON numbers. Name-like properties (title, release_group,
    /// episode_title, etc.) always serialize as strings, even if the value
    /// happens to be all digits (e.g., the movie "2001").
    pub fn to_flat_map(&self) -> BTreeMap<String, serde_json::Value> {
        let mut map = BTreeMap::new();
        for (k, v) in &self.props {
            let key = k.to_string();
            let numeric = k.is_numeric();
            if v.len() == 1 {
                if numeric {
                    if let Ok(n) = v[0].parse::<i64>() {
                        map.insert(key, serde_json::Value::Number(n.into()));
                        continue;
                    }
                }
                map.insert(key, serde_json::Value::String(v[0].clone()));
            } else {
                let arr: Vec<serde_json::Value> = v
                    .iter()
                    .map(|s| {
                        if numeric {
                            if let Ok(n) = s.parse::<i64>() {
                                return serde_json::Value::Number(n.into());
                            }
                        }
                        serde_json::Value::String(s.clone())
                    })
                    .collect();
                map.insert(key, serde_json::Value::Array(arr));
            }
        }
        map
    }
}

impl std::fmt::Display for HunchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = self.to_flat_map();
        match serde_json::to_string_pretty(&map) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "<serialization error: {e}>"),
        }
    }
}
