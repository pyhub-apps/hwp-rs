use hwp_parser::parse;
use hwp_core::HwpError;
use std::fs;
use std::path::Path;

/// Test handling of truncated files
#[test]
fn test_truncated_file_handling() {
    let file_path = Path::new("tests/fixtures/corrupted/truncated.hwp");
    
    if !file_path.exists() {
        // Create a truncated file for testing
        let truncated_data = vec![
            // HWP signature (partial)
            b'H', b'W', b'P', b' ', b'D', b'o',
            // Abruptly cut off
        ];
        
        println!("Testing truncated file handling (in-memory)");
        
        let result = parse(&truncated_data);
        assert!(result.is_err(), "Truncated file should fail to parse");
        
        match result {
            Err(e) => {
                println!("  ✓ Failed gracefully: {:?}", e);
                // Should be a header-related or underflow error
                assert!(
                    format!("{:?}", e).contains("Header") || 
                    format!("{:?}", e).contains("Invalid") ||
                    format!("{:?}", e).contains("BufferUnderflow") ||
                    format!("{:?}", e).contains("Underflow"),
                    "Expected header/underflow error for truncated file, got: {:?}", e
                );
            }
            Ok(_) => panic!("Truncated file should not parse successfully"),
        }
    } else {
        println!("Testing truncated file handling");
        
        let data = fs::read(file_path).expect("Failed to read truncated.hwp");
        let result = parse(&data);
        
        assert!(result.is_err(), "Truncated file should fail to parse");
        
        match result {
            Err(e) => {
                println!("  ✓ Failed gracefully: {:?}", e);
            }
            Ok(_) => panic!("Truncated file should not parse successfully"),
        }
    }
}

/// Test handling of files with bad headers
#[test]
fn test_bad_header_handling() {
    let file_path = Path::new("tests/fixtures/corrupted/bad_header.hwp");
    
    if !file_path.exists() {
        // Create a file with bad header for testing
        let mut bad_header_data = vec![
            // Invalid signature
            b'B', b'A', b'D', b'!', b'H', b'E', b'A', b'D',
            b'E', b'R', b'!', b'!', b'!', b'!', b'!', b'!',
            b'!', 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            // Version
            5, 0, 0, 0,
            // Flags
            0, 0, 0, 0,
        ];
        // Fill rest of header
        bad_header_data.extend(vec![0u8; 216]);
        
        println!("Testing bad header handling (in-memory)");
        
        let result = parse(&bad_header_data);
        assert!(result.is_err(), "Bad header file should fail to parse");
        
        match result {
            Err(e) => {
                println!("  ✓ Failed gracefully: {:?}", e);
                // Should detect invalid signature or header issue
                assert!(
                    format!("{:?}", e).contains("Signature") || 
                    format!("{:?}", e).contains("Invalid") ||
                    format!("{:?}", e).contains("Header"),
                    "Expected signature/header error for bad header, got: {:?}", e
                );
            }
            Ok(_) => panic!("Bad header file should not parse successfully"),
        }
    } else {
        println!("Testing bad header handling");
        
        let data = fs::read(file_path).expect("Failed to read bad_header.hwp");
        let result = parse(&data);
        
        assert!(result.is_err(), "Bad header file should fail to parse");
        
        match result {
            Err(e) => {
                println!("  ✓ Failed gracefully: {:?}", e);
            }
            Ok(_) => panic!("Bad header file should not parse successfully"),
        }
    }
}

/// Test handling of empty files
#[test]
fn test_empty_file_handling() {
    println!("Testing empty file handling");
    
    let empty_data = vec![];
    let result = parse(&empty_data);
    
    assert!(result.is_err(), "Empty file should fail to parse");
    
    match result {
        Err(e) => {
            println!("  ✓ Failed gracefully: {:?}", e);
            // Should be an EOF, insufficient data, or buffer underflow error
            assert!(
                format!("{:?}", e).contains("EOF") || 
                format!("{:?}", e).contains("Insufficient") ||
                format!("{:?}", e).contains("Invalid") ||
                format!("{:?}", e).contains("BufferUnderflow") ||
                format!("{:?}", e).contains("Underflow"),
                "Expected EOF/insufficient data/underflow error for empty file, got: {:?}", e
            );
        }
        Ok(_) => panic!("Empty file should not parse successfully"),
    }
}

/// Test handling of files with invalid version
#[test]
fn test_invalid_version_handling() {
    println!("Testing invalid version handling");
    
    let mut invalid_version_data = vec![
        // Valid HWP signature
        b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u',
        b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l',
        b'e', 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        // Invalid version (255.255.255.255)
        255, 255, 255, 255,
        // Flags
        0, 0, 0, 0,
    ];
    // Fill rest of header
    invalid_version_data.extend(vec![0u8; 216]);
    
    let result = parse(&invalid_version_data);
    
    // Parser might accept this but warn, or reject it
    match result {
        Ok(doc) => {
            println!("  ⚠ Parsed with unusual version: {}.{}.{}.{}", 
                doc.header.version.major,
                doc.header.version.minor,
                doc.header.version.build,
                doc.header.version.revision);
            // Version 255 is technically valid but unusual
            assert_eq!(doc.header.version.major, 255);
        }
        Err(e) => {
            println!("  ✓ Failed gracefully: {:?}", e);
        }
    }
}

/// Test handling of files with corrupted CFB structure
#[test]
fn test_corrupted_cfb_handling() {
    println!("Testing corrupted CFB handling");
    
    // Create a file with CFB signature but corrupted data
    let mut corrupted_cfb_data = vec![
        // CFB signature
        0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1,
        // Some more bytes to make it look like a CFB header
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3E,
        0x00, 0x03, 0x00, 0xFE, 0xFF, 0x09, 0x00, 0x06,
    ];
    
    // Add garbage instead of valid CFB structure
    corrupted_cfb_data.extend(b"This is not a valid CFB container structure!");
    
    let result = parse(&corrupted_cfb_data);
    
    assert!(result.is_err(), "Corrupted CFB should fail to parse");
    
    match result {
        Err(e) => {
            println!("  ✓ Failed gracefully: {:?}", e);
            // Should detect CFB issues or invalid data
            assert!(
                format!("{:?}", e).contains("CFB") || 
                format!("{:?}", e).contains("Container") ||
                format!("{:?}", e).contains("Invalid") ||
                format!("{:?}", e).contains("signature") ||
                format!("{:?}", e).contains("Signature"),
                "Expected CFB/container error for corrupted container, got: {:?}", e
            );
        }
        Ok(_) => panic!("Corrupted CFB should not parse successfully"),
    }
}

/// Test handling of password-protected files
#[test]
fn test_password_protected_handling() {
    println!("Testing password-protected file handling");
    
    // Create a file with password flag set
    let mut password_data = vec![
        // Valid HWP signature
        b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u',
        b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l',
        b'e', 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        // Version (5.0.0.0) - as little-endian u32 where major is high byte
        // u32 = 0x05000000, little-endian bytes = 00 00 00 05
        0, 0, 0, 0x05,
        // Flags (password protected - bit 1 set, value 0x0002)
        0x02, 0, 0, 0,
    ];
    // Fill rest of header
    password_data.extend(vec![0u8; 216]);
    
    let result = parse(&password_data);
    
    // Parser should detect password protection
    match result {
        Ok(doc) => {
            println!("  Version: {}", doc.header.version);
            println!("  Has password: {}", doc.header.has_password());
            assert!(doc.header.has_password(), "Should detect password flag");
            println!("  ✓ Password protection detected");
        }
        Err(e) => {
            // Or it might fail immediately
            println!("  ✓ Failed on password-protected file: {:?}", e);
            assert!(
                format!("{:?}", e).contains("Password") || 
                format!("{:?}", e).contains("Protected") ||
                format!("{:?}", e).contains("Unsupported"),
                "Expected password/protected error, got: {:?}", e
            );
        }
    }
}

/// Test recovery from partial corruption
#[test]
fn test_partial_corruption_recovery() {
    println!("Testing partial corruption recovery");
    
    // This would test if the parser can recover from partial corruption
    // and still extract some useful data
    
    // For now, just verify the parser doesn't panic
    let partially_corrupt = vec![
        // Valid header
        b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u',
        b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l',
        b'e', 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        5, 0, 0, 0, // Version
        0, 0, 0, 0, // Flags
        // Some valid data followed by corruption
    ];
    
    let _ = parse(&partially_corrupt); // Should not panic
    println!("  ✓ No panic on partial corruption");
}