//! # rs-ach
//!
//! ACH (Automated Clearing House) file parser for Rust.
//!
//! This crate provides a safe interface for parsing ACH files
//! following the NACHA (National Automated Clearing House Association) specifications.
//!
//! ## Features
//!
//! - Parse ACH file headers and batch records
//! - Support for PPD, CCD, and other standard entry class codes
//! - Entry detail records with addenda support
//! - Type-safe parsing with comprehensive error handling
//! - Zero-copy parsing where possible
//!
//! ## Example
//!
//! ```no_run
//! use rs_ach::AchFile;
//!
//! let ach_content = std::fs::read_to_string("sample.ach").unwrap();
//! let ach_file = AchFile::parse(&ach_content).unwrap();
//!
//! for batch in &ach_file.batches {
//!     println!("Batch: {}", batch.header.company_name);
//!     for entry in &batch.entries {
//!         println!("  Entry: {} - ${}", entry.individual_name, entry.amount);
//!     }
//! }
//! ```

mod error;
mod parser;
mod records;

pub use error::AchError;
pub use records::{Addenda, BatchControl, BatchHeader, EntryDetail, FileControl, FileHeader};

/// Represents a complete ACH file with file header, batches, and file control.
#[derive(Debug, Clone)]
pub struct AchFile<'a> {
    /// File header record (record type 1)
    pub file_header: FileHeader<'a>,

    /// Collection of batches in the file
    pub batches: Vec<Batch<'a>>,

    /// File control record (record type 9)
    pub file_control: FileControl,
}

impl<'a> AchFile<'a> {
    /// Parse an ACH file from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The complete ACH file content as a string
    ///
    /// # Returns
    ///
    /// Returns a parsed `AchFile` on success, or an `AchError` if parsing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rs_ach::AchFile;
    ///
    /// let ach_content = std::fs::read_to_string("sample.ach").unwrap();
    /// let ach_file = AchFile::parse(&ach_content).unwrap();
    /// ```
    pub fn parse(content: &'a str) -> Result<Self, AchError> {
        parser::parse_ach_file(content)
    }
}

/// Represents a batch within an ACH file.
#[derive(Debug, Clone)]
pub struct Batch<'a> {
    /// Batch header record (record type 5)
    pub header: BatchHeader<'a>,

    /// Collection of entry detail records
    pub entries: Vec<EntryDetail<'a>>,

    /// Batch control record (record type 8)
    pub control: BatchControl,
}
