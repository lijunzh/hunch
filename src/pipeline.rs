//! Pipeline: orchestrates matchers → conflict resolution → post-processing → Guess.

use crate::guess::Guess;
use crate::matcher::engine::MatchEngine;
use crate::matcher::span::{MatchSpan, Property};
use crate::options::Options;
use crate::properties::PropertyMatcher;
use crate::properties::aspect_ratio::AspectRatioMatcher;
use crate::properties::audio_codec::AudioCodecMatcher;
use crate::properties::audio_profile::AudioProfileMatcher;
use crate::properties::bonus::BonusMatcher;
use crate::properties::color_depth::ColorDepthMatcher;
use crate::properties::container::ContainerMatcher;
use crate::properties::country::CountryMatcher;
use crate::properties::crc32::Crc32Matcher;
use crate::properties::date::DateMatcher;
use crate::properties::edition::EditionMatcher;
use crate::properties::episode_details::EpisodeDetailsMatcher;
use crate::properties::episodes::EpisodeMatcher;
use crate::properties::language::LanguageMatcher;
use crate::properties::other::OtherMatcher;
use crate::properties::part::PartMatcher;
use crate::properties::release_group::ReleaseGroupMatcher;
use crate::properties::screen_size::ScreenSizeMatcher;
use crate::properties::size::SizeMatcher;
use crate::properties::source::SourceMatcher;
use crate::properties::streaming_service::StreamingServiceMatcher;
use crate::properties::subtitle_language::SubtitleLanguageMatcher;
use crate::properties::title;
use crate::properties::uuid::UuidMatcher;
use crate::properties::video_codec::VideoCodecMatcher;
use crate::properties::video_profile::VideoProfileMatcher;
use crate::properties::website::WebsiteMatcher;
use crate::properties::year::YearMatcher;

/// The parsing pipeline.
pub struct Pipeline {
    #[allow(dead_code)]
    options: Options,
    matchers: Vec<Box<dyn PropertyMatcher>>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new(Options::default())
    }
}

impl Pipeline {
    pub fn new(options: Options) -> Self {
        let matchers: Vec<Box<dyn PropertyMatcher>> = vec![
            Box::new(ContainerMatcher),
            Box::new(VideoCodecMatcher),
            Box::new(AudioCodecMatcher),
            Box::new(AudioProfileMatcher),
            Box::new(VideoProfileMatcher),
            Box::new(ColorDepthMatcher),
            Box::new(SourceMatcher),
            Box::new(ScreenSizeMatcher),
            Box::new(AspectRatioMatcher),
            Box::new(YearMatcher),
            Box::new(DateMatcher),
            Box::new(EpisodeMatcher),
            Box::new(EpisodeDetailsMatcher),
            Box::new(EditionMatcher),
            Box::new(OtherMatcher),
            Box::new(LanguageMatcher),
            Box::new(SubtitleLanguageMatcher),
            Box::new(CountryMatcher),
            Box::new(StreamingServiceMatcher),
            Box::new(Crc32Matcher),
            Box::new(UuidMatcher),
            Box::new(WebsiteMatcher),
            Box::new(SizeMatcher),
            Box::new(PartMatcher),
            Box::new(BonusMatcher),
            Box::new(ReleaseGroupMatcher),
        ];
        Self { options, matchers }
    }

    /// Run the full pipeline on an input string.
    pub fn run(&self, input: &str) -> Guess {
        // Step 1: Collect all matches from all matchers.
        let mut all_matches: Vec<MatchSpan> = self
            .matchers
            .iter()
            .flat_map(|m| m.find_matches(input))
            .collect();

        // Step 2: Resolve overlapping conflicts.
        MatchEngine::resolve_conflicts(&mut all_matches);

        // Step 2b: Remove language matches that appear in the title zone.
        // The "title zone" is the region before the first technical property.
        // Language words like "Italian" in "The.Italian.Job" should not be treated
        // as language tags if they appear before any codec/year/source/resolution.
        self.prune_language_in_title_zone(input, &mut all_matches);

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
        if let Some(type_match) = title::infer_media_type(&all_matches) {
            all_matches.push(type_match);
        }

        // 3d: Compute proper_count from Other:Proper matches in the filename.
        let fn_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let mut proper_count: u32 = 0;
        for m in all_matches
            .iter()
            .filter(|m| m.property == Property::Other && m.value == "Proper" && m.start >= fn_start)
        {
            let raw = &input[m.start..m.end];
            // Check for trailing digit: REPACK5 → 5.
            let re = fancy_regex::Regex::new(r"(?i)(?:REPACK|RERIP)(\d+)$").unwrap();
            if let Ok(Some(caps)) = re.captures(raw)
                && let Some(num) = caps.get(1)
            {
                proper_count += num.as_str().parse::<u32>().unwrap_or(1);
                continue;
            }
            proper_count += 1;
        }
        if proper_count > 0 {
            all_matches.push(MatchSpan::new(
                0,
                0,
                Property::ProperCount,
                proper_count.to_string(),
            ));
        }

        // Step 4: Build the Guess result.
        Guess::from_matches(&all_matches)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_movie_parse() {
        let pipeline = Pipeline::default();
        let guess = pipeline.run("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");

        assert_eq!(guess.title(), Some("The Matrix"));
        assert_eq!(guess.year(), Some(1999));
        assert_eq!(guess.screen_size(), Some("1080p"));
        assert_eq!(guess.source(), Some("Blu-ray"));
        assert_eq!(guess.video_codec(), Some("H.264"));
        assert_eq!(guess.release_group(), Some("GROUP"));
        assert_eq!(guess.container(), Some("mkv"));
    }

    #[test]
    fn test_episode_parse() {
        let pipeline = Pipeline::default();
        let guess = pipeline.run("Breaking.Bad.S05E16.720p.BluRay.x264-DEMAND.mkv");

        assert_eq!(guess.title(), Some("Breaking Bad"));
        assert_eq!(guess.season(), Some(5));
        assert_eq!(guess.episode(), Some(16));
        assert_eq!(guess.screen_size(), Some("720p"));
        assert_eq!(guess.video_codec(), Some("H.264"));
        assert_eq!(guess.release_group(), Some("DEMAND"));
    }

    #[test]
    fn test_minimal_input() {
        let pipeline = Pipeline::default();
        let guess = pipeline.run("Movie.mkv");

        assert_eq!(guess.title(), Some("Movie"));
        assert_eq!(guess.container(), Some("mkv"));
    }

    #[test]
    fn test_4k_hdr() {
        let pipeline = Pipeline::default();
        let guess = pipeline.run("Movie.2024.2160p.UHD.BluRay.Remux.HDR.HEVC.DTS-HD.MA-GROUP.mkv");

        assert_eq!(guess.title(), Some("Movie"));
        assert_eq!(guess.year(), Some(2024));
        assert_eq!(guess.screen_size(), Some("2160p"));
        assert_eq!(guess.video_codec(), Some("H.265"));
        assert!(guess.other().contains(&"HDR10"));
        assert!(guess.other().contains(&"Remux"));
    }
}
