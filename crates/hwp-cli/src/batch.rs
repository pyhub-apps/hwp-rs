use crate::error::CliError;
use anyhow::{Context, Result};
use glob::glob;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Strategy for handling errors in batch operations
#[derive(Debug, Clone, Copy)]
pub enum ErrorStrategy {
    /// Skip failed files and continue
    Skip,
    /// Stop on first error
    FailFast,
    /// Retry failed operations once
    Retry,
}

/// Result of a single file operation
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub path: PathBuf,
    pub success: bool,
    pub message: String,
    pub duration: std::time::Duration,
}

/// Aggregated results from batch processing
#[derive(Debug)]
pub struct BatchResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ProcessResult>,
    pub total_duration: std::time::Duration,
}

impl BatchResult {
    pub fn summary(&self) -> String {
        format!(
            "Processed {} files: {} successful, {} failed in {:.2}s",
            self.total,
            self.successful,
            self.failed,
            self.total_duration.as_secs_f64()
        )
    }
}

/// Batch processor for parallel file operations
pub struct BatchProcessor {
    parallel_jobs: usize,
    error_strategy: ErrorStrategy,
    multi_progress: MultiProgress,
}

impl BatchProcessor {
    pub fn new(parallel_jobs: usize, error_strategy: ErrorStrategy) -> Self {
        Self {
            parallel_jobs,
            error_strategy,
            multi_progress: MultiProgress::new(),
        }
    }

    /// Discover HWP files in a directory
    pub fn discover_files(&self, path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "hwp") {
                files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            let pattern = if recursive {
                format!("{}/**/*.hwp", path.display())
            } else {
                format!("{}/*.hwp", path.display())
            };

            for entry in glob(&pattern).context("Failed to read glob pattern")? {
                match entry {
                    Ok(path) => files.push(path),
                    Err(e) => eprintln!("Warning: {}", e),
                }
            }
        }

        Ok(files)
    }

    /// Process files with a glob pattern
    pub fn discover_glob(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in glob(pattern).context("Failed to read glob pattern")? {
            match entry {
                Ok(path) if path.extension().map_or(false, |ext| ext == "hwp") => {
                    files.push(path);
                }
                Ok(_) => {} // Skip non-HWP files
                Err(e) => eprintln!("Warning: {}", e),
            }
        }

        Ok(files)
    }

    /// Process multiple files in parallel
    pub fn process_files<F>(
        &self,
        files: Vec<PathBuf>,
        operation_name: &str,
        operation: F,
    ) -> Result<BatchResult>
    where
        F: Fn(&Path) -> Result<String> + Send + Sync,
    {
        let total = files.len();
        if total == 0 {
            return Ok(BatchResult {
                total: 0,
                successful: 0,
                failed: 0,
                results: vec![],
                total_duration: std::time::Duration::from_secs(0),
            });
        }

        // Create progress bar
        let pb = self.create_progress_bar(total, operation_name);
        let results = Arc::new(Mutex::new(Vec::new()));
        let start_time = Instant::now();

        // Configure thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.parallel_jobs)
            .build()
            .context("Failed to create thread pool")?;

        // Process files in parallel
        let operation = Arc::new(operation);
        let pb = Arc::new(pb);
        let error_strategy = self.error_strategy;

        pool.install(|| {
            files.par_iter().for_each(|file| {
                let start = Instant::now();
                let result = self.process_single_file(file, operation.as_ref(), error_strategy);

                let process_result = ProcessResult {
                    path: file.clone(),
                    success: result.is_ok(),
                    message: result.unwrap_or_else(|e| e.to_string()),
                    duration: start.elapsed(),
                };

                // Update progress
                pb.inc(1);
                pb.set_message(format!("Processing: {}", file.display()));

                // Store result
                results.lock().unwrap().push(process_result);
            });
        });

        pb.finish_with_message(format!("{} complete", operation_name));

        // Aggregate results
        let results = Arc::try_unwrap(results)
            .map(|mutex| mutex.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone());

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        Ok(BatchResult {
            total,
            successful,
            failed,
            results,
            total_duration: start_time.elapsed(),
        })
    }

    /// Process a single file with error handling
    fn process_single_file<F>(
        &self,
        file: &Path,
        operation: &F,
        error_strategy: ErrorStrategy,
    ) -> Result<String>
    where
        F: Fn(&Path) -> Result<String> + Send + Sync,
    {
        let result = operation(file);

        match (result, error_strategy) {
            (Ok(msg), _) => Ok(msg),
            (Err(e), ErrorStrategy::Skip) => {
                eprintln!("Skipping {}: {}", file.display(), e);
                Err(e)
            }
            (Err(e), ErrorStrategy::FailFast) => {
                return Err(CliError::BatchError {
                    file: file.to_path_buf(),
                    details: e.to_string(),
                }
                .into());
            }
            (Err(e), ErrorStrategy::Retry) => {
                // Retry once
                eprintln!("Retrying {}", file.display());
                operation(file).map_err(|retry_err| {
                    eprintln!("Retry failed for {}: {}", file.display(), retry_err);
                    e
                })
            }
        }
    }

    /// Create a progress bar with standard style
    fn create_progress_bar(&self, total: usize, operation: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(total as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("{} starting...", operation));
        pb
    }

    /// Show the multi-progress display
    pub fn show_progress(&self) {
        // MultiProgress display is handled automatically by indicatif
    }
}
