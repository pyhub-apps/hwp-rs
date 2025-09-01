use hwp_parser::parser::doc_info::parse_doc_info;
use hwp_parser::parser::record::RecordParser;

/// Helper function to create a proper record header
fn create_header(tag_id: u16, level: u8, size: usize) -> Vec<u8> {
    // Correct bit layout: tag_id (10 bits) | level (2 bits) | size (20 bits)
    let value = (tag_id as u32) | ((level as u32) << 10) | ((size as u32) << 12);
    value.to_le_bytes().to_vec()
}

#[test]
fn debug_docinfo_parsing() {
    let mut data = Vec::new();
    
    // Simple document properties record
    data.extend(create_header(0x0010, 0, 36));
    // Add exactly 36 bytes of data
    data.extend_from_slice(&[0x01, 0x00]); // section_count: 1
    data.extend_from_slice(&[0x01, 0x00]); // page_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // footnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // endnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // picture_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // table_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // equation_start_number: 1
    data.extend_from_slice(&[0x64, 0x00, 0x00, 0x00]); // total_character_count: 100
    data.extend_from_slice(&[0x05, 0x00, 0x00, 0x00]); // total_page_count: 5
    data.extend_from_slice(&[0x00; 14]); // padding to reach 36 bytes
    
    println!("Test data size: {} bytes", data.len());
    
    // First, let's test that our record parser can handle this
    let mut record_parser = RecordParser::new(&data);
    match record_parser.parse_next_record() {
        Ok(Some(record)) => {
            println!("Record parsed successfully:");
            println!("  Tag ID: 0x{:04X}", record.tag_id);
            println!("  Level: {}", record.level);
            println!("  Size: {}", record.size);
            println!("  Data length: {}", record.data.len());
        }
        Ok(None) => {
            println!("No record found");
        }
        Err(e) => {
            println!("Record parsing failed: {:?}", e);
        }
    }
    
    // Now let's test DocInfo parsing
    match parse_doc_info(&data) {
        Ok(doc_info) => {
            println!("DocInfo parsed successfully:");
            println!("  Section count: {}", doc_info.properties.section_count);
        }
        Err(e) => {
            println!("DocInfo parsing failed: {:?}", e);
        }
    }
}

#[test] 
fn debug_char_shape_parsing() {
    let mut data = Vec::new();
    
    // Create a minimal character shape record
    let mut char_data = Vec::new();
    
    // Face name IDs (7 u16 values)
    for i in 0..7 {
        char_data.extend(&(i as u16).to_le_bytes());
    }
    
    // Ratios (7 u8 values)
    for _ in 0..7 {
        char_data.push(100);
    }
    
    // Character spaces (7 i8 values)
    for _ in 0..7 {
        char_data.push(0);
    }
    
    // Relative sizes (7 u8 values)  
    for _ in 0..7 {
        char_data.push(100);
    }
    
    // Character offsets (7 i8 values)
    for _ in 0..7 {
        char_data.push(0);
    }
    
    // Remaining fields
    char_data.extend(&1200u32.to_le_bytes()); // base_size
    char_data.extend(&0u32.to_le_bytes()); // properties
    char_data.push(0); // shadow_gap_x
    char_data.push(0); // shadow_gap_y
    char_data.extend(&0x000000u32.to_le_bytes()); // text_color
    char_data.extend(&0x000000u32.to_le_bytes()); // underline_color
    char_data.extend(&0xFFFFFFu32.to_le_bytes()); // shade_color
    char_data.extend(&0x808080u32.to_le_bytes()); // shadow_color
    
    println!("Character shape data size: {} bytes", char_data.len());
    
    // Create the record
    data.extend(create_header(0x0015, 0, char_data.len()));
    data.extend_from_slice(&char_data);
    
    println!("Total char shape record size: {} bytes", data.len());
    
    // Parse it
    match parse_doc_info(&data) {
        Ok(doc_info) => {
            println!("DocInfo with char shape parsed successfully:");
            println!("  Char shapes: {}", doc_info.char_shapes.len());
            if !doc_info.char_shapes.is_empty() {
                println!("  Base size: {}", doc_info.char_shapes[0].base_size);
            }
        }
        Err(e) => {
            println!("DocInfo parsing failed: {:?}", e);
        }
    }
}