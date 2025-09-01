use hwp_parser::compression::{decompress_hwp, is_hwp_compressed, decompress_raw};
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};

#[test]
fn test_hwp_compression_with_real_like_data() {
    // Simulate a real DocInfo record structure
    let mut uncompressed_data = Vec::new();
    
    // Document Properties record (tag_id=0x0010)
    // Record header: tag_id (10 bits) | level (2 bits) | size (20 bits)
    let tag_id: u16 = 0x0010;
    let level: u8 = 0;
    let record_size: u32 = 36;
    let header = (tag_id as u32) | ((level as u32) << 10) | ((record_size as u32) << 12);
    uncompressed_data.write_u32::<LittleEndian>(header).unwrap();
    
    // Record data (36 bytes)
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // section_count
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // page_start_number
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // footnote_start_number
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // endnote_start_number
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // picture_start_number
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // table_start_number
    uncompressed_data.write_u16::<LittleEndian>(1).unwrap(); // equation_start_number
    uncompressed_data.write_u32::<LittleEndian>(100).unwrap(); // total_character_count
    uncompressed_data.write_u32::<LittleEndian>(5).unwrap(); // total_page_count
    uncompressed_data.extend_from_slice(&[0u8; 14]); // padding
    
    println!("Uncompressed data size: {} bytes", uncompressed_data.len());
    println!("First 16 bytes (uncompressed): {:02X?}", &uncompressed_data[..16.min(uncompressed_data.len())]);
    
    // Compress with raw deflate (like HWP does)
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&uncompressed_data).unwrap();
    let compressed_data = encoder.finish().unwrap();
    
    println!("Compressed data size: {} bytes", compressed_data.len());
    println!("First 16 bytes (compressed): {:02X?}", &compressed_data[..16.min(compressed_data.len())]);
    
    // Create HWP format: [4-byte uncompressed size][compressed data]
    let mut hwp_format_data = Vec::new();
    hwp_format_data.write_u32::<LittleEndian>(uncompressed_data.len() as u32).unwrap();
    hwp_format_data.extend_from_slice(&compressed_data);
    
    println!("\nHWP format data size: {} bytes", hwp_format_data.len());
    println!("First 16 bytes: {:02X?}", &hwp_format_data[..16.min(hwp_format_data.len())]);
    
    // Test detection
    assert!(is_hwp_compressed(&hwp_format_data), "Should detect HWP compression");
    
    // Test decompression
    let decompressed = decompress_hwp(&hwp_format_data).expect("Decompression should succeed");
    assert_eq!(decompressed.len(), uncompressed_data.len(), "Decompressed size should match");
    assert_eq!(decompressed, uncompressed_data, "Decompressed data should match original");
    
    println!("\nDecompression successful!");
    println!("Decompressed size: {} bytes", decompressed.len());
}

#[test]
fn test_raw_deflate_without_header() {
    // Test case simulating the problematic bytes [EC, 57, 3D, 6B, 53, 51, 18, 7E]
    // These might be raw deflate data without any header
    
    // Create some test data that might compress to similar patterns
    let original = b"HWP Document Information Section with various records and metadata";
    
    // Compress with raw deflate
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original).unwrap();
    let compressed = encoder.finish().unwrap();
    
    println!("Testing raw deflate:");
    println!("Original size: {} bytes", original.len());
    println!("Compressed size: {} bytes", compressed.len());
    println!("First 8 bytes of compressed: {:02X?}", &compressed[..8.min(compressed.len())]);
    
    // Test raw decompression
    let decompressed = decompress_raw(&compressed).expect("Raw deflate decompression should work");
    assert_eq!(decompressed, original);
    
    println!("Raw deflate decompression successful!");
}

#[test]
fn test_problematic_bytes() {
    // Simulate the exact problematic case from the issue
    let problematic_bytes = vec![0xEC, 0x57, 0x3D, 0x6B, 0x53, 0x51, 0x18, 0x7E];
    
    println!("\nTesting problematic bytes: {:02X?}", problematic_bytes);
    
    // Interpretation 1: These are the first 8 bytes including a 4-byte size header
    let size_from_header = u32::from_le_bytes([0xEC, 0x57, 0x3D, 0x6B]);
    println!("If interpreted as size header: {} bytes", size_from_header);
    println!("That's {} MB - clearly wrong!", size_from_header / 1024 / 1024);
    
    // Interpretation 2: These are pure compressed data (no header)
    println!("\nTrying as raw deflate data...");
    match decompress_raw(&problematic_bytes) {
        Ok(result) => {
            println!("Raw deflate succeeded: {} bytes decompressed", result.len());
            println!("Content: {:?}", String::from_utf8_lossy(&result));
        }
        Err(e) => {
            println!("Raw deflate failed (expected): {:?}", e);
        }
    }
    
    // This test shows that the bytes are likely compressed data,
    // not a size header followed by data
}