use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific error types
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to parse HWP file '{file}': {details}")]
    ParseError {
        file: PathBuf,
        details: String,
    },
    
    #[error("Unsupported output format '{format}'. Supported formats: {}", .supported.join(", "))]
    UnsupportedFormat {
        format: String,
        supported: Vec<String>,
    },
    
    #[error("Batch operation failed for '{file}': {details}")]
    BatchError {
        file: PathBuf,
        details: String,
    },
    
    #[error("Batch processing failed: {successful} succeeded, {failed} failed out of {total}")]
    BatchSummaryError {
        total: usize,
        successful: usize,
        failed: usize,
    },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Search pattern error: {0}")]
    SearchError(String),
    
    #[error("No files found matching pattern: {pattern}")]
    NoFilesFound {
        pattern: String,
    },
    
    #[error("Output directory does not exist: {path}")]
    OutputDirectoryNotFound {
        path: PathBuf,
    },
    
    #[error("Pipeline error: {0}")]
    PipelineError(String),
}

/// Helper function to get supported formats
pub fn supported_formats() -> Vec<String> {
    vec![
        "text".to_string(),
        "txt".to_string(),
        "json".to_string(),
        "markdown".to_string(),
        "md".to_string(),
        "html".to_string(),
        "yaml".to_string(),
        "yml".to_string(),
        "csv".to_string(),
    ]
}