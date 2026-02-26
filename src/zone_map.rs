//! Zone map: structural analysis of filename zones.
//!
//! Identifies zones *before* full matching runs, inverting the v0.2
//! "match-then-prune" flow. See ARCHITECTURE.md D006 for rationale.
//!
//! # Two-phase anchor detection
//!
//! Phase 1: Find Tier 1+2 anchors (structural markers + tech vocabulary)
//!          → establishes `tech_zone_start`
//! Phase 2: Disambiguate Tier 3 tokens (year candidates) using that boundary
//!          → refines zone boundaries

use std::ops::Range;
use std::sync::LazyLock;

use crate::tokenizer::TokenStream;

/// Structural zone map for a single filename segment.
///
/// Byte ranges are absolute offsets into the original input string.
#[derive(Debug, Clone)]
pub struct ZoneMap {
    /// The title zone: tokens before the first anchor.
    /// Ambiguous matchers (Other, Edition, Language) should be
    /// suppressed in this zone.
    pub title_zone: Range<usize>,

    /// The tech zone: tokens from the first anchor to the release group.
    /// All matchers are active here.
    pub tech_zone: Range<usize>,

    /// The year value (if disambiguated). `None` if no year found.
    /// When two year-like numbers exist, the first may be title content.
    pub year: Option<YearInfo>,
}

/// Disambiguated year information.
#[derive(Debug, Clone)]
pub struct YearInfo {
    /// The actual release year value.
    pub value: u32,
    /// Byte offset start of the year in the input.
    pub start: usize,
    /// Byte offset end of the year in the input.
    pub end: usize,
    /// Year candidates that were classified as title content (not metadata).
    pub title_years: Vec<TitleYear>,
}

/// A year-like number that was classified as title content.
#[derive(Debug, Clone)]
pub struct TitleYear {
    pub value: u32,
    pub start: usize,
    pub end: usize,
}

// ── Tier 1: Structural anchors (always unambiguous) ─────────────────────

// These regexes use `[^a-zA-Z0-9]` boundaries instead of lookbehind
// (the `regex` crate doesn't support lookaround). The match includes
// the boundary char, so we offset +1 when extracting position.

static SXXEXX_ANCHOR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)(?:^|[^a-zA-Z0-9])S\d{1,3}[. ]?E\d{1,4}").unwrap());

static NXN_ANCHOR: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?:^|[^a-zA-Z0-9])\d{1,2}[xX]\d{1,4}(?:$|[^a-zA-Z0-9])").unwrap()
});

static SUFFIXED_RESOLUTION: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)(?:^|[^a-zA-Z0-9])(?:480|576|720|1080|1440|2160|4320)[pi](?:$|[^a-zA-Z0-9])",
    )
    .unwrap()
});

// ── Tier 2: Unambiguous tech vocabulary ──────────────────────────────────

/// Tokens that are virtually never title words.
/// Kept small and precise — false positives here would break title extraction.
const TIER2_TOKENS: &[&str] = &[
    // Video codecs
    "x264", "x265", "h264", "h265", "hevc", "xvid", "divx", "av1", "avc",
    // Audio codecs
    "aac", "ac3", "dts", "flac", "opus", "truehd", "atmos", "eac3", "pcm", // Sources
    "bluray", "bdrip", "brrip", "dvdrip", "webrip", "hdrip", "hdtv", "pdtv", "sdtv", "dsr",
    "dvdscr", "hddvd",
    // Compound sources (matched as multi-token windows by TOML, but
    // single tokens when hyphenated: WEB-DL, WEB-Rip)
    "web-dl", "web-rip", // Other unambiguous tech
    "remux", "repack", "proper",
];

/// Check if a token text (lowercase) is a Tier 2 tech token.
fn is_tier2_token(text: &str) -> bool {
    let lower = text.to_lowercase();
    TIER2_TOKENS.contains(&lower.as_str())
}

// ── Tier 3: Year candidates ─────────────────────────────────────────────

static YEAR_CANDIDATE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?P<year>(?:19|20)\d{2})").unwrap());

/// Check that a year candidate has non-digit boundaries.
fn year_has_boundaries(input: &[u8], start: usize, end: usize) -> bool {
    let left_ok = start == 0 || !input[start - 1].is_ascii_digit();
    let right_ok = end >= input.len() || !input[end].is_ascii_digit();
    left_ok && right_ok
}

static PAREN_YEAR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\((?P<year>(?:19|20)\d{2})\)").unwrap());

/// A raw year candidate before disambiguation.
#[derive(Debug, Clone)]
struct YearCandidate {
    value: u32,
    start: usize,
    end: usize,
    parenthesized: bool,
}

// ── Zone map construction ───────────────────────────────────────────────

/// Build a zone map for the filename portion of the input.
///
/// This is the core of the two-phase anchor detection algorithm.
pub fn build_zone_map(input: &str, token_stream: &TokenStream) -> ZoneMap {
    let fn_start = token_stream.filename_start;
    let fn_end = input.len();
    let filename = &input[fn_start..];

    // ── Phase 1: Find tech_zone_start from Tier 1 + Tier 2 anchors ──

    let mut tech_zone_start = fn_end; // default: no tech zone found

    // Tier 1: structural anchors (regex on filename).
    // The regex includes a boundary char prefix (`[^a-zA-Z0-9]` or `^`),
    // so the actual anchor starts 1 byte after match start (unless at pos 0).
    for re in [&*SXXEXX_ANCHOR, &*NXN_ANCHOR, &*SUFFIXED_RESOLUTION] {
        if let Some(m) = re.find(filename) {
            // Skip the boundary char if match doesn't start at beginning.
            let offset = if m.start() == 0 { 0 } else { 1 };
            let abs_pos = fn_start + m.start() + offset;
            if abs_pos < tech_zone_start {
                tech_zone_start = abs_pos;
            }
        }
    }

    // Tier 2: unambiguous tech vocabulary (token scan)
    for segment in &token_stream.segments {
        for token in &segment.tokens {
            if token.start < fn_start {
                continue;
            }
            if token.start >= tech_zone_start {
                break; // already found something earlier
            }
            if is_tier2_token(&token.text) {
                tech_zone_start = token.start;
            }
            // Multi-token: check 2-token compounds (e.g., "WEB" + "DL")
            // These are handled by the fact that TOML rules match them,
            // but for zone detection we check adjacent tokens.
        }
    }

    // ── Phase 2: Disambiguate year candidates ────────────────────────

    let year_info = disambiguate_years(input, fn_start, tech_zone_start);

    // Refine tech_zone_start: if the actual year is before the current
    // tech_zone_start, it becomes the new boundary.
    if let Some(ref yi) = year_info
        && yi.start < tech_zone_start
    {
        tech_zone_start = yi.start;
    }

    ZoneMap {
        title_zone: fn_start..tech_zone_start,
        tech_zone: tech_zone_start..fn_end,
        year: year_info,
    }
}

/// Disambiguate year-like numbers in the filename.
///
/// Uses `tech_zone_start` (derived from Tier 1+2 anchors) to determine
/// which year candidates are title content vs actual release years.
fn disambiguate_years(input: &str, fn_start: usize, _tech_zone_start: usize) -> Option<YearInfo> {
    let filename = &input[fn_start..];

    // Collect all year candidates.
    let mut candidates: Vec<YearCandidate> = Vec::new();

    // Parenthesized years first (highest confidence).
    for cap in PAREN_YEAR.captures_iter(filename) {
        let year_match = cap.name("year").unwrap();
        let value: u32 = year_match.as_str().parse().unwrap_or(0);
        let full = cap.get(0).unwrap();
        candidates.push(YearCandidate {
            value,
            start: fn_start + full.start(),
            end: fn_start + full.end(),
            parenthesized: true,
        });
    }

    // Bare year candidates (boundary-validated).
    let bytes = input.as_bytes();
    for cap in YEAR_CANDIDATE.captures_iter(filename) {
        let year_match = cap.name("year").unwrap();
        let value: u32 = year_match.as_str().parse().unwrap_or(0);
        let abs_start = fn_start + year_match.start();
        let abs_end = fn_start + year_match.end();

        // Validate non-digit boundaries.
        if !year_has_boundaries(bytes, abs_start, abs_end) {
            continue;
        }

        // Skip if already covered by a parenthesized candidate.
        if candidates
            .iter()
            .any(|c| c.parenthesized && abs_start >= c.start && abs_end <= c.end)
        {
            continue;
        }

        // Skip codec-like numbers.
        if value == 264 || value == 265 {
            continue;
        }

        candidates.push(YearCandidate {
            value,
            start: abs_start,
            end: abs_end,
            parenthesized: false,
        });
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort by position.
    candidates.sort_by_key(|c| c.start);

    // If only one candidate: it's the year.
    if candidates.len() == 1 {
        let c = &candidates[0];
        return Some(YearInfo {
            value: c.value,
            start: c.start,
            end: c.end,
            title_years: vec![],
        });
    }

    // Multiple candidates: classify each.
    //
    // Strategy:
    // - Parenthesized → always year (pick last parenthesized).
    // - Last candidate at or after tech_zone_start → year.
    // - Last candidate before tech_zone_start → year.
    // - Earlier candidates before tech_zone_start → title content.

    // Prefer parenthesized candidates.
    let paren_candidates: Vec<&YearCandidate> =
        candidates.iter().filter(|c| c.parenthesized).collect();
    if let Some(paren) = paren_candidates.last() {
        let title_years: Vec<TitleYear> = candidates
            .iter()
            .filter(|c| c.start != paren.start)
            .map(|c| TitleYear {
                value: c.value,
                start: c.start,
                end: c.end,
            })
            .collect();
        return Some(YearInfo {
            value: paren.value,
            start: paren.start,
            end: paren.end,
            title_years,
        });
    }

    // No parenthesized: last candidate is the year, earlier ones are title.
    let last = candidates.last().unwrap();
    let title_years: Vec<TitleYear> = candidates[..candidates.len() - 1]
        .iter()
        .map(|c| TitleYear {
            value: c.value,
            start: c.start,
            end: c.end,
        })
        .collect();

    Some(YearInfo {
        value: last.value,
        start: last.start,
        end: last.end,
        title_years,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;

    fn zones(input: &str) -> ZoneMap {
        let ts = tokenizer::tokenize(input);
        build_zone_map(input, &ts)
    }

    #[test]
    fn test_basic_movie() {
        let zm = zones("The.Matrix.1999.1080p.BluRay.x264-GROUP.mkv");
        assert!(zm.year.is_some());
        assert_eq!(zm.year.as_ref().unwrap().value, 1999);
        assert!(zm.year.as_ref().unwrap().title_years.is_empty());
        // Title zone should end at year (1999)
        assert!(zm.title_zone.end <= 15); // "The.Matrix." is ~11 chars
    }

    #[test]
    fn test_year_as_title_2001() {
        let zm = zones("2001.A.Space.Odyssey.1968.HDDVD.1080p.DTS.x264.mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 1968);
        assert_eq!(yi.title_years.len(), 1);
        assert_eq!(yi.title_years[0].value, 2001);
    }

    #[test]
    fn test_year_as_title_2012() {
        let zm = zones("2012.2009.720p.BluRay.x264.DTS.mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 2009);
        assert_eq!(yi.title_years.len(), 1);
        assert_eq!(yi.title_years[0].value, 2012);
    }

    #[test]
    fn test_year_as_title_1917() {
        let zm = zones("1917.2019.1080p.BluRay.x264-GROUP.mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 2019);
        assert_eq!(yi.title_years.len(), 1);
        assert_eq!(yi.title_years[0].value, 1917);
    }

    #[test]
    fn test_year_as_title_1922() {
        let zm = zones("1922.2017.WEB-DL.x264.mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 2017);
        assert_eq!(yi.title_years.len(), 1);
        assert_eq!(yi.title_years[0].value, 1922);
    }

    #[test]
    fn test_parenthesized_year() {
        let zm = zones("Movie (2019).mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 2019);
        assert!(yi.title_years.is_empty());
    }

    #[test]
    fn test_episode_anchor() {
        let zm = zones("Show.Name.S01E02.720p.HDTV.x264-GROUP.mkv");
        // tech_zone_start should be at S01E02
        assert!(zm.title_zone.end <= 11); // "Show.Name." is ~10 chars
        assert!(zm.year.is_none());
    }

    #[test]
    fn test_no_tech_tokens() {
        let zm = zones("Just A Simple Title.mkv");
        // No anchors found → title zone is entire filename
        assert_eq!(zm.title_zone.start, 0);
        assert_eq!(zm.title_zone.end, zm.tech_zone.start);
    }

    #[test]
    fn test_single_year_no_tech() {
        let zm = zones("Movie.2019.mkv");
        let yi = zm.year.as_ref().unwrap();
        assert_eq!(yi.value, 2019);
        assert!(yi.title_years.is_empty());
    }
}
