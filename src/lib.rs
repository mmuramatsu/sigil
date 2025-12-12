//! # Sigil Library
//!
//! This library provides the core functionality for the Sigil application.
//! It is responsible for reading file signatures, building a Trie for efficient searching,
//! and verifying file types based on their magic numbers.

mod trie;
mod file_manager;

use crate::trie::MagicNumberTrie;
use crate::file_manager::{FileSignature, get_file_info};
use std::fs;
use std::path::PathBuf;
use colored::*;
use walkdir::WalkDir;

/// Configuration for the Sigil application.
///
/// This struct holds all the necessary configuration parameters that are
/// required for the application to run.
pub struct AppConfig {
    /// The path to the file or directory that needs to be verified.
    pub path: PathBuf,
    /// The path to the JSON file containing the file signatures.
    pub input_json_file: PathBuf,
    /// Flag to activate recursive directory traversal.
    pub recursive: bool,
}

/// Runs the main logic of the Sigil application.
///
/// This function takes an `AppConfig` struct and performs file type verification.
/// It initializes a Trie from a JSON file of signatures. If the provided path is a file,
/// it analyzes that file. If the path is a directory, it discovers and analyzes all
/// files within it. Directory traversal is recursive if the `recursive` flag is set.
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
/// * Any other I/O error occurs during file or directory processing.
pub fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let trie = MagicNumberTrie::from_file(&config.input_json_file)?;
    println!("Trie initialized successfully from JSON file.");
    println!("Max buffer size: {} bytes.", trie.max_buffer_size);

    let path = config.path;

    if path.is_dir() {
        let path_list = resolve_path(&path, config.recursive)?;

        for file_path in path_list {
            process_file(file_path, &trie)?;
        }
    } else {
        process_file(path, &trie)?;
    }
    
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
fn resolve_path(folder_path: &PathBuf, recursive_flag: bool) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
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

/// Processes a single file to verify its type against the MagicNumberTrie.
///
/// It reads the file's magic numbers and compares them with the known signatures in the trie,
/// then prints the declared and actual file types.
///
/// # Arguments
///
/// * `path` - The path to the file to be processed.
/// * `trie` - A reference to the `MagicNumberTrie` containing known file signatures.
///
/// # Errors
///
/// Returns an error if the file cannot be read or its metadata cannot be accessed.
fn process_file(path: PathBuf, trie: &MagicNumberTrie) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_file() {
        println!("\nAnalyzing file: {}", path.display());
        let mut file_info: FileSignature = get_file_info(path, trie.max_buffer_size)?;

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