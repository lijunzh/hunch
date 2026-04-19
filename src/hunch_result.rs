//! The [`HunchResult`] type — a structured bag of extracted metadata.
//!
//! This is the return type of [`hunch`](crate::hunch) and
//! [`hunch_with_context`](crate::hunch_with_context). It holds all properties extracted
//! from a media filename, with typed accessors for common fields and
//! generic [`first`](HunchResult::first) / [`all`](HunchResult::all)
//! methods for the full 50-property set.
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
///
/// `#[non_exhaustive]` so future variants (e.g. a `VeryHigh` for context-
/// resolved cross-file matches) can be added in minor releases without a
/// SemVer break. Downstream `match`es must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Confidence {
    /// Few or no properties extracted; title may be wrong.
    Low,
    /// Reasonable extraction but some ambiguity remains.
    Medium,
    /// Strong anchors found; high certainty in title and properties.
    High,
}

/// The type of media detected.
///
/// `#[non_exhaustive]` so future variants (e.g. `Music`, `TVMovie`) can be
/// added in minor releases without a SemVer break. Downstream `match`es must
/// include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
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

        // Derive Mimetype from Container (#158). This is a pure mapping —
        // no parsing — so it lives at result-build time rather than as a
        // matcher. Done after the dedup pass so we read the canonical
        // container value that survived.
        if let Some(container) = props.get(&Property::Container).and_then(|v| v.first())
            && let Some(mime) = container_to_mimetype(container)
        {
            props
                .entry(Property::Mimetype)
                .or_default()
                .push(mime.to_string());
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

    /// Audio bit rate (e.g., `"320Kbps"`, `"448Kbps"`).
    ///
    /// All bit-rate matches with `Kbps` units are routed here. See
    /// [`Property::AudioBitRate`] for the unit-based disambiguation rationale.
    pub fn audio_bit_rate(&self) -> Option<&str> {
        self.first(Property::AudioBitRate)
    }

    /// Video bit rate (e.g., `"1.5Mbps"`, `"19.1Mbps"`).
    ///
    /// All bit-rate matches with `Mbps` units are routed here. See
    /// [`Property::VideoBitRate`] for the unit-based disambiguation rationale.
    pub fn video_bit_rate(&self) -> Option<&str> {
        self.first(Property::VideoBitRate)
    }

    /// MIME type derived from the file container (e.g., `"video/mp4"`).
    ///
    /// This is a derived getter — the MIME type is not parsed from the
    /// filename but mapped from [`container`](Self::container) using a fixed
    /// table of common video/audio/subtitle containers. Returns `None` if
    /// the container is unknown or unmapped.
    ///
    /// The mapping is populated when the [`HunchResult`] is built so that
    /// callers don't have to maintain their own container → MIME table.
    pub fn mimetype(&self) -> Option<&str> {
        self.first(Property::Mimetype)
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

    /// `true` if the detected media type is [`MediaType::Movie`].
    ///
    /// Convenience over `media_type() == Some(MediaType::Movie)` for routing
    /// downstream lookups (e.g., TMDb movie endpoint vs. TVDb episode endpoint).
    /// Returns `false` when the media type is unknown — callers that need to
    /// distinguish "definitely not a movie" from "unknown" should use
    /// [`media_type`](Self::media_type) directly.
    pub fn is_movie(&self) -> bool {
        self.media_type() == Some(MediaType::Movie)
    }

    /// `true` if the detected media type is [`MediaType::Episode`].
    ///
    /// See [`is_movie`](Self::is_movie) for caveats around unknown media type.
    pub fn is_episode(&self) -> bool {
        self.media_type() == Some(MediaType::Episode)
    }

    /// `true` if the detected media type is [`MediaType::Extra`]
    /// (bonus features, openings, endings, specials, etc.).
    ///
    /// See [`is_movie`](Self::is_movie) for caveats around unknown media type.
    pub fn is_extra(&self) -> bool {
        self.media_type() == Some(MediaType::Extra)
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

/// Map a normalized container value (e.g., `"mp4"`, `"mkv"`) to its IANA
/// MIME type. Returns `None` for unknown containers (defensive default —
/// the alternative would be to fabricate a `application/octet-stream`
/// fallback, but a missing mimetype is more honest than a wrong one).
///
/// The table covers the containers actually emitted by hunch's parser
/// (see `src/properties/container.rs`) plus a few common aliases. Add
/// entries here if a new container is supported upstream.
fn container_to_mimetype(container: &str) -> Option<&'static str> {
    // Lower-case lookup so callers don't have to normalize. The values
    // returned are canonical IANA registrations where they exist; for
    // formats without an IANA entry (e.g., .mkv) we use the de-facto
    // standard MIME type widely accepted by browsers and players.
    match container.to_ascii_lowercase().as_str() {
        // Video containers.
        "mp4" | "m4v" => Some("video/mp4"),
        "mkv" => Some("video/x-matroska"),
        "avi" => Some("video/x-msvideo"),
        "mov" | "qt" => Some("video/quicktime"),
        "webm" => Some("video/webm"),
        "flv" => Some("video/x-flv"),
        "wmv" => Some("video/x-ms-wmv"),
        "mpg" | "mpeg" => Some("video/mpeg"),
        "ts" | "m2ts" | "mts" => Some("video/mp2t"),
        "vob" => Some("video/dvd"),
        "3gp" => Some("video/3gpp"),
        // Audio containers.
        "mp3" => Some("audio/mpeg"),
        "flac" => Some("audio/flac"),
        "m4a" => Some("audio/mp4"),
        "ogg" | "oga" => Some("audio/ogg"),
        "wav" => Some("audio/wav"),
        "wma" => Some("audio/x-ms-wma"),
        "aac" => Some("audio/aac"),
        // Subtitle containers.
        "srt" => Some("application/x-subrip"),
        "ass" | "ssa" => Some("text/x-ssa"),
        "vtt" => Some("text/vtt"),
        "sub" => Some("text/plain"),
        "idx" => Some("application/x-vobsub"),
        // Unknown container — return None rather than guess.
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    //! Unit tests for typed-accessor helpers added on top of the [`MediaType`]
    //! enum. The helpers themselves are pure derived getters — no parsing
    //! change — so we test them by manually setting the underlying property
    //! rather than running the full pipeline.

    use super::*;
    use crate::matcher::span::Property;

    fn empty_result() -> HunchResult {
        HunchResult {
            props: BTreeMap::new(),
            confidence: Confidence::Medium,
        }
    }

    #[test]
    fn is_movie_true_when_media_type_is_movie() {
        let mut r = empty_result();
        r.set(Property::MediaType, "movie");
        assert!(r.is_movie());
        assert!(!r.is_episode());
        assert!(!r.is_extra());
    }

    #[test]
    fn is_episode_true_when_media_type_is_episode() {
        let mut r = empty_result();
        r.set(Property::MediaType, "episode");
        assert!(r.is_episode());
        assert!(!r.is_movie());
        assert!(!r.is_extra());
    }

    #[test]
    fn is_extra_true_when_media_type_is_extra() {
        let mut r = empty_result();
        r.set(Property::MediaType, "extra");
        assert!(r.is_extra());
        assert!(!r.is_movie());
        assert!(!r.is_episode());
    }

    #[test]
    fn all_three_helpers_false_when_media_type_unknown() {
        // Explicit choice: helpers return `false` when the media type is
        // unknown, NOT "true for movie because there's no episode marker"
        // (which is what go-ptn does). Callers that need the trichotomy
        // (movie / episode / unknown) should use `media_type()` directly.
        let r = empty_result();
        assert_eq!(r.media_type(), None);
        assert!(!r.is_movie());
        assert!(!r.is_episode());
        assert!(!r.is_extra());
    }

    #[test]
    fn is_movie_case_insensitive_via_media_type() {
        // media_type() lower-cases internally, so any casing of the stored
        // value should still produce the right helper answer.
        let mut r = empty_result();
        r.set(Property::MediaType, "MOVIE");
        assert!(r.is_movie());
    }
}
