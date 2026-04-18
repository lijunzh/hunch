//! Compile-time registration of all matcher rule sets and legacy matchers.
//!
//! This module is a **declarative configuration table**: it lists every
//! TOML-driven rule set and every legacy matcher function the pipeline
//! runs in Pass 1. The orchestration logic lives in [`super::Pipeline`];
//! this module exists so that adding or tweaking a rule is a one-line
//! diff in a flat list rather than a hunt through a 970-line orchestrator.
//!
//! ## Adding a rule set
//!
//! 1. Add a TOML file under `rules/`.
//! 2. Add a `static FOO_RULES: LazyLock<RuleSet> = ...;` below.
//! 3. Add one [`TomlRule`] entry to the vec returned by
//!    [`build_toml_rules`].
//!
//! That's the entire surface area.
//!
//! ## Adding a legacy matcher
//!
//! Add the function pointer to [`build_legacy_matchers`]. The migration
//! goal is to convert these to TOML over time; until then, the table here
//! makes the inventory explicit.

use std::sync::LazyLock;

use crate::matcher::rule_loader::RuleSet;
use crate::matcher::span::{MatchSpan, Property};
use crate::priority;
use crate::properties::{
    aspect_ratio, bit_rate, bonus, crc32, date, episode_count, episodes, language, part, size,
    subtitle_language, uuid, version, website, year,
};

// ── TOML rule sets (embedded at compile time) ──────────────────────────────

pub(super) static VIDEO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_codec.toml")));
pub(super) static COLOR_DEPTH_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/color_depth.toml")));
pub(super) static COUNTRY_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/country.toml")));
pub(super) static STREAMING_SERVICE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/streaming_service.toml")));
pub(super) static VIDEO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_profile.toml")));
pub(super) static EPISODE_DETAILS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/episode_details.toml")));
pub(super) static ANIME_BONUS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/anime_bonus.toml")));
pub(super) static EDITION_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/edition.toml")));
pub(super) static AUDIO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_codec.toml")));
pub(super) static AUDIO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_profile.toml")));
pub(super) static AUDIO_CHANNELS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_channels.toml")));
pub(super) static OTHER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/other.toml")));
pub(super) static OTHER_POSITIONAL_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/other_positional.toml")));
pub(super) static VIDEO_API_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_api.toml")));
pub(super) static SOURCE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/source.toml")));
pub(super) static SCREEN_SIZE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/screen_size.toml")));
pub(super) static CONTAINER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/container.toml")));
pub(super) static FRAME_RATE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/frame_rate.toml")));
pub(super) static LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/language.toml")));
pub(super) static SUBTITLE_LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/subtitle_language.toml")));
pub(super) static EPISODE_FORMAT_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/episode_format.toml")));

// ── Types ──────────────────────────────────────────────────────────────────

/// Whether a TOML rule set should match tokens from directory segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SegmentScope {
    /// Match only filename tokens. Use for tech properties (source, codec, etc.)
    /// where directory names like "TV Shows" or "HD" would cause false positives.
    FilenameOnly,
    /// Match tokens from all path segments. Directory matches receive a priority
    /// penalty (`DIR_PRIORITY_PENALTY`) so filename matches always win in conflicts.
    /// Use for contextual properties (season, year, language) where directories
    /// carry genuine metadata ("Season 01/", "(2008)/", "VF/").
    AllSegments,
}

/// A legacy matcher function: takes raw input, returns property matches.
///
/// These are the matchers that have not yet been migrated to TOML. The
/// long-term goal is to delete this type and the corresponding registry
/// list once every property has a TOML rule set.
pub(super) type LegacyMatcherFn = fn(&str) -> Vec<MatchSpan>;

/// One TOML rule set's pipeline registration record.
///
/// Replaces the previous `(rules, property, priority, scope)` 4-tuple
/// with named fields so the registration table reads as data, not as a
/// puzzle.
pub(super) struct TomlRule {
    pub rules: &'static LazyLock<RuleSet>,
    pub property: Property,
    pub priority: i32,
    pub scope: SegmentScope,
}

// ── Registry tables ────────────────────────────────────────────────────────

// Local shorthands keep the tables one entry per line. `rustfmt` normally
// wants to split a struct literal across lines; the `#[rustfmt::skip]` on
// each table keeps them dense and grep-friendly.
use SegmentScope::{AllSegments, FilenameOnly};
use priority::{DEFAULT, HEURISTIC, POSITIONAL, STRUCTURAL, VOCABULARY};

const fn r(
    rules: &'static LazyLock<RuleSet>,
    property: Property,
    priority: i32,
    scope: SegmentScope,
) -> TomlRule {
    TomlRule {
        rules,
        property,
        priority,
        scope,
    }
}

/// All TOML-driven rule sets, in registration order.
///
/// **Order does not affect correctness** — Pass 1 conflict resolution
/// resolves overlaps by priority, not by registration order. The grouping
/// below is purely for human navigation.
#[rustfmt::skip]
pub(super) fn build_toml_rules() -> Vec<TomlRule> {
    use Property::*;
    vec![
        // Tech properties: unambiguous tokens, safe across all path segments.
        // (XviD, x264, 720p, AAC don't false-positive in directory names.)
        r(&VIDEO_CODEC_RULES,       VideoCodec,       DEFAULT,    AllSegments),
        r(&COLOR_DEPTH_RULES,       ColorDepth,       DEFAULT,    AllSegments),
        r(&AUDIO_CODEC_RULES,       AudioCodec,       DEFAULT,    AllSegments),
        r(&AUDIO_PROFILE_RULES,     AudioProfile,     VOCABULARY, AllSegments),
        r(&AUDIO_CHANNELS_RULES,    AudioChannels,    HEURISTIC,  AllSegments),
        r(&FRAME_RATE_RULES,        FrameRate,        DEFAULT,    AllSegments),
        r(&SCREEN_SIZE_RULES,       ScreenSize,       DEFAULT,    AllSegments),

        // Tech properties: ambiguous tokens, filename only.
        // Short tokens (HD, DV, TV, TS) would false-positive in dir names.
        r(&STREAMING_SERVICE_RULES, StreamingService, VOCABULARY, FilenameOnly),
        r(&VIDEO_PROFILE_RULES,     VideoProfile,     POSITIONAL, FilenameOnly),
        r(&EPISODE_DETAILS_RULES,   EpisodeDetails,   HEURISTIC,  FilenameOnly),
        r(&ANIME_BONUS_RULES,       EpisodeDetails,   HEURISTIC,  FilenameOnly),
        r(&EPISODE_FORMAT_RULES,    EpisodeFormat,    HEURISTIC,  FilenameOnly),

        r(&EDITION_RULES,           Edition,          DEFAULT,    AllSegments),

        // Other: AllSegments with dir priority penalty.
        // Per-directory zone maps filter false positives in title zones.
        r(&OTHER_RULES,             Other,            DEFAULT,    AllSegments),
        r(&OTHER_POSITIONAL_RULES,  Other,            POSITIONAL, FilenameOnly),

        r(&VIDEO_API_RULES,         VideoApi,         DEFAULT,    FilenameOnly),
        r(&SOURCE_RULES,            Source,           DEFAULT,    AllSegments),
        r(&CONTAINER_RULES,         Container,        STRUCTURAL, FilenameOnly),

        // Contextual properties: match all segments (dirs carry real metadata).
        // NOTE: Language, SubtitleLanguage, and Country are kept FilenameOnly
        // (or HEURISTIC) because directory names contain title words that
        // false-match language patterns (e.g. "Por" → Portuguese, "Fr" → French).
        // Directory-level language/season/year extraction is handled by the
        // legacy algorithmic matchers (language.rs, episodes.rs, year.rs).
        // When those legacy matchers are retired, segment-aware zone rules
        // will need to filter directory title words from language matches.
        r(&COUNTRY_RULES,           Country,          POSITIONAL, FilenameOnly),
        r(&LANGUAGE_RULES,          Language,         HEURISTIC,  AllSegments),
        r(&SUBTITLE_LANGUAGE_RULES, SubtitleLanguage, HEURISTIC,  FilenameOnly),
    ]
}

/// All legacy (non-TOML) matchers, in registration order.
///
/// `release_group` is intentionally absent — it runs in Pass 2 because it
/// needs the resolved tech matches as anchors.
#[rustfmt::skip]
pub(super) fn build_legacy_matchers() -> Vec<LegacyMatcherFn> {
    vec![
        aspect_ratio::find_matches,
        year::find_matches,
        date::find_matches,
        episodes::find_matches,
        episode_count::find_matches,
        language::find_matches,
        subtitle_language::find_matches,
        crc32::find_matches,
        uuid::find_matches,
        website::find_matches,
        size::find_matches,
        bit_rate::find_matches,
        part::find_matches,
        bonus::find_matches,
        version::find_matches,
        // NOTE: release_group is NOT here — it runs in Pass 2 (post-resolution).
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: every TOML rule set parses and contains at least the
    /// minimum number of entries we'd expect after a successful build.
    /// Catches "I emptied the rules file by accident" regressions.
    #[test]
    fn toml_rule_sets_parse_and_have_entries() {
        assert!(VIDEO_CODEC_RULES.exact_count() >= 10);
        assert!(COLOR_DEPTH_RULES.exact_count() >= 3);
        assert!(STREAMING_SERVICE_RULES.exact_count() >= 10);
        assert!(VIDEO_PROFILE_RULES.exact_count() >= 2);
        assert!(EPISODE_DETAILS_RULES.exact_count() >= 4);
        assert!(EDITION_RULES.exact_count() >= 10);
    }

    /// Sanity check: the registry tables build without panicking and have
    /// non-trivial size. Catches accidental `vec![]` edits.
    #[test]
    fn registry_tables_are_populated() {
        assert!(build_toml_rules().len() >= 20, "TOML rule registry shrank?");
        assert!(
            build_legacy_matchers().len() >= 10,
            "legacy matcher registry shrank?"
        );
    }
}
