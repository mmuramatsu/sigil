//! # Sigil Library
//!
//! This library provides the core functionality for the Sigil application.
//! It is responsible for reading file signatures, building a Trie for efficient searching,
//! and verifying file types based on their magic numbers.

mod trie;
mod file_manager;

use crate::trie::MagicNumberTrie;
use crate::file_manager::{FileSignature, get_file_info};
use std::fs::{self};
use std::path::{PathBuf};
use colored::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::WalkDir;

/// Configuration for the Sigil application.
///
/// This struct holds all the necessary configuration parameters that are
/// required for the application to run.
pub struct AppConfig {
    /// The path to the file or directory that needs to be verified.
    pub path: PathBuf,
    /// The path to the JSON file containing the file signatures.
    pub input_json_file: Option<PathBuf>,
    /// Flag to activate recursive directory traversal.
    pub recursive: bool,
}

pub enum FileResult {
    Correct(PathBuf),
    Incorrect {
        path: PathBuf,
        declared_type: String,
        actual_type: String,
    },
    Error {
        path: PathBuf,
        error_message: String,
    },
}

/// Runs the main logic of the Sigil application.
///
/// This function orchestrates the file type verification process. It initializes the
/// signature Trie and then, based on the input path, either processes a single file or
/// discovers and processes files within a directory. Directory processing is done in
/// parallel using Rayon for efficiency.
///
/// Finally, it calls the `report` function to print a summary of the results.
///
/// # Arguments
///
/// * `config` - A struct containing the application configuration.
///
/// # Errors
///
/// This function will return an error if:
/// * The JSON file with signatures cannot be read.
/// * The path to be verified does not exist or cannot be read.
pub fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let trie = match config.input_json_file {
        Some(path) => {
            println!("Trie initialized successfully from '{}'.", path.display());
            MagicNumberTrie::from_file(&path)?
        }
        None => {
            println!("Trie initialized successfully from embedded JSON.");
            let json_data = include_str!("../data/magic_numbers_reference.json");
            MagicNumberTrie::from_str(json_data)?
        }
    };
    println!("Max buffer size: {} bytes.", trie.max_buffer_size);

    let path = config.path;
    let mut results: Vec<FileResult> = Vec::new();

    println!("\nStarting verification...");

    if path.is_dir() {
        let path_list = resolve_path(&path, config.recursive)?;

        results = path_list.into_par_iter().map(|file_path| process_file(file_path, &trie)).collect();
    } else {
        results.push(process_file(path, &trie));
    }

    report(results);
    
    Ok(())
}

/// Discovers files to be processed based on the given path and recursive flag.
///
/// If `recursive_flag` is false, it returns a list of files directly within `folder_path`.
/// If `recursive_flag` is true, it performs a recursive search for all files within `folder_path`.
///
/// # Arguments
///
/// * `folder_path` - The directory to search for files.
/// * `recursive_flag` - A boolean to control recursive search.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
fn resolve_path(folder_path: &PathBuf, recursive_flag: bool) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut files_path = Vec::new();

    if !recursive_flag {
        for entry in fs::read_dir(folder_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files_path.push(path);
            }
        }
    } else {
        files_path = WalkDir::new(folder_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();
    }

    Ok(files_path)
}

/// Processes a single file to verify its type and returns a `FileResult`.
///
/// This function does not print anything. Instead, it encapsulates the outcome
/// of the verification in a `FileResult` enum, which can be `Correct`, `Incorrect`,
/// or `Error`. This allows the calling function to aggregate results for later reporting.
///
/// # Arguments
///
/// * `path` - The path to the file to be processed.
/// * `trie` - A reference to the `MagicNumberTrie` containing known file signatures.
fn process_file(path: PathBuf, trie: &MagicNumberTrie) -> FileResult {
    if !path.is_file() {
        return FileResult::Error {
            path,
            error_message: "The provided path is not a file.".to_string(),
        };
    }

    let mut file_info: FileSignature = match get_file_info(path.clone(), trie.max_buffer_size) {
        Ok(info) => info,
        Err(e) => {
            return FileResult::Error { path, error_message: e.to_string() };
        }
    };

    if let Some(actual_type) = trie.search(&file_info.buffer) {
        file_info.actual_type = actual_type;

        if file_info.actual_type.contains(&file_info.declared_type) {
            FileResult::Correct(path)
        } else {
            FileResult::Incorrect { path, declared_type: file_info.declared_type, actual_type: file_info.actual_type }
        }
    } else {
        FileResult::Incorrect { path, declared_type: file_info.declared_type, actual_type: "Unknown".to_string() }
    }
}

/// Prints a formatted summary report of the verification results.
///
/// This function takes a vector of `FileResult` and prints a summary including
/// total files scanned, number of correct, incorrect, and errored files. It also
/// lists the details for each incorrect or errored file.
///
/// The output formatting is conditional: emojis and colors are disabled if the
/// output is not an interactive terminal (TTY), making it suitable for redirection
/// to a file.
///
/// # Arguments
///
/// * `results` - A vector of `FileResult` containing the outcome for each processed file.
fn report(results: Vec<FileResult>) {
    let total_files = results.len();
    let mut correct_files = 0;
    let mut incorrect_files = Vec::new();
    let mut error_files = Vec::new();

    let should_use_emojis = atty::is(atty::Stream::Stdout);

    for r in results {
        match r {
            FileResult::Correct(_) => {
                correct_files += 1;
            }
            FileResult::Incorrect { path, declared_type, actual_type } => {
                let emoji_prefix = if should_use_emojis { "❌ " } else { "" };
                incorrect_files.push(format!("{} {}: Declared as '{}', but is '{}'", emoji_prefix, path.display(), declared_type.blue(), actual_type.red()));
            }
            FileResult::Error { path, error_message } => {
                let emoji_prefix = if should_use_emojis { "⚠️ " } else { "" };
                error_files.push(format!("{} {}: Error processing file - {}", emoji_prefix, path.display(), error_message.red()));
            }
        }
    }

    println!("\n--- Verification Complete ---");
    println!("Total files scanned: {}", total_files);
    if should_use_emojis {
        println!("{}", format!("✔️ Correct: {}", correct_files).green());
        println!("{}", format!("❌ Incorrect: {}", incorrect_files.len()).red());
        println!("{}", format!("⚠️ Errors: {}", error_files.len()).yellow());
    } else {
        println!("{}", format!("Correct: {}", correct_files).green());
        println!("{}", format!("Incorrect: {}", incorrect_files.len()).red());
        println!("{}", format!("Errors: {}", error_files.len()).yellow());
    }

    if !incorrect_files.is_empty() {
        println!("\n--- Incorrect Files ---");
        for r in incorrect_files {
            println!("{}", r);
        }
    }

    if !error_files.is_empty() {
        println!("\n--- Error Files ---");
        for r in error_files {
            println!("{}", r);
        }
    }
}