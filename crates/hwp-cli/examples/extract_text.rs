use std::fs;
use std::env;
use hwp_parser::{TextExtractor, parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <hwp-file> [--formatted]", args[0]);
        eprintln!("  <hwp-file>    Path to the HWP file to extract text from");
        eprintln!("  --formatted   Optional: Extract with formatting information");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let formatted = args.len() > 2 && args[2] == "--formatted";
    
    println!("Reading HWP file: {}", file_path);
    let hwp_data = fs::read(file_path)?;
    println!("File size: {} bytes", hwp_data.len());
    
    if formatted {
        // Parse the document first for formatted extraction
        println!("\nParsing document for formatted extraction...");
        let doc = parse(&hwp_data)?;
        
        let formatted_text = TextExtractor::extract_with_formatting(&doc)?;
        
        println!("\n=== Formatted Text Extraction ===\n");
        for (i, para) in formatted_text.paragraphs.iter().enumerate() {
            println!("Paragraph {}:", i + 1);
            if para.level > 0 {
                println!("  [Heading Level {}]", para.level);
            }
            if para.is_list_item {
                println!("  [List Item]");
            }
            println!("  Text: {}", para.text);
            println!();
        }
    } else {
        // Direct text extraction from bytes
        println!("\nExtracting text directly from HWP file...");
        
        match TextExtractor::extract_from_bytes(&hwp_data) {
            Ok(text) => {
                println!("\n=== Extracted Text ===\n");
                println!("{}", text);
                println!("\n=== Statistics ===");
                println!("Total characters: {}", text.chars().count());
                println!("Total lines: {}", text.lines().count());
                
                // Count Korean characters
                let korean_chars = text.chars()
                    .filter(|c| (*c >= '\u{AC00}' && *c <= '\u{D7AF}') || // Hangul syllables
                               (*c >= '\u{1100}' && *c <= '\u{11FF}') || // Hangul Jamo
                               (*c >= '\u{3130}' && *c <= '\u{318F}'))   // Hangul compatibility Jamo
                    .count();
                println!("Korean characters: {}", korean_chars);
            }
            Err(e) => {
                eprintln!("Error extracting text: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}