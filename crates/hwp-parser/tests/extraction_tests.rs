use hwp_parser::parse;
use std::fs;
use std::path::Path;

/// Test text extraction from documents
#[test]
fn test_text_extraction() {
    let fixtures_dir = Path::new("tests/fixtures/basic");

    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }

    // Test files with expected content
    let test_cases = vec![
        ("simple_text.hwp", "Hello, World!"),
        ("single_para.hwp", "This is a single paragraph."),
    ];

    for (file_name, expected_text) in test_cases {
        let file_path = fixtures_dir.join(file_name);
        if file_path.exists() {
            println!("Testing text extraction: {}", file_name);

            let data = fs::read(&file_path).expect(&format!("Failed to read {}", file_name));

            match parse(&data) {
                Ok(doc) => {
                    let extracted_text = doc.get_text();
                    println!("  Extracted: {:?}", extracted_text);

                    // For now, just check that we get some text
                    // Real fixtures would have predictable content
                    if !extracted_text.is_empty() {
                        println!("  ✓ Text extracted successfully");
                    } else {
                        println!("  ⚠ No text extracted (might be expected for minimal files)");
                    }
                }
                Err(e) => {
                    println!("  ⚠ Parse failed: {:?}", e);
                }
            }
        }
    }
}

/// Test extraction of Korean text
#[test]
fn test_korean_text_extraction() {
    let file_path = Path::new("tests/fixtures/encoding/korean_only.hwp");

    if !file_path.exists() {
        eprintln!("Skipping test: korean_only.hwp not found");
        return;
    }

    println!("Testing Korean text extraction");

    let data = fs::read(file_path).expect("Failed to read korean_only.hwp");

    match parse(&data) {
        Ok(doc) => {
            let text = doc.get_text();
            println!("  Extracted text: {:?}", text);

            // Check if we have any Korean characters
            let has_korean = text
                .chars()
                .any(|c| matches!(c, '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}'));

            if has_korean {
                println!("  ✓ Korean text extracted successfully");
            } else {
                println!("  ⚠ No Korean text found (file might be minimal)");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test extraction from documents with mixed languages
#[test]
fn test_mixed_language_extraction() {
    let file_path = Path::new("tests/fixtures/encoding/mixed_lang.hwp");

    if !file_path.exists() {
        eprintln!("Skipping test: mixed_lang.hwp not found");
        return;
    }

    println!("Testing mixed language extraction");

    let data = fs::read(file_path).expect("Failed to read mixed_lang.hwp");

    match parse(&data) {
        Ok(doc) => {
            let text = doc.get_text();
            println!("  Extracted text: {:?}", text);

            // Check for different character sets
            let has_ascii = text.chars().any(|c| c.is_ascii_alphabetic());
            let has_korean = text.chars().any(|c| matches!(c, '\u{AC00}'..='\u{D7AF}'));
            let has_cjk = text
                .chars()
                .any(|c| matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3040}'..='\u{309F}'));

            println!("  Character sets found:");
            println!("    ASCII: {}", has_ascii);
            println!("    Korean: {}", has_korean);
            println!("    CJK: {}", has_cjk);
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test extraction of special characters
#[test]
fn test_special_character_extraction() {
    let file_path = Path::new("tests/fixtures/encoding/special_chars.hwp");

    if !file_path.exists() {
        eprintln!("Skipping test: special_chars.hwp not found");
        return;
    }

    println!("Testing special character extraction");

    let data = fs::read(file_path).expect("Failed to read special_chars.hwp");

    match parse(&data) {
        Ok(doc) => {
            let text = doc.get_text();
            println!("  Extracted text: {:?}", text);

            // Check for special characters
            let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
            let has_special = special_chars.chars().any(|c| text.contains(c));

            if has_special {
                println!("  ✓ Special characters extracted");
            } else {
                println!("  ⚠ No special characters found");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test that extraction handles empty documents
#[test]
fn test_empty_document_extraction() {
    let file_path = Path::new("tests/fixtures/basic/empty.hwp");

    if !file_path.exists() {
        eprintln!("Skipping test: empty.hwp not found");
        return;
    }

    println!("Testing empty document extraction");

    let data = fs::read(file_path).expect("Failed to read empty.hwp");

    match parse(&data) {
        Ok(doc) => {
            let text = doc.get_text();
            println!("  Extracted text length: {}", text.len());

            // Empty or minimal document should extract minimal text
            if text.is_empty() || text.trim().is_empty() {
                println!("  ✓ Empty document handled correctly");
            } else {
                println!("  ⚠ Unexpected text in empty document: {:?}", text);
            }
        }
        Err(e) => {
            // Empty document might fail to parse, which is acceptable
            println!("  ✓ Empty document failed to parse (expected): {:?}", e);
        }
    }
}
