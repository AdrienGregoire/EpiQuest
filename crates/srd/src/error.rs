/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** error.rs
*/

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SrdError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("checksum mismatch for {file}: manifest says {expected}, file is {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    #[error("unsupported snapshot format version {found} (expected {expected})")]
    FormatVersion { found: u32, expected: u32 },
    #[error("normalization failed for {kind} '{key}': {reason}")]
    Normalize {
        kind: &'static str,
        key: String,
        reason: String,
    },
    #[error("snapshot validation failed:\n  - {}", .0.join("\n  - "))]
    Validation(Vec<String>),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Open5e error: {0}")]
    Open5e(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

pub type Result<T> = std::result::Result<T, SrdError>;
