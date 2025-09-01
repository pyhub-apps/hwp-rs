use crate::parser::record::RecordDataParser;
use crate::reader::ByteReader;
use hwp_core::models::document::{
    DocumentProperties, CharShape, ParaShape, Style, FaceName, FaceNameType, BorderFill, BorderLine,
    BinDataEntry, TabDef, TabInfo, Numbering, NumberingLevel, Bullet
};
use hwp_core::Result;

/// Parse DOCUMENT_PROPERTIES record (tag 0x0010)
pub fn parse_document_properties(data: &[u8]) -> Result<DocumentProperties> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    Ok(DocumentProperties {
        section_count: reader.read_u16()?,
        page_start_number: reader.read_u16()?,
        footnote_start_number: reader.read_u16()?,
        endnote_start_number: reader.read_u16()?,
        picture_start_number: reader.read_u16()?,
        table_start_number: reader.read_u16()?,
        equation_start_number: reader.read_u16()?,
        total_character_count: reader.read_u32()?,
        total_page_count: reader.read_u32()?,
    })
}

/// Parse FACE_NAME record (tag 0x0013)
pub fn parse_face_name(data: &[u8]) -> Result<FaceName> {
    let mut parser = RecordDataParser::new(data);
    
    let properties = parser.reader().read_u8()?;
    let name = parser.read_hwp_string()?;
    
    // Check if there's type info (optional based on properties)
    let type_info = if (properties & 0x01) != 0 && parser.has_more_data() {
        FaceNameType {
            family: parser.reader().read_u8()?,
            serif: parser.reader().read_u8()?,
            weight: parser.reader().read_u8()?,
            proportion: parser.reader().read_u8()?,
            contrast: parser.reader().read_u8()?,
            stroke_variation: parser.reader().read_u8()?,
            arm_style: parser.reader().read_u8()?,
            letter_form: parser.reader().read_u8()?,
            midline: parser.reader().read_u8()?,
            x_height: parser.reader().read_u8()?,
        }
    } else {
        FaceNameType {
            family: 0, serif: 0, weight: 0, proportion: 0, contrast: 0,
            stroke_variation: 0, arm_style: 0, letter_form: 0, midline: 0, x_height: 0,
        }
    };
    
    // Check for substitute font info (optional)
    let (substitute_font_type, substitute_font_name) = if (properties & 0x02) != 0 && parser.has_more_data() {
        let font_type = parser.reader().read_u8()?;
        let font_name = parser.read_hwp_string()?;
        (Some(font_type), Some(font_name))
    } else {
        (None, None)
    };
    
    // Check for base font name (optional)
    let base_font_name = if (properties & 0x04) != 0 && parser.has_more_data() {
        Some(parser.read_hwp_string()?)
    } else {
        None
    };
    
    Ok(FaceName {
        properties,
        name,
        substitute_font_type,
        substitute_font_name,
        type_info,
        base_font_name,
    })
}

/// Parse CHAR_SHAPE record (tag 0x0015)
pub fn parse_char_shape(data: &[u8]) -> Result<CharShape> {
    let mut parser = RecordDataParser::new(data);
    
    // Read face name IDs (array of 7 u16 values)
    let mut face_name_ids = Vec::with_capacity(7);
    for _ in 0..7 {
        face_name_ids.push(parser.reader().read_u16()?);
    }
    
    // Read ratios (array of 7 u8 values)
    let mut ratios = Vec::with_capacity(7);
    for _ in 0..7 {
        ratios.push(parser.reader().read_u8()?);
    }
    
    // Read character spaces (array of 7 i8 values)
    let mut char_spaces = Vec::with_capacity(7);
    for _ in 0..7 {
        char_spaces.push(parser.reader().read_i8()?);
    }
    
    // Read relative sizes (array of 7 u8 values)
    let mut rel_sizes = Vec::with_capacity(7);
    for _ in 0..7 {
        rel_sizes.push(parser.reader().read_u8()?);
    }
    
    // Read character offsets (array of 7 i8 values)
    let mut char_offsets = Vec::with_capacity(7);
    for _ in 0..7 {
        char_offsets.push(parser.reader().read_i8()?);
    }
    
    // Read remaining fields
    let base_size = parser.reader().read_u32()?;
    let properties = parser.reader().read_u32()?;
    let shadow_gap_x = parser.reader().read_i8()?;
    let shadow_gap_y = parser.reader().read_i8()?;
    let text_color = parser.reader().read_u32()?;
    let underline_color = parser.reader().read_u32()?;
    let shade_color = parser.reader().read_u32()?;
    let shadow_color = parser.reader().read_u32()?;
    
    // Border fill ID is optional
    let border_fill_id = if parser.has_more_data() {
        Some(parser.reader().read_u16()?)
    } else {
        None
    };
    
    Ok(CharShape {
        face_name_ids,
        ratios,
        char_spaces,
        rel_sizes,
        char_offsets,
        base_size,
        properties,
        shadow_gap_x,
        shadow_gap_y,
        text_color,
        underline_color,
        shade_color,
        shadow_color,
        border_fill_id,
    })
}

/// Parse PARA_SHAPE record (tag 0x0019)
pub fn parse_para_shape(data: &[u8]) -> Result<ParaShape> {
    let mut parser = RecordDataParser::new(data);
    
    let properties1 = parser.reader().read_u32()?;
    let left_margin = parser.reader().read_i32()?;
    let right_margin = parser.reader().read_i32()?;
    let indent = parser.reader().read_i32()?;
    let prev_spacing = parser.reader().read_i32()?;
    let next_spacing = parser.reader().read_i32()?;
    let line_spacing = parser.reader().read_i32()?;
    let tab_def_id = parser.reader().read_u16()?;
    let numbering_id = parser.reader().read_u16()?;
    let border_fill_id = parser.reader().read_u16()?;
    let border_offset_left = parser.reader().read_i16()?;
    let border_offset_right = parser.reader().read_i16()?;
    let border_offset_top = parser.reader().read_i16()?;
    let border_offset_bottom = parser.reader().read_i16()?;
    
    let properties2 = if parser.has_more_data() { parser.reader().read_u32()? } else { 0 };
    let properties3 = if parser.has_more_data() { parser.reader().read_u32()? } else { 0 };
    let line_spacing_type = if parser.has_more_data() { parser.reader().read_u32()? } else { 0 };
    
    Ok(ParaShape {
        properties1,
        left_margin,
        right_margin,
        indent,
        prev_spacing,
        next_spacing,
        line_spacing,
        tab_def_id,
        numbering_id,
        border_fill_id,
        border_offset_left,
        border_offset_right,
        border_offset_top,
        border_offset_bottom,
        properties2,
        properties3,
        line_spacing_type,
    })
}

/// Parse STYLE record (tag 0x001A)
pub fn parse_style(data: &[u8]) -> Result<Style> {
    let mut parser = RecordDataParser::new(data);
    
    let name = parser.read_hwp_string()?;
    let english_name = parser.read_hwp_string()?;
    
    let properties = parser.reader().read_u8()?;
    let next_style_id = parser.reader().read_u8()?;
    let lang_id = parser.reader().read_u16()?;
    let para_shape_id = parser.reader().read_u16()?;
    let char_shape_id = parser.reader().read_u16()?;
    
    Ok(Style {
        name,
        english_name,
        properties,
        next_style_id,
        lang_id,
        para_shape_id,
        char_shape_id,
    })
}

/// Parse BORDER_FILL record (tag 0x0014)
pub fn parse_border_fill(data: &[u8]) -> Result<BorderFill> {
    let mut parser = RecordDataParser::new(data);
    
    let properties = parser.reader().read_u16()?;
    
    // Parse border lines
    let left_border = parse_border_line(parser.reader())?;
    let right_border = parse_border_line(parser.reader())?;
    let top_border = parse_border_line(parser.reader())?;
    let bottom_border = parse_border_line(parser.reader())?;
    let diagonal_border = parse_border_line(parser.reader())?;
    
    let fill_type = parser.reader().read_u8()?;
    
    // Read remaining fill data
    let fill_data = if parser.has_more_data() {
        let remaining = parser.remaining();
        parser.reader().read_bytes(remaining)?
    } else {
        Vec::new()
    };
    
    Ok(BorderFill {
        properties,
        left_border,
        right_border,
        top_border,
        bottom_border,
        diagonal_border,
        fill_type,
        fill_data,
    })
}

/// Helper function to parse a border line
fn parse_border_line(reader: &mut ByteReader) -> Result<BorderLine> {
    Ok(BorderLine {
        line_type: reader.read_u8()?,
        thickness: reader.read_u8()?,
        color: reader.read_u32()?,
    })
}

/// Parse ID_MAPPINGS record (tag 0x0011)
pub fn parse_id_mappings(data: &[u8]) -> Result<Vec<u32>> {
    let mut parser = RecordDataParser::new(data);
    
    let count = parser.reader().read_u32()? as usize;
    let mut mappings = Vec::with_capacity(count);
    
    for _ in 0..count {
        mappings.push(parser.reader().read_u32()?);
    }
    
    Ok(mappings)
}

/// Parse BIN_DATA record (tag 0x0012)
pub fn parse_bin_data(data: &[u8]) -> Result<BinDataEntry> {
    let mut parser = RecordDataParser::new(data);
    
    // Read BIN_DATA properties
    let properties = parser.reader().read_u16()?;
    
    // Extract ID from properties (lower 16 bits)
    let id = properties & 0xFFFF;
    
    // Link type and compression type
    let link_type = parser.reader().read_u8()?;
    let compression_type = parser.reader().read_u8()?;
    
    // Read the actual binary data
    let data_size = parser.remaining();
    let data = parser.reader().read_bytes(data_size)?;
    
    Ok(BinDataEntry {
        id,
        link_type,
        compression_type,
        data,
    })
}

/// Parse DOC_DATA record (tag 0x001B)
pub fn parse_doc_data(data: &[u8]) -> Result<Vec<u8>> {
    // DOC_DATA is application-specific data
    Ok(data.to_vec())
}

/// Parse TAB_DEF record (tag 0x0016)
pub fn parse_tab_def(data: &[u8]) -> Result<TabDef> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    let properties = reader.read_u32()?;
    let count = reader.read_u32()?;
    
    let mut tabs = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let position = reader.read_i32()?;
        let tab_type = reader.read_u8()?;
        let fill_type = reader.read_u8()?;
        // Skip 2 bytes of reserved data
        reader.read_u16()?;
        
        tabs.push(TabInfo {
            position,
            tab_type,
            fill_type,
        });
    }
    
    Ok(TabDef {
        properties,
        count,
        tabs,
    })
}

/// Parse NUMBERING record (tag 0x0017)
pub fn parse_numbering(data: &[u8]) -> Result<Numbering> {
    let mut parser = RecordDataParser::new(data);
    
    let mut levels = Vec::new();
    
    // HWP supports up to 7 levels of numbering
    for _ in 0..7 {
        if !parser.has_more_data() {
            break;
        }
        
        let properties = parser.reader().read_u32()?;
        let paragraph_shape_id = parser.reader().read_u16()?;
        
        // Read the numbering format string
        let format = parser.read_hwp_string()?;
        
        let start_number = if parser.has_more_data() {
            parser.reader().read_u16()?
        } else {
            1
        };
        
        levels.push(NumberingLevel {
            properties,
            paragraph_shape_id,
            format,
            start_number,
        });
    }
    
    Ok(Numbering { levels })
}

/// Parse BULLET record (tag 0x0018)
pub fn parse_bullet(data: &[u8]) -> Result<Bullet> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    let properties = reader.read_u32()?;
    let paragraph_shape_id = reader.read_u16()?;
    
    // Check if using character or image
    let uses_image = (properties & 0x01) != 0;
    
    let (bullet_char, image_id) = if uses_image {
        // Using image
        let img_id = reader.read_u16()?;
        (None, Some(img_id))
    } else {
        // Using text character
        let char = parser.read_hwp_string()?;
        (Some(char), None)
    };
    
    Ok(Bullet {
        properties,
        paragraph_shape_id,
        bullet_char,
        image_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_document_properties() {
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
        assert_eq!(props.total_character_count, 100);
        assert_eq!(props.total_page_count, 5);
    }
    
    #[test]
    fn test_parse_face_name() {
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
    fn test_parse_style() {
        let data = vec![
            0x04, 0x00, // name length: 4 characters
            0x14, 0xBC, // '바' in UTF-16LE
            0xD5, 0xD0, // '탕' in UTF-16LE
            0x38, 0xBB, // '문' in UTF-16LE
            0xB4, 0xCC, // '체' in UTF-16LE
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
            0x00, 0x00, // para_shape_id: 0
            0x00, 0x00, // char_shape_id: 0
        ];
        
        let style = parse_style(&data).unwrap();
        assert_eq!(style.name, "바탕문체");
        assert_eq!(style.english_name, "Normal");
        assert_eq!(style.properties, 1);
        assert_eq!(style.next_style_id, 255);
        assert_eq!(style.lang_id, 0x0412);
    }
    
    #[test]
    fn test_parse_id_mappings() {
        let data = vec![
            0x03, 0x00, 0x00, 0x00, // count: 3
            0x01, 0x00, 0x00, 0x00, // mapping[0]: 1
            0x02, 0x00, 0x00, 0x00, // mapping[1]: 2
            0x03, 0x00, 0x00, 0x00, // mapping[2]: 3
        ];
        
        let mappings = parse_id_mappings(&data).unwrap();
        assert_eq!(mappings.len(), 3);
        assert_eq!(mappings[0], 1);
        assert_eq!(mappings[1], 2);
        assert_eq!(mappings[2], 3);
    }
    
    #[test]
    fn test_parse_bin_data() {
        let data = vec![
            0x01, 0x00, // properties/id: 1
            0x00,       // link_type: 0 (embedded)
            0x01,       // compression_type: 1 (compressed)
            0xDE, 0xAD, 0xBE, 0xEF, // binary data
        ];
        
        let bin_data = parse_bin_data(&data).unwrap();
        assert_eq!(bin_data.id, 1);
        assert_eq!(bin_data.link_type, 0);
        assert_eq!(bin_data.compression_type, 1);
        assert_eq!(bin_data.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }
    
    #[test]
    fn test_parse_tab_def() {
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // properties: 1
            0x02, 0x00, 0x00, 0x00, // count: 2
            // Tab 1
            0x00, 0x05, 0x00, 0x00, // position: 1280 (5 * 256)
            0x00,       // tab_type: 0 (left)
            0x01,       // fill_type: 1 (dots)
            0x00, 0x00, // reserved
            // Tab 2
            0x00, 0x0A, 0x00, 0x00, // position: 2560 (10 * 256)
            0x01,       // tab_type: 1 (center)
            0x00,       // fill_type: 0 (none)
            0x00, 0x00, // reserved
        ];
        
        let tab_def = parse_tab_def(&data).unwrap();
        assert_eq!(tab_def.properties, 1);
        assert_eq!(tab_def.count, 2);
        assert_eq!(tab_def.tabs.len(), 2);
        assert_eq!(tab_def.tabs[0].position, 0x0500);
        assert_eq!(tab_def.tabs[0].tab_type, 0);
        assert_eq!(tab_def.tabs[0].fill_type, 1);
        assert_eq!(tab_def.tabs[1].position, 0x0A00);
        assert_eq!(tab_def.tabs[1].tab_type, 1);
        assert_eq!(tab_def.tabs[1].fill_type, 0);
    }
    
    #[test]
    fn test_parse_numbering() {
        let data = vec![
            // Level 1
            0x01, 0x00, 0x00, 0x00, // properties: 1
            0x00, 0x00, // paragraph_shape_id: 0
            0x02, 0x00, // format string length: 2
            0x31, 0x00, // '1'
            0x2E, 0x00, // '.'
            0x01, 0x00, // start_number: 1
        ];
        
        let numbering = parse_numbering(&data).unwrap();
        assert_eq!(numbering.levels.len(), 1);
        assert_eq!(numbering.levels[0].properties, 1);
        assert_eq!(numbering.levels[0].paragraph_shape_id, 0);
        assert_eq!(numbering.levels[0].format, "1.");
        assert_eq!(numbering.levels[0].start_number, 1);
    }
    
    #[test]
    fn test_parse_bullet() {
        // Test text bullet
        let data_text = vec![
            0x00, 0x00, 0x00, 0x00, // properties: 0 (text bullet)
            0x00, 0x00, // paragraph_shape_id: 0
            0x01, 0x00, // char string length: 1 character (not bytes)
            0x22, 0x20, // '•' (U+2022 bullet character in UTF-16LE)
        ];
        
        let bullet = parse_bullet(&data_text).unwrap();
        assert_eq!(bullet.properties, 0);
        assert_eq!(bullet.paragraph_shape_id, 0);
        assert!(bullet.bullet_char.is_some());
        assert_eq!(bullet.bullet_char.unwrap(), "•");
        assert!(bullet.image_id.is_none());
        
        // Test image bullet
        let data_image = vec![
            0x01, 0x00, 0x00, 0x00, // properties: 1 (uses image)
            0x00, 0x00, // paragraph_shape_id: 0
            0x05, 0x00, // image_id: 5
        ];
        
        let bullet = parse_bullet(&data_image).unwrap();
        assert_eq!(bullet.properties, 1);
        assert_eq!(bullet.paragraph_shape_id, 0);
        assert!(bullet.bullet_char.is_none());
        assert!(bullet.image_id.is_some());
        assert_eq!(bullet.image_id.unwrap(), 5);
    }
}