use hwp_parser::parser::doc_info::parse_doc_info;

/// Helper function to create a proper record header
fn create_header(tag_id: u16, level: u8, size: usize) -> Vec<u8> {
    let value = (tag_id as u32) | ((level as u32) << 16) | ((size as u32) << 18);
    value.to_le_bytes().to_vec()
}

/// Create sample DocInfo data with multiple records
fn create_sample_doc_info_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Record 1: DOCUMENT_PROPERTIES (tag 0x0010)
    data.extend(create_header(0x0010, 0, 36));
    // Document properties data
    data.extend_from_slice(&[0x03, 0x00]); // section_count: 3
    data.extend_from_slice(&[0x01, 0x00]); // page_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // footnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // endnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // picture_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // table_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // equation_start_number: 1
    data.extend_from_slice(&[0x64, 0x00, 0x00, 0x00]); // total_character_count: 100
    data.extend_from_slice(&[0x05, 0x00, 0x00, 0x00]); // total_page_count: 5
    // Padding to reach 36 bytes
    data.extend_from_slice(&[0x00; 16]);
    
    // Record 2: FACE_NAME (tag 0x0013) - Arial
    data.extend(create_header(0x0013, 0, 13)); // 1 (props) + 2 (len) + 10 (5 chars * 2 bytes)
    data.push(0x00); // properties: 0
    data.extend_from_slice(&[0x05, 0x00]); // string length: 5
    data.extend_from_slice(&"Arial".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    
    // Record 3: FACE_NAME (tag 0x0013) - Times New Roman
    let times_name = "Times New Roman";
    let times_utf16: Vec<u8> = times_name.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let times_record_size = 1 + 2 + times_utf16.len(); // properties + length + string data
    data.extend(create_header(0x0013, 0, times_record_size));
    data.push(0x00); // properties: 0
    data.extend_from_slice(&(times_name.len() as u16).to_le_bytes()); // string length
    data.extend_from_slice(&times_utf16);
    
    // Record 4: CHAR_SHAPE (tag 0x0015) - Basic character shape
    let char_shape_data = create_sample_char_shape_data();
    data.extend(create_header(0x0015, 0, char_shape_data.len()));
    data.extend_from_slice(&char_shape_data);
    
    // Record 5: PARA_SHAPE (tag 0x0019) - Basic paragraph shape
    let para_shape_data = create_sample_para_shape_data();
    data.extend(create_header(0x0019, 0, para_shape_data.len()));
    data.extend_from_slice(&para_shape_data);
    
    // Record 6: STYLE (tag 0x001A) - Normal style
    let style_data = create_sample_style_data();
    data.extend(create_header(0x001A, 0, style_data.len()));
    data.extend_from_slice(&style_data);
    
    // Record 7: BORDER_FILL (tag 0x0014) - Basic border
    let border_fill_data = create_sample_border_fill_data();
    data.extend(create_header(0x0014, 0, border_fill_data.len()));
    data.extend_from_slice(&border_fill_data);
    
    // Record 8: ID_MAPPINGS (tag 0x0011)
    data.extend(create_header(0x0011, 0, 12)); // count (4) + 2 mappings (8) = 12 bytes
    data.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // count: 2
    data.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]); // mapping 1: 16
    data.extend_from_slice(&[0x20, 0x00, 0x00, 0x00]); // mapping 2: 32
    
    data
}

fn create_sample_char_shape_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Face name IDs (7 u16 values)
    for i in 0..7 {
        data.extend_from_slice(&(i as u16).to_le_bytes());
    }
    
    // Ratios (7 u8 values)
    for i in 0..7 {
        data.push(100);
    }
    
    // Character spaces (7 i8 values)
    for _ in 0..7 {
        data.push(0);
    }
    
    // Relative sizes (7 u8 values)  
    for _ in 0..7 {
        data.push(100);
    }
    
    // Character offsets (7 i8 values)
    for _ in 0..7 {
        data.push(0);
    }
    
    // Remaining fields
    data.extend_from_slice(&1200u32.to_le_bytes()); // base_size
    data.extend_from_slice(&0u32.to_le_bytes()); // properties
    data.push(0); // shadow_gap_x
    data.push(0); // shadow_gap_y
    data.extend_from_slice(&0x000000u32.to_le_bytes()); // text_color (black)
    data.extend_from_slice(&0x000000u32.to_le_bytes()); // underline_color
    data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // shade_color (white)
    data.extend_from_slice(&0x808080u32.to_le_bytes()); // shadow_color (gray)
    
    data
}

fn create_sample_para_shape_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    data.extend_from_slice(&0u32.to_le_bytes()); // properties1
    data.extend_from_slice(&0i32.to_le_bytes()); // left_margin
    data.extend_from_slice(&0i32.to_le_bytes()); // right_margin
    data.extend_from_slice(&0i32.to_le_bytes()); // indent
    data.extend_from_slice(&0i32.to_le_bytes()); // prev_spacing
    data.extend_from_slice(&0i32.to_le_bytes()); // next_spacing
    data.extend_from_slice(&1600i32.to_le_bytes()); // line_spacing
    data.extend_from_slice(&0u16.to_le_bytes()); // tab_def_id
    data.extend_from_slice(&0u16.to_le_bytes()); // numbering_id
    data.extend_from_slice(&0u16.to_le_bytes()); // border_fill_id
    data.extend_from_slice(&0i16.to_le_bytes()); // border_offset_left
    data.extend_from_slice(&0i16.to_le_bytes()); // border_offset_right
    data.extend_from_slice(&0i16.to_le_bytes()); // border_offset_top
    data.extend_from_slice(&0i16.to_le_bytes()); // border_offset_bottom
    data.extend_from_slice(&0u32.to_le_bytes()); // properties2
    data.extend_from_slice(&0u32.to_le_bytes()); // properties3
    data.extend_from_slice(&0u32.to_le_bytes()); // line_spacing_type
    
    data
}

fn create_sample_style_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Korean name: "바탕"
    let korean_name = "바탕";
    let korean_utf16: Vec<u8> = korean_name.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    data.extend_from_slice(&(korean_name.len() as u16).to_le_bytes());
    data.extend_from_slice(&korean_utf16);
    
    // English name: "Normal"
    let english_name = "Normal";
    let english_utf16: Vec<u8> = english_name.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    data.extend_from_slice(&(english_name.len() as u16).to_le_bytes());
    data.extend_from_slice(&english_utf16);
    
    data.push(0x01); // properties
    data.push(0xFF); // next_style_id
    data.extend_from_slice(&0x0412u16.to_le_bytes()); // lang_id (Korean)
    data.extend_from_slice(&0u16.to_le_bytes()); // para_shape_id
    data.extend_from_slice(&0u16.to_le_bytes()); // char_shape_id
    
    data
}

fn create_sample_border_fill_data() -> Vec<u8> {
    let mut data = Vec::new();
    
    data.extend_from_slice(&0u16.to_le_bytes()); // properties
    
    // All borders (left, right, top, bottom, diagonal) - no borders
    for _ in 0..5 {
        data.push(0); // line_type: none
        data.push(0); // thickness: 0
        data.extend_from_slice(&0u32.to_le_bytes()); // color: none
    }
    
    data.push(0); // fill_type: none
    
    data
}

#[test]
fn test_complete_doc_info_parsing() {
    let data = create_sample_doc_info_data();
    let doc_info = parse_doc_info(&data).unwrap();
    
    // Check document properties
    assert_eq!(doc_info.properties.section_count, 3);
    assert_eq!(doc_info.properties.total_character_count, 100);
    assert_eq!(doc_info.properties.total_page_count, 5);
    
    // Check face names
    assert_eq!(doc_info.face_names.len(), 2);
    assert_eq!(doc_info.face_names[0].name, "Arial");
    assert_eq!(doc_info.face_names[1].name, "Times New Roman");
    
    // Check character shapes
    assert_eq!(doc_info.char_shapes.len(), 1);
    assert_eq!(doc_info.char_shapes[0].base_size, 1200);
    assert_eq!(doc_info.char_shapes[0].face_name_ids.len(), 7);
    
    // Check paragraph shapes
    assert_eq!(doc_info.para_shapes.len(), 1);
    assert_eq!(doc_info.para_shapes[0].line_spacing, 1600);
    
    // Check styles
    assert_eq!(doc_info.styles.len(), 1);
    assert_eq!(doc_info.styles[0].name, "바탕");
    assert_eq!(doc_info.styles[0].english_name, "Normal");
    assert_eq!(doc_info.styles[0].lang_id, 0x0412);
    
    // Check border fills
    assert_eq!(doc_info.border_fills.len(), 1);
    assert_eq!(doc_info.border_fills[0].fill_type, 0);
}

#[test]
fn test_empty_doc_info_parsing() {
    let data = Vec::new();
    let doc_info = parse_doc_info(&data).unwrap();
    
    // Should have default values
    assert_eq!(doc_info.properties.section_count, 0);
    assert_eq!(doc_info.face_names.len(), 0);
    assert_eq!(doc_info.char_shapes.len(), 0);
    assert_eq!(doc_info.para_shapes.len(), 0);
    assert_eq!(doc_info.styles.len(), 0);
    assert_eq!(doc_info.border_fills.len(), 0);
}

#[test]
fn test_malformed_doc_info_handling() {
    // Test with incomplete record
    let data = vec![
        0x10, 0x00, 0x10, 0x00, // header: tag=0x0010, level=0, size=16
        0x01, 0x02, // Only 2 bytes instead of 16
    ];
    
    let result = parse_doc_info(&data);
    assert!(result.is_err()); // Should return error for malformed data
}

#[test]
fn test_unknown_record_handling() {
    let mut data = Vec::new();
    
    // Valid document properties record
    data.extend(create_header(0x0010, 0, 36)); // header
    // Add 36 bytes of data
    data.extend_from_slice(&[0x01, 0x00]); // section_count: 1
    data.extend_from_slice(&[0x00; 34]); // padding
    
    // Unknown record type
    data.extend(create_header(0xFFFF, 0, 4)); // header: unknown tag, size=4
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // unknown data
    
    let doc_info = parse_doc_info(&data).unwrap();
    
    // Should parse the valid record and skip the unknown one
    assert_eq!(doc_info.properties.section_count, 1);
}

#[test]
fn test_multiple_records_same_type() {
    let mut data = Vec::new();
    
    // First FACE_NAME record - Arial
    data.extend(create_header(0x0013, 0, 13)); // header: 1 (props) + 2 (len) + 10 (5 chars * 2 bytes)
    data.push(0x00); // properties
    data.extend_from_slice(&[0x05, 0x00]); // length: 5
    data.extend_from_slice(&"Arial".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    
    // Second FACE_NAME record - Consolas
    data.extend(create_header(0x0013, 0, 19)); // header: 1 (props) + 2 (len) + 16 (8 chars * 2 bytes)
    data.push(0x00); // properties
    data.extend_from_slice(&[0x08, 0x00]); // length: 8
    data.extend_from_slice(&"Consolas".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    
    let doc_info = parse_doc_info(&data).unwrap();
    
    // Should have both face names
    assert_eq!(doc_info.face_names.len(), 2);
    assert_eq!(doc_info.face_names[0].name, "Arial");
    assert_eq!(doc_info.face_names[1].name, "Consolas");
}

#[test]
fn test_extended_size_record() {
    let mut data = Vec::new();
    
    // Create a record with extended size
    data.extend(create_header(0x0010, 0, 0xFFF)); // header with extended size marker
    data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // actual size: 8 bytes
    // Document properties data (minimal)
    data.extend_from_slice(&[0x01, 0x00]); // section_count: 1
    data.extend_from_slice(&[0x00; 6]); // padding
    
    let doc_info = parse_doc_info(&data).unwrap();
    assert_eq!(doc_info.properties.section_count, 1);
}