//! The `Guess` result type — a structured bag of extracted metadata.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::matcher::span::{MatchSpan, Property};

/// The type of media detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Episode,
}

/// The result of parsing a media filename.
///
/// Provides typed accessors for common properties and a generic
/// `get(property)` for everything else.
#[derive(Debug, Clone)]
pub struct Guess {
    /// All properties extracted, keyed by property.
    props: BTreeMap<Property, Vec<String>>,
}

impl Guess {
    /// Build a `Guess` from resolved match spans, deduplicating values.
    pub(crate) fn from_matches(matches: &[MatchSpan]) -> Self {
        let mut props: BTreeMap<Property, Vec<String>> = BTreeMap::new();
        for m in matches {
            let values = props.entry(m.property).or_default();
            if !values.contains(&m.value) {
                values.push(m.value.clone());
            }
        }
        Self { props }
    }

    /// Set a computed property value directly (not from a match span).
    pub(crate) fn set(&mut self, property: Property, value: impl Into<String>) {
        let values = self.props.entry(property).or_default();
        let v = value.into();
        if !values.contains(&v) {
            values.push(v);
        }
    }

    // ── Typed accessors (return first value) ──

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
            _ => None,
        }
    }

    /// All "other" flags (e.g., "HDR", "Remux", "Proper").
    pub fn other(&self) -> Vec<&str> {
        self.all(Property::Other)
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
    pub fn to_flat_map(&self) -> BTreeMap<String, serde_json::Value> {
        let mut map = BTreeMap::new();
        for (k, v) in &self.props {
            let key = k.to_string();
            if v.len() == 1 {
                // Try to parse as number for year/season/episode
                if let Ok(n) = v[0].parse::<i64>() {
                    map.insert(key, serde_json::Value::Number(n.into()));
                } else {
                    map.insert(key, serde_json::Value::String(v[0].clone()));
                }
            } else {
                let arr: Vec<serde_json::Value> = v
                    .iter()
                    .map(|s| {
                        if let Ok(n) = s.parse::<i64>() {
                            serde_json::Value::Number(n.into())
                        } else {
                            serde_json::Value::String(s.clone())
                        }
                    })
                    .collect();
                map.insert(key, serde_json::Value::Array(arr));
            }
        }
        map
    }
}

impl std::fmt::Display for Guess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = self.to_flat_map();
        match serde_json::to_string_pretty(&map) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "<serialization error: {e}>"),
        }
    }
}
