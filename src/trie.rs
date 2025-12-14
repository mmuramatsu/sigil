//! # Trie
//!
//! This module provides a Trie data structure for storing and searching file
//! signatures (magic numbers). The Trie is optimized for efficient prefix
//! matching, which is ideal for identifying file types based on their
//! initial bytes.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::read_to_string;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct SignatureEntry {
    pub r#type: String,
    pub offset: u32,
    pub signature: Vec<u8>,
}

#[derive(Default, Debug)]
struct TrieNode {
    file_type: Option<String>,
    children: HashMap<u8, Box<TrieNode>>,
}

/// A Trie for storing and searching file signatures.
///
/// This data structure is used to efficiently store and search for file
/// signatures (magic numbers). It allows for quick identification of file
/// types based on a file's initial bytes.
#[derive(Default, Debug)]
pub struct MagicNumberTrie {
    root: TrieNode,
    /// The maximum buffer size required to identify a file type.
    pub max_buffer_size: u32,
    max_offset_len: u32,
    max_signature_len: u32,
    possible_offsets: Vec<u32>,
}

impl MagicNumberTrie {
    /// Creates a new `MagicNumberTrie` from a JSON file.
    ///
    /// This function reads a JSON file containing file signatures and builds
    /// a Trie from them.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the JSON file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The file cannot be read.
    /// * The file content cannot be parsed as JSON.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let content = read_to_string(path)?;
        let signatures: Vec<SignatureEntry> = serde_json::from_str(&content)?;

        let mut trie = MagicNumberTrie::default();
        let mut unique_offsets: HashSet<u32> = HashSet::new();

        for entry in signatures {
            trie.insert(&entry.signature, entry.r#type);

            unique_offsets.insert(entry.offset);

            if trie.max_offset_len < entry.offset {
                trie.max_offset_len = entry.offset;
            }

            let signature_len = entry.signature.len() as u32;
            if trie.max_signature_len < signature_len {
                trie.max_signature_len = signature_len;
            }
        }

        trie.max_buffer_size = trie.max_offset_len + trie.max_signature_len;
        trie.possible_offsets = unique_offsets.into_iter().collect();
        trie.possible_offsets.sort();

        Ok(trie)
    }

    /// Inserts a file signature into the Trie.
    fn insert(&mut self, file_header: &[u8], file_type: String) {
        let mut curr_node = &mut self.root;

        for &byte in file_header {
            curr_node = curr_node.children.entry(byte).or_default();
        }
        curr_node.file_type = Some(file_type);
    }

    /// Searches the Trie for a matching file signature.
    ///
    /// This function takes a byte slice and searches the Trie for a matching
    /// file signature. It returns the file type if a match is found.
    ///
    /// # Arguments
    ///
    /// * `file_header` - A byte slice from the beginning of a file.
    pub fn search(&self, file_header: &[u8]) -> Option<String> {
        for offset in &self.possible_offsets {
            let offset_usize = *offset as usize;

            if file_header.len() < offset_usize {
                continue;
            }

            let slice_to_check = &file_header[offset_usize..];

            if let Some(file_type) = self.trie_match(slice_to_check) {
                return Some(file_type);
            }
        }

        None
    }

    /// Performs a match operation within the Trie.
    fn trie_match(&self, file_header: &[u8]) -> Option<String> {
        let mut curr_node = &self.root;
        let mut best_match: Option<String> = None;

        for &byte in file_header {
            if let Some(next_node) = curr_node.children.get(&byte) {
                curr_node = next_node;
                if let Some(ref file_type) = curr_node.file_type {
                    best_match = Some(file_type.clone());
                }
            } else {
                break;
            }
        }

        best_match
    }
}