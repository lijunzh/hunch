//! Pipeline v0.2: tokenize → match → zones → title → result.
//!
//! The v0.2 pipeline tokenizes the input first, then matches tokens
//! against TOML rules and raw-string patterns. Zone detection replaces
//! the v0.1 prune_* heuristics.

use crate::hunch_result::HunchResult;
use crate::matcher::engine;
use crate::matcher::rule_loader::RuleSet;
use crate::matcher::span::{MatchSpan, Property};
use crate::options::Options;
use crate::tokenizer::{self, TokenStream};

use std::sync::LazyLock;

// ── TOML rule sets (embedded at compile time) ──────────────────────────────

static VIDEO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/video_codec.toml")));
static COLOR_DEPTH_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/color_depth.toml")));
static COUNTRY_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/country.toml")));
static STREAMING_SERVICE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/streaming_service.toml")));
static VIDEO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/video_profile.toml")));
static EPISODE_DETAILS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/episode_details.toml")));
static EDITION_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/edition.toml")));
static AUDIO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/audio_codec.toml")));
static AUDIO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/audio_profile.toml")));

// ── Legacy matchers (not yet migrated to TOML) ─────────────────────────────

use crate::properties::title;
use crate::properties::{
    aspect_ratio, audio_codec, audio_profile, bonus, color_depth, container, country, crc32, date,
    edition, episode_count, episode_details, episodes, frame_rate, language, other, part,
    release_group, screen_size, size, source, streaming_service, subtitle_language, uuid, version,
    video_codec, video_profile, website, year,
};

/// A legacy matcher function: takes raw input, returns property matches.
type LegacyMatcherFn = fn(&str) -> Vec<MatchSpan>;

/// The parsing pipeline.
pub struct Pipeline {
    #[allow(dead_code)]
    options: Options,
    /// TOML-driven rule sets: (rules, property, priority).
    toml_rules: Vec<(&'static LazyLock<RuleSet>, Property, i32)>,
    /// Legacy matchers that run against raw input (to be migrated).
    legacy_matchers: Vec<LegacyMatcherFn>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new(Options::default())
    }
}

impl Pipeline {
    pub fn new(options: Options) -> Self {
        let toml_rules: Vec<(&'static LazyLock<RuleSet>, Property, i32)> = vec![
            (&VIDEO_CODEC_RULES, Property::VideoCodec, 0),
            (&COLOR_DEPTH_RULES, Property::ColorDepth, 0),
            (&STREAMING_SERVICE_RULES, Property::StreamingService, 1),
            (&VIDEO_PROFILE_RULES, Property::VideoProfile, -2),
            (&EPISODE_DETAILS_RULES, Property::EpisodeDetails, -1),
            (&EDITION_RULES, Property::Edition, 0),
            (&COUNTRY_RULES, Property::Country, -2),
            (&AUDIO_CODEC_RULES, Property::AudioCodec, 0),
            (&AUDIO_PROFILE_RULES, Property::AudioProfile, 1),
        ];

        // Legacy matchers — everything not yet in TOML.
        let legacy_matchers: Vec<LegacyMatcherFn> = vec![
            container::find_matches,
            video_codec::find_matches, // legacy kept alongside TOML for compound-codec edge cases
            audio_codec::find_matches,
            audio_profile::find_matches,
            video_profile::find_matches, // legacy kept for lookaround edge cases
            color_depth::find_matches,   // legacy kept for multi-token compounds
            source::find_matches,
            screen_size::find_matches,
            aspect_ratio::find_matches,
            year::find_matches,
            date::find_matches,
            episodes::find_matches,
            episode_details::find_matches, // legacy kept for negative lookahead (Special Edition)
            episode_count::find_matches,
            edition::find_matches, // legacy kept for multi-token compounds
            other::find_matches,
            language::find_matches,
            subtitle_language::find_matches,
            country::find_matches, // legacy kept for 2-char boundary detection
            streaming_service::find_matches, // legacy kept for compound patterns
            crc32::find_matches,
            uuid::find_matches,
            website::find_matches,
            size::find_matches,
            part::find_matches,
            bonus::find_matches,
            version::find_matches,
            frame_rate::find_matches,
            release_group::find_matches,
        ];

        Self {
            options,
            toml_rules,
            legacy_matchers,
        }
    }

    /// Run the full pipeline on an input string.
    pub fn run(&self, input: &str) -> HunchResult {
        // Step 1: Tokenize.
        let token_stream = tokenizer::tokenize(input);

        // Step 2: Match — TOML rules against tokens + legacy matchers against raw input.
        let mut all_matches = self.match_all(input, &token_stream);

        // Step 3: Resolve overlapping conflicts.
        engine::resolve_conflicts(&mut all_matches);

        // Step 4: Zone-based disambiguation (replaces prune_* functions).
        self.apply_zone_rules(input, &token_stream, &mut all_matches);

        // Step 5: Post-processing.
        if let Some(title_match) = title::extract_title(input, &all_matches) {
            all_matches.push(title_match);
        }
        if let Some(ep_title) = title::extract_episode_title(input, &all_matches) {
            all_matches.push(ep_title);
        }

        let media_type = title::infer_media_type(&all_matches);
        let proper_count = compute_proper_count(input, &all_matches);

        // Step 6: Build result.
        let mut result = HunchResult::from_matches(&all_matches);
        result.set(Property::MediaType, media_type);
        if proper_count > 0 {
            result.set(Property::ProperCount, proper_count.to_string());
        }
        result
    }

    /// Run all matchers: TOML token rules + legacy raw-string matchers.
    fn match_all(&self, input: &str, token_stream: &TokenStream) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // TOML rules: match tokens and multi-token windows.
        let tokens = &token_stream.tokens;
        for (rule_set, property, priority) in &self.toml_rules {
            let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

            // Try windows of 1, 2, and 3 tokens (longest first).
            for window_size in (1..=3).rev() {
                for i in 0..tokens.len() {
                    if i + window_size > tokens.len() {
                        break;
                    }

                    // Skip if any part of this window is already matched.
                    let win_start = tokens[i].start;
                    let win_end = tokens[i + window_size - 1].end;
                    if matched_ranges
                        .iter()
                        .any(|(s, e)| win_start < *e && win_end > *s)
                    {
                        continue;
                    }

                    // Build the compound text from the raw input (preserving separators).
                    let compound = if window_size == 1 {
                        tokens[i].text.clone()
                    } else {
                        input[win_start..win_end].to_string()
                    };

                    if let Some(value) = rule_set.match_token(&compound) {
                        matches.push(
                            MatchSpan::new(win_start, win_end, *property, value)
                                .with_priority(*priority),
                        );
                        matched_ranges.push((win_start, win_end));
                    }
                }
            }
        }

        // Legacy matchers: run against raw input.
        for matcher in &self.legacy_matchers {
            matches.extend(matcher(input));
        }

        matches
    }

    /// Zone-based disambiguation.
    ///
    /// Replaces the v0.1 prune_* functions with structural zone detection.
    /// The "title zone" is everything before the first anchor (tech token).
    /// Language/source/episode_details in the title zone are likely title words.
    fn apply_zone_rules(
        &self,
        input: &str,
        _token_stream: &TokenStream,
        matches: &mut Vec<MatchSpan>,
    ) {
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

        // ── Rule 1: Language in title zone → likely a title word ─────────
        // Anchor set for language zone: ALL technical properties.
        let lang_anchor_props = [
            Property::Year,
            Property::VideoCodec,
            Property::AudioCodec,
            Property::Source,
            Property::ScreenSize,
            Property::Edition,
            Property::Other,
            Property::AudioChannels,
            Property::Season,
            Property::Episode,
            Property::StreamingService,
        ];

        let first_tech_pos = matches
            .iter()
            .filter(|m| m.start >= fn_start && lang_anchor_props.contains(&m.property))
            .map(|m| m.start)
            .min();

        if let Some(tech_pos) = first_tech_pos {
            let title_zone_mid = fn_start + (tech_pos - fn_start) / 2;
            matches.retain(|m| {
                !(m.property == Property::Language
                    && m.start >= fn_start
                    && m.start < title_zone_mid)
            });
        } else {
            // No technical tokens at all — prune all language matches.
            matches.retain(|m| m.property != Property::Language);
        }

        // ── Rule 2: Duplicate source in title zone → title word ─────────
        // Uses year/season/episode as anchor (NOT full tech set).
        let source_anchor_pos = matches
            .iter()
            .filter(|m| {
                m.start >= fn_start
                    && matches!(
                        m.property,
                        Property::Year | Property::Season | Property::Episode
                    )
            })
            .map(|m| m.start)
            .min();

        if let Some(anchor) = source_anchor_pos {
            let has_early_source = matches
                .iter()
                .any(|m| m.property == Property::Source && m.start >= fn_start && m.start < anchor);
            let has_late_source = matches
                .iter()
                .any(|m| m.property == Property::Source && m.start >= anchor);

            if has_early_source && has_late_source {
                matches.retain(|m| {
                    !(m.property == Property::Source && m.start >= fn_start && m.start < anchor)
                });
            }
        }

        // ── Rule 3: Redundant HD tags when source has UHD ────────────────
        let source_has_uhd = matches
            .iter()
            .any(|m| m.property == Property::Source && m.value.contains("Ultra HD"));
        if source_has_uhd {
            matches.retain(|m| !(m.property == Property::Other && m.value == "Ultra HD"));
        }

        // ── Rule 4: EpisodeDetails before any episode marker → title ─────
        let first_ep_pos = matches
            .iter()
            .filter(|m| {
                m.start >= fn_start
                    && (m.property == Property::Season || m.property == Property::Episode)
            })
            .map(|m| m.start)
            .min();

        matches.retain(|m| {
            if m.property != Property::EpisodeDetails || m.start < fn_start {
                return true;
            }
            match first_ep_pos {
                Some(ep_pos) => m.start >= ep_pos,
                None => false,
            }
        });

        // ── Rule 5: Other overlapping ReleaseGroup → drop ambiguous Other ─
        let rg_spans: Vec<(usize, usize)> = matches
            .iter()
            .filter(|m| m.property == Property::ReleaseGroup)
            .map(|m| (m.start, m.end))
            .collect();

        if !rg_spans.is_empty() {
            const AMBIGUOUS_OTHER: &[&str] = &["High Quality", "High Resolution"];
            matches.retain(|m| {
                if m.property != Property::Other || !AMBIGUOUS_OTHER.contains(&m.value.as_ref()) {
                    return true;
                }
                !rg_spans.iter().any(|(rs, re)| m.start < *re && m.end > *rs)
            });
        }
    }
}

// ── Proper count computation ───────────────────────────────────────────────

static REAL_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)^REAL$").unwrap());

static REPACK_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)^(?:REPACK|RERIP)(\d+)?$").unwrap());

fn compute_proper_count(input: &str, matches: &[MatchSpan]) -> u32 {
    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let mut has_real = false;
    let mut proper_count_raw: u32 = 0;
    let mut repack_count: u32 = 0;

    let tech_start = matches
        .iter()
        .filter(|m| {
            m.start >= fn_start
                && matches!(
                    m.property,
                    Property::VideoCodec
                        | Property::AudioCodec
                        | Property::Source
                        | Property::ScreenSize
                )
        })
        .map(|m| m.start)
        .min();

    if let Some(ts) = tech_start {
        // Use tokenizer to check for standalone REAL in tech zone.
        let tech_tokens = tokenizer::tokenize(&input[ts..]);
        if tech_tokens
            .tokens
            .iter()
            .any(|t| t.text.eq_ignore_ascii_case("REAL"))
        {
            has_real = true;
        }
    }

    for m in matches
        .iter()
        .filter(|m| m.property == Property::Other && m.value == "Proper" && m.start >= fn_start)
    {
        let raw = &input[m.start..m.end];
        if REAL_RE.is_match(raw) {
            has_real = true;
            continue;
        }
        if let Some(caps) = REPACK_RE.captures(raw) {
            if let Some(num) = caps.get(1) {
                repack_count += num.as_str().parse::<u32>().unwrap_or(1);
            } else {
                repack_count += 1;
            }
            continue;
        }
        proper_count_raw += 1;
    }

    let base = if has_real { 2 } else { proper_count_raw };
    base + repack_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_movie_parse() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");

        assert_eq!(result.title(), Some("The Matrix"));
        assert_eq!(result.year(), Some(1999));
        assert_eq!(result.screen_size(), Some("1080p"));
        assert_eq!(result.source(), Some("Blu-ray"));
        assert_eq!(result.video_codec(), Some("H.264"));
        assert_eq!(result.release_group(), Some("GROUP"));
        assert_eq!(result.container(), Some("mkv"));
    }

    #[test]
    fn test_episode_parse() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv");

        assert_eq!(result.title(), Some("Breaking Bad"));
        assert_eq!(result.season(), Some(5));
        assert_eq!(result.episode(), Some(16));
        assert_eq!(result.screen_size(), Some("720p"));
        assert_eq!(result.video_codec(), Some("H.264"));
        assert_eq!(result.release_group(), Some("DEMAND"));
    }

    #[test]
    fn test_minimal_input() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.mkv");

        assert_eq!(result.title(), Some("Movie"));
        assert_eq!(result.container(), Some("mkv"));
    }

    #[test]
    fn test_4k_hdr() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.2024.2160p.UHD.BluRay.Remux.HDR.HEVC.DTS-HD.MA-GROUP.mkv");

        assert_eq!(result.title(), Some("Movie"));
        assert_eq!(result.year(), Some(2024));
        assert_eq!(result.screen_size(), Some("2160p"));
        assert_eq!(result.video_codec(), Some("H.265"));
        assert!(result.other().contains(&"HDR10"));
        assert!(result.other().contains(&"Remux"));
    }

    #[test]
    fn test_toml_video_codec_basic() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.HEVC.1080p.mkv");
        assert_eq!(result.video_codec(), Some("H.265"));
    }

    #[test]
    fn test_toml_color_depth() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.10bit.1080p.mkv");
        assert_eq!(result.color_depth(), Some("10-bit"));
    }

    #[test]
    fn test_toml_streaming_service() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Show.S01E01.AMZN.WEB-DL.1080p.mkv");
        assert_eq!(result.streaming_service(), Some("Amazon Prime"));
    }

    #[test]
    fn test_toml_edition_multi_token() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.Directors.Cut.1080p.BluRay.mkv");
        assert_eq!(result.edition(), Some("Director's Cut"));
    }

    #[test]
    fn test_toml_edition_single_token() {
        let pipeline = Pipeline::default();
        let result = pipeline.run("Movie.Remastered.1080p.BluRay.mkv");
        assert_eq!(result.edition(), Some("Remastered"));
    }

    #[test]
    fn test_toml_rules_load() {
        // Smoke test: all TOML rule sets parse and have entries.
        assert!(VIDEO_CODEC_RULES.exact_count() >= 10);
        assert!(COLOR_DEPTH_RULES.exact_count() >= 3);
        assert!(STREAMING_SERVICE_RULES.exact_count() >= 10);
        assert!(VIDEO_PROFILE_RULES.exact_count() >= 5);
        assert!(EPISODE_DETAILS_RULES.exact_count() >= 4);
        assert!(EDITION_RULES.exact_count() >= 10);
    }
}
