use anyhow::Result;
use clap::{Parser, Subcommand};
use hwp_parser::parse;
use std::fs;

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

fn inspect_file(path: &str) -> Result<()> {
    println!("Inspecting file: {}", path);
    
    // Read the file
    let data = fs::read(path)?;
    
    // Parse the HWP document
    let document = parse(&data)?;
    
    // Display header information
    println!("\n=== HWP File Information ===");
    println!("Version: {}", document.header.version);
    println!("Properties: 0x{:08X}", document.header.properties.to_u32());
    println!("Compressed: {}", if document.header.is_compressed() { "Yes" } else { "No" });
    println!("Has password: {}", if document.header.has_password() { "Yes" } else { "No" });
    println!("DRM protected: {}", if document.header.is_drm_document() { "Yes" } else { "No" });
    
    // Display document properties
    println!("\n=== Document Properties ===");
    println!("Section count: {}", document.doc_info.properties.section_count);
    println!("Total pages: {}", document.doc_info.properties.total_page_count);
    println!("Total characters: {}", document.doc_info.properties.total_character_count);
    
    // Display DocInfo summary
    println!("\n=== DocInfo Summary ===");
    println!("Character shapes: {}", document.doc_info.char_shapes.len());
    println!("Paragraph shapes: {}", document.doc_info.para_shapes.len());
    println!("Styles: {}", document.doc_info.styles.len());
    println!("Face names (fonts): {}", document.doc_info.face_names.len());
    println!("Border fills: {}", document.doc_info.border_fills.len());
    
    // Display sections
    println!("\n=== Sections ===");
    println!("Total sections: {}", document.sections.len());
    for (idx, section) in document.sections.iter().enumerate() {
        println!("  Section {}: {} paragraphs", idx, section.paragraphs.len());
    }
    
    // Extract and display text
    println!("\n=== Extracted Text (first 500 chars) ===");
    let text = document.get_text();
    if text.is_empty() {
        println!("(No text content found)");
    } else {
        let preview = if text.len() > 500 {
            format!("{}...", &text[..500])
        } else {
            text.clone()
        };
        println!("{}", preview);
        println!("\nTotal text length: {} characters", text.len());
    }
    
    Ok(())
}

fn convert_file(path: &str, format: &str) -> Result<()> {
    // Read and parse the file
    let data = fs::read(path)?;
    let document = parse(&data)?;
    
    match format {
        "text" | "txt" => {
            // Extract and output plain text
            let text = document.get_text();
            println!("{}", text);
        }
        "json" => {
            // Output document structure as JSON
            // For now, just output a simple structure
            println!("{{");
            println!("  \"version\": \"{}\",", document.header.version);
            println!("  \"sections\": {},", document.sections.len());
            println!("  \"text_length\": {},", document.get_text().len());
            println!("  \"paragraphs\": {}", 
                document.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>());
            println!("}}");
        }
        _ => {
            eprintln!("Unsupported format: {}. Use 'text' or 'json'", format);
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Inspect { file } => {
            inspect_file(&file)?;
        }
        Commands::Convert { file, format } => {
            convert_file(&file, &format)?;
        }
        Commands::Validate { file } => {
            println!("Validating file: {}", file);
            // TODO: Implement validation
        }
    }
    
    Ok(())
}