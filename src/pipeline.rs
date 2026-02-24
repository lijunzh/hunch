//! Pipeline: orchestrates matchers → conflict resolution → post-processing → HunchResult.

use crate::hunch_result::HunchResult;
use crate::matcher::engine;
use crate::matcher::span::{MatchSpan, Property};
use crate::options::Options;

use std::sync::LazyLock;

static REAL_RE: LazyLock<fancy_regex::Regex> =
    LazyLock::new(|| fancy_regex::Regex::new(r"(?i)^REAL$").unwrap());

static REPACK_RE: LazyLock<fancy_regex::Regex> =
    LazyLock::new(|| fancy_regex::Regex::new(r"(?i)^(?:REPACK|RERIP)(\d+)?$").unwrap());

use crate::properties::title;
use crate::properties::{
    aspect_ratio, audio_codec, audio_profile, bonus, color_depth, container, country, crc32, date,
    edition, episode_count, episode_details, episodes, frame_rate, language, other, part,
    release_group, screen_size, size, source, streaming_service, subtitle_language, uuid, version,
    video_codec, video_profile, website, year,
};

/// A matcher function: takes input, returns property matches.
type MatcherFn = fn(&str) -> Vec<MatchSpan>;

/// The parsing pipeline.
pub struct Pipeline {
    #[allow(dead_code)]
    options: Options,
    matchers: Vec<MatcherFn>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new(Options::default())
    }
}

impl Pipeline {
    pub fn new(options: Options) -> Self {
        let matchers: Vec<MatcherFn> = vec![
            container::find_matches,
            video_codec::find_matches,
            audio_codec::find_matches,
            audio_profile::find_matches,
            video_profile::find_matches,
            color_depth::find_matches,
            source::find_matches,
            screen_size::find_matches,
            aspect_ratio::find_matches,
            year::find_matches,
            date::find_matches,
            episodes::find_matches,
            episode_details::find_matches,
            episode_count::find_matches,
            edition::find_matches,
            other::find_matches,
            language::find_matches,
            subtitle_language::find_matches,
            country::find_matches,
            streaming_service::find_matches,
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
        Self { options, matchers }
    }

    /// Run the full pipeline on an input string.
    pub fn run(&self, input: &str) -> HunchResult {
        // Step 1: Collect all matches from all matchers.
        let mut all_matches: Vec<MatchSpan> = self
            .matchers
            .iter()
            .flat_map(|matcher| matcher(input))
            .collect();

        // Step 2: Resolve overlapping conflicts.
        engine::resolve_conflicts(&mut all_matches);

        // Step 2b: Remove language matches that appear in the title zone.
        // The "title zone" is the region before the first technical property.
        // Language words like "Italian" in "The.Italian.Job" should not be treated
        // as language tags if they appear before any codec/year/source/resolution.
        self.prune_language_in_title_zone(input, &mut all_matches);

        // Step 2c: Prune duplicate source matches in the title zone.
        // e.g., "The.Girl.in.the.Spiders.Web.2019.WEB-DL" — "Web" before year
        // is a title word, not a source, because "WEB-DL" after year is the real source.
        self.prune_early_source_duplicates(input, &mut all_matches);

        // Step 3: Post-processing.
        // 3a: Extract title from remaining gaps.
        if let Some(title_match) = title::extract_title(input, &all_matches) {
            all_matches.push(title_match);
        }

        // 3b: Extract episode title (text between episode marker and next property).
        if let Some(ep_title) = title::extract_episode_title(input, &all_matches) {
            all_matches.push(ep_title);
        }

        // 3c: Infer media type.
        let media_type = title::infer_media_type(&all_matches);

        // 3d: Compute proper_count from Other:Proper matches in the filename.
        let proper_count = compute_proper_count(input, &all_matches);

        // Step 4: Build the HunchResult from real matches, then set computed values.
        let mut result = HunchResult::from_matches(&all_matches);
        result.set(Property::MediaType, media_type);
        if proper_count > 0 {
            result.set(Property::ProperCount, proper_count.to_string());
        }
        result
    }

    /// Remove language matches that appear before any "technical" property.
    /// This prevents language names (French, Italian, English, etc.) from
    /// eating title words like "The Italian Job" or "Immersion French".
    fn prune_language_in_title_zone(&self, input: &str, matches: &mut Vec<MatchSpan>) {
        // Find the filename portion start.
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

        // Find the start position of the first technical match.
        let technical_props = [
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
            .filter(|m| m.start >= fn_start && technical_props.contains(&m.property))
            .map(|m| m.start)
            .min();

        if let Some(tech_pos) = first_tech_pos {
            // Remove language matches that appear before the first technical token,
            // but only if they appear in the first half of the pre-tech zone
            // (likely part of the title rather than metadata).
            let title_zone_end = fn_start + (tech_pos - fn_start) / 2;
            matches.retain(|m| {
                if m.property == Property::Language
                    && m.start < title_zone_end
                    && m.start >= fn_start
                {
                    false // prune it — likely a title word
                } else {
                    true
                }
            });
        } else {
            // No technical tokens at all — prune all language matches.
            matches.retain(|m| m.property != Property::Language);
        }
    }

    /// Remove source matches that appear before a year/season/episode when
    /// there's a later source match. This prevents short source keywords
    /// like "Web" from eating title words.
    fn prune_early_source_duplicates(&self, input: &str, matches: &mut Vec<MatchSpan>) {
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

        // Find the first year/season/episode position.
        let anchor_pos = matches
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

        let Some(anchor) = anchor_pos else {
            return;
        };

        // Check if there are source matches both before and after the anchor.
        let has_early_source = matches
            .iter()
            .any(|m| m.property == Property::Source && m.start < anchor && m.start >= fn_start);
        let has_late_source = matches
            .iter()
            .any(|m| m.property == Property::Source && m.start >= anchor);

        if has_early_source && has_late_source {
            matches.retain(|m| {
                !(m.property == Property::Source && m.start < anchor && m.start >= fn_start)
            });
        }
    }
}

/// Compute the proper count from PROPER/REPACK/REAL matches in the filename.
///
/// Rules:
/// - REAL replaces PROPER (counts as 2)
/// - REPACK/RERIP adds 1 (or the trailing digit, e.g., REPACK5 → 5)
/// - Each PROPER keyword adds 1
fn compute_proper_count(input: &str, matches: &[MatchSpan]) -> u32 {
    let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let mut has_real = false;
    let mut proper_count_raw: u32 = 0;
    let mut repack_count: u32 = 0;

    for m in matches
        .iter()
        .filter(|m| m.property == Property::Other && m.value == "Proper" && m.start >= fn_start)
    {
        let raw = &input[m.start..m.end];
        if REAL_RE.is_match(raw).unwrap_or(false) {
            has_real = true;
            continue;
        }
        if let Ok(Some(caps)) = REPACK_RE.captures(raw) {
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
}
