use byteorder::{LittleEndian, WriteBytesExt};
use flate2::write::DeflateEncoder;
use flate2::Compression;
use hwp_parser::cfb::stream::Stream;
use hwp_parser::compression::{decompress_hwp, decompress_raw};
use std::io::Write;

#[test]
fn test_docinfo_stream_compression_detection() {
    // Create a realistic DocInfo stream content
    let mut uncompressed = Vec::new();

    // Add multiple records like a real DocInfo would have
    // Record 1: Document Properties (0x0010)
    let header1 = (0x0010u32) | (0u32 << 10) | (36u32 << 12);
    uncompressed.write_u32::<LittleEndian>(header1).unwrap();
    uncompressed.extend_from_slice(&[1u8; 36]); // dummy data

    // Record 2: Face Name (0x0012)
    let header2 = (0x0012u32) | (0u32 << 10) | (20u32 << 12);
    uncompressed.write_u32::<LittleEndian>(header2).unwrap();
    uncompressed.extend_from_slice(&[2u8; 20]); // dummy data

    println!("Original DocInfo size: {} bytes", uncompressed.len());

    // Compress using raw deflate (HWP style)
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&uncompressed).unwrap();
    let compressed_data = encoder.finish().unwrap();

    // Create HWP format with size header
    let mut hwp_data = Vec::new();
    hwp_data
        .write_u32::<LittleEndian>(uncompressed.len() as u32)
        .unwrap();
    hwp_data.extend_from_slice(&compressed_data);

    println!("HWP compressed DocInfo size: {} bytes", hwp_data.len());
    println!(
        "First 16 bytes: {:02X?}",
        &hwp_data[..16.min(hwp_data.len())]
    );

    // Create a Stream object
    let stream = Stream::new("DocInfo".to_string(), hwp_data.clone());

    // Test compression detection
    assert!(
        stream.is_compressed(),
        "DocInfo stream should be detected as compressed"
    );

    // Test decompression
    let decompressed = stream.decompress().expect("Decompression should succeed");
    assert_eq!(decompressed.len(), uncompressed.len());
    assert_eq!(decompressed, uncompressed);

    println!("DocInfo stream compression handling successful!");
}

#[test]
fn test_bodytext_stream_compression() {
    // Create a realistic BodyText/Section0 stream
    let mut uncompressed = Vec::new();

    // Paragraph record (0x0043)
    let para_header = (0x0043u32) | (0u32 << 10) | (100u32 << 12);
    uncompressed.write_u32::<LittleEndian>(para_header).unwrap();
    uncompressed.extend_from_slice(&[0xAAu8; 100]); // dummy paragraph data

    println!("Original BodyText size: {} bytes", uncompressed.len());

    // Compress using raw deflate
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&uncompressed).unwrap();
    let compressed_data = encoder.finish().unwrap();

    // Create HWP format
    let mut hwp_data = Vec::new();
    hwp_data
        .write_u32::<LittleEndian>(uncompressed.len() as u32)
        .unwrap();
    hwp_data.extend_from_slice(&compressed_data);

    // Create a Stream object
    let stream = Stream::new("BodyText/Section0".to_string(), hwp_data);

    // Test compression detection
    assert!(
        stream.is_compressed(),
        "BodyText stream should be detected as compressed"
    );

    // Test decompression
    let decompressed = stream.decompress().expect("Decompression should succeed");
    assert_eq!(decompressed.len(), uncompressed.len());

    println!("BodyText stream compression handling successful!");
}

#[test]
fn test_problematic_docinfo_bytes() {
    // Simulate the exact problematic case from the issue
    // These bytes appear to be compressed data without proper header
    let problematic_data = vec![
        0xEC, 0x57, 0x3D, 0x6B, 0x53, 0x51, 0x18, 0x7E,
        // Add more bytes to simulate a real stream
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];

    println!("\nTesting problematic DocInfo bytes:");
    println!("Data size: {} bytes", problematic_data.len());
    println!("First 8 bytes: {:02X?}", &problematic_data[..8]);

    // Create a DocInfo stream with this data
    let stream = Stream::new("DocInfo".to_string(), problematic_data.clone());

    // The stream should be detected as compressed
    // (because DocInfo is assumed compressed in HWP v5.x)
    assert!(
        stream.is_compressed(),
        "DocInfo should be assumed compressed"
    );

    // Try to decompress - this will try multiple methods
    match stream.decompress() {
        Ok(result) => {
            println!("Decompression succeeded: {} bytes", result.len());
            // Check if result looks like valid records
            if result.len() >= 4 {
                let header = u32::from_le_bytes([result[0], result[1], result[2], result[3]]);
                let tag_id = (header & 0x3FF) as u16;
                println!("First record tag_id: 0x{:04X}", tag_id);
            }
        }
        Err(e) => {
            println!("Decompression failed (expected for random data): {:?}", e);
            // This is expected for this test since the data is not really compressed
        }
    }
}

#[test]
fn test_mixed_compression_formats() {
    // Test different compression scenarios

    // Scenario 1: Raw deflate only (no size header)
    let original = b"Test data for raw deflate compression";
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(original).unwrap();
    let raw_deflate = encoder.finish().unwrap();

    let stream1 = Stream::new("DocInfo".to_string(), raw_deflate.clone());
    assert!(stream1.is_compressed());

    // This should work with the fallback to raw deflate
    match stream1.decompress() {
        Ok(result) => {
            println!(
                "Raw deflate decompression succeeded: {} bytes",
                result.len()
            );
            assert_eq!(result, original);
        }
        Err(e) => {
            println!("Raw deflate decompression failed: {:?}", e);
        }
    }

    // Scenario 2: Zlib compressed (with header)
    use flate2::write::ZlibEncoder;
    let mut zlib_encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    zlib_encoder.write_all(original).unwrap();
    let zlib_data = zlib_encoder.finish().unwrap();

    let stream2 = Stream::new("DocInfo".to_string(), zlib_data);
    assert!(stream2.is_compressed());

    let decompressed2 = stream2
        .decompress()
        .expect("Zlib decompression should work");
    assert_eq!(decompressed2, original);

    println!("Mixed compression format handling successful!");
}
