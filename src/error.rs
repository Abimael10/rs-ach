//! Error types for ACH file parsing.

use thiserror::Error;

/// Errors that can occur during ACH file parsing.
#[derive(Error, Debug)]
pub enum AchError {
    /// The record type is invalid or unsupported.
    #[error("Invalid record type: {0}")]
    InvalidRecordType(String),

    /// The line length does not match the expected 94 characters.
    #[error("Invalid line length: expected 94, got {0}")]
    InvalidLineLength(usize),

    /// A numeric field could not be parsed.
    #[error("Invalid numeric field '{field}': {source}")]
    InvalidNumber {
        field: &'static str,
        source: std::num::ParseIntError,
    },

    /// The file structure is invalid (e.g., missing header or control records).
    #[error("Invalid file structure: {0}")]
    InvalidStructure(String),

    /// The file is empty or contains no valid records.
    #[error("Empty file")]
    EmptyFile,

    /// A batch is missing required records.
    #[error("Incomplete batch: {0}")]
    IncompleteBatch(String),
}
