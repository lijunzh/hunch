//! Hunch CLI — parse media filenames from the command line.

use std::path::PathBuf;

use clap::Parser;
use hunch::{Confidence, Pipeline};

/// Media file extensions recognized for batch/context scanning.
const MEDIA_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "wmv", "flv", "ts", "m4v", "webm", "ogv", "mov", "mpg", "mpeg",
    "m2ts", "iso", "img", "rmvb", "rm",
];

#[derive(Parser)]
#[command(
    name = "hunch",
    about = "Fast, offline media filename parser — extract title, year, codec, and 40+ properties"
)]
#[command(version)]
struct Cli {
    /// Filename or release name to parse.
    filename: Vec<String>,

    /// Directory of sibling files to use as context for title detection.
    #[arg(long = "context", value_name = "DIR", conflicts_with = "batch_dir")]
    context_dir: Option<PathBuf>,

    /// Parse all media files in a directory (siblings used as mutual context).
    #[arg(long = "batch", value_name = "DIR", conflicts_with = "context_dir")]
    batch_dir: Option<PathBuf>,

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
        let files = list_media_files(batch_dir);
        if files.is_empty() {
            eprintln!("No media files found in {}", batch_dir.display());
            std::process::exit(1);
        }
        let filenames: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(String::from))
            .collect();

        for (i, filename) in filenames.iter().enumerate() {
            let siblings: Vec<&str> = filenames
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, s)| s.as_str())
                .collect();
            let result = pipeline.run_with_context(filename, &siblings);
            print_result(filename, &result, cli.json);
        }
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
            eprintln!(
                "\u{26a0} Low confidence result. Try: hunch --context . \"{filename}\""
            );
            eprintln!("  (sibling files can improve title detection)");
        }
    }
}

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

/// List all media files in a directory (non-recursive).
fn list_media_files(dir: &PathBuf) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("Error: cannot read directory {}", dir.display());
        std::process::exit(1);
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        MEDIA_EXTENSIONS
                            .iter()
                            .any(|me| me.eq_ignore_ascii_case(ext))
                    })
        })
        .collect();
    files.sort();
    files
}
