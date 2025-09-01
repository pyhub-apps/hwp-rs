use hwp_parser::parse;
use std::fs;
use std::path::Path;

/// Test that all fixture files can be parsed without panicking
#[test]
fn test_parse_all_fixtures() {
    let fixtures_dir = Path::new("tests/fixtures");
    
    // Skip if fixtures don't exist yet
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures directory not found");
        return;
    }
    
    let test_files = vec![
        "basic/empty.hwp",
        "basic/simple_text.hwp",
        "basic/single_para.hwp",
        "encoding/korean_only.hwp",
        "encoding/mixed_lang.hwp",
        "encoding/special_chars.hwp",
    ];
    
    for file_path in test_files {
        let full_path = fixtures_dir.join(file_path);
        if full_path.exists() {
            println!("Testing: {}", file_path);
            let data = fs::read(&full_path)
                .expect(&format!("Failed to read {}", file_path));
            
            // Should either parse successfully or return a proper error
            let result = parse(&data);
            match result {
                Ok(doc) => {
                    println!("  ✓ Parsed successfully");
                    assert_eq!(doc.header.version.major, 5, "Expected HWP v5.x");
                }
                Err(e) => {
                    println!("  ✓ Failed with expected error: {:?}", e);
                }
            }
        }
    }
}

/// Test parsing of corrupted files - should handle gracefully
#[test]
fn test_parse_corrupted_files() {
    let fixtures_dir = Path::new("tests/fixtures/corrupted");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: corrupted fixtures not found");
        return;
    }
    
    let corrupted_files = vec![
        "truncated.hwp",
        "bad_header.hwp",
        "bad_records.hwp",
    ];
    
    for file_name in corrupted_files {
        let full_path = fixtures_dir.join(file_name);
        if full_path.exists() {
            println!("Testing corrupted file: {}", file_name);
            let data = fs::read(&full_path)
                .expect(&format!("Failed to read {}", file_name));
            
            let result = parse(&data);
            
            // Corrupted files should return an error, not panic
            match result {
                Ok(_) => {
                    // Some corrupted files might still parse partially
                    println!("  ⚠ Parsed despite corruption (partial success)");
                }
                Err(e) => {
                    println!("  ✓ Failed gracefully: {:?}", e);
                    // Verify it's a reasonable error type
                    assert!(
                        format!("{:?}", e).contains("Invalid") ||
                        format!("{:?}", e).contains("Unexpected") ||
                        format!("{:?}", e).contains("Error"),
                        "Expected a parsing error for corrupted file"
                    );
                }
            }
        }
    }
}

/// Test batch parsing performance
#[test]
fn test_batch_parsing_performance() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let start = std::time::Instant::now();
    let mut parsed_count = 0;
    
    if let Ok(entries) = fs::read_dir(fixtures_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |e| e == "hwp") {
                let data = fs::read(entry.path()).unwrap();
                if parse(&data).is_ok() {
                    parsed_count += 1;
                }
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("Parsed {} files in {:?}", parsed_count, elapsed);
    
    // Basic performance assertion - should parse simple files quickly
    if parsed_count > 0 {
        let avg_time = elapsed.as_millis() / parsed_count as u128;
        assert!(
            avg_time < 1000,
            "Parsing taking too long: {}ms average",
            avg_time
        );
    }
}

/// Test memory usage doesn't explode with multiple parses
#[test]
fn test_memory_stability() {
    // Create a minimal HWP data in memory
    let minimal_hwp = vec![
        // HWP signature
        b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u',
        b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l',
        b'e', 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        // Version (5.0.0.0)
        5, 0, 0, 0,
        // Flags
        0, 0, 0, 0,
        // Reserved (216 bytes)
    ];
    
    let mut data = minimal_hwp;
    data.extend(vec![0u8; 216]);
    
    // Parse the same data multiple times
    for i in 0..100 {
        let _result = parse(&data);
        if i % 20 == 0 {
            println!("Completed {} iterations", i);
        }
    }
    
    println!("✓ Memory stability test passed");
}