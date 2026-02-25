//! Tokenizer: splits a media filename into a stream of tokens.
//!
//! Tokens are the atomic units that matchers operate on. By tokenizing first,
//! we get position awareness (title zone vs tech zone) for free, and matchers
//! can work on isolated tokens without needing lookaround assertions.
//!
//! # Design
//!
//! The tokenizer handles:
//! - Separator splitting (`.`, `-`, `_`, ` `)
//! - Dot-acronym preservation (`S.H.I.E.L.D` → one token)
//! - Bracket group detection (`[group]`, `(group)`)
//! - Path separator handling (`/`, `\`)
//! - File extension stripping

/// A single token extracted from the input string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token text (original casing preserved).
    pub text: String,
    /// Byte offset of the token start in the original input.
    pub start: usize,
    /// Byte offset of the token end (exclusive) in the original input.
    pub end: usize,
    /// What separator preceded this token.
    pub separator: Separator,
    /// Whether this token is inside brackets `[...]` or parentheses `(...)`.
    pub in_brackets: bool,
}

impl Token {
    /// Case-insensitive text for matching.
    pub fn lower(&self) -> String {
        self.text.to_lowercase()
    }

    /// Byte length of the token text.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the token is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// The separator that preceded a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    /// No separator (start of input, or inside a compound token).
    None,
    /// `.` separator.
    Dot,
    /// `-` separator.
    Dash,
    /// ` ` separator.
    Space,
    /// `_` separator.
    Underscore,
    /// `/` or `\` path separator.
    PathSep,
}

/// Which part of the path a segment represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// A directory component (e.g., "Movies", "Season 01").
    Directory,
    /// The final filename component (e.g., "movie.720p.mkv").
    Filename,
}

/// A tokenized path segment (one directory or the filename).
#[derive(Debug, Clone)]
pub struct PathSegment {
    /// What kind of segment this is.
    pub kind: SegmentKind,
    /// Tokens within this segment.
    pub tokens: Vec<Token>,
    /// Byte offset where this segment starts in the original input.
    pub start: usize,
    /// Byte offset where this segment ends (exclusive) in the original input.
    pub end: usize,
    /// Depth from root (0 = first segment, increases toward filename).
    pub depth: usize,
}

/// Result of tokenizing an input string.
#[derive(Debug, Clone)]
pub struct TokenStream {
    /// The original input string.
    pub input: String,
    /// Flattened view of ALL tokens across all segments, ordered by byte position.
    pub tokens: Vec<Token>,
    /// Structured view by path segment.
    pub segments: Vec<PathSegment>,
    /// Byte offset where the filename starts (after last `/` or `\`).
    pub filename_start: usize,
    /// File extension if detected (lowercase, without the dot).
    pub extension: Option<String>,
}

/// Tokenize a media filename/path into a stream of tokens.
///
/// Splits the input at path separators (`/`, `\`) into segments, tokenizes each
/// segment independently, and produces both a structured segment view and a
/// flattened token view for backward compatibility.
///
/// Dot-acronyms like `S.H.I.E.L.D.` are preserved as single tokens.
/// Bracket groups like `[rarbg]` are marked with `in_brackets: true`.
pub fn tokenize(input: &str) -> TokenStream {
    let filename_start = input.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);

    // Split input into raw path segments.
    let raw_parts = split_path_segments(input);

    let mut segments = Vec::new();
    let mut extension = None;
    let last_idx = raw_parts.len().saturating_sub(1);

    for (depth, &(seg_text, seg_start)) in raw_parts.iter().enumerate() {
        // Skip empty segments (e.g., leading `/`).
        if seg_text.is_empty() {
            continue;
        }

        // Skip Windows drive letter segments like "D:".
        if is_drive_letter(seg_text) {
            continue;
        }

        let is_filename = depth == last_idx;
        let kind = if is_filename {
            SegmentKind::Filename
        } else {
            SegmentKind::Directory
        };

        // Only strip extension from the filename segment.
        let (name_part, seg_ext) = if is_filename {
            split_extension(seg_text)
        } else {
            (seg_text, None)
        };

        if is_filename {
            extension = seg_ext;
        }

        let protected = find_dot_acronyms(name_part);
        let tokens = split_into_tokens(name_part, seg_start, &protected);

        let actual_depth = segments.len();
        segments.push(PathSegment {
            kind,
            tokens,
            start: seg_start,
            end: seg_start + seg_text.len(),
            depth: actual_depth,
        });
    }

    // Flatten all segment tokens into one vec, ordered by byte position.
    let tokens: Vec<Token> = segments
        .iter()
        .flat_map(|seg| seg.tokens.iter().cloned())
        .collect();

    TokenStream {
        input: input.to_string(),
        tokens,
        segments,
        filename_start,
        extension,
    }
}

/// Split input at path separators, returning `(segment_text, byte_offset)` pairs.
fn split_path_segments(input: &str) -> Vec<(&str, usize)> {
    let mut parts = Vec::new();
    let mut start = 0;

    for (i, ch) in input.char_indices() {
        if ch == '/' || ch == '\\' {
            parts.push((&input[start..i], start));
            start = i + 1;
        }
    }
    // Final segment (the filename).
    parts.push((&input[start..], start));
    parts
}

/// Check if a path segment is a Windows drive letter (e.g., "D:").
fn is_drive_letter(seg: &str) -> bool {
    let bytes = seg.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Split off a file extension if it looks like a media/subtitle/archive extension.
fn split_extension(filename: &str) -> (&str, Option<String>) {
    if let Some(dot_pos) = filename.rfind('.') {
        let ext = &filename[dot_pos + 1..];
        if !ext.is_empty() && ext.len() <= 5 && is_known_extension(ext) {
            return (&filename[..dot_pos], Some(ext.to_lowercase()));
        }
    }
    (filename, None)
}

/// Check if a string looks like a known file extension.
fn is_known_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
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

/// Find byte ranges of dot-acronyms (e.g., `S.H.I.E.L.D.`) that should
/// be preserved as single tokens.
fn find_dot_acronyms(s: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for pattern: single char, dot, single char, dot, ...
        // Minimum: X.Y.Z (5 chars, 3 letters + 2 dots)
        // The char must be "isolated" — not preceded by another alphanumeric.
        let preceded_by_alpha = i > 0 && bytes[i - 1].is_ascii_alphanumeric();
        if bytes[i].is_ascii_alphanumeric()
            && !preceded_by_alpha
            && i + 2 < len
            && bytes[i + 1] == b'.'
        {
            let start = i;
            let mut end = i + 1; // after first char

            // Consume .X pairs (dot + single alphanumeric).
            while end < len
                && bytes[end] == b'.'
                && end + 1 < len
                && bytes[end + 1].is_ascii_alphanumeric()
            {
                end += 2; // skip dot + char
            }

            // If the last consumed char is the start of a multi-char word
            // (followed by another alphanumeric), roll it back.
            if end < len && bytes[end].is_ascii_alphanumeric() {
                end -= 2;
            }

            // Need at least 3 letters (X.Y.Z).
            let letter_count = (end - start).div_ceil(2);
            if letter_count >= 3 {
                // Do NOT consume trailing dot — it acts as a separator
                // between the acronym and the next token.
                ranges.push((start, end));
                i = end;
                continue;
            }
        }
        i += 1;
    }

    ranges
}

/// Check if a byte position falls within a protected dot-acronym range.
fn in_protected(pos: usize, protected: &[(usize, usize)]) -> bool {
    protected.iter().any(|(s, e)| pos >= *s && pos < *e)
}

/// Split a filename (without extension) into tokens at separator boundaries.
fn split_into_tokens(name: &str, base_offset: usize, protected: &[(usize, usize)]) -> Vec<Token> {
    split_into_tokens_inner(name, base_offset, protected, 0)
}

fn split_into_tokens_inner(
    name: &str,
    base_offset: usize,
    protected: &[(usize, usize)],
    depth: u32,
) -> Vec<Token> {
    // Guard against pathological nesting.
    if depth > 3 {
        if !name.is_empty() {
            return vec![Token {
                text: name.to_string(),
                start: base_offset,
                end: base_offset + name.len(),
                separator: Separator::None,
                in_brackets: true,
            }];
        }
        return Vec::new();
    }
    let mut tokens = Vec::new();
    let bytes = name.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    let mut current_sep = Separator::None;
    let mut bracket_depth: u32 = 0;

    while i < len {
        // Handle bracket opens.
        if bytes[i] == b'[' || bytes[i] == b'(' {
            bracket_depth += 1;
            let close_char = if bytes[i] == b'[' { b']' } else { b')' };
            let content_start = i + 1; // skip the bracket
            let mut j = content_start;
            while j < len && bytes[j] != close_char {
                j += 1;
            }
            // Recursively tokenize the bracket content, marking tokens as in_brackets.
            let bracket_content = &name[content_start..j];
            if !bracket_content.is_empty() {
                // Don't pass protected ranges into bracket recursion — they
                // reference byte positions in the outer string, not the bracket substring.
                let inner_tokens = split_into_tokens_inner(
                    bracket_content,
                    base_offset + content_start,
                    &[],
                    depth + 1,
                );
                for mut t in inner_tokens {
                    t.in_brackets = true;
                    tokens.push(t);
                }
            }
            // Skip past close bracket.
            i = if j < len { j + 1 } else { j };
            current_sep = Separator::None;
            bracket_depth = bracket_depth.saturating_sub(1);
            continue;
        }

        // Handle bracket closes (unmatched).
        if bytes[i] == b']' || bytes[i] == b')' {
            bracket_depth = bracket_depth.saturating_sub(1);
            i += 1;
            continue;
        }

        // Handle separators.
        if is_separator(bytes[i]) && !in_protected(i, protected) {
            current_sep = byte_to_separator(bytes[i]);
            i += 1;
            // Consume consecutive separators (e.g., `..` or `- `).
            while i < len && is_separator(bytes[i]) && !in_protected(i, protected) {
                // Keep the "most significant" separator.
                let next_sep = byte_to_separator(bytes[i]);
                if sep_priority(next_sep) > sep_priority(current_sep) {
                    current_sep = next_sep;
                }
                i += 1;
            }
            continue;
        }

        // Collect a token: run of non-separator, non-bracket chars.
        let token_start = i;
        while i < len && !is_separator(bytes[i]) || in_protected(i, protected) {
            if bytes[i] == b'[' || bytes[i] == b'(' || bytes[i] == b']' || bytes[i] == b')' {
                break;
            }
            i += 1;
        }

        let text = &name[token_start..i];
        if !text.is_empty() {
            tokens.push(Token {
                text: text.to_string(),
                start: base_offset + token_start,
                end: base_offset + i,
                separator: current_sep,
                in_brackets: bracket_depth > 0,
            });
            current_sep = Separator::None;
        }
    }

    tokens
}

fn is_separator(b: u8) -> bool {
    matches!(b, b'.' | b'-' | b'_' | b' ' | b',')
}

fn byte_to_separator(b: u8) -> Separator {
    match b {
        b'.' => Separator::Dot,
        b'-' => Separator::Dash,
        b'_' => Separator::Underscore,
        b' ' | b',' => Separator::Space,
        b'/' | b'\\' => Separator::PathSep,
        _ => Separator::None,
    }
}

/// Priority for choosing between consecutive separators.
/// Higher = more significant.
fn sep_priority(s: Separator) -> u8 {
    match s {
        Separator::None => 0,
        Separator::Dot => 1,
        Separator::Underscore => 2,
        Separator::Dash => 3,
        Separator::Space => 4,
        Separator::PathSep => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_dot_separated() {
        let ts = tokenize("The.Walking.Dead.S05E03.720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["The", "Walking", "Dead", "S05E03", "720p"]);
        assert_eq!(ts.extension, Some("mkv".to_string()));
    }

    #[test]
    fn test_space_separated() {
        let ts = tokenize("The Walking Dead S05E03 720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["The", "Walking", "Dead", "S05E03", "720p"]);
    }

    #[test]
    fn test_underscore_separated() {
        let ts = tokenize("The_Walking_Dead_S05E03_720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["The", "Walking", "Dead", "S05E03", "720p"]);
    }

    #[test]
    fn test_dot_acronym_shield() {
        let ts = tokenize("Marvels.Agents.of.S.H.I.E.L.D.S01E06.720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["Marvels", "Agents", "of", "S.H.I.E.L.D", "S01E06", "720p"]
        );
    }

    #[test]
    fn test_bracket_group() {
        let ts = tokenize("Movie.720p.x264-GROUP[rarbg].mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "720p", "x264", "GROUP", "rarbg"]);
        // "rarbg" should be in_brackets
        assert!(!ts.tokens[3].in_brackets); // GROUP
        assert!(ts.tokens[4].in_brackets); // rarbg
    }

    #[test]
    fn test_dash_release_group() {
        let ts = tokenize("Movie.720p.BluRay.x264-DEMAND.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "720p", "BluRay", "x264", "DEMAND"]);
        assert_eq!(ts.tokens[4].separator, Separator::Dash);
    }

    #[test]
    fn test_path_with_directory() {
        let ts = tokenize("/media/movies/Movie.720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        // Flattened tokens now include directory segments too.
        assert_eq!(texts, vec!["media", "movies", "Movie", "720p"]);
        assert_eq!(ts.filename_start, 14); // after "/media/movies/"
    }

    #[test]
    fn test_consecutive_separators() {
        let ts = tokenize("Movie..720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "720p"]);
    }

    #[test]
    fn test_mixed_separators() {
        let ts = tokenize("Movie.Name - 720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "Name", "720p"]);
        // Space has higher priority than dot, so the separator before "720p" is Space.
        assert_eq!(ts.tokens[2].separator, Separator::Space);
    }

    #[test]
    fn test_no_extension() {
        let ts = tokenize("Movie.Name.S01E02");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "Name", "S01E02"]);
        assert_eq!(ts.extension, None);
    }

    #[test]
    fn test_parenthesized_year() {
        let ts = tokenize("Movie Name (2024) 720p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "Name", "2024", "720p"]);
        assert!(ts.tokens[2].in_brackets);
    }

    #[test]
    fn test_anime_brackets() {
        let ts = tokenize("[SubGroup] Series Name - 01 [720p].mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["SubGroup", "Series", "Name", "01", "720p"]);
        assert!(ts.tokens[0].in_brackets); // SubGroup
        assert!(!ts.tokens[1].in_brackets); // Series
        assert!(ts.tokens[4].in_brackets); // 720p
    }

    #[test]
    fn test_dot_acronym_minimum() {
        // X.Y is NOT an acronym (only 2 letters), X.Y.Z is (3 letters)
        let ts = tokenize("A.B.Movie.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["A", "B", "Movie"]);
    }

    #[test]
    fn test_dot_acronym_three_letters() {
        let ts = tokenize("A.B.C.Movie.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["A.B.C", "Movie"]);
    }

    #[test]
    fn test_separator_types() {
        let ts = tokenize("A.B-C_D E.mkv");
        assert_eq!(ts.tokens[0].separator, Separator::None); // A
        assert_eq!(ts.tokens[1].separator, Separator::Dot); // B
        assert_eq!(ts.tokens[2].separator, Separator::Dash); // C
        assert_eq!(ts.tokens[3].separator, Separator::Underscore); // D
        assert_eq!(ts.tokens[4].separator, Separator::Space); // E
    }

    #[test]
    fn test_empty_input() {
        let ts = tokenize("");
        assert!(ts.tokens.is_empty());
        assert_eq!(ts.extension, None);
    }

    #[test]
    fn test_extension_only() {
        let ts = tokenize("movie.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["movie"]);
        assert_eq!(ts.extension, Some("mkv".to_string()));
    }

    #[test]
    fn test_dts_hd_ma_tokens() {
        // DTS-HD.MA should tokenize as ["DTS", "HD", "MA"]
        // The pipeline will handle multi-token matching.
        let ts = tokenize("Movie.DTS-HD.MA.5.1.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "DTS", "HD", "MA", "5", "1"]);
    }

    #[test]
    fn test_web_dl_tokens() {
        let ts = tokenize("Movie.WEB-DL.1080p.mkv");
        let texts: Vec<&str> = ts.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["Movie", "WEB", "DL", "1080p"]);
    }

    // --- Path segment tests ---

    #[test]
    fn test_path_segments_basic() {
        let ts = tokenize("Movies/Movie.720p.mkv");
        assert_eq!(ts.segments.len(), 2);
        assert_eq!(ts.segments[0].kind, SegmentKind::Directory);
        assert_eq!(ts.segments[0].tokens[0].text, "Movies");
        assert_eq!(ts.segments[0].depth, 0);
        assert_eq!(ts.segments[1].kind, SegmentKind::Filename);
        assert_eq!(ts.segments[1].depth, 1);
        // Flattened tokens include directory tokens.
        assert!(ts.tokens.iter().any(|t| t.text == "Movies"));
    }

    #[test]
    fn test_path_segments_deep() {
        let ts = tokenize("TV/Show Name/Season 01/Show.S01E01.720p.mkv");
        assert_eq!(ts.segments.len(), 4);
        assert_eq!(ts.segments[0].kind, SegmentKind::Directory);
        assert_eq!(ts.segments[3].kind, SegmentKind::Filename);
        assert_eq!(ts.segments[0].depth, 0);
        assert_eq!(ts.segments[3].depth, 3);
    }

    #[test]
    fn test_path_segments_no_path() {
        let ts = tokenize("Movie.720p.mkv");
        assert_eq!(ts.segments.len(), 1);
        assert_eq!(ts.segments[0].kind, SegmentKind::Filename);
        assert_eq!(ts.segments[0].depth, 0);
    }

    #[test]
    fn test_path_segments_dir_metadata() {
        let ts = tokenize("movies/Movie Name (2009) BRrip 720p/abbreviated.avi");
        assert_eq!(ts.segments.len(), 3);
        let dir_tokens: Vec<&str> = ts.segments[1].tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(dir_tokens.contains(&"BRrip"));
        assert!(dir_tokens.contains(&"720p"));
        assert!(dir_tokens.contains(&"2009"));
    }

    #[test]
    fn test_windows_path() {
        let ts = tokenize("D:\\TV\\Show.S01E01.mkv");
        // Drive letter "D:" should be skipped.
        assert_eq!(ts.segments.last().unwrap().kind, SegmentKind::Filename);
        let filename_tokens: Vec<&str> = ts.segments.last().unwrap()
            .tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(filename_tokens.contains(&"Show"));
        assert!(filename_tokens.contains(&"S01E01"));
    }

    #[test]
    fn test_leading_slash_skips_empty() {
        let ts = tokenize("/movies/Movie.mkv");
        // Leading empty segment from "/" should be skipped.
        assert!(ts.segments.iter().all(|s| !s.tokens.is_empty()));
        assert_eq!(ts.segments.last().unwrap().kind, SegmentKind::Filename);
    }

    #[test]
    fn test_dir_no_extension_stripping() {
        // Directory "movie.2009" should NOT have "2009" stripped as extension.
        let ts = tokenize("movie.2009/file.mkv");
        let dir_tokens: Vec<&str> = ts.segments[0].tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(dir_tokens.contains(&"2009"));
    }
}
