use hwp_parser::parser::doc_info::parse_doc_info;

/// Helper function to create a proper record header
fn create_header(tag_id: u16, level: u8, size: usize) -> Vec<u8> {
    // Correct bit layout: tag_id (10 bits) | level (2 bits) | size (20 bits)
    let value = (tag_id as u32) | ((level as u32) << 10) | ((size as u32) << 12);
    value.to_le_bytes().to_vec()
}

#[test]
fn debug_multiple_records() {
    let mut data = Vec::new();

    // Record 1: Document Properties (36 bytes)
    data.extend(create_header(0x0010, 0, 36));
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

    println!("After record 1: {} bytes", data.len());

    // Record 2: Face Name for Arial (13 bytes)
    data.extend(create_header(0x0013, 0, 13));
    data.push(0x00); // properties
    data.extend_from_slice(&[0x05, 0x00]); // length: 5
    data.extend_from_slice(
        &"Arial"
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect::<Vec<_>>(),
    );

    println!("After record 2: {} bytes", data.len());

    // Record 3: Simple Character Shape (68 bytes)
    let mut char_data = Vec::new();
    // Face name IDs (7 u16 values) = 14 bytes
    for i in 0..7 {
        char_data.extend(&(i as u16).to_le_bytes());
    }
    // Ratios, spaces, sizes, offsets (7 bytes each) = 28 bytes
    for _ in 0..4 {
        for _ in 0..7 {
            char_data.push(100);
        }
    }
    // Base fields (26 bytes)
    char_data.extend(&1200u32.to_le_bytes()); // base_size (4)
    char_data.extend(&0u32.to_le_bytes()); // properties (4)
    char_data.push(0); // shadow_gap_x (1)
    char_data.push(0); // shadow_gap_y (1)
    char_data.extend(&0x000000u32.to_le_bytes()); // text_color (4)
    char_data.extend(&0x000000u32.to_le_bytes()); // underline_color (4)
    char_data.extend(&0xFFFFFFu32.to_le_bytes()); // shade_color (4)
    char_data.extend(&0x808080u32.to_le_bytes()); // shadow_color (4)

    println!("Character shape data: {} bytes", char_data.len());

    data.extend(create_header(0x0015, 0, char_data.len()));
    data.extend_from_slice(&char_data);

    println!("Total test data: {} bytes", data.len());

    // Parse it
    match parse_doc_info(&data) {
        Ok(doc_info) => {
            println!("✅ DocInfo parsed successfully:");
            println!("  Section count: {}", doc_info.properties.section_count);
            println!("  Face names: {}", doc_info.face_names.len());
            println!("  Char shapes: {}", doc_info.char_shapes.len());
        }
        Err(e) => {
            println!("❌ DocInfo parsing failed: {:?}", e);
        }
    }
}
