//! Hunch CLI — parse media filenames from the command line.

use clap::Parser;
use hunch::hunch;

#[derive(Parser)]
#[command(
    name = "hunch",
    about = "Fast, offline media filename parser — extract title, year, codec, and 40+ properties"
)]
#[command(version)]
struct Cli {
    /// Filename or release name to parse.
    filename: Vec<String>,

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
    // Users can override with RUST_LOG for finer control (e.g., RUST_LOG=hunch=trace).
    if cli.verbose {
        env_logger::Builder::new()
            .filter_module("hunch", log::LevelFilter::Debug)
            .init();
    } else {
        // Respect RUST_LOG if set, otherwise stay silent.
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off")).init();
    }

    if cli.filename.is_empty() {
        eprintln!("Usage: hunch <filename>");
        std::process::exit(1);
    }

    for filename in &cli.filename {
        let result = hunch(filename);

        if cli.json {
            let map = result.to_flat_map();
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
}
