use hwp_parser::cfb::{parse_cfb_bytes, CfbContainer};
use std::fs;
use std::io::Cursor;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <hwp-file> [command]", args[0]);
        eprintln!("Commands:");
        eprintln!("  info     - Show CFB container information (default)");
        eprintln!("  list     - List all streams in the container");
        eprintln!("  extract  - Extract all streams to files");
        eprintln!("  read <stream-name> - Read and display a specific stream");
        return Ok(());
    }
    
    let file_path = &args[1];
    let command = args.get(2).map(|s| s.as_str()).unwrap_or("info");
    
    // Read the HWP file
    println!("Reading file: {}", file_path);
    let data = fs::read(file_path)?;
    
    // Check if it's a CFB file
    if data.len() < 8 || &data[0..8] != &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        eprintln!("Error: Not a valid CFB/HWP v5.x file");
        return Ok(());
    }
    
    // Parse the CFB container
    let mut container = parse_cfb_bytes(&data)?;
    let mut cursor = Cursor::new(&data);
    
    match command {
        "info" => show_info(&container),
        "list" => list_streams(&container),
        "extract" => extract_streams(&mut container, &mut cursor, file_path)?,
        "read" => {
            if let Some(stream_name) = args.get(3) {
                read_stream(&mut container, &mut cursor, stream_name)?;
            } else {
                eprintln!("Error: Stream name required for 'read' command");
            }
        }
        _ => eprintln!("Unknown command: {}", command),
    }
    
    Ok(())
}

fn show_info(container: &CfbContainer) {
    println!("\n=== CFB Container Information ===");
    println!("Version: {}.{}", container.header.major_version, container.header.minor_version);
    println!("Sector size: {} bytes", container.header.sector_size());
    println!("Mini sector size: {} bytes", container.header.mini_sector_size());
    println!("Mini stream cutoff: {} bytes", container.header.mini_stream_cutoff_size);
    
    if let Some(root) = container.root_entry() {
        println!("\nRoot entry: {}", root.name);
    }
    
    let streams = container.list_streams();
    let storages = container.list_storages();
    
    println!("\nStatistics:");
    println!("  Streams: {}", streams.len());
    println!("  Storages: {}", storages.len());
    println!("  Total entries: {}", streams.len() + storages.len());
}

fn list_streams(container: &CfbContainer) {
    println!("\n=== Streams in Container ===");
    
    let streams = container.list_streams();
    if streams.is_empty() {
        println!("No streams found");
        return;
    }
    
    println!("{:<30} | {:<10}", "Stream Name", "Type");
    println!("{:-<30}-+-{:-<10}", "", "");
    
    for name in &streams {
        let stream_type = if name.contains("BodyText") {
            "Body"
        } else if name == "FileHeader" {
            "Header"
        } else if name == "DocInfo" {
            "Info"
        } else if name.starts_with("BinData") {
            "Binary"
        } else {
            "Other"
        };
        
        println!("{:<30} | {:<10}", name, stream_type);
    }
    
    println!("\nTotal streams: {}", streams.len());
}

fn extract_streams(
    container: &mut CfbContainer,
    cursor: &mut Cursor<&Vec<u8>>,
    base_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Extracting Streams ===");
    
    // Create output directory
    let base_name = Path::new(base_path)
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("output");
    let output_dir = format!("{}_extracted", base_name);
    fs::create_dir_all(&output_dir)?;
    
    let streams = container.list_streams();
    
    for stream_name in &streams {
        println!("Extracting: {}", stream_name);
        
        let stream = container.read_stream(cursor, stream_name)?;
        
        // Create safe filename
        let safe_name = stream_name.replace('/', "_").replace('\\', "_");
        let file_path = Path::new(&output_dir).join(&safe_name);
        
        // Check if compressed and decompress if needed
        let data = if stream.is_compressed() {
            println!("  Decompressing stream...");
            match stream.decompress() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("  Failed to decompress: {}", e);
                    stream.as_bytes().to_vec()
                }
            }
        } else {
            stream.as_bytes().to_vec()
        };
        
        // Write to file
        fs::write(&file_path, &data)?;
        println!("  Saved to: {}", file_path.display());
        println!("  Size: {} bytes{}", 
            data.len(),
            if stream.is_compressed() { " (decompressed)" } else { "" }
        );
    }
    
    println!("\nExtracted {} streams to '{}'", streams.len(), output_dir);
    Ok(())
}

fn read_stream(
    container: &mut CfbContainer,
    cursor: &mut Cursor<&Vec<u8>>,
    stream_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Reading Stream: {} ===", stream_name);
    
    let stream = container.read_stream(cursor, stream_name)?;
    
    println!("Stream size: {} bytes", stream.size);
    println!("Compressed: {}", stream.is_compressed());
    
    // Get data (decompress if needed)
    let data = if stream.is_compressed() {
        println!("Decompressing stream...");
        stream.decompress()?
    } else {
        stream.as_bytes().to_vec()
    };
    
    println!("\nData ({} bytes):", data.len());
    
    // Display first 500 bytes or entire content if smaller
    let display_len = data.len().min(500);
    
    // Try to display as text if it looks like text
    if is_likely_text(&data[..display_len]) {
        println!("--- Text Content ---");
        let text = String::from_utf8_lossy(&data[..display_len]);
        println!("{}", text);
        if data.len() > display_len {
            println!("... ({} more bytes)", data.len() - display_len);
        }
    } else {
        println!("--- Hex Dump ---");
        print_hex_dump(&data[..display_len]);
        if data.len() > display_len {
            println!("... ({} more bytes)", data.len() - display_len);
        }
    }
    
    Ok(())
}

fn is_likely_text(data: &[u8]) -> bool {
    // Simple heuristic: if most bytes are printable ASCII or common UTF-8, it's likely text
    let printable_count = data.iter()
        .filter(|&&b| (b >= 0x20 && b < 0x7F) || b == b'\n' || b == b'\r' || b == b'\t')
        .count();
    
    printable_count > data.len() * 3 / 4
}

fn print_hex_dump(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}: ", i * 16);
        
        // Hex bytes
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        
        // Padding for incomplete lines
        for _ in chunk.len()..16 {
            print!("   ");
        }
        
        print!(" |");
        
        // ASCII representation
        for byte in chunk {
            if *byte >= 0x20 && *byte < 0x7F {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        
        println!("|");
    }
}