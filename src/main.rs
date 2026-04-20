//! Hunch CLI — parse media filenames from the command line.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::Parser;
use hunch::{Confidence, Pipeline};

/// Media file extensions recognized for batch/context scanning.
const MEDIA_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "wmv", "flv", "ts", "m4v", "webm", "ogv", "mov", "mpg", "mpeg", "m2ts",
    "iso", "img", "rmvb", "rm",
];

#[derive(Parser)]
#[command(
    name = "hunch",
    about = "Fast, offline media filename parser — extract title, year, codec, and 40+ properties",
    after_help = "EXAMPLES:
  Parse a single file:
    hunch 'Show.S01E03.720p.BluRay.x264-GROUP.mkv'

  Parse with sibling context (improves title detection):
    hunch 'S01E03.mkv' --context /path/to/show/

  Batch-parse a single directory:
    hunch --batch /path/to/show/ -j

  Batch-parse an entire media library (RECOMMENDED):
    hunch --batch /path/to/tv/ -r -j

    The -r flag recurses into subdirectories and preserves the full
    relative path (e.g. tv/Anime/Show/Extra/file.mkv). This gives
    the parser critical context from directory names like 'tv/',
    'Anime/', 'Season 1/' for accurate type detection.

    Without -r, files in deep subdirectories lose their path context
    and bonus content may be misclassified as movies."
)]
#[command(version)]
struct Cli {
    /// Filename or release name to parse.
    #[arg(conflicts_with = "batch_dir")]
    filename: Vec<String>,

    /// Directory of sibling files to use as context for title detection.
    #[arg(long = "context", value_name = "DIR", conflicts_with = "batch_dir")]
    context_dir: Option<PathBuf>,

    /// Parse all media files in a directory (siblings used as mutual context).
    ///
    /// For media libraries, use with -r to preserve full path context:
    ///   hunch --batch /path/to/tv/ -r -j
    #[arg(long = "batch", value_name = "DIR", conflicts_with_all = ["context_dir", "filename"])]
    batch_dir: Option<PathBuf>,

    /// Recurse into subdirectories (only with --batch).
    ///
    /// Preserves relative paths so directory names (tv/, Anime/, Season 1/)
    /// provide context for type inference. Recommended for media libraries.
    #[arg(short = 'r', long = "recursive", requires = "batch_dir")]
    recursive: bool,

    /// Output raw JSON (default is pretty-printed).
    #[arg(short = 'j', long = "json")]
    json: bool,

    /// Enable verbose/debug logging (set RUST_LOG for finer control).
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Initialise logging: --verbose enables debug level for the hunch crate.
    if cli.verbose {
        env_logger::Builder::new()
            .filter_module("hunch", log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off")).init();
    }

    let pipeline = Pipeline::new();

    // ── Batch mode ──────────────────────────────────────────────────────
    if let Some(ref batch_dir) = cli.batch_dir {
        run_batch(&pipeline, batch_dir, cli.recursive, cli.json);
        return;
    }

    // ── Normal / context mode ──────────────────────────────────────────
    if cli.filename.is_empty() {
        eprintln!("Usage: hunch <filename>");
        eprintln!("       hunch --batch <dir>");
        std::process::exit(1);
    }

    // Load sibling filenames from --context directory if provided.
    let siblings: Vec<String> = if let Some(ref ctx_dir) = cli.context_dir {
        list_media_files(ctx_dir)
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(String::from))
            .collect()
    } else {
        Vec::new()
    };

    for filename in &cli.filename {
        let result = if siblings.is_empty() {
            pipeline.run(filename)
        } else {
            // Exclude the target from its own siblings.
            let sibs: Vec<&str> = siblings
                .iter()
                .filter(|s| s.as_str() != filename.as_str())
                .map(|s| s.as_str())
                .collect();
            pipeline.run_with_context(filename, &sibs)
        };

        print_result(filename, &result, cli.json);

        // Low-confidence hint (only in non-JSON mode).
        if !cli.json
            && result.confidence() == Confidence::Low
            && cli.context_dir.is_none()
            && cli.batch_dir.is_none()
        {
            eprintln!("\u{26a0} Low confidence result. Try: hunch --context . \"{filename}\"");
            eprintln!("  (sibling files can improve title detection)");
        }
    }
}

// ── Batch processing ────────────────────────────────────────────────────

/// Run batch mode: collect media files, group by directory, parse with
/// sibling context.
///
/// Files in the same directory are siblings of each other. Each file's input
/// string is its relative path from the batch root, so
/// `extract_title_from_parent` can walk the full directory chain.
fn run_batch(pipeline: &Pipeline, batch_dir: &Path, recursive: bool, json: bool) {
    // Collect files: flat or recursive.
    let files = if recursive {
        list_media_files_recursive(batch_dir)
    } else {
        list_media_files(batch_dir)
    };

    // In flat mode, warn if subdirectories contain media files being skipped.
    if !recursive {
        warn_if_subdirs_have_media(batch_dir);
    }

    if files.is_empty() {
        eprintln!("No media files found in {}", batch_dir.display());
        std::process::exit(1);
    }

    // Build relative paths from the batch root, prefixed with the batch
    // dir name itself. This ensures extract_title_from_parent can walk
    // the full directory chain.
    //
    // Flat:      batch_dir="Paw Patrol/" → "Paw Patrol/S01E10.mkv"
    // Recursive: batch_dir="tv/"         → "tv/Paw Patrol/Season 1/01.mkv"
    let batch_name = batch_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let rel_paths: Vec<String> = files
        .iter()
        .filter_map(|p| {
            let rel = p.strip_prefix(batch_dir).ok()?.to_str()?;
            if batch_name.is_empty() {
                Some(rel.to_string())
            } else {
                Some(format!("{batch_name}/{rel}"))
            }
        })
        .collect();

    // Group files by their parent directory (siblings = same parent dir).
    // Key: parent dir relative path (e.g., "Show/Season 1")
    // Value: indices into rel_paths
    let groups = group_by_parent(&rel_paths);

    // In recursive mode, process groups top-down and cache titles so child
    // directories can inherit parent context. BTreeMap iteration is already
    // sorted lexically, which means parents come before children. (#94)
    let mut dir_titles: BTreeMap<String, String> = BTreeMap::new();

    for (parent_key, indices) in &groups {
        let group_paths: Vec<&str> = indices.iter().map(|&i| rel_paths[i].as_str()).collect();

        // Look up the nearest ancestor's cached title as a fallback hint.
        // This propagates invariance results from parent directories to
        // child directories like SP/, 特典映像/. (#94)
        //
        // Suppress the fallback for child directories whose names signal
        // "this is auxiliary content, not show episodes" — Sample/,
        // Subs/, Extras/, Specials/, Bonus/, Featurettes/. These should
        // not inherit the parent's title because the parent's invariance
        // was computed over a different content type. (#97, #208)
        let fallback_title: Option<&str> = if recursive {
            let dir_name = Path::new(parent_key)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if is_inheritance_blocking_dir(dir_name) {
                None
            } else {
                find_ancestor_title(parent_key, &dir_titles)
            }
        } else {
            None
        };

        let mut group_titles: Vec<String> = Vec::new();

        for (pos, &idx) in indices.iter().enumerate() {
            let input = &rel_paths[idx];
            let siblings: Vec<&str> = group_paths
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != pos)
                .map(|(_, s)| *s)
                .collect();

            let result = pipeline.run_with_context_and_fallback(input, &siblings, fallback_title);

            // Collect titles for parent context caching.
            if let Some(title) = result.title() {
                group_titles.push(title.to_string());
            }

            // Display filename: bare filename for flat, relative path for recursive.
            let display_name = if recursive {
                input.as_str()
            } else {
                files[idx]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(input)
            };
            print_result(display_name, &result, json);
        }

        // Cache the most common title from this directory group so child
        // directories can inherit it as fallback context. (#94)
        if recursive {
            if let Some(title) = most_common_title(&group_titles) {
                dir_titles.insert(parent_key.clone(), title);
            }
        }
    }
}

/// Group relative paths by their parent directory.
///
/// Returns a sorted map of parent dir → indices, so sibling detection
/// is scoped per-directory.
fn group_by_parent(rel_paths: &[String]) -> BTreeMap<String, Vec<usize>> {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, path) in rel_paths.iter().enumerate() {
        let parent = Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();
        groups.entry(parent).or_default().push(i);
    }
    groups
}

/// Find the cached title from the nearest ancestor directory.
///
/// Walks up the directory tree from `child_key` and returns the first
/// cached title found. This propagates titles from parent directories
/// to child directories in recursive batch mode.
fn find_ancestor_title<'a>(
    child_key: &str,
    dir_titles: &'a BTreeMap<String, String>,
) -> Option<&'a str> {
    let mut current = child_key;
    loop {
        let parent = Path::new(current)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("");

        if parent == current {
            break;
        }

        if let Some(title) = dir_titles.get(parent) {
            return Some(title.as_str());
        }

        current = parent;
    }
    None
}

/// Find the most common title in a group of results.
///
/// Returns the title that appears most frequently, breaking ties by
/// first occurrence. Used to cache a representative title for parent
/// context propagation.
fn most_common_title(titles: &[String]) -> Option<String> {
    if titles.is_empty() {
        return None;
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for title in titles {
        *counts.entry(title.as_str()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(title, _)| title.to_string())
}

// ── Output ──────────────────────────────────────────────────────────────

/// Print a parse result as pretty JSON or compact JSON.
fn print_result(filename: &str, result: &hunch::HunchResult, json: bool) {
    if json {
        let mut map = result.to_flat_map();
        map.insert(
            "_filename".to_string(),
            serde_json::Value::String(filename.to_string()),
        );
        match serde_json::to_string(&map) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error: failed to serialize result: {e}");
                std::process::exit(1);
            }
        }
    } else {
        println!("{result}");
    }
}

// ── File listing ────────────────────────────────────────────────────────

/// List media files in a directory (non-recursive).
///
/// **Symlink-safe** — uses [`std::fs::DirEntry::file_type`] (which does
/// NOT follow symlinks) and skips symlinked entries entirely, mirroring
/// the defense in [`walk_dir_inner`]. Without this guard, `--context`
/// mode could collect basenames of files outside the user-chosen
/// directory via a symlink, slightly broadening the parser's input
/// surface to attacker-controlled bytes. Hunch only reads basenames,
/// not file contents, so the impact is low — but matching `walk_dir`'s
/// hardening keeps the defense story consistent across both `--context`
/// and `--batch -r` entry points. (#209)
fn list_media_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("Error: cannot read directory {}", dir.display());
        std::process::exit(1);
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            // Use file_type() (does NOT follow symlinks) instead of
            // path.is_file() (which DOES). Skip symlinks entirely for
            // parity with walk_dir_inner.
            let ft = e.file_type().ok()?;
            if ft.is_symlink() || !ft.is_file() {
                return None;
            }
            let path = e.path();
            is_media_extension(&path).then_some(path)
        })
        .collect();
    files.sort();
    files
}

/// Maximum recursion depth for [`walk_dir`] / [`dir_contains_media`].
///
/// Real-world media libraries are very rarely deeper than 6 levels
/// (`Movies/Genre/Year/Title/Disc/file.mkv`). 32 leaves a generous safety
/// margin while bounding worst-case stack usage and preventing
/// pathological-input DoS (e.g., a directory tree built deliberately
/// to exhaust stack on traversal).
const MAX_WALK_DEPTH: usize = 32;

/// List media files in a directory tree (recursive).
fn list_media_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files);
    files.sort();
    files
}

/// Recursively walk a directory tree, collecting media files.
///
/// **Defensive guarantees** (added in PR-B of the v1.1.8 release-prep wave):
///
/// - **Depth-bounded**: stops recursing past [`MAX_WALK_DEPTH`] (32) to
///   prevent stack overflow from pathologically deep trees.
/// - **Symlink-safe**: skips symlinked entries entirely (uses
///   [`std::fs::DirEntry::file_type`], which does NOT follow symlinks).
///   This avoids:
///     - Infinite recursion on symlink loops
///     - Filesystem-escape via a symlink to `/`, `/home`, etc.
///     - Surprising double-traversal when a directory is symlinked into
///       its own descendant.
///
///   Trade-off: legitimate symlink farms (e.g., a curated `Movies/` of
///   symlinks to a NAS share) will be skipped. The CLI does not currently
///   advertise symlink support, so this is the safer default. If users
///   request symlink-following, add an opt-in flag with a visited-inode
///   set rather than removing the guard.
fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_dir_inner(dir, out, 0);
}

fn walk_dir_inner(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth >= MAX_WALK_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut dirs = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        // Use file_type() (does NOT follow symlinks) instead of
        // path.is_file() / path.is_dir() (which DO follow symlinks).
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_symlink() {
            continue;
        }
        let path = entry.path();
        if ft.is_file() && is_media_extension(&path) {
            out.push(path);
        } else if ft.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    for d in dirs {
        walk_dir_inner(&d, out, depth + 1);
    }
}

/// Check if a path has a recognized media file extension.
fn is_media_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            MEDIA_EXTENSIONS
                .iter()
                .any(|me| me.eq_ignore_ascii_case(ext))
        })
}

/// When running flat `--batch` (no `-r`), check if subdirectories contain
/// media files. If so, print a hint suggesting `-r` for better results.
///
/// This catches the #1 UX footgun: flat batch silently loses path context
/// from ancestor directories (tv/, Anime/, Season 1/), producing plausible
/// but wrong results (e.g., bonus content classified as movies).
fn warn_if_subdirs_have_media(batch_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(batch_dir) else {
        return;
    };
    let subdirs_with_media: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Use file_type() (does NOT follow symlinks) to stay
            // consistent with walk_dir's defensive guarantees.
            e.file_type()
                .map(|ft| ft.is_dir() && !ft.is_symlink())
                .unwrap_or(false)
        })
        .filter(|e| dir_contains_media(&e.path()))
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    if subdirs_with_media.is_empty() {
        return;
    }

    let n = subdirs_with_media.len();
    let dir_display = batch_dir.display();
    eprintln!(
        "hint: found media files in {n} subdirector{} being skipped. \
         Use -r to include them\n      \
         with full path context (improves type detection and title extraction).\n      \
         Example: hunch --batch {dir_display} -r -j",
        if n == 1 { "y" } else { "ies" },
    );
}

/// Check if a directory (recursively) contains at least one media file.
/// Short-circuits on the first match for performance.
///
/// Same defensive guarantees as [`walk_dir`]: depth-bounded by
/// [`MAX_WALK_DEPTH`] and skips symlinks.
fn dir_contains_media(dir: &Path) -> bool {
    dir_contains_media_inner(dir, 0)
}

fn dir_contains_media_inner(dir: &Path, depth: usize) -> bool {
    if depth >= MAX_WALK_DEPTH {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let mut subdirs = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_symlink() {
            continue;
        }
        let path = entry.path();
        if ft.is_file() && is_media_extension(&path) {
            return true;
        } else if ft.is_dir() {
            subdirs.push(path);
        }
    }
    subdirs
        .iter()
        .any(|d| dir_contains_media_inner(d, depth + 1))
}

/// Check whether a directory name signals "auxiliary content, not show
/// episodes" — i.e., its files should NOT inherit the parent's title via
/// the ancestor-title fallback.
///
/// Two flavors of blocking:
///
/// - **Sample/preview content** (`sample`, `samples`, `subs`, `subtitles`,
///   `featurettes`) — clips and packaging, not the show itself. (#97)
/// - **Extras/specials content** (`extras`, `extra`, `specials`, `bonus`)
///   — bonus features that live alongside a show but don't share its
///   title metadata. Without blocking, a `Show/Extras/Bonus.mkv` file in
///   a batch alongside an unrelated `Movie.mkv` could inherit "Movie"
///   from the batch-root cache. (#208)
fn is_inheritance_blocking_dir(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "sample"
            | "samples"
            | "subs"
            | "subtitles"
            | "featurettes"
            | "extras"
            | "extra"
            | "specials"
            | "bonus"
    )
}
