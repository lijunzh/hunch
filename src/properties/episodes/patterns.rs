//! Regex pattern definitions for season/episode detection.

use crate::matcher::regex_utils::BoundedRegex;
use std::sync::LazyLock;

pub(super) type Regex = BoundedRegex;

// ── SxxExx patterns ──

/// S01E02, S01E02E03, S01E02-E05, S01E02-05, S01E02+E03, S01.E02.E03.
pub(super) static SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[. ]?E(?:P)?(?P<ep_start>\d{1,4})(?P<ep_rest>(?:(?:[-+]E?|[. ]E|E)\d{1,4})+)?(?![a-z0-9])",
    )
});

/// S03-E01 (dash between S and E).
pub(super) static SXX_DASH_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+E(?P<episode>\d{1,4})(?![a-z0-9])")
});

/// S01E01-S01E21 full range.
pub(super) static SXXEXX_TO_SXXEXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})E(?P<e1>\d{1,4})[-]S(?P<s2>\d{1,3})E(?P<e2>\d{1,4})(?![a-z0-9])",
    )
});

/// S06xE01 (x separator).
pub(super) static SXX_X_EXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[xX]E(?P<episode>\d{1,4})(?![a-z0-9])")
});

/// S03-X01 for bonus/extras.
pub(super) static SXX_DASH_XXX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})[-. ]+[xX](?P<episode>\d{1,4})(?![a-z0-9])")
});

// ── NxN patterns ──

/// NxN format: 1x03, 5x9, 5x44x45, 4x05-06.
pub(super) static NXN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])(?P<season>\d{1,2})[xX](?P<ep_start>\d{1,4})(?:[-xX](?P<ep2>\d{1,4}))*(?![a-z0-9])",
    )
});

// ── Standalone episode patterns ──

/// E01, Ep01, E02-03, E02-E03, etc.
pub(super) static EP_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z0-9])(?:E|Ep\.?)\s*(?P<ep_start>\d{1,4})(?P<ep_rest>(?:(?:[-+]E?|E)\d{1,4})+)?(?![a-z0-9])",
    )
});

/// Episode 1, Episode.01.
pub(super) static EPISODE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])Episodes?\s*\.?\s*(?P<episode>\d{1,4})(?:\s*[-~]\s*(?P<ep_end>\d{1,4}))?(?![a-z0-9])",
    )
});

/// Versioned episode: `07v4`, `312v1`.
pub(super) static VERSIONED_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])(?P<episode>\d{1,4})v\d{1,2}(?![a-z0-9])"));

/// Leading episode number: `01 - Ep Name`, `003. Show Name`.
pub(super) static LEADING_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?P<episode>0\d{1,3}|\d{1,3})(?:\s*[-.]\s+[A-Za-z])"));

/// Anime episode: `- 01`, `- 001`.
pub(super) static ANIME_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<![a-z0-9])[-]\s+(?P<episode>\d{1,4})(?:\s|[.]|$)"));

/// Bare episode after dots: `Show.05.Title`.
pub(super) static BARE_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\.(?P<episode>0\d|\d{2})\.(?![0-9])"));

// ── Season patterns ──

pub(super) static SEASON_ONLY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione|Tem\.?)\s*\.?\s*(?P<season>\d{1,2})(?![a-z0-9])",
    )
});

pub(super) static SEASON_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<season>(?:X{0,3})(?:IX|IV|V?I{0,3}))(?![a-z])",
    )
});

pub(super) static SEASON_DIR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<season>\d{1,2})(?:[/\\])")
});

/// S01-only without episode.
pub(super) static S_ONLY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?<![a-z0-9])S(?P<season>\d{1,3})"));

pub(super) static S_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})[-]S(?P<s2>\d{1,3})(?![a-z0-9])")
});

pub(super) static SEASON_MULTI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<seasons>\d{1,2}(?:\s*[-&.,]\s*\d{1,2})+)(?![a-z0-9])",
    )
});

pub(super) static SEASON_MULTI_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<prefix>\d{1,2}(?:[. ]\d{1,2})*)\s*[. ]?\s*(?:~|to)\s*\.?\s*(?P<end>\d{1,2})(?![a-z0-9])",
    )
});

pub(super) static SEASON_RANGE_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<s1>\d{1,2})\s*\.?\s*(?:to|~|a|\.\.)\s*\.?\s*(?P<s2>\d{1,2})(?![a-z0-9])",
    )
});

pub(super) static S_CONCAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<first>\d{1,3})(?:S(?P<rest>\d{1,3}))+(?![a-z0-9])")
});

pub(super) static S_MULTI_NUM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<seasons>\d{2,3}(?:[-. ]\d{2,3})+)(?![a-z0-9])")
});

pub(super) static S_TO_S: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?<![a-z0-9])S(?P<s1>\d{1,3})\.?(?:to|\.to\.)\.?S(?P<s2>\d{1,3})(?![a-z0-9])")
});

pub(super) static SEASON_LIST_AND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])(?:Season|Saison|Temporada|Stagione)\s*\.?\s*(?P<nums>\d{1,2}(?:[. ]\d{1,2})*)[. ](?:and|&)\s*(?P<last>\d{1,2})(?![a-z0-9])",
    )
});

// ── Spanish Cap patterns ──

pub(super) static CAP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?<![a-z])Cap\.?\s*(?P<num1>\d{3,4})(?:[_](?P<num2>\d{3,4}))?(?:\.[A-Za-z]|[\]\[]|$)",
    )
});

// ── Digit decomposition ──

pub(super) static THREE_DIGIT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[.\-_ ](?P<num>\d{3,4})").unwrap());
