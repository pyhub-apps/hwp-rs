use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hwp")]
#[command(about = "HWP file processing tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect HWP file metadata
    Inspect {
        /// Path to the HWP file
        file: String,
    },
    /// Convert HWP file to another format
    Convert {
        /// Path to the HWP file
        file: String,
        /// Output format (json, text)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Validate HWP file structure
    Validate {
        /// Path to the HWP file
        file: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Inspect { file } => {
            println!("Inspecting file: {}", file);
            // TODO: Implement inspection
        }
        Commands::Convert { file, format } => {
            println!("Converting {} to {}", file, format);
            // TODO: Implement conversion
        }
        Commands::Validate { file } => {
            println!("Validating file: {}", file);
            // TODO: Implement validation
        }
    }
    
    Ok(())
}