use sigil::{run, AppConfig};

use clap::Parser;

/// Sigil checks the integrity by comparing the file type to the type infered by the Magic Number of it
#[derive(Parser, Debug)]
#[command(name = "Sigil", version = "1.0")]
struct Cli {
    /// File/Directory's path
    path: std::path::PathBuf,

    /// File path for an input JSON file with file signatures
    #[arg(short, long, default_value = "data/magic_numbers_reference.json")]
    input_json_file: std::path::PathBuf,

    /// Recursively check all folders inside of that path
    #[arg(short, long)]
    recursive: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    let config = AppConfig {
        path: cli.path,
        input_json_file: cli.input_json_file,
        recursive: cli.recursive,
    };

    if atty::isnt(atty::Stream::Stdout) {
        colored::control::set_override(false);
    }

    println!("The path is: '{}'", config.path.display());
    println!(
        "The input file path is: '{}'",
        config.input_json_file.display()
    );

    run(config)
}