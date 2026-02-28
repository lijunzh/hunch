//! Hunch CLI — parse media filenames from the command line.

use clap::Parser;
use hunch::{Options, hunch, hunch_with};

#[derive(Parser)]
#[command(
    name = "hunch",
    about = "Fast, offline media filename parser — extract title, year, codec, and 40+ properties"
)]
#[command(version)]
struct Cli {
    /// Filename or release name to parse.
    filename: Vec<String>,

    /// Hint the media type: "movie" or "episode".
    #[arg(short = 't', long = "type")]
    media_type: Option<String>,

    /// Treat input as name only (no path separators).
    #[arg(short = 'n', long = "name-only")]
    name_only: bool,

    /// Output raw JSON (default is pretty-printed).
    #[arg(short = 'j', long = "json")]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.filename.is_empty() {
        eprintln!("Usage: hunch <filename>");
        std::process::exit(1);
    }

    let mut options = Options::new();
    if let Some(ref t) = cli.media_type {
        options = options.with_type(t);
    }
    if cli.name_only {
        options = options.name_only();
    }

    for filename in &cli.filename {
        let result = if cli.media_type.is_some() || cli.name_only {
            hunch_with(filename, options.clone())
        } else {
            hunch(filename)
        };

        if cli.json {
            let map = result.to_flat_map();
            println!("{}", serde_json::to_string(&map).unwrap_or_default());
        } else {
            println!("{result}");
        }
    }
}
