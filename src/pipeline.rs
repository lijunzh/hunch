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
use crate::zone_map::{self, ZoneMap};

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
static AUDIO_CHANNELS_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/audio_channels.toml")));
static OTHER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/other.toml")));
static OTHER_WEAK_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/other_weak.toml")));
static VIDEO_API_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/video_api.toml")));
static SOURCE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/source.toml")));
static SCREEN_SIZE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/screen_size.toml")));
static CONTAINER_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/container.toml")));
static FRAME_RATE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/frame_rate.toml")));
static LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/language.toml")));
static SUBTITLE_LANGUAGE_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/subtitle_language.toml")));
static EPISODE_FORMAT_RULES: LazyLock<RuleSet> =
    LazyLock::new(|| RuleSet::from_toml(include_str!("../rules/episode_format.toml")));

// ── Legacy matchers (not yet migrated to TOML) ─────────────────────────────

use crate::properties::title;
use crate::properties::{
    aspect_ratio, bit_rate, bonus, crc32, date, episode_count, episodes, language, part,
    release_group, size, source, subtitle_language, uuid, version, website, year,
};

/// A legacy matcher function: takes raw input, returns property matches.
type LegacyMatcherFn = fn(&str) -> Vec<MatchSpan>;

/// The parsing pipeline.
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

pub struct Pipeline {
    #[allow(dead_code)]
    options: Options,
    /// TOML-driven rule sets: (rules, property, priority, segment_scope).
    toml_rules: Vec<(&'static LazyLock<RuleSet>, Property, i32, SegmentScope)>,
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
                &EPISODE_FORMAT_RULES,
                Property::EpisodeFormat,
                -1,
                SegmentScope::FilenameOnly,
            ),
            (
                &EDITION_RULES,
                Property::Edition,
                0,
                SegmentScope::FilenameOnly,
            ),
            (&OTHER_RULES, Property::Other, 0, SegmentScope::FilenameOnly),
            (
                &OTHER_WEAK_RULES,
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
                SegmentScope::FilenameOnly,
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
                SegmentScope::FilenameOnly,
            ),
            (
                &SUBTITLE_LANGUAGE_RULES,
                Property::SubtitleLanguage,
                -1,
                SegmentScope::FilenameOnly,
            ),
        ];

        // Legacy matchers — everything not yet in TOML.
        // Note: audio_codec is kept here only for combined codec+channel patterns (DD5.1,
        // etc.) and standalone channel counts. Simple codec patterns are in audio_codec.toml.
        // audio_profile is handled entirely by audio_profile.toml — no legacy needed.
        let legacy_matchers: Vec<LegacyMatcherFn> = vec![
            source::find_matches,
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

        // Step 1b: Build zone map (anchor detection + year disambiguation).
        let zone_map = zone_map::build_zone_map(input, &token_stream);

        // Step 2: Match — TOML rules against tokens + legacy matchers against raw input.
        let mut all_matches = self.match_all(input, &token_stream, &zone_map);

        // Step 3: Resolve overlapping conflicts.
        engine::resolve_conflicts(&mut all_matches);

        // Step 4: Zone-based disambiguation (replaces prune_* functions).
        self.apply_zone_rules(input, &token_stream, &mut all_matches);

        // Step 5: Post-processing.
        if let Some(title_match) = title::extract_title(input, &all_matches) {
            all_matches.push(title_match);
        }
        // Film title: when -fNN- marker exists, split franchise from movie title.
        if let Some((film_title, adjusted_title)) = title::extract_film_title(input, &all_matches) {
            all_matches.retain(|m| m.property != Property::Title);
            all_matches.push(film_title);
            all_matches.push(adjusted_title);
        }
        if let Some(ep_title) = title::extract_episode_title(input, &all_matches) {
            all_matches.push(ep_title);
        }
        if let Some(alt_title) = title::extract_alternative_title(input, &all_matches) {
            all_matches.push(alt_title);
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
            for segment in &token_stream.segments {
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

                let tokens = &segment.tokens;
                self.match_tokens_in_segment(
                    input,
                    tokens,
                    rule_set,
                    *property,
                    effective_priority,
                    zone_map,
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

    /// Match tokens within a single segment against a TOML rule set.
    ///
    /// Uses a sliding window of 1–3 tokens (longest first) to handle compound
    /// patterns like "WEB-DL" or "HD-DVD". Emits primary matches and any
    /// side-effect spans declared in the TOML pattern.
    #[allow(clippy::too_many_arguments)]
    fn match_tokens_in_segment(
        &self,
        input: &str,
        tokens: &[tokenizer::Token],
        rule_set: &RuleSet,
        property: Property,
        priority: i32,
        zone_map: &ZoneMap,
        matches: &mut Vec<MatchSpan>,
    ) {
        use crate::matcher::rule_loader::ZoneScope;

        let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

        for window_size in (1..=3).rev() {
            for i in 0..tokens.len() {
                if i + window_size > tokens.len() {
                    break;
                }

                let win_start = tokens[i].start;
                let win_end = tokens[i + window_size - 1].end;

                // ── Zone scope filtering ─────────────────────────────
                // Only filter when we have reliable zone boundaries.
                if zone_map.has_anchors {
                    let in_title_zone = zone_map.title_zone.contains(&win_start);
                    match rule_set.zone_scope {
                        ZoneScope::TechOnly if in_title_zone => continue,
                        ZoneScope::AfterAnchor if in_title_zone => continue,
                        _ => {}
                    }
                }
                if matched_ranges
                    .iter()
                    .any(|(s, e)| win_start < *e && win_end > *s)
                {
                    continue;
                }

                let compound = if window_size == 1 {
                    tokens[i].text.clone()
                } else {
                    input[win_start..win_end].to_string()
                };

                if let Some(token_match) = rule_set.match_token(&compound) {
                    // ── Neighbor constraint checks ──────────────────
                    let last_idx = i + window_size - 1;
                    if let Some(ref blocked) = token_match.not_before
                        && last_idx + 1 < tokens.len()
                        && blocked
                            .iter()
                            .any(|b| b == &tokens[last_idx + 1].text.to_lowercase())
                    {
                        continue;
                    }
                    if let Some(ref blocked) = token_match.not_after
                        && i > 0
                        && blocked
                            .iter()
                            .any(|b| b == &tokens[i - 1].text.to_lowercase())
                    {
                        continue;
                    }
                    if let Some(ref required) = token_match.requires_after {
                        let ok = last_idx + 1 < tokens.len()
                            && required
                                .iter()
                                .any(|r| r == &tokens[last_idx + 1].text.to_lowercase());
                        if !ok {
                            continue;
                        }
                    }

                    // ── Primary match ───────────────────────────────
                    matches.push(
                        MatchSpan::new(win_start, win_end, property, token_match.value)
                            .with_priority(priority),
                    );
                    matched_ranges.push((win_start, win_end));

                    // ── Side effects ────────────────────────────────
                    for se in &token_match.side_effects {
                        if let Some(se_prop) = Property::from_name(&se.property) {
                            matches.push(
                                MatchSpan::new(win_start, win_end, se_prop, &se.value)
                                    .with_priority(priority),
                            );
                        }
                    }
                }
            }
        }
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
            // No technical tokens. Prune language when there's substantial
            // unmatched content that looks like a title (e.g., "Italian"
            // in "The_Italian_Job.mkv" is a title word, not a language).
            // Keep language when it's the only content or nearly so.
            let lang_matches: Vec<&MatchSpan> = matches
                .iter()
                .filter(|m| m.start >= fn_start && m.property == Property::Language)
                .collect();

            if !lang_matches.is_empty() {
                // Count unmatched bytes in the filename (content not covered
                // by any match). Many unmatched bytes = title words present.
                let fn_end = input.len();
                let matched_bytes: usize = matches
                    .iter()
                    .filter(|m| m.start >= fn_start)
                    .map(|m| m.end.saturating_sub(m.start))
                    .sum();
                let unmatched = (fn_end - fn_start).saturating_sub(matched_bytes);

                // If unmatched content is longer than the language match,
                // the language is likely a title word.
                let lang_bytes: usize = lang_matches
                    .iter()
                    .map(|m| m.end.saturating_sub(m.start))
                    .sum();
                if unmatched > lang_bytes {
                    matches.retain(|m| !(m.property == Property::Language && m.start >= fn_start));
                }
            }
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

        // ── Rule 5: Other overlapping ReleaseGroup → drop ambiguous Other ───
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

        // ── Rule 6: Language/SubtitleLanguage contained within a tech span ───
        //
        // Replaces the fancy_regex lookbehind (?<!WEB[-. ]) that used to
        // prevent "DL" from matching inside "WEB-DL". With token-based
        // matching, "WEB-DL" becomes a Source span and "DL" within it
        // is a nested language span — drop the inner language span.
        //
        // "Contained" = language span start AND end both fall within the
        // tech span (not just overlapping).
        let tech_spans: Vec<(usize, usize)> = matches
            .iter()
            .filter(|m| {
                matches!(
                    m.property,
                    Property::Source
                        | Property::VideoCodec
                        | Property::AudioCodec
                        | Property::ScreenSize
                        | Property::StreamingService
                )
            })
            .map(|m| (m.start, m.end))
            .collect();

        if !tech_spans.is_empty() {
            matches.retain(|m| {
                if !matches!(m.property, Property::Language | Property::SubtitleLanguage) {
                    return true;
                }
                // Drop if the language span is fully contained within any tech span.
                !tech_spans
                    .iter()
                    .any(|(ts, te)| m.start >= *ts && m.end <= *te)
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
