//! Named priority constants for match conflict resolution.
//!
//! Higher values win when two matches of the same property overlap.
//! These constants replace magic integer literals throughout the
//! codebase to make the priority system self-documenting (P1).
//!
//! ## Tier overview
//!
//! ```text
//! EXTENSION   (10)  File extension → container (always wins)
//! STRUCTURAL   (5)  Unambiguous structural markers (SxxExx)
//! PATTERN      (3)  Compound patterns (NxN, Cap.NNN, CJK brackets)
//! KEYWORD      (2)  Keyword-driven matches ("Episode 1", full dates, CRC32)
//! VOCABULARY   (1)  Word-based lookups ("Season 1", Part, size, anime ep)
//! DEFAULT      (0)  Standard TOML exact/pattern matches
//! HEURISTIC   (-1)  Single-file guesses (bare year, bare episode)
//! POSITIONAL  (-2)  Position-dependent fallbacks (release_group, country)
//! DIR_PENALTY (-5)  Applied to directory segment matches
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::priority;
//!
//! MatchSpan::new(start, end, Property::Episode, value)
//!     .with_priority(priority::STRUCTURAL);
//! ```
//!
//! For fine-grained ordering within a tier, use arithmetic:
//! `priority::STRUCTURAL - 1` for slightly-less-confident structural.

/// File extension → container. Always wins over in-filename matches.
pub const EXTENSION: i32 = 10;

/// Unambiguous structural markers: SxxExx, parenthesized year, S01E01-S01E21.
pub const STRUCTURAL: i32 = 5;

/// Compound patterns: NxN compact notation, Cap.NNN, CJK bracket episodes.
pub const PATTERN: i32 = 3;

/// Keyword-driven matches: "Episode 1", full dates (YYYY-MM-DD), CRC32,
/// UUID, subtitle language with explicit markers ("Sub French").
pub const KEYWORD: i32 = 2;

/// Word-based lookups: "Season 1", "Part 2", size, anime-style episodes,
/// versioned episodes, bit_rate.
pub const VOCABULARY: i32 = 1;

/// Standard TOML exact/pattern matches, bonus, language codes, leading
/// episodes, digit decomposition. The baseline.
pub const DEFAULT: i32 = 0;

/// Single-file heuristic guesses: bare year numbers, bare episodes,
/// primary release_group candidates, aspect ratio.
pub const HEURISTIC: i32 = -1;

/// Position-dependent fallbacks: release_group secondary candidates,
/// country codes, video_profile, other_positional, episode_details.
pub const POSITIONAL: i32 = -2;

/// Priority penalty applied to matches from directory path segments.
/// Added to a rule's base priority so filename matches always win
/// over directory matches for the same property.
pub const DIR_PENALTY: i32 = -5;
