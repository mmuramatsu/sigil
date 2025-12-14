//! # File Manager
//!
//! This module is responsible for handling file-related operations, such as
//! reading file metadata and extracting byte signatures. It provides the necessary
//! tools to inspect files and prepare them for type verification.

use std::error::Error;
use std::fs::File;
use std::ffi::OsStr;
use std::io::{Read};
use std::path::{Path, PathBuf};
use std::fmt;

#[derive(Debug)]
enum FileManagerError {
    MissingExtension,
}

impl Error for FileManagerError {}

impl fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileManagerError::MissingExtension => write!(f, "File has no extension"),
        }
    }
}

/// Represents the signature of a file.
///
/// This struct stores information about a file's declared type, its actual
/// type as determined by its magic numbers, and a buffer containing the
/// initial bytes of the file.
#[derive(Default, Debug)]
pub struct FileSignature {
    /// The file type as declared by its extension.
    pub declared_type: String,
    /// The actual file type as determined by its magic numbers.
    pub actual_type: String,
    /// A buffer containing the initial bytes of the file.
    pub buffer: Vec<u8>,
}

/// Reads a specified number of bytes from a file.
fn get_bytes_from_file(path: &Path, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; buffer_size];
    let mut file = File::open(path)?;
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

/// Extracts the file extension from a path.
fn get_file_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

/// Gathers information about a file.
///
/// This function takes a file path and a buffer size, and returns a
/// `FileSignature` struct containing the file's declared type and a
/// buffer with the initial bytes of the file.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `buffer_size` - The number of bytes to read from the beginning of the file.
///
/// # Errors
///
/// This function will return an error if:
/// * The file has no extension.
/// * The file cannot be read.
pub fn get_file_info(path: PathBuf, buffer_size: u32) -> Result<FileSignature, Box<dyn Error + Send + Sync>> {
    let buffer_size = buffer_size as usize;
    let declared_type = get_file_extension(&path)
        .ok_or(FileManagerError::MissingExtension)?
        .to_uppercase();

    let buffer = get_bytes_from_file(&path, buffer_size)?;

    Ok(FileSignature {
        declared_type,
        actual_type: String::new(),
        buffer,
    })
}