mod commands;
mod batch;
mod error;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use commands::{ExtractCommand, InfoCommand, ConvertCommand, ValidateCommand, SearchCommand, BatchCommand};

#[derive(Parser)]
#[command(name = "hwp")]
#[command(version, about = "HWP file processing tool", long_about = None)]
#[command(author = "HWP-RS Contributors")]
struct Cli {
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    
    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract content from HWP files with advanced options
    Extract(ExtractCommand),
    
    /// Display comprehensive file information and analysis
    Info(InfoCommand),
    
    /// Convert HWP files to other formats
    Convert(ConvertCommand),
    
    /// Validate HWP file integrity and structure
    Validate(ValidateCommand),
    
    /// Search for content in HWP files
    Search(SearchCommand),
    
    /// Process multiple HWP files in batch
    Batch(BatchCommand),
    
    /// Inspect HWP file metadata (legacy, use 'info' instead)
    #[command(hide = true)]
    Inspect {
        /// Path to the HWP file
        file: String,
    },
}

fn setup_logging(verbosity: u8, quiet: bool) {
    if quiet {
        return;
    }
    
    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(cli.verbose, cli.quiet);
    
    // Execute command
    let result = match cli.command {
        Commands::Extract(cmd) => cmd.execute(),
        Commands::Info(cmd) => cmd.execute(),
        Commands::Convert(cmd) => cmd.execute(),
        Commands::Validate(cmd) => cmd.execute(),
        Commands::Search(cmd) => cmd.execute(),
        Commands::Batch(cmd) => cmd.execute(),
        Commands::Inspect { file } => {
            // Legacy command - redirect to info
            eprintln!("{}", "Note: 'inspect' is deprecated, use 'info' instead".yellow());
            let info_cmd = InfoCommand {
                input: file.into(),
                format: "text".to_string(),
                output: None,
                verbose: false,
                stats: false,
                fonts: false,
                styles: false,
                check_integrity: false,
                metadata_only: false,
                analyze_complexity: false,
                word_frequency: false,
                paragraph_stats: false,
                style_analysis: false,
            };
            info_cmd.execute()
        }
    };
    
    // Handle errors with colored output
    if let Err(e) = result {
        if !cli.quiet {
            eprintln!("{}: {}", "Error".red().bold(), e);
            
            // Print error chain if verbose
            if cli.verbose > 0 {
                let mut source = e.source();
                while let Some(err) = source {
                    eprintln!("{}: {}", "Caused by".yellow(), err);
                    source = err.source();
                }
            }
        }
        std::process::exit(1);
    }
    
    Ok(())
}