/// Example: Parse and display HWP file header information
/// 
/// Run with: cargo run --example parse_header -- <path_to_hwp_file>

use hwp_parser;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <hwp_file>", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    // Read file
    let data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };
    
    // Parse HWP document
    match hwp_parser::parse(&data) {
        Ok(document) => {
            println!("HWP File Header Information");
            println!("===========================");
            println!("Version: {}", document.header.version);
            println!("Compressed: {}", document.header.is_compressed());
            println!("Has Password: {}", document.header.has_password());
            println!("DRM Protected: {}", document.header.is_drm_document());
            println!("\nProperties:");
            println!("  - Has Script: {}", document.header.properties.has_script);
            println!("  - Has Change Tracking: {}", document.header.properties.has_change_tracking);
            println!("  - Mobile Optimized: {}", document.header.properties.is_mobile_optimized);
            println!("\nDocument Info:");
            println!("  - Sections: {}", document.sections.len());
        }
        Err(e) => {
            eprintln!("Failed to parse HWP file: {}", e);
            std::process::exit(1);
        }
    }
}