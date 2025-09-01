use crate::batch::{BatchProcessor, ErrorStrategy};
use crate::commands::{ConvertCommand, ExtractCommand, InfoCommand};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::*;
use std::fs;
use std::path::PathBuf;

/// Batch processing command
#[derive(Args, Debug)]
pub struct BatchCommand {
    /// Input directory or glob pattern
    pub input: String,

    /// Output directory
    #[arg(short, long)]
    pub output_dir: PathBuf,

    /// Process files recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long, default_value = "4")]
    pub parallel: usize,

    /// Continue processing on errors
    #[arg(long)]
    pub continue_on_error: bool,

    /// Generate summary report
    #[arg(long)]
    pub report: bool,

    /// Report output file
    #[arg(long)]
    pub report_file: Option<PathBuf>,

    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,

    #[command(subcommand)]
    pub operation: BatchOperation,
}

#[derive(Subcommand, Debug)]
pub enum BatchOperation {
    /// Extract content from multiple files
    Extract {
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Preserve formatting
        #[arg(long)]
        preserve_formatting: bool,

        /// Include metadata
        #[arg(long)]
        include_metadata: bool,
    },

    /// Convert multiple files to another format
    Convert {
        /// Target format
        #[arg(short = 't', long = "to", default_value = "text")]
        format: String,

        /// Pretty print JSON
        #[arg(long)]
        json_pretty: bool,

        /// Generate TOC for Markdown
        #[arg(long)]
        markdown_toc: bool,
    },

    /// Generate info reports for multiple files
    Info {
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Include statistics
        #[arg(long)]
        stats: bool,

        /// Include font information
        #[arg(long)]
        fonts: bool,

        /// Include style information
        #[arg(long)]
        styles: bool,
    },

    /// Validate multiple files
    Validate {
        /// Strict validation
        #[arg(long)]
        strict: bool,

        /// Check integrity
        #[arg(long)]
        check_integrity: bool,
    },
}

impl BatchCommand {
    pub fn execute(&self) -> Result<()> {
        // Ensure output directory exists
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir).context("Failed to create output directory")?;
        }

        // Create batch processor
        let error_strategy = if self.continue_on_error {
            ErrorStrategy::Skip
        } else {
            ErrorStrategy::FailFast
        };

        let batch_processor = BatchProcessor::new(self.parallel, error_strategy);

        // Discover files
        let files = if self.input.contains('*') || self.input.contains('?') {
            batch_processor.discover_glob(&self.input)?
        } else {
            let path = PathBuf::from(&self.input);
            batch_processor.discover_files(&path, self.recursive)?
        };

        if files.is_empty() {
            eprintln!(
                "{}: No HWP files found in '{}'",
                "Warning".yellow(),
                self.input
            );
            return Ok(());
        }

        eprintln!("Found {} HWP files to process", files.len());

        // Execute batch operation
        let operation_name = match &self.operation {
            BatchOperation::Extract { .. } => "Batch Extract",
            BatchOperation::Convert { .. } => "Batch Convert",
            BatchOperation::Info { .. } => "Batch Info",
            BatchOperation::Validate { .. } => "Batch Validate",
        };

        let result = batch_processor
            .process_files(files, operation_name, |file| self.process_single_file(file))?;

        // Print summary
        println!("\n{}", "=".repeat(60));
        println!("{}", result.summary().green().bold());

        if result.failed > 0 {
            println!("\n{}", "Failed files:".red().bold());
            for process_result in &result.results {
                if !process_result.success {
                    println!(
                        "  {} - {}",
                        process_result.path.display(),
                        process_result.message
                    );
                }
            }
        }

        // Generate report if requested
        if self.report {
            self.generate_report(&result)?;
        }

        // Return error if any files failed and not continuing on error
        if result.failed > 0 && !self.continue_on_error {
            return Err(anyhow::anyhow!(
                "Batch processing failed: {} files failed",
                result.failed
            ));
        }

        Ok(())
    }

    fn process_single_file(&self, file: &std::path::Path) -> Result<String> {
        let output_path = self.get_output_path(file)?;

        // Check if file exists and overwrite flag
        if output_path.exists() && !self.overwrite {
            return Ok(format!("Skipped (file exists)"));
        }

        match &self.operation {
            BatchOperation::Extract {
                format,
                preserve_formatting,
                include_metadata,
            } => {
                let cmd = ExtractCommand {
                    input: file.to_path_buf(),
                    format: format.clone(),
                    output: Some(output_path),
                    preserve_formatting: *preserve_formatting,
                    include_metadata: *include_metadata,
                    sections: None,
                    search: None,
                    context: 0,
                    text_width: None,
                    markdown_toc: false,
                    json_pretty: false,
                    json_include_styles: false,
                    paragraphs: None,
                    tables_only: false,
                    images_only: false,
                    equations_only: false,
                };
                cmd.execute()?;
                Ok("Extracted".to_string())
            }

            BatchOperation::Convert {
                format,
                json_pretty,
                markdown_toc,
            } => {
                let cmd = ConvertCommand {
                    input: file.display().to_string(),
                    format: format.clone(),
                    output: Some(output_path),
                    output_dir: None,
                    recursive: false,
                    json_pretty: *json_pretty,
                    json_include_styles: false,
                    text_width: None,
                    text_page_breaks: false,
                    markdown_flavor: "commonmark".to_string(),
                    markdown_toc: *markdown_toc,
                    overwrite: self.overwrite,
                };
                cmd.execute()?;
                Ok("Converted".to_string())
            }

            BatchOperation::Info {
                format,
                stats,
                fonts,
                styles,
            } => {
                let cmd = InfoCommand {
                    input: file.to_path_buf(),
                    format: format.clone(),
                    output: Some(output_path),
                    verbose: false,
                    stats: *stats,
                    fonts: *fonts,
                    styles: *styles,
                    check_integrity: false,
                    metadata_only: false,
                    analyze_complexity: false,
                    word_frequency: false,
                    paragraph_stats: false,
                    style_analysis: false,
                };
                cmd.execute()?;
                Ok("Info generated".to_string())
            }

            BatchOperation::Validate {
                strict,
                check_integrity,
            } => {
                // For validate, we just check and report
                use crate::commands::ValidateCommand;
                let cmd = ValidateCommand {
                    input: file.to_path_buf(),
                    strict: *strict,
                    check_integrity: *check_integrity,
                    verify_structure: false,
                    performance: false,
                    verbose: false,
                };
                cmd.execute()?;
                Ok("Validated".to_string())
            }
        }
    }

    fn get_output_path(&self, input_file: &std::path::Path) -> Result<PathBuf> {
        let file_name = input_file
            .file_stem()
            .context("Invalid file name")?
            .to_string_lossy();

        let extension = match &self.operation {
            BatchOperation::Extract { format, .. }
            | BatchOperation::Convert { format, .. }
            | BatchOperation::Info { format, .. } => match format.as_str() {
                "json" => "json",
                "markdown" | "md" => "md",
                "html" => "html",
                "yaml" | "yml" => "yaml",
                "csv" => "csv",
                _ => "txt",
            },
            BatchOperation::Validate { .. } => "validation.json",
        };

        Ok(self.output_dir.join(format!("{}.{}", file_name, extension)))
    }

    fn generate_report(&self, result: &crate::batch::BatchResult) -> Result<()> {
        let report = serde_json::json!({
            "operation": format!("{:?}", self.operation),
            "input": self.input,
            "output_dir": self.output_dir.display().to_string(),
            "total_files": result.total,
            "successful": result.successful,
            "failed": result.failed,
            "duration_seconds": result.total_duration.as_secs_f64(),
            "files_per_second": result.total as f64 / result.total_duration.as_secs_f64(),
            "results": result.results.iter().map(|r| {
                serde_json::json!({
                    "file": r.path.display().to_string(),
                    "success": r.success,
                    "message": r.message,
                    "duration_ms": r.duration.as_millis(),
                })
            }).collect::<Vec<_>>(),
        });

        let report_str = serde_json::to_string_pretty(&report)?;

        if let Some(report_file) = &self.report_file {
            fs::write(report_file, report_str)?;
            eprintln!("\nReport written to: {}", report_file.display());
        } else {
            println!("\n{}", "Batch Processing Report:".cyan().bold());
            println!("{}", report_str);
        }

        Ok(())
    }
}
