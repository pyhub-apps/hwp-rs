use hwp_parser::parser::doc_info::parse_doc_info;
use hwp_core::constants::tag_id::doc_info;

fn header(tag_id: u16, level: u8, size: usize) -> [u8; 4] {
    // HWP record header: tag_id(10) | level(2) | size(20)
    let value = (tag_id as u32) | ((level as u32) << 10) | ((size as u32) << 12);
    value.to_le_bytes()
}

/// Create a sample DocInfo data stream for demonstration
fn create_sample_docinfo() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Document Properties Record (0x0010) size = 36
    data.extend_from_slice(&header(doc_info::DOCUMENT_PROPERTIES, 0, 36));
    data.extend_from_slice(&[0x05, 0x00]); // section_count: 5
    data.extend_from_slice(&[0x01, 0x00]); // page_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // footnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // endnote_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // picture_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // table_start_number: 1
    data.extend_from_slice(&[0x01, 0x00]); // equation_start_number: 1
    data.extend_from_slice(&[0x50, 0x46, 0x00, 0x00]); // total_character_count: 18000
    data.extend_from_slice(&[0x0A, 0x00, 0x00, 0x00]); // total_page_count: 10
    data.extend_from_slice(&[0x00; 16]); // padding
    
    // Face Name Record (0x0013) - Arial
    let arial_utf16: Vec<u8> = "Arial".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let face_name_size = 1 + 2 + arial_utf16.len();
    data.extend_from_slice(&header(doc_info::FACE_NAME, 0, face_name_size));
    data.push(0x01); // properties: has type info
    data.extend_from_slice(&(5u16).to_le_bytes()); // string length
    data.extend_from_slice(&arial_utf16);
    // Type info
    data.extend_from_slice(&[0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    // Face Name Record (0x0013) - 맑은 고딕 (Malgun Gothic)
    let malgun_utf16: Vec<u8> = "맑은 고딕".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let face_name2_size = 1 + 2 + malgun_utf16.len();
    data.extend_from_slice(&header(doc_info::FACE_NAME, 0, face_name2_size));
    data.push(0x00); // properties: no optional data
    data.extend_from_slice(&(5u16).to_le_bytes()); // string length (5 characters)
    data.extend_from_slice(&malgun_utf16);

    // Border Fill Record (0x0014) - minimal
    let mut border_fill = Vec::new();
    border_fill.extend_from_slice(&0u16.to_le_bytes()); // properties
    for _ in 0..5 { // borders (left,right,top,bottom,diagonal)
        border_fill.push(0); // line_type
        border_fill.push(0); // thickness
        border_fill.extend_from_slice(&0u32.to_le_bytes()); // color
    }
    border_fill.push(0); // fill_type
    data.extend_from_slice(&header(doc_info::BORDER_FILL, 0, border_fill.len()));
    data.extend_from_slice(&border_fill);

    // Char Shape Record (0x0015) - minimal
    let mut char_shape = Vec::new();
    for i in 0..7 { char_shape.extend_from_slice(&(i as u16).to_le_bytes()); } // face_name_ids
    for _ in 0..7 { char_shape.push(100); } // ratios
    for _ in 0..7 { char_shape.push(0); } // char_spaces
    for _ in 0..7 { char_shape.push(100); } // rel_sizes
    for _ in 0..7 { char_shape.push(0); } // char_offsets
    char_shape.extend_from_slice(&1200u32.to_le_bytes()); // base_size
    char_shape.extend_from_slice(&0u32.to_le_bytes()); // properties
    char_shape.push(0); // shadow_gap_x
    char_shape.push(0); // shadow_gap_y
    char_shape.extend_from_slice(&0x000000u32.to_le_bytes()); // text_color
    char_shape.extend_from_slice(&0x000000u32.to_le_bytes()); // underline_color
    char_shape.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // shade_color
    char_shape.extend_from_slice(&0x808080u32.to_le_bytes()); // shadow_color
    data.extend_from_slice(&header(doc_info::CHAR_SHAPE, 0, char_shape.len()));
    data.extend_from_slice(&char_shape);

    // Style Record (0x001A) - Normal style
    let style_name_utf16: Vec<u8> = "본문".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let style_eng_utf16: Vec<u8> = "Normal".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let style_size = 2 + style_name_utf16.len() + 2 + style_eng_utf16.len() + 1 + 1 + 2 + 2 + 2;
    data.extend_from_slice(&header(doc_info::STYLE, 0, style_size));
    data.extend_from_slice(&(2u16).to_le_bytes()); // name length
    data.extend_from_slice(&style_name_utf16);
    data.extend_from_slice(&(6u16).to_le_bytes()); // english name length
    data.extend_from_slice(&style_eng_utf16);
    data.push(0x01); // properties: paragraph style
    data.push(0xFF); // next_style_id: none
    data.extend_from_slice(&0x0412u16.to_le_bytes()); // lang_id: Korean
    data.extend_from_slice(&0u16.to_le_bytes()); // para_shape_id
    data.extend_from_slice(&0u16.to_le_bytes()); // char_shape_id
    
    data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HWP DocInfo Parser Example");
    println!("==========================");
    
    // Create sample DocInfo data
    let doc_info_data = create_sample_docinfo();
    println!("Created sample DocInfo data: {} bytes", doc_info_data.len());
    
    // Parse the DocInfo
    let doc_info = parse_doc_info(&doc_info_data)?;
    
    // Display parsed information
    println!("\nDocument Properties:");
    println!("  Section count: {}", doc_info.properties.section_count);
    println!("  Total characters: {}", doc_info.properties.total_character_count);
    println!("  Total pages: {}", doc_info.properties.total_page_count);
    
    println!("\nFonts ({} found):", doc_info.face_names.len());
    for (i, face_name) in doc_info.face_names.iter().enumerate() {
        println!("  {}: {} (properties: 0x{:02X})", i, face_name.name, face_name.properties);
        if face_name.properties & 0x01 != 0 {
            println!("    Type info: family={}, weight={}", 
                    face_name.type_info.family, face_name.type_info.weight);
        }
    }
    
    println!("\nStyles ({} found):", doc_info.styles.len());
    for (i, style) in doc_info.styles.iter().enumerate() {
        println!("  {}: '{}' / '{}' (lang: 0x{:04X})", 
                i, style.name, style.english_name, style.lang_id);
        println!("    Para shape ID: {}, Char shape ID: {}", 
                style.para_shape_id, style.char_shape_id);
    }
    
    println!("\nCharacter Shapes: {}", doc_info.char_shapes.len());
    println!("Paragraph Shapes: {}", doc_info.para_shapes.len());
    println!("Border Fills: {}", doc_info.border_fills.len());
    
    println!("\n✓ DocInfo parsing completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sample_docinfo_creation() {
        let data = create_sample_docinfo();
        assert!(!data.is_empty());
        
        let doc_info = parse_doc_info(&data).unwrap();
        assert_eq!(doc_info.properties.section_count, 5);
        assert_eq!(doc_info.properties.total_page_count, 10);
        assert_eq!(doc_info.face_names.len(), 2);
        assert_eq!(doc_info.styles.len(), 1);
    }
}
