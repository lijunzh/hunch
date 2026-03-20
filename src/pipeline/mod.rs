//! Pipeline v0.2.1: tokenize → zones → match → disambiguate → title → result.
//!
//! The pipeline tokenizes the input, builds zone boundaries, then matches
//! tokens against TOML rules and legacy matchers. Zone-aware disambiguation
//! replaces v0.1 prune_* heuristics.

pub(crate) mod context;
mod invariance;
mod matching;
mod pass2_helpers;
mod proper_count;
pub(crate) mod token_context;
mod zone_rules;

use crate::hunch_result::HunchResult;
use crate::matcher::engine;
use crate::matcher::rule_loader::RuleSet;
use crate::matcher::span::{MatchSpan, Property};
use crate::tokenizer::{self, TokenStream};
use crate::zone_map::{self, ZoneMap};
use matching::MatchContext;

use log::{debug, trace};
use std::sync::LazyLock;

// ── TOML rule sets (embedded at compile time) ──────────────────────────────

static VIDEO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_codec.toml")));
static COLOR_DEPTH_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/color_depth.toml")));
static COUNTRY_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/country.toml")));
static STREAMING_SERVICE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/streaming_service.toml")));
static VIDEO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_profile.toml")));
static EPISODE_DETAILS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/episode_details.toml")));
static ANIME_BONUS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/anime_bonus.toml")));
static EDITION_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/edition.toml")));
static AUDIO_CODEC_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_codec.toml")));
static AUDIO_PROFILE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_profile.toml")));
static AUDIO_CHANNELS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/audio_channels.toml")));
static OTHER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/other.toml")));
static OTHER_POSITIONAL_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/other_positional.toml")));
static VIDEO_API_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/video_api.toml")));
static SOURCE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/source.toml")));
static SCREEN_SIZE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/screen_size.toml")));
static CONTAINER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/container.toml")));
static FRAME_RATE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/frame_rate.toml")));
static LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/language.toml")));
static SUBTITLE_LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/subtitle_language.toml")));
static EPISODE_FORMAT_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../../rules/episode_format.toml")));

// ── Legacy matchers (not yet migrated to TOML) ─────────────────────────────

use crate::properties::title;
use crate::properties::{
    aspect_ratio, bit_rate, bonus, crc32, date, episode_count, episodes, language, part,
    release_group, size, subtitle_language, uuid, version, website, year,
};

/// A legacy matcher function: takes raw input, returns property matches.
type LegacyMatcherFn = fn(&str) -> Vec<MatchSpan>;

/// Whether a TOML rule set should match tokens from directory segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentScope {
    /// Match only filename tokens. Use for tech properties (source, codec, etc.)
    /// where directory names like "TV Shows" or "HD" would cause false positives.
    FilenameOnly,
    /// Match tokens from all path segments. Directory matches receive a priority
    /// penalty (`DIR_PRIORITY_PENALTY`) so filename matches always win in conflicts.
    /// Use for contextual properties (season, year, language) where directories
    /// carry genuine metadata ("Season 01/", "(2008)/", "VF/").
    AllSegments,
}

/// Priority penalty applied to matches from directory segments.
/// Ensures filename matches always win over directory matches in conflict resolution.
const DIR_PRIORITY_PENALTY: i32 = -5;

/// The two-pass parsing pipeline.
///
/// Orchestrates the full parsing flow: tokenization → zone mapping
/// → TOML + legacy matching → conflict resolution → zone disambiguation
/// → release group / title extraction → result assembly.
///
/// See [`Pipeline::run`] for the main entry point, or use
/// [`hunch`](crate::hunch) / [`hunch_with_context`](crate::hunch_with_context)
/// for convenience.
pub struct Pipeline {
    /// TOML-driven rule sets: (rules, property, priority, segment_scope).
    toml_rules: Vec<(&'static LazyLock<RuleSet>, Property, i32, SegmentScope)>,
    /// Legacy matchers that run against raw input (to be migrated).
    legacy_matchers: Vec<LegacyMatcherFn>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    /// Create a new pipeline.
    ///
    /// Prefer [`hunch`](crate::hunch) for one-shot parsing.
    /// Construct a `Pipeline` directly when you want to reuse the same
    /// configuration across many inputs.
    pub fn new() -> Self {
        let toml_rules: Vec<(&'static LazyLock<RuleSet>, Property, i32, SegmentScope)> = vec![
            // Tech properties: unambiguous tokens safe for all segments.
            // These were previously scanned across full paths by legacy matchers.
            // Tokens like XviD, x264, 720p, AAC are unambiguous in directory names.
            (
                &VIDEO_CODEC_RULES,
                Property::VideoCodec,
                0,
                SegmentScope::AllSegments,
            ),
            (
                &COLOR_DEPTH_RULES,
                Property::ColorDepth,
                0,
                SegmentScope::AllSegments,
            ),
            (
                &AUDIO_CODEC_RULES,
                Property::AudioCodec,
                0,
                SegmentScope::AllSegments,
            ),
            (
                &AUDIO_PROFILE_RULES,
                Property::AudioProfile,
                1,
                SegmentScope::AllSegments,
            ),
            (
                &AUDIO_CHANNELS_RULES,
                Property::AudioChannels,
                -1,
                SegmentScope::AllSegments,
            ),
            (
                &FRAME_RATE_RULES,
                Property::FrameRate,
                0,
                SegmentScope::AllSegments,
            ),
            (
                &SCREEN_SIZE_RULES,
                Property::ScreenSize,
                0,
                SegmentScope::AllSegments,
            ),
            // Tech properties: ambiguous tokens, filename only.
            // Short tokens (HD, DV, TV, TS) would false-positive in dir names.
            (
                &STREAMING_SERVICE_RULES,
                Property::StreamingService,
                1,
                SegmentScope::FilenameOnly,
            ),
            (
                &VIDEO_PROFILE_RULES,
                Property::VideoProfile,
                -2,
                SegmentScope::FilenameOnly,
            ),
            (
                &EPISODE_DETAILS_RULES,
                Property::EpisodeDetails,
                -1,
                SegmentScope::FilenameOnly,
            ),
            (
                &ANIME_BONUS_RULES,
                Property::EpisodeDetails,
                -1,
                SegmentScope::FilenameOnly,
            ),
            (
                &EPISODE_FORMAT_RULES,
                Property::EpisodeFormat,
                -1,
                SegmentScope::FilenameOnly,
            ),
            (
                &EDITION_RULES,
                Property::Edition,
                0,
                SegmentScope::AllSegments,
            ),
            // Other: AllSegments with dir priority penalty.
            // Per-directory zone maps filter false positives in title zones.
            (&OTHER_RULES, Property::Other, 0, SegmentScope::AllSegments),
            (
                &OTHER_POSITIONAL_RULES,
                Property::Other,
                -2,
                SegmentScope::FilenameOnly,
            ),
            (
                &VIDEO_API_RULES,
                Property::VideoApi,
                0,
                SegmentScope::FilenameOnly,
            ),
            (
                &SOURCE_RULES,
                Property::Source,
                0,
                SegmentScope::AllSegments,
            ),
            (
                &CONTAINER_RULES,
                Property::Container,
                5,
                SegmentScope::FilenameOnly,
            ),
            // Contextual properties: match all segments (dirs carry real metadata)
            // NOTE: Language, SubtitleLanguage, and Country are kept FilenameOnly
            // for now because directory names contain title words that false-match
            // language patterns (e.g., "Por" → Portuguese, "Fr" → French).
            // Directory-level language/season/year extraction is handled by the
            // legacy algorithmic matchers (language.rs, episodes.rs, year.rs)
            // which run on the raw input string.
            // When those legacy matchers are retired, we'll need segment-aware
            // zone rules to filter directory title words from language matches.
            (
                &COUNTRY_RULES,
                Property::Country,
                -2,
                SegmentScope::FilenameOnly,
            ),
            (
                &LANGUAGE_RULES,
                Property::Language,
                -1,
                SegmentScope::AllSegments,
            ),
            (
                &SUBTITLE_LANGUAGE_RULES,
                Property::SubtitleLanguage,
                -1,
                SegmentScope::FilenameOnly,
            ),
        ];

        // Legacy matchers — algorithmic patterns not expressible in TOML.
        // Note: source is now fully TOML (source.toml with side_effects).
        // audio_profile is handled entirely by audio_profile.toml.
        let legacy_matchers: Vec<LegacyMatcherFn> = vec![
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
        ];

        Self {
            toml_rules,
            legacy_matchers,
        }
    }

    /// Run the full pipeline on an input string.
    ///
    /// ## Two-pass architecture (v0.3)
    ///
    /// **Pass 1**: Tech property resolution — TOML rules + legacy matchers
    /// (everything except release_group). Conflict resolution + zone
    /// disambiguation produces `resolved_tech_matches`.
    ///
    /// **Pass 2**: Positional property extraction — release_group uses
    /// resolved match positions (no more `is_known_token` exclusion list).
    /// Title, episode_title, alternative_title use all resolved matches.
    pub fn run(&self, input: &str) -> HunchResult {
        let (mut matches, token_stream, zone_map) = self.pass1(input);
        self.pass2(input, &mut matches, &zone_map, &token_stream, None, None)
    }

    /// Parse a filename using sibling filenames for cross-file title detection.
    ///
    /// Siblings should be raw filenames (no directory paths). Even 1–2 siblings
    /// can dramatically improve title extraction for CJK and non-standard
    /// formats.
    ///
    /// Cross-file analysis produces an `InvarianceReport` that informs:
    /// - **Title**: invariant text across files
    /// - **Year signals**: year-like numbers classified as title vs metadata
    /// - **Episode signals**: sequential variant numbers as episode evidence
    ///
    /// # Example
    ///
    /// ```rust
    /// use hunch::Pipeline;
    ///
    /// let pipeline = Pipeline::new();
    /// let result = pipeline.run_with_context(
    ///     "Show.S01E03.720p.mkv",
    ///     &["Show.S01E01.720p.mkv", "Show.S01E02.720p.mkv"],
    /// );
    /// assert_eq!(result.title(), Some("Show"));
    /// ```
    pub fn run_with_context(&self, input: &str, siblings: &[&str]) -> HunchResult {
        if siblings.is_empty() {
            return self.run(input);
        }

        // 1. Run Pass 1 on target + all siblings.
        let (target_matches, target_ts, target_zm) = self.pass1(input);
        let sibling_results: Vec<_> = siblings.iter().map(|s| self.pass1(s)).collect();

        // 2. Unified invariance analysis (title + year + episode signals).
        let sibling_analyses: Vec<_> = siblings
            .iter()
            .zip(&sibling_results)
            .map(|(s, (matches, _, _))| invariance::FileAnalysis {
                input: s,
                matches: matches.as_slice(),
            })
            .collect();
        let report = invariance::analyze_invariance(
            &invariance::FileAnalysis {
                input,
                matches: &target_matches,
            },
            &sibling_analyses,
        );

        debug!(
            "cross-file context: {} sibling(s), title={:?}, {} year signal(s), {} episode signal(s)",
            siblings.len(),
            report.title,
            report.year_signals.len(),
            report.episode_signals.len(),
        );
        for ys in &report.year_signals {
            trace!(
                "  [YEAR] {} at {}..{} invariant={}",
                ys.value, ys.start, ys.end, ys.is_invariant
            );
        }
        for es in &report.episode_signals {
            trace!(
                "  [EPISODE] {} at {}..{} sequential={} digits={}",
                es.value, es.start, es.end, es.is_sequential, es.digit_count
            );
        }

        // 3. Run Pass 2 with invariance report.
        let mut matches = target_matches;
        self.pass2(
            input,
            &mut matches,
            &target_zm,
            &target_ts,
            report.title.as_deref(),
            Some(&report),
        )
    }

    /// Run Pass 1: tokenize → zone map → match → conflict resolve → zone disambiguate.
    ///
    /// Returns the resolved tech matches, token stream, and zone map.
    /// This is the reusable core that `run_with_context()` calls on both
    /// the target file and each sibling.
    fn pass1(&self, input: &str) -> (Vec<MatchSpan>, TokenStream, ZoneMap) {
        // Step 1: Tokenize.
        let token_stream = tokenizer::tokenize(input);
        debug!(
            "step 1: tokenized into {} segment(s), {} total token(s)",
            token_stream.segments.len(),
            token_stream
                .segments
                .iter()
                .map(|s| s.tokens.len())
                .sum::<usize>()
        );

        // Step 1b: Build zone map (anchor detection + year disambiguation).
        let zone_map = zone_map::build_zone_map(input, &token_stream);
        debug!(
            "step 1b: zone map — has_anchors={}, title_zone={}..{}, year={:?}",
            zone_map.has_anchors,
            zone_map.title_zone.start,
            zone_map.title_zone.end,
            zone_map.year.as_ref().map(|y| y.value)
        );

        // Step 2: Match — TOML rules against tokens + legacy matchers against raw input.
        // NOTE: release_group is NOT included here — it runs in Pass 2.
        let mut all_matches = self.match_all(input, &token_stream, &zone_map);
        debug!(
            "step 2: matching produced {} raw match(es)",
            all_matches.len()
        );
        for m in &all_matches {
            trace!(
                "  raw match: {:?}={} at {}..{} (pri={})",
                m.property, m.value, m.start, m.end, m.priority
            );
        }

        // Step 2b: Year disambiguation using ZoneMap.
        if let Some(ref yi) = zone_map.year
            && !yi.title_years.is_empty()
        {
            all_matches.retain(|m| {
                if m.property != Property::Year {
                    return true;
                }
                !yi.title_years
                    .iter()
                    .any(|ty| m.start < ty.end && m.end > ty.start)
            });
        }

        // Step 3: Resolve overlapping conflicts.
        let pre_resolve_count = all_matches.len();
        engine::resolve_conflicts(&mut all_matches);
        debug!(
            "step 3: conflict resolution — {} → {} match(es)",
            pre_resolve_count,
            all_matches.len()
        );

        // Step 4: Zone-based disambiguation.
        let pre_zone_count = all_matches.len();
        zone_rules::apply_zone_rules(input, &zone_map, &token_stream, &mut all_matches);
        debug!(
            "step 4: zone disambiguation — {} → {} match(es)",
            pre_zone_count,
            all_matches.len()
        );
        for m in &all_matches {
            trace!(
                "  resolved: {:?}={} at {}..{}",
                m.property, m.value, m.start, m.end
            );
        }

        (all_matches, token_stream, zone_map)
    }

    /// Run Pass 2: positional extraction (release group, title, episode title, etc.).
    ///
    /// When `title_override` is `Some(...)`, the provided title is used directly
    /// instead of running the standard positional title extractor. This is the
    /// hook for cross-file invariance detection (`run_with_context`).
    ///
    /// When `report` is `Some(...)`, year and episode signals from cross-file
    /// analysis are applied to disambiguate year-in-title numbers and confirm
    /// episode evidence.
    fn pass2(
        &self,
        input: &str,
        all_matches: &mut Vec<MatchSpan>,
        zone_map: &ZoneMap,
        token_stream: &TokenStream,
        title_override: Option<&str>,
        report: Option<&invariance::InvarianceReport>,
    ) -> HunchResult {
        // Step 5a: Release group (post-resolution — can see claimed positions).
        let rg_matches = release_group::find_matches(input, all_matches, zone_map, token_stream);
        if !rg_matches.is_empty() {
            debug!(
                "step 5a: release group — found {:?}",
                rg_matches
                    .iter()
                    .map(|m| m.value.as_str())
                    .collect::<Vec<_>>()
            );
        }
        all_matches.extend(rg_matches);

        // Step 5a.1: Zone rules that depend on release group positions.
        zone_rules::apply_post_release_group_rules(all_matches);

        // Step 5a.2: Cross-file year/episode disambiguation.
        // When an InvarianceReport is available, use its signals to:
        //   - Suppress Year matches for invariant year-like numbers (they're title content)
        //   - Inject episode matches for sequential variant numbers
        if let Some(report) = report {
            pass2_helpers::apply_invariance_signals(input, all_matches, report);
        }

        // Step 5b: Title extraction.
        if let Some(override_title) = title_override {
            // Cross-file context provided a title — use it directly.
            // Find the title's byte range in the input for a proper MatchSpan.
            if let Some(start) = input.find(override_title) {
                let end = start + override_title.len();
                let title_match = MatchSpan::new(start, end, Property::Title, override_title);
                debug!(
                    "step 5b: title override — \"{}\" at {}..{}",
                    title_match.value, title_match.start, title_match.end
                );
                title::absorb_reclaimable(&title_match, all_matches);
                all_matches.push(title_match);
            } else {
                // Title text not found verbatim — set it without a byte range.
                debug!(
                    "step 5b: title override (no byte range) — \"{}\"",
                    override_title
                );
                all_matches.push(MatchSpan::new(0, 0, Property::Title, override_title));
            }
        } else if let Some(title_match) =
            title::extract_title(input, all_matches, zone_map, token_stream)
        {
            debug!(
                "step 5b: title extracted — \"{}\" at {}..{}",
                title_match.value, title_match.start, title_match.end
            );
            // Remove reclaimable matches absorbed into the title.
            title::absorb_reclaimable(&title_match, all_matches);
            all_matches.push(title_match);
        }
        // Film title: when -fNN- marker exists, split franchise from movie title.
        if let Some((film_title, adjusted_title)) =
            title::extract_film_title(input, all_matches, token_stream)
        {
            all_matches.retain(|m| m.property != Property::Title);
            all_matches.push(film_title);
            all_matches.push(adjusted_title);
        }

        // Step 5c: Episode title.
        if let Some(ep_title) = title::extract_episode_title(input, all_matches, token_stream) {
            debug!("step 5c: episode title — \"{}\"", ep_title.value);
            // Remove release_group if it overlaps with the episode title.
            // Plex-dash format (`Show - S01E01 - Episode Title.mkv`) triggers
            // last-word fallback release_group extraction on the final word of
            // the episode title (e.g., "Ninja" from "Rising Ninja"). Fixes #38.
            let ep_start = ep_title.start;
            let ep_end = ep_title.end;
            all_matches.retain(|m| {
                if m.property != Property::ReleaseGroup {
                    return true;
                }
                // Drop RG if it's fully inside or substantially overlaps the episode title.
                let overlap_start = m.start.max(ep_start);
                let overlap_end = m.end.min(ep_end);
                let overlap = overlap_end.saturating_sub(overlap_start);
                let rg_len = m.end.saturating_sub(m.start).max(1);
                // If ≥50% of the release_group span is inside the episode title, drop it.
                overlap * 2 < rg_len
            });
            all_matches.push(ep_title);
        }

        // Step 5d: Alternative title(s).
        let alt_titles = title::extract_alternative_titles(input, all_matches, token_stream);
        for alt_title in alt_titles {
            all_matches.push(alt_title);
        }

        let media_type = title::infer_media_type(input, all_matches);
        let proper_count = proper_count::compute_proper_count(input, all_matches);

        // Step 5e: Strip video/audio tech properties from subtitle containers.
        // Files like .ass, .srt, .sub should not carry video_codec, color_depth, etc.
        pass2_helpers::strip_tech_from_subtitle_containers(all_matches);

        // Step 6: Build result.
        debug!(
            "step 6: building result from {} final match(es), media_type={}",
            all_matches.len(),
            media_type
        );
        let mut result = HunchResult::from_matches(all_matches);
        result.set(Property::MediaType, media_type);
        if proper_count > 0 {
            result.set(Property::ProperCount, proper_count.to_string());
        }

        // Step 7: Compute confidence.
        let confidence =
            pass2_helpers::compute_confidence(&result, title_override.is_some(), all_matches);
        result.set_confidence(confidence);
        debug!("step 7: confidence = {:?}", confidence);

        result
    }

    /// Run all matchers: TOML token rules + legacy raw-string matchers.
    fn match_all(
        &self,
        input: &str,
        token_stream: &TokenStream,
        zone_map: &ZoneMap,
    ) -> Vec<MatchSpan> {
        let mut matches = Vec::new();

        // TOML rules: segment-aware matching.
        // Each rule set declares its SegmentScope:
        //   FilenameOnly  → skip directory segments entirely
        //   AllSegments   → match dirs too, but with a priority penalty
        for (rule_set, property, priority, scope) in &self.toml_rules {
            for (seg_idx, segment) in token_stream.segments.iter().enumerate() {
                let is_dir = segment.kind == tokenizer::SegmentKind::Directory;

                // Skip directory segments for filename-only rules.
                if is_dir && *scope == SegmentScope::FilenameOnly {
                    continue;
                }

                // Directory matches get a priority penalty so filename wins in conflicts.
                let effective_priority = if is_dir {
                    *priority + DIR_PRIORITY_PENALTY
                } else {
                    *priority
                };

                // Use per-directory zone map for directory segments.
                let dir_zone = if is_dir {
                    zone_map
                        .dir_zones
                        .iter()
                        .find(|dz| dz.segment_idx == seg_idx)
                } else {
                    None
                };

                let tokens = &segment.tokens;
                matching::match_tokens_in_segment(
                    &MatchContext {
                        input,
                        tokens,
                        rule_set,
                        property: *property,
                        priority: effective_priority,
                        zone_map,
                        dir_zone,
                    },
                    &mut matches,
                );
            }
        }

        // Legacy matchers: run against raw input.
        for matcher in &self.legacy_matchers {
            matches.extend(matcher(input));
        }

        // Extension → Container: emit directly from the tokenizer's extension
        // field. This is PATH A for container detection (see container.toml).
        // Priority 10 beats all other container matches.
        if let Some(ext) = &token_stream.extension {
            let ext_start = input.len() - ext.len();
            matches.push(
                MatchSpan::new(ext_start, input.len(), Property::Container, ext.as_str())
                    .as_extension()
                    .with_priority(10),
            );
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_rules_load() {
        // Smoke test: all TOML rule sets parse and have entries.
        assert!(VIDEO_CODEC_RULES.exact_count() >= 10);
        assert!(COLOR_DEPTH_RULES.exact_count() >= 3);
        assert!(STREAMING_SERVICE_RULES.exact_count() >= 10);
        assert!(VIDEO_PROFILE_RULES.exact_count() >= 2);
        assert!(EPISODE_DETAILS_RULES.exact_count() >= 4);
        assert!(EDITION_RULES.exact_count() >= 10);
    }
}
