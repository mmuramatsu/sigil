//! # Sigil Library
//!
//! This library provides the core functionality for the Sigil application.
//! It is responsible for reading file signatures, building a Trie for efficient searching,
//! and verifying file types based on their magic numbers.

mod trie;
mod file_manager;

use crate::trie::MagicNumberTrie;
use crate::file_manager::{FileSignature, get_file_info};
use std::path::PathBuf;
use colored::*;

/// Configuration for the Sigil application.
///
/// This struct holds all the necessary configuration parameters that are
/// required for the application to run.
pub struct AppConfig {
    /// The path to the file that needs to be verified.
    pub path: PathBuf,
    /// The path to the JSON file containing the file signatures.
    pub input_json_file: PathBuf,
}

/// Runs the main logic of the Sigil application.
///
/// This function takes an `AppConfig` struct and performs the file type
/// verification. It reads the file signatures from the provided JSON file,
/// builds a Trie, and then compares the file's magic numbers against the
/// signatures in the Trie.
///
/// # Arguments
///
/// * `config` - A struct containing the application configuration.
///
/// # Errors
///
/// This function will return an error if:
/// * The JSON file with signatures cannot be read.
/// * The file to be verified cannot be read.
/// * The file extension cannot be determined.
pub fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let trie = MagicNumberTrie::from_file(&config.input_json_file)?;
    println!("Trie initialized successfully from JSON file.");
    println!("Max buffer size: {} bytes.", trie.max_buffer_size);

    if config.path.is_file() {
        println!("\nAnalyzing file: {}", config.path.display());
        let mut file_info: FileSignature = get_file_info(config.path, trie.max_buffer_size)?;

        println!("Declared type: {}", file_info.declared_type.yellow());

        if let Some(actual_type) = trie.search(&file_info.buffer) {
            file_info.actual_type = actual_type;
            println!("Actual type:   {}", file_info.actual_type.green());

            if file_info.actual_type.contains(&file_info.declared_type) {
                println!("\n{}", "✔️  File type is correct.".green());
            } else {
                println!("\n{}", "❌  File type is incorrect!".red());
            }
        } else {
            println!("Actual type:   {}", "Unknown".red());
            println!("\n{}", "⚠️  Could not determine file type from magic numbers.".yellow());
        }
    } else {
        println!("{}", "❌  Error: The provided path is not a file.".red());
    }

    Ok(())
}