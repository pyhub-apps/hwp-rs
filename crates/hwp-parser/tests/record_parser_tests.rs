use hwp_parser::parser::record::{RecordParser, RecordDataParser};
use hwp_parser::parser::doc_info_records::*;
use hwp_core::constants::tag_id::doc_info;
use hwp_core::models::record::{Record, RecordHeader};

#[test]
fn test_record_header_parsing() {
    // Test normal size record header
    // Correct bit layout: tag_id (10 bits) | level (2 bits) | size (20 bits)
    // 0x0010 (tag) | 0 (level) << 10 | 4 (size) << 12
    let value = (0x0010_u32) | (0_u32 << 10) | (4_u32 << 12);
    let header_bytes = value.to_le_bytes();
    let header = RecordHeader::from_bytes(header_bytes);
    
    assert_eq!(header.tag_id(), 0x0010);
    assert_eq!(header.level(), 0);
    assert_eq!(header.size(), 4);
    assert!(!header.has_extended_size());
}

#[test]
fn test_extended_size_record_header() {
    // Test extended size record header
    // Extended size marker is 0xFFFFF (20 bits all set to 1)
    // Bit layout: tag_id (10 bits) | level (2 bits) | size (20 bits)
    // 0x0010 (tag) | 0 (level) << 10 | 0xFFFFF (size) << 12
    let value = (0x0010_u32) | (0_u32 << 10) | (0xFFFFF_u32 << 12);
    let header_bytes = value.to_le_bytes();
    let header = RecordHeader::from_bytes(header_bytes);
    
    assert_eq!(header.tag_id(), 0x0010);
    assert_eq!(header.level(), 0);
    assert_eq!(header.size(), 0xFFFFF);
    assert!(header.has_extended_size());
}

#[test]
fn test_record_parsing() {
    // Create proper headers with correct bit layout
    let doc_props_header = ((0x0010_u32) | (0_u32 << 10) | (36_u32 << 12)).to_le_bytes();
    let face_name_header = ((0x0013_u32) | (0_u32 << 10) | (13_u32 << 12)).to_le_bytes();
    
    let mut data = vec![];
    
    // Record 1: DOCUMENT_PROPERTIES
    data.extend_from_slice(&doc_props_header); // header: tag=0x0010, level=0, size=36
    // Document properties data (36 bytes)
    data.extend_from_slice(&[
        0x03, 0x00, // section_count: 3
        0x01, 0x00, // page_start_number: 1
        0x01, 0x00, // footnote_start_number: 1
        0x01, 0x00, // endnote_start_number: 1
        0x01, 0x00, // picture_start_number: 1
        0x01, 0x00, // table_start_number: 1
        0x01, 0x00, // equation_start_number: 1
        0x64, 0x00, 0x00, 0x00, // total_character_count: 100
        0x05, 0x00, 0x00, 0x00, // total_page_count: 5
        // Padding to reach 36 bytes (we have 22 bytes of data, need 14 more)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    
    // Record 2: FACE_NAME  
    data.extend_from_slice(&face_name_header); // header: tag=0x0013, level=0, size=13
    data.extend_from_slice(&[
        0x00, // properties: 0
        0x05, 0x00, // string length: 5
        0x41, 0x00, // 'A'
        0x72, 0x00, // 'r'
        0x69, 0x00, // 'i'
        0x61, 0x00, // 'a'
        0x6C, 0x00, // 'l'
    ]);
    
    let mut parser = RecordParser::new(&data);
    let records = parser.parse_all_records().unwrap();
    
    assert_eq!(records.len(), 2);
    
    // Check first record (DOCUMENT_PROPERTIES)
    assert_eq!(records[0].tag_id, doc_info::DOCUMENT_PROPERTIES);
    assert_eq!(records[0].level, 0);
    assert_eq!(records[0].size, 36);
    assert_eq!(records[0].data.len(), 36);
    
    // Check second record (FACE_NAME)
    assert_eq!(records[1].tag_id, doc_info::FACE_NAME);
    assert_eq!(records[1].level, 0);
    assert_eq!(records[1].size, 13);
    assert_eq!(records[1].data.len(), 13);
}

#[test]
fn test_document_properties_parsing() {
    let data = vec![
        0x03, 0x00, // section_count: 3
        0x01, 0x00, // page_start_number: 1
        0x01, 0x00, // footnote_start_number: 1
        0x01, 0x00, // endnote_start_number: 1
        0x01, 0x00, // picture_start_number: 1
        0x01, 0x00, // table_start_number: 1
        0x01, 0x00, // equation_start_number: 1
        0x64, 0x00, 0x00, 0x00, // total_character_count: 100
        0x05, 0x00, 0x00, 0x00, // total_page_count: 5
    ];
    
    let props = parse_document_properties(&data).unwrap();
    assert_eq!(props.section_count, 3);
    assert_eq!(props.page_start_number, 1);
    assert_eq!(props.footnote_start_number, 1);
    assert_eq!(props.endnote_start_number, 1);
    assert_eq!(props.picture_start_number, 1);
    assert_eq!(props.table_start_number, 1);
    assert_eq!(props.equation_start_number, 1);
    assert_eq!(props.total_character_count, 100);
    assert_eq!(props.total_page_count, 5);
}

#[test]
fn test_face_name_parsing_basic() {
    let data = vec![
        0x00, // properties: 0 (no optional data)
        0x05, 0x00, // string length: 5
        0x41, 0x00, // 'A'
        0x72, 0x00, // 'r'
        0x69, 0x00, // 'i'
        0x61, 0x00, // 'a'
        0x6C, 0x00, // 'l'
    ];
    
    let face_name = parse_face_name(&data).unwrap();
    assert_eq!(face_name.properties, 0);
    assert_eq!(face_name.name, "Arial");
    assert!(face_name.substitute_font_type.is_none());
    assert!(face_name.substitute_font_name.is_none());
    assert!(face_name.base_font_name.is_none());
}

#[test]
fn test_face_name_parsing_with_type_info() {
    let data = vec![
        0x01, // properties: 1 (has type info)
        0x05, 0x00, // string length: 5
        0x41, 0x00, // 'A'
        0x72, 0x00, // 'r'
        0x69, 0x00, // 'i'
        0x61, 0x00, // 'a'
        0x6C, 0x00, // 'l'
        // Type info (10 bytes)
        0x02, // family
        0x01, // serif
        0x04, // weight
        0x00, // proportion
        0x00, // contrast
        0x00, // stroke_variation
        0x00, // arm_style
        0x00, // letter_form
        0x00, // midline
        0x00, // x_height
    ];
    
    let face_name = parse_face_name(&data).unwrap();
    assert_eq!(face_name.properties, 1);
    assert_eq!(face_name.name, "Arial");
    assert_eq!(face_name.type_info.family, 2);
    assert_eq!(face_name.type_info.serif, 1);
    assert_eq!(face_name.type_info.weight, 4);
}

#[test]
fn test_char_shape_parsing() {
    let mut data = Vec::new();
    
    // Face name IDs (7 u16 values)
    for i in 0..7 {
        data.extend_from_slice(&(i as u16).to_le_bytes());
    }
    
    // Ratios (7 u8 values)
    for i in 0..7 {
        data.push(100 + i);
    }
    
    // Character spaces (7 i8 values)
    for i in 0..7 {
        data.push(i);
    }
    
    // Relative sizes (7 u8 values)  
    for i in 0..7 {
        data.push(100 + i);
    }
    
    // Character offsets (7 i8 values)
    for i in 0..7 {
        data.push(i);
    }
    
    // Remaining fields
    data.extend_from_slice(&1200u32.to_le_bytes()); // base_size
    data.extend_from_slice(&0x12345678u32.to_le_bytes()); // properties
    data.push(1); // shadow_gap_x
    data.push(2); // shadow_gap_y
    data.extend_from_slice(&0xFF0000u32.to_le_bytes()); // text_color (red)
    data.extend_from_slice(&0x00FF00u32.to_le_bytes()); // underline_color (green)
    data.extend_from_slice(&0x0000FFu32.to_le_bytes()); // shade_color (blue)
    data.extend_from_slice(&0x888888u32.to_le_bytes()); // shadow_color (gray)
    data.extend_from_slice(&123u16.to_le_bytes()); // border_fill_id
    
    let char_shape = parse_char_shape(&data).unwrap();
    
    // Verify face name IDs
    for i in 0..7 {
        assert_eq!(char_shape.face_name_ids[i], i as u16);
    }
    
    // Verify ratios
    for i in 0..7 {
        assert_eq!(char_shape.ratios[i], 100 + i as u8);
    }
    
    assert_eq!(char_shape.base_size, 1200);
    assert_eq!(char_shape.properties, 0x12345678);
    assert_eq!(char_shape.shadow_gap_x, 1);
    assert_eq!(char_shape.shadow_gap_y, 2);
    assert_eq!(char_shape.text_color, 0xFF0000);
    assert_eq!(char_shape.underline_color, 0x00FF00);
    assert_eq!(char_shape.shade_color, 0x0000FF);
    assert_eq!(char_shape.shadow_color, 0x888888);
    assert_eq!(char_shape.border_fill_id, Some(123));
}

#[test]
fn test_para_shape_parsing() {
    let data = vec![
        0x78, 0x56, 0x34, 0x12, // properties1: 0x12345678
        0x00, 0x10, 0x00, 0x00, // left_margin: 4096
        0x00, 0x08, 0x00, 0x00, // right_margin: 2048
        0x00, 0x02, 0x00, 0x00, // indent: 512
        0x00, 0x01, 0x00, 0x00, // prev_spacing: 256
        0x00, 0x01, 0x00, 0x00, // next_spacing: 256
        0x40, 0x06, 0x00, 0x00, // line_spacing: 1600
        0x01, 0x00, // tab_def_id: 1
        0x02, 0x00, // numbering_id: 2
        0x03, 0x00, // border_fill_id: 3
        0x10, 0x00, // border_offset_left: 16
        0x10, 0x00, // border_offset_right: 16
        0x08, 0x00, // border_offset_top: 8
        0x08, 0x00, // border_offset_bottom: 8
        0x87, 0x65, 0x43, 0x21, // properties2: 0x21436587
        0xEF, 0xCD, 0xAB, 0x89, // properties3: 0x89ABCDEF
        0x01, 0x00, 0x00, 0x00, // line_spacing_type: 1
    ];
    
    let para_shape = parse_para_shape(&data).unwrap();
    assert_eq!(para_shape.properties1, 0x12345678);
    assert_eq!(para_shape.left_margin, 4096);
    assert_eq!(para_shape.right_margin, 2048);
    assert_eq!(para_shape.indent, 512);
    assert_eq!(para_shape.prev_spacing, 256);
    assert_eq!(para_shape.next_spacing, 256);
    assert_eq!(para_shape.line_spacing, 1600);
    assert_eq!(para_shape.tab_def_id, 1);
    assert_eq!(para_shape.numbering_id, 2);
    assert_eq!(para_shape.border_fill_id, 3);
    assert_eq!(para_shape.border_offset_left, 16);
    assert_eq!(para_shape.border_offset_right, 16);
    assert_eq!(para_shape.border_offset_top, 8);
    assert_eq!(para_shape.border_offset_bottom, 8);
    assert_eq!(para_shape.properties2, 0x21436587);
    assert_eq!(para_shape.properties3, 0x89ABCDEF);
    assert_eq!(para_shape.line_spacing_type, 1);
}

#[test]
fn test_style_parsing() {
    let data = vec![
        // Korean name: "바탕"  
        0x02, 0x00, // name length: 2
        0x14, 0xBC, // '바' (correct UTF-16LE)
        0xD5, 0xD0, // '탕' (correct UTF-16LE)
        // English name: "Normal"
        0x06, 0x00, // english_name length: 6
        0x4E, 0x00, // 'N'
        0x6F, 0x00, // 'o'
        0x72, 0x00, // 'r'
        0x6D, 0x00, // 'm'
        0x61, 0x00, // 'a'
        0x6C, 0x00, // 'l'
        0x01,       // properties: 1
        0xFF,       // next_style_id: 255
        0x12, 0x04, // lang_id: 0x0412 (Korean)
        0x05, 0x00, // para_shape_id: 5
        0x03, 0x00, // char_shape_id: 3
    ];
    
    let style = parse_style(&data).unwrap();
    assert_eq!(style.name, "바탕");
    assert_eq!(style.english_name, "Normal");
    assert_eq!(style.properties, 1);
    assert_eq!(style.next_style_id, 255);
    assert_eq!(style.lang_id, 0x0412);
    assert_eq!(style.para_shape_id, 5);
    assert_eq!(style.char_shape_id, 3);
}

#[test]
fn test_border_fill_parsing() {
    let data = vec![
        0x01, 0x00, // properties: 1
        // Left border
        0x01, // line_type
        0x02, // thickness
        0x00, 0x00, 0xFF, 0x00, // color: 0x00FF0000 (red)
        // Right border
        0x02, // line_type
        0x03, // thickness
        0x00, 0xFF, 0x00, 0x00, // color: 0x0000FF00 (green)
        // Top border
        0x03, // line_type
        0x04, // thickness
        0xFF, 0x00, 0x00, 0x00, // color: 0x000000FF (blue)
        // Bottom border
        0x04, // line_type
        0x05, // thickness
        0x88, 0x88, 0x88, 0x00, // color: 0x00888888 (gray)
        // Diagonal border
        0x00, // line_type: none
        0x00, // thickness
        0x00, 0x00, 0x00, 0x00, // color: none
        0x01, // fill_type: 1
        // Additional fill data (optional)
        0x01, 0x02, 0x03, 0x04,
    ];
    
    let border_fill = parse_border_fill(&data).unwrap();
    assert_eq!(border_fill.properties, 1);
    assert_eq!(border_fill.left_border.line_type, 1);
    assert_eq!(border_fill.left_border.thickness, 2);
    assert_eq!(border_fill.left_border.color, 0x00FF0000);
    assert_eq!(border_fill.right_border.line_type, 2);
    assert_eq!(border_fill.right_border.thickness, 3);
    assert_eq!(border_fill.right_border.color, 0x0000FF00);
    assert_eq!(border_fill.fill_type, 1);
    assert_eq!(border_fill.fill_data, vec![0x01, 0x02, 0x03, 0x04]);
}

#[test]
fn test_id_mappings_parsing() {
    let data = vec![
        0x03, 0x00, 0x00, 0x00, // count: 3
        0x10, 0x00, 0x00, 0x00, // mapping 1: 16
        0x20, 0x00, 0x00, 0x00, // mapping 2: 32
        0x30, 0x00, 0x00, 0x00, // mapping 3: 48
    ];
    
    let mappings = parse_id_mappings(&data).unwrap();
    assert_eq!(mappings.len(), 3);
    assert_eq!(mappings[0], 16);
    assert_eq!(mappings[1], 32);
    assert_eq!(mappings[2], 48);
}

#[test]
fn test_record_data_parser_hwp_string() {
    let data = vec![
        0x03, 0x00, // string length: 3 characters
        0x48, 0x00, // 'H'
        0x57, 0x00, // 'W'
        0x50, 0x00, // 'P'
    ];
    
    let mut parser = RecordDataParser::new(&data);
    let string = parser.read_hwp_string().unwrap();
    assert_eq!(string, "HWP");
}

#[test]
fn test_record_data_parser_varint() {
    let data = vec![
        0x96, 0x01, // 150 (0x96)
        0x80, 0x02, // 256 (0x80 + 0x02 << 7)
        0xFF, 0xFF, 0xFF, 0xFF, 0x0F, // 0xFFFFFFFF
    ];
    
    let mut parser = RecordDataParser::new(&data);
    assert_eq!(parser.read_varint().unwrap(), 150);
    assert_eq!(parser.read_varint().unwrap(), 256);
    assert_eq!(parser.read_varint().unwrap(), 0xFFFFFFFF);
}

#[test]
fn test_malformed_record_handling() {
    // Test incomplete record header
    let data = vec![0x10, 0x00]; // Only 2 bytes instead of 4
    let mut parser = RecordParser::new(&data);
    let result = parser.parse_next_record().unwrap();
    assert!(result.is_none()); // Should return None for incomplete data
    
    // Test record with size larger than available data
    let data = vec![
        0x10, 0x00, 0x64, 0x00, // header: tag=0x0010, level=0, size=100
        0x01, 0x02, // Only 2 bytes of data instead of 100
    ];
    let mut parser = RecordParser::new(&data);
    let result = parser.parse_next_record();
    assert!(result.is_err()); // Should return error for insufficient data
}