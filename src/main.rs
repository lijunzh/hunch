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

    for indices in groups.values() {
        let group_paths: Vec<&str> = indices.iter().map(|&i| rel_paths[i].as_str()).collect();

        for (pos, &idx) in indices.iter().enumerate() {
            let input = &rel_paths[idx];
            let siblings: Vec<&str> = group_paths
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != pos)
                .map(|(_, s)| *s)
                .collect();

            let result = pipeline.run_with_context(input, &siblings);

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
fn list_media_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("Error: cannot read directory {}", dir.display());
        std::process::exit(1);
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_media_extension(p))
        .collect();
    files.sort();
    files
}

/// List media files in a directory tree (recursive).
fn list_media_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files);
    files.sort();
    files
}

/// Recursively walk a directory tree, collecting media files.
fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut dirs = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_media_extension(&path) {
            out.push(path);
        } else if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    for d in dirs {
        walk_dir(&d, out);
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
