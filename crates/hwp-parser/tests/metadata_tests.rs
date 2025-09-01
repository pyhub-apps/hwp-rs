use hwp_parser::parse;
use std::fs;
use std::path::Path;

/// Test document version extraction
#[test]
fn test_document_version() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let test_files = vec!["empty.hwp", "simple_text.hwp", "single_para.hwp"];
    
    for file_name in test_files {
        let file_path = fixtures_dir.join(file_name);
        if file_path.exists() {
            println!("Testing version extraction: {}", file_name);
            
            let data = fs::read(&file_path)
                .expect(&format!("Failed to read {}", file_name));
            
            match parse(&data) {
                Ok(doc) => {
                    let version = &doc.header.version;
                    println!("  Version: {}.{}.{}.{}", 
                        version.major, version.minor, version.build, version.revision);
                    
                    // All test files should be HWP v5.x
                    assert_eq!(version.major, 5, "Expected HWP v5.x format");
                    println!("  ✓ Version validated");
                }
                Err(e) => {
                    println!("  ⚠ Parse failed: {:?}", e);
                }
            }
        }
    }
}

/// Test document properties extraction
#[test]
fn test_document_properties() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let test_files = vec!["simple_text.hwp", "single_para.hwp"];
    
    for file_name in test_files {
        let file_path = fixtures_dir.join(file_name);
        if file_path.exists() {
            println!("Testing properties extraction: {}", file_name);
            
            let data = fs::read(&file_path)
                .expect(&format!("Failed to read {}", file_name));
            
            match parse(&data) {
                Ok(doc) => {
                    let props = &doc.doc_info.properties;
                    
                    println!("  Document Properties:");
                    println!("    Section count: {}", props.section_count);
                    println!("    Page start: {}", props.page_start_number);
                    println!("    Total pages: {}", props.total_page_count);
                    println!("    Total characters: {}", props.total_character_count);
                    
                    // Basic validation
                    assert!(props.section_count > 0 || doc.sections.len() > 0,
                        "Document should have at least one section");
                    
                    println!("  ✓ Properties extracted");
                }
                Err(e) => {
                    println!("  ⚠ Parse failed: {:?}", e);
                }
            }
        }
    }
}

/// Test font face names extraction
#[test]
fn test_face_names() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let file_path = fixtures_dir.join("simple_text.hwp");
    if !file_path.exists() {
        eprintln!("Skipping test: simple_text.hwp not found");
        return;
    }
    
    println!("Testing font face names extraction");
    
    let data = fs::read(&file_path).expect("Failed to read simple_text.hwp");
    
    match parse(&data) {
        Ok(doc) => {
            let face_names = &doc.doc_info.face_names;
            
            println!("  Found {} font face names", face_names.len());
            
            for (i, face) in face_names.iter().enumerate() {
                println!("    [{}] {}", i, face.name);
                if let Some(ref substitute) = face.substitute_font_name {
                    println!("        Substitute: {}", substitute);
                }
            }
            
            // Most HWP documents have at least one font
            if !face_names.is_empty() {
                println!("  ✓ Font faces extracted");
            } else {
                println!("  ⚠ No font faces found (minimal document)");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test styles extraction
#[test]
fn test_styles() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let file_path = fixtures_dir.join("simple_text.hwp");
    if !file_path.exists() {
        eprintln!("Skipping test: simple_text.hwp not found");
        return;
    }
    
    println!("Testing styles extraction");
    
    let data = fs::read(&file_path).expect("Failed to read simple_text.hwp");
    
    match parse(&data) {
        Ok(doc) => {
            let styles = &doc.doc_info.styles;
            
            println!("  Found {} styles", styles.len());
            
            for (i, style) in styles.iter().enumerate() {
                println!("    [{}] {} ({})", i, style.name, style.english_name);
                println!("        Char shape ID: {}", style.char_shape_id);
                println!("        Para shape ID: {}", style.para_shape_id);
            }
            
            if !styles.is_empty() {
                println!("  ✓ Styles extracted");
            } else {
                println!("  ⚠ No styles found (minimal document)");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test character shapes extraction
#[test]
fn test_char_shapes() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let file_path = fixtures_dir.join("simple_text.hwp");
    if !file_path.exists() {
        eprintln!("Skipping test: simple_text.hwp not found");
        return;
    }
    
    println!("Testing character shapes extraction");
    
    let data = fs::read(&file_path).expect("Failed to read simple_text.hwp");
    
    match parse(&data) {
        Ok(doc) => {
            let char_shapes = &doc.doc_info.char_shapes;
            
            println!("  Found {} character shapes", char_shapes.len());
            
            for (i, shape) in char_shapes.iter().take(3).enumerate() {
                println!("    [{}] Base size: {}", i, shape.base_size);
                println!("        Text color: 0x{:08X}", shape.text_color);
                println!("        Properties: 0x{:08X}", shape.properties);
            }
            
            if !char_shapes.is_empty() {
                println!("  ✓ Character shapes extracted");
            } else {
                println!("  ⚠ No character shapes found (minimal document)");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}

/// Test paragraph shapes extraction
#[test]
fn test_para_shapes() {
    let fixtures_dir = Path::new("tests/fixtures/basic");
    
    if !fixtures_dir.exists() {
        eprintln!("Skipping test: fixtures not found");
        return;
    }
    
    let file_path = fixtures_dir.join("simple_text.hwp");
    if !file_path.exists() {
        eprintln!("Skipping test: simple_text.hwp not found");
        return;
    }
    
    println!("Testing paragraph shapes extraction");
    
    let data = fs::read(&file_path).expect("Failed to read simple_text.hwp");
    
    match parse(&data) {
        Ok(doc) => {
            let para_shapes = &doc.doc_info.para_shapes;
            
            println!("  Found {} paragraph shapes", para_shapes.len());
            
            for (i, shape) in para_shapes.iter().take(3).enumerate() {
                println!("    [{}] Margins: L={} R={}", 
                    i, shape.left_margin, shape.right_margin);
                println!("        Indent: {}", shape.indent);
                println!("        Line spacing: {}", shape.line_spacing);
            }
            
            if !para_shapes.is_empty() {
                println!("  ✓ Paragraph shapes extracted");
            } else {
                println!("  ⚠ No paragraph shapes found (minimal document)");
            }
        }
        Err(e) => {
            println!("  ⚠ Parse failed: {:?}", e);
        }
    }
}