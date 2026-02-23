//! Pipeline: orchestrates matchers → conflict resolution → post-processing → Guess.

use crate::guess::Guess;
use crate::matcher::engine::MatchEngine;
use crate::matcher::span::MatchSpan;
use crate::options::Options;
use crate::properties::PropertyMatcher;
use crate::properties::audio_codec::AudioCodecMatcher;
use crate::properties::container::ContainerMatcher;
use crate::properties::edition::EditionMatcher;
use crate::properties::episodes::EpisodeMatcher;
use crate::properties::other::OtherMatcher;
use crate::properties::release_group::ReleaseGroupMatcher;
use crate::properties::screen_size::ScreenSizeMatcher;
use crate::properties::source::SourceMatcher;
use crate::properties::title;
use crate::properties::video_codec::VideoCodecMatcher;
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
            Box::new(SourceMatcher),
            Box::new(ScreenSizeMatcher),
            Box::new(YearMatcher),
            Box::new(EpisodeMatcher),
            Box::new(EditionMatcher),
            Box::new(OtherMatcher),
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

        // Step 3: Post-processing.
        // 3a: Extract title from remaining gaps.
        if let Some(title_match) = title::extract_title(input, &all_matches) {
            all_matches.push(title_match);
        }

        // 3b: Infer media type.
        if let Some(type_match) = title::infer_media_type(&all_matches) {
            all_matches.push(type_match);
        }

        // Step 4: Build the Guess result.
        Guess::from_matches(&all_matches)
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
        assert!(guess.other().contains(&"HDR"));
        assert!(guess.other().contains(&"Remux"));
    }
}
