//! Title string cleaning — composed from small, single-purpose transforms.
//!
//! ## Design
//!
//! Title cleaning is a pipeline. Each step does *one* thing and returns a
//! `String`. The public entry points (`clean_title`, `clean_episode_title`,
//! `clean_title_preserve_dashes`) compose these steps.
//!
//! Steps (in pipeline order):
//!
//! 1. [`strip_leading_brackets`] — drop `[XCT]`, `[阿维达]`, ... at the start.
//! 2. [`strip_paren_year`] — drop a trailing `(YYYY)`.
//! 3. [`strip_paren_groups`] — drop all `(...)` groups, with empty fallback.
//! 4. [`normalize_separators`] — convert `.`, `_`, `+`, brackets, `*` to
//!    spaces. Dash handling is parameterized by [`DashPolicy`].
//! 5. [`trim_trailing_punct`] — drop stray `:-,;` at the end.
//! 6. [`strip_trailing_keywords`] — drop trailing `Part N`, `Season N`,
//!    `Episode`, `-xNN` bonus markers (caller opt-in).
//!
//! See `D10: Refactor before accreting` in `DESIGN.md` — this module
//! exists because we hit the "2nd cleaning mode + bool flag" tripwire.

use super::{BRACKETS, SEPS};

use std::sync::LazyLock;

// ── Public composers ───────────────────────────────────────────────────────

/// Standard title cleaning: separators → spaces, brackets stripped,
/// trailing `Part N` / `Season N` / bonus markers removed.
pub(super) fn clean_title(raw: &str) -> String {
    let s = strip_leading_brackets(raw);
    let s = strip_paren_year(&s);
    let s = strip_paren_groups(&s);
    let s = normalize_separators(&s, DashPolicy::WordDashOnly);
    let s = trim_trailing_punct(&s);
    strip_trailing_keywords(&s)
}

/// Episode-title cleaning: same as [`clean_title`] but keeps trailing
/// `Part N` / `Season N` (those words are valid episode-title content)
/// and trims leading separator junk first.
pub(super) fn clean_episode_title(raw: &str) -> String {
    let trimmed = raw.trim_start_matches(['.', '_', ' ', '-']);
    let s = strip_leading_brackets(trimmed);
    let s = strip_paren_year(&s);
    let s = strip_paren_groups(&s);
    let s = normalize_separators(&s, DashPolicy::WordDashOnly);
    trim_trailing_punct(&s)
}

/// Clean a raw title while preserving internal `" - "` (and equivalents
/// `_-_`, `.-.`) as literal `" - "` separators, and without stripping
/// trailing `Part N` keywords.
///
/// Use this when the title boundary has already been correctly identified
/// by upstream logic (e.g., anime bracket releases
/// `[Group] Show - Sub Part 2 - 13 [tags]`) and the dashes / `Part N`
/// are genuinely part of the title.
///
/// Composition: same pipeline as [`clean_title`] but with
/// [`DashPolicy::PreserveStructuralDash`] and no trailing-keyword strip.
pub(super) fn clean_title_preserve_dashes(raw: &str) -> String {
    let s = strip_leading_brackets(raw);
    let s = strip_paren_year(&s);
    let s = strip_paren_groups(&s);
    let s = normalize_separators(&s, DashPolicy::PreserveStructuralDash);
    trim_trailing_punct(&s)
}

// ── DashPolicy ─────────────────────────────────────────────────────────────

/// How [`normalize_separators`] handles `-` characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashPolicy {
    /// Keep `-` only between alphanumerics (e.g. `Spider-Man`); convert
    /// every other dash to a space. This is the standard mode used by
    /// [`clean_title`] and [`clean_episode_title`].
    WordDashOnly,
    /// Keep `-` between alphanumerics, AND preserve a separator-flanked
    /// dash (e.g. `_-_`, ` - `, `.-.`) as a literal ` - ` (space-dash-space)
    /// in the output. Used when the structural separator carries title
    /// content that would otherwise be lost (anime
    /// `[Group] Title - Sub - Ep [tags]` where `Sub` belongs to the title).
    PreserveStructuralDash,
}

// ── Step 1: leading brackets ───────────────────────────────────────────────

/// Strip `[…]` groups at the start (and any trailing separators) until the
/// string no longer begins with `[`.
pub(super) fn strip_leading_brackets(raw: &str) -> String {
    let mut s = raw.to_string();
    while s.starts_with('[') {
        if let Some(end) = s.find(']') {
            s = s[end + 1..].to_string();
            s = s.trim_start_matches(SEPS).to_string();
        } else {
            break;
        }
    }
    s
}

// ── Step 2: trailing parenthesized year ────────────────────────────────────

static RE_PAREN_YEAR: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\s*\((?:19|20)\d{2}\)\s*$").unwrap());

/// Strip a trailing `(YYYY)`: `Movie Name (2005)` → `Movie Name`.
pub(super) fn strip_paren_year(s: &str) -> String {
    if let Some(m) = RE_PAREN_YEAR.find(s) {
        s[..m.start()].to_string()
    } else {
        s.to_string()
    }
}

// ── Step 3: parenthesized groups ───────────────────────────────────────────

static RE_PAREN_GROUP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\s*\([^)]*\)\s*").unwrap());

/// Strip all `(...)` groups (alternative titles, country tags, etc.).
///
/// If stripping would empty the string, the original is returned — a title
/// that is *only* a parenthesized phrase is better than nothing.
pub(super) fn strip_paren_groups(s: &str) -> String {
    let stripped = RE_PAREN_GROUP.replace_all(s, " ").into_owned();
    if stripped.trim().is_empty() {
        s.to_string()
    } else {
        stripped
    }
}

// ── Step 4: separator normalization ────────────────────────────────────────

static RE_DOT_ACRONYM: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?:^|[\s._])([A-Za-z0-9](?:\.[A-Za-z0-9]){2,}\.?)").unwrap()
});

/// Replace separators (`.`, `_`, `+`, brackets, `*`) with spaces, with
/// dash handling controlled by [`DashPolicy`].
///
/// Preserves dot-acronyms like `S.H.I.E.L.D.` by computing protected
/// byte ranges before the per-char rewrite.
pub(super) fn normalize_separators(s: &str, dash: DashPolicy) -> String {
    // Find dot-acronym byte ranges to protect from dot→space conversion.
    let protected_ranges: Vec<(usize, usize)> = RE_DOT_ACRONYM
        .find_iter(s)
        .map(|m| {
            let actual_start =
                if m.start() > 0 && matches!(s.as_bytes()[m.start()], b' ' | b'\t' | b'.' | b'_') {
                    m.start() + 1
                } else {
                    m.start()
                };
            (actual_start, m.end())
        })
        .collect();

    let in_protected =
        |pos: usize| -> bool { protected_ranges.iter().any(|(s, e)| pos >= *s && pos < *e) };

    let chars: Vec<char> = s.chars().collect();
    let mut byte_positions: Vec<usize> = Vec::with_capacity(chars.len());
    let mut byte_pos = 0;
    for &c in &chars {
        byte_positions.push(byte_pos);
        byte_pos += c.len_utf8();
    }

    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '-' => {
                let kind = classify_dash(&chars, i);
                match (kind, dash) {
                    (DashKind::WordDash, _) => out.push('-'),
                    (DashKind::SeparatorFlanked, DashPolicy::PreserveStructuralDash) => {
                        // Collapse any trailing space we just emitted so we
                        // don't get "  - " / " -  ". `collapse_spaces` at the
                        // end will normalize any remaining doubles, but emit
                        // exactly " - " here for clarity.
                        if out.ends_with(' ') {
                            out.pop();
                        }
                        out.push_str(" - ");
                    }
                    _ => out.push(' '),
                }
            }
            '.' if in_protected(byte_positions[i]) => out.push('.'),
            ch if SEPS.contains(&ch) || BRACKETS.contains(&ch) || ch == '*' => out.push(' '),
            ch => out.push(ch),
        }
    }
    collapse_spaces(&out)
}

/// Classification of a `-` character based on the chars immediately
/// surrounding it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashKind {
    /// Both neighbors are alphanumerics. Always preserved as `-`
    /// regardless of policy. Example: `Spider-Man`.
    WordDash,
    /// Both neighbors are filename separators (`.`, `_`, `+`, ` `). This
    /// is the structural " - " form. Preserved by
    /// [`DashPolicy::PreserveStructuralDash`], collapsed otherwise.
    SeparatorFlanked,
    /// Anything else (start/end of string, mixed neighbors, brackets, ...).
    /// Always collapsed to a space.
    Other,
}

fn classify_dash(chars: &[char], i: usize) -> DashKind {
    let prev = if i > 0 { Some(chars[i - 1]) } else { None };
    let next = chars.get(i + 1).copied();
    let is_alnum = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric());
    let is_sep = |c: Option<char>| c.is_some_and(|c| SEPS.contains(&c));
    if is_alnum(prev) && is_alnum(next) {
        DashKind::WordDash
    } else if is_sep(prev) && is_sep(next) {
        DashKind::SeparatorFlanked
    } else {
        DashKind::Other
    }
}

// (legacy `rewrite_dash` removed — logic now lives in `classify_dash`
//  + the per-char match in `normalize_separators`.)

// ── Step 5: trailing punctuation ───────────────────────────────────────────

/// Strip trailing punctuation that leaks from separator boundaries.
pub(super) fn trim_trailing_punct(s: &str) -> String {
    s.trim_end_matches([':', '-', ',', ';']).trim().to_string()
}

// ── Step 6: trailing keywords ──────────────────────────────────────────────

static RE_TRAILING_PART: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\s+Part\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?\s*$").unwrap()
});

static RE_TRAILING_SEASON: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"(?i)\s+(?:Saison|Temporada|Stagione|Tem\.?|Season|Seasons?)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+)?(?:\s*(?:&|and)\s*(?:I{1,4}|IV|VI{0,3}|IX|X{0,3}|[0-9]+))?\s*$"
    ).unwrap()
});

static RE_TRAILING_EP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)\s+(?:Episodes?|Ep\.?)\s*$").unwrap());

static RE_TRAILING_BONUS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)[-]x\d{1,3}\s*$").unwrap());

/// Strip trailing `Part`, `Season`, `Episode` keywords and `-xNN` bonus
/// markers from titles. Each strip is skipped if it would empty the title.
pub(super) fn strip_trailing_keywords(input: &str) -> String {
    let mut s = input.to_string();
    s = strip_if_nonempty(&s, &RE_TRAILING_PART);
    s = strip_if_nonempty(&s, &RE_TRAILING_SEASON);
    s = strip_if_nonempty(&s, &RE_TRAILING_EP);
    s = strip_if_nonempty(&s, &RE_TRAILING_BONUS);
    s
}

fn strip_if_nonempty(s: &str, re: &regex::Regex) -> String {
    if let Some(m) = re.find(s) {
        let stripped = &s[..m.start()];
        if !stripped.trim().is_empty() {
            return stripped.to_string();
        }
    }
    s.to_string()
}

// ── Whitespace utilities (used by other modules in the title subsystem) ───

/// Collapse multiple spaces into one and trim.
pub(super) fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = true;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(' ');
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

/// Strip a file extension from the end of a string.
pub(super) fn strip_extension(s: &str) -> &str {
    if let Some(dot) = s.rfind('.') {
        let ext = &s[dot + 1..];
        let ext_lower = ext.to_lowercase();
        if ext.len() <= 5 && is_likely_extension(&ext_lower) {
            return &s[..dot];
        }
    }
    s
}

/// Check if a string looks like a real file extension.
pub(super) fn is_likely_extension(ext: &str) -> bool {
    matches!(
        ext,
        "mkv"
            | "mp4"
            | "avi"
            | "wmv"
            | "flv"
            | "mov"
            | "webm"
            | "ogm"
            | "ogv"
            | "ts"
            | "m2ts"
            | "m4v"
            | "mpg"
            | "mpeg"
            | "vob"
            | "divx"
            | "3gp"
            | "srt"
            | "sub"
            | "ssa"
            | "ass"
            | "idx"
            | "sup"
            | "vtt"
            | "nfo"
            | "txt"
            | "jpg"
            | "jpeg"
            | "png"
            | "nzb"
            | "par"
            | "par2"
            | "iso"
            | "img"
            | "rar"
            | "zip"
            | "7z"
    )
}

/// Detect if a title looks like a scene abbreviation.
pub(super) fn is_abbreviated(title: &str) -> bool {
    let segments: Vec<&str> = title
        .split(|c: char| c.is_whitespace() || c == '-')
        .collect();
    segments.iter().all(|w| {
        w.len() <= 6
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    }) && title.len() <= 20
}

/// Pick the string with better casing when two titles match case-insensitively.
pub(super) fn pick_better_casing<'a>(a: &'a str, b: &'a str) -> &'a str {
    fn casing_score(s: &str) -> i32 {
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
        {
            return -10;
        }
        if s.chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase())
        {
            return -5;
        }
        s.split_whitespace()
            .filter(|w| w.starts_with(|c: char| c.is_uppercase()))
            .count() as i32
    }
    if casing_score(a) >= casing_score(b) {
        a
    } else {
        b
    }
}

/// Check if a directory name is generic (should be skipped for title).
///
/// Generic directories are structural (e.g., "Season 1", "Extras") or
/// organizational (e.g., "Movies", "Downloads"). When walking parent
/// directories for title fallback, these are skipped so the real title
/// directory (e.g., "Transformers 1984") is found.
pub(crate) fn is_generic_dir(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Exact matches (case-insensitive).
    if matches!(
        lower.as_str(),
        // Library root / organizational
        "movies"
            | "movie"
            | "films"
            | "film"
            | "series"
            | "tv shows"
            | "tvshows"
            | "tv"
            | "media"
            | "video"
            | "videos"
            | "anime"
            | "donghua"
            | "kids"
            | "cartoons"
            | "shows"
            | "documentary"
            | "documentaries"
            | "music"
            | "concert"
            | "concerts"
            // Language categories (library organization, not show titles)
            | "chinese"
            | "english"
            | "japanese"
            | "korean"
            | "french"
            | "german"
            | "spanish"
            | "italian"
            | "portuguese"
            | "russian"
            | "thai"
            | "hindi"
            | "arabic"
            // Download / system
            | "downloads"
            | "download"
            | "completed"
            | "mnt"
            | "nas"
            | "share"
            | "shares"
            | "data"
            | "public"
            | "home"
            | "tmp"
            | "temp"
            // Bonus / extras
            | "extras"
            | "extra"
            | "specials"
            | "special"
            | "bonus"
            | "featurettes"
            | "featurette"
            | "behind the scenes"
            | "deleted scenes"
            | "interviews"
            | "interview"
            | "trailers"
            | "trailer"
            | "samples"
            | "sample"
            // CJK bonus / extras directory names
            | "特典映像"  // tokuten eizou — special footage (JP)
            | "特典"      // tokuten — bonus/extras (JP)
            | "映像特典"  // eizou tokuten — video bonus (JP)
            | "sp"
            // Subtitles / audio
            | "subs"
            | "subtitles"
            | "subtitle"
            | "ost"
            | "soundtrack"
            | "soundtracks"
    ) {
        return true;
    }

    // Prefix matches (e.g., "Season 1", "Disc 2", "CD1").
    if lower.starts_with("season")
        || lower.starts_with("saison")
        || lower.starts_with("temporada")
        || lower.starts_with("stagione")
        || lower.starts_with("disc")
        || lower.starts_with("disk")
        || lower.starts_with("dvd")
    {
        return true;
    }

    // CD1, CD2, etc.
    if lower.starts_with("cd") && lower[2..].chars().all(|c| c.is_ascii_digit()) && lower.len() <= 4
    {
        return true;
    }

    // Quality-as-dir: "1080p", "720p", "2160p", "4K", "4k"
    if lower.ends_with('p') && lower[..lower.len() - 1].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if lower == "4k" {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Pipeline integration ─────────────────────────────────────────

    #[test]
    fn test_clean_title_dots() {
        assert_eq!(clean_title("The.Matrix"), "The Matrix");
    }

    #[test]
    fn test_clean_title_underscores() {
        assert_eq!(clean_title("The_Matrix_Reloaded"), "The Matrix Reloaded");
    }

    #[test]
    fn test_strip_leading_bracket() {
        assert_eq!(clean_title("[XCT].Le.Prestige"), "Le Prestige");
    }

    #[test]
    fn test_strip_paren_year() {
        assert_eq!(clean_title("Movie Name (2005)"), "Movie Name");
    }

    // ── Per-step unit tests ──────────────────────────────────────────

    #[test]
    fn step_strip_leading_brackets_drops_multiple() {
        assert_eq!(strip_leading_brackets("[A][B] Show"), "Show");
        assert_eq!(strip_leading_brackets("[XCT].Le.Prestige"), "Le.Prestige");
    }

    #[test]
    fn step_strip_leading_brackets_unclosed_passes_through() {
        // No closing ']' → leave the input alone (don't loop forever).
        assert_eq!(strip_leading_brackets("[unclosed Show"), "[unclosed Show");
    }

    #[test]
    fn step_strip_paren_year_only_at_end() {
        // The regex consumes leading whitespace before `(YYYY)`.
        assert_eq!(strip_paren_year("Movie (2005)"), "Movie");
        // Year in the middle is preserved (only trailing year is stripped).
        assert_eq!(strip_paren_year("(2005) Movie"), "(2005) Movie");
    }

    #[test]
    fn step_strip_paren_groups_empty_fallback() {
        // Stripping all parens would empty the string → return the original.
        assert_eq!(strip_paren_groups("(only paren)"), "(only paren)");
        // Otherwise strip them.
        assert_eq!(strip_paren_groups("Movie (alt)"), "Movie ");
    }

    #[test]
    fn step_normalize_preserves_word_dash() {
        let out = normalize_separators("Spider-Man.2002", DashPolicy::WordDashOnly);
        assert_eq!(out, "Spider-Man 2002");
    }

    #[test]
    fn step_normalize_drops_separator_flanked_dash() {
        // Default policy: " - " becomes a single space.
        let out = normalize_separators("Show - Sub", DashPolicy::WordDashOnly);
        assert_eq!(out, "Show Sub");
    }

    #[test]
    fn step_normalize_preserves_separator_flanked_dash_in_preserve_mode() {
        // PreserveStructuralDash keeps space-flanked, dot-flanked, and
        // underscore-flanked dashes as a literal " - ".
        assert_eq!(
            normalize_separators("Show - Sub", DashPolicy::PreserveStructuralDash),
            "Show - Sub"
        );
        assert_eq!(
            normalize_separators("Show_-_Sub_-_Final", DashPolicy::PreserveStructuralDash),
            "Show - Sub - Final"
        );
        assert_eq!(
            normalize_separators("Show.-.Sub", DashPolicy::PreserveStructuralDash),
            "Show - Sub"
        );
        // But word-dashes still survive even in preserve mode.
        assert_eq!(
            normalize_separators("Spider-Man", DashPolicy::PreserveStructuralDash),
            "Spider-Man"
        );
    }

    #[test]
    fn preserve_dashes_keeps_inner_separator() {
        // Standard clean_title collapses " - " into a single space.
        assert_eq!(clean_title("Show - Subtitle"), "Show Subtitle");
        // The preserve variant keeps the structural separator literal.
        assert_eq!(
            clean_title_preserve_dashes("Show - Subtitle"),
            "Show - Subtitle"
        );
        // And it does not strip a trailing "Part N" (which is genuine title content).
        assert_eq!(
            clean_title_preserve_dashes("San no Shou Part 2"),
            "San no Shou Part 2"
        );
        // "_-_" and ".-." should normalize to " - " too.
        assert_eq!(
            clean_title_preserve_dashes("Show_-_Sub_-_Final"),
            "Show - Sub - Final"
        );
    }

    #[test]
    fn preserve_dashes_kitchen_sink_composition() {
        // Kitchen-sink test: locks in the *composition order* of every
        // transform inside `clean_title_preserve_dashes` simultaneously.
        //
        // Pre-PR-C this composition was only exercised piecewise (one
        // transform per test). When the pipeline was decomposed in #130,
        // the per-transform tests caught regressions inside each step,
        // but no single test caught "Step 2 strips the year that Step 4
        // expected to see, leaving Step 4 a no-op". This test pins the
        // full chain end-to-end so any future re-decomposition of
        // `clean.rs` (and there will be one — see DESIGN D10) cannot
        // silently change interaction order.
        //
        // Composition under exercise (left to right, all in one input):
        //   1. strip_leading_brackets:   `[Group] ` → ""
        //   2. strip_paren_year:         ` (2014)` at end → ""
        //                                (only fires at end-of-string)
        //   3. strip_paren_groups:       `(Director's Cut)` mid-string → " "
        //   4. normalize_separators(PreserveStructuralDash):
        //        - dot-acronyms preserved
        //        - `_-_` and `.-.` and ` - ` → ` - `
        //        - other separators → single space
        //   5. trim_trailing_punct:      strip trailing `:-,;`
        //
        // Note: the trailing-keyword strip (Step 6) is intentionally NOT
        // applied by `clean_title_preserve_dashes` — "Part N" is genuine
        // title content for this code path.
        let input =
            "[Group].Show_-_Sub.-.Detail.(Director's.Cut).Part.2.S.H.I.E.L.D._-_End.(2014).";
        // Note: the dot-acronym detector greedily extends "S.H.I.E.L.D."
        // backwards to include the preceding `2.` (since `2` is also
        // alphanumeric, the run `2.S.H.I.E.L.D.` matches the
        // `[A-Za-z0-9](\.[A-Za-z0-9]){2,}\.?` pattern in one shot). The
        // trailing dot of the acronym also survives. This is the
        // documented (and tested below in `step_normalize_preserves_dot_acronyms`)
        // behavior; pinning it explicitly here so any change shows up
        // as a clear cross-step regression rather than as a mysterious
        // failure in a single sub-step test.
        let expected = "Show - Sub - Detail Part 2.S.H.I.E.L.D. - End";
        assert_eq!(
            clean_title_preserve_dashes(input),
            expected,
            "composition order regression: each step must run on the \
             output of the previous one in the documented sequence"
        );
    }

    #[test]
    fn step_normalize_preserves_dot_acronyms() {
        let out = normalize_separators("Agents.of.S.H.I.E.L.D.S01", DashPolicy::WordDashOnly);
        assert!(out.contains("S.H.I.E.L.D"), "got: {out}");
    }

    #[test]
    fn step_trim_trailing_punct() {
        assert_eq!(trim_trailing_punct("Title -"), "Title");
        assert_eq!(trim_trailing_punct("Title:,;"), "Title");
        assert_eq!(trim_trailing_punct("Title"), "Title");
    }

    #[test]
    fn step_strip_trailing_keywords() {
        // The Part / Season / Episode regexes start with `\s+` so they
        // consume the space before the keyword. The bonus regex starts
        // with `[-]` so it does NOT eat the leading space — the trailing
        // space remains in the result. This matches pre-refactor behavior
        // exactly; trimming is the caller's job.
        assert_eq!(strip_trailing_keywords("Show Part 2"), "Show");
        assert_eq!(strip_trailing_keywords("Show Season 3"), "Show");
        assert_eq!(strip_trailing_keywords("Show Episode"), "Show");
        assert_eq!(strip_trailing_keywords("Show -x05"), "Show ");
        // Empty fallback: don't strip if the result would be empty.
        assert_eq!(strip_trailing_keywords("Part 2"), "Part 2");
    }

    #[test]
    fn episode_title_keeps_part_n() {
        // clean_episode_title must NOT strip trailing "Part N" — that's
        // valid episode-title content.
        assert_eq!(
            clean_episode_title("The Battle Part 2"),
            "The Battle Part 2"
        );
    }

    #[test]
    fn episode_title_trims_leading_seps() {
        assert_eq!(clean_episode_title(" - The Battle"), "The Battle");
        assert_eq!(clean_episode_title(".._The Battle"), "The Battle");
    }

    // ── is_generic_dir ──────────────────────────────────────────────

    #[test]
    fn generic_dir_originals() {
        // Original entries still work.
        assert!(is_generic_dir("Movies"));
        assert!(is_generic_dir("tv"));
        assert!(is_generic_dir("Season 1"));
        assert!(is_generic_dir("Saison 03"));
    }

    #[test]
    fn generic_dir_extras_and_bonus() {
        assert!(is_generic_dir("Extras"));
        assert!(is_generic_dir("Specials"));
        assert!(is_generic_dir("Bonus"));
        assert!(is_generic_dir("Featurettes"));
        assert!(is_generic_dir("Behind The Scenes"));
        assert!(is_generic_dir("Deleted Scenes"));
        assert!(is_generic_dir("Trailers"));
        assert!(is_generic_dir("Sample"));
    }

    #[test]
    fn generic_dir_disc_and_cd() {
        assert!(is_generic_dir("Disc 1"));
        assert!(is_generic_dir("Disc2"));
        assert!(is_generic_dir("Disk 3"));
        assert!(is_generic_dir("DVD1"));
        assert!(is_generic_dir("CD1"));
        assert!(is_generic_dir("CD2"));
        assert!(!is_generic_dir("CD123")); // too long for CD pattern
    }

    #[test]
    fn generic_dir_quality() {
        assert!(is_generic_dir("1080p"));
        assert!(is_generic_dir("720p"));
        assert!(is_generic_dir("2160p"));
        assert!(is_generic_dir("4K"));
    }

    #[test]
    fn generic_dir_subtitles_and_audio() {
        assert!(is_generic_dir("Subs"));
        assert!(is_generic_dir("Subtitles"));
        assert!(is_generic_dir("OST"));
        assert!(is_generic_dir("Soundtrack"));
    }

    #[test]
    fn generic_dir_structural() {
        assert!(is_generic_dir("Anime"));
        assert!(is_generic_dir("Kids"));
        assert!(is_generic_dir("Cartoons"));
        assert!(is_generic_dir("Shows"));
        assert!(is_generic_dir("Documentary"));
        assert!(is_generic_dir("Documentaries"));
        assert!(is_generic_dir("Music"));
        assert!(is_generic_dir("Concert"));
        assert!(is_generic_dir("Concerts"));
    }

    #[test]
    fn generic_dir_language_categories() {
        assert!(is_generic_dir("Chinese"));
        assert!(is_generic_dir("English"));
        assert!(is_generic_dir("Japanese"));
        assert!(is_generic_dir("Korean"));
        assert!(is_generic_dir("French"));
        assert!(is_generic_dir("German"));
        assert!(is_generic_dir("Spanish"));
        assert!(is_generic_dir("Italian"));
        assert!(is_generic_dir("Portuguese"));
        assert!(is_generic_dir("Russian"));
        assert!(is_generic_dir("Thai"));
        assert!(is_generic_dir("Hindi"));
        assert!(is_generic_dir("Arabic"));
    }

    #[test]
    fn generic_dir_cjk_bonus() {
        assert!(is_generic_dir("特典映像"));
        assert!(is_generic_dir("特典"));
        assert!(is_generic_dir("映像特典"));
        assert!(is_generic_dir("SP"));
    }

    #[test]
    fn non_generic_dirs() {
        // Real titles should NOT be generic.
        assert!(!is_generic_dir("Paw Patrol"));
        assert!(!is_generic_dir("Transformers 1984"));
        assert!(!is_generic_dir("Breaking Bad"));
        assert!(!is_generic_dir("十二国記"));
    }
}
