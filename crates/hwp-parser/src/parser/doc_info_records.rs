use crate::parser::record::RecordDataParser;
use crate::reader::ByteReader;
use hwp_core::models::document::{
    DocumentProperties, CharShape, ParaShape, Style, FaceName, FaceNameType, BorderFill, BorderLine,
    BinDataEntry, TabDef, TabInfo, Numbering, NumberingLevel, Bullet,
    DistributeDocData, CompatibleDocument, LayoutCompatibility, TrackChange, TrackChangeAuthor,
    MemoShape, ForbiddenChar
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

/// Parse DISTRIBUTE_DOC_DATA record (tag 0x001C)
pub fn parse_distribute_doc_data(data: &[u8]) -> Result<DistributeDocData> {
    Ok(DistributeDocData {
        data: data.to_vec(),
    })
}

/// Parse COMPATIBLE_DOCUMENT record (tag 0x0020)
pub fn parse_compatible_document(data: &[u8]) -> Result<CompatibleDocument> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    let target_program = reader.read_u32()?;
    
    Ok(CompatibleDocument {
        target_program,
    })
}

/// Parse LAYOUT_COMPATIBILITY record (tag 0x0021)
pub fn parse_layout_compatibility(data: &[u8]) -> Result<LayoutCompatibility> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    let letter_spacing = reader.read_u32()?;
    let paragraph_spacing = reader.read_u32()?;
    let line_grid = reader.read_u32()?;
    let paragraph_grid = reader.read_u32()?;
    let snap_to_grid = reader.read_u32()?;
    
    Ok(LayoutCompatibility {
        letter_spacing,
        paragraph_spacing,
        line_grid,
        paragraph_grid,
        snap_to_grid,
    })
}

/// Parse TRACK_CHANGE record (tag 0x0022)
pub fn parse_track_change(data: &[u8]) -> Result<TrackChange> {
    let mut parser = RecordDataParser::new(data);
    
    let properties = parser.reader().read_u32()?;
    let author_id = parser.reader().read_u16()?;
    let timestamp = parser.reader().read_u64()?;
    let change_type = parser.reader().read_u16()?;
    
    // Read the remaining data
    let data_size = parser.remaining();
    let change_data = if data_size > 0 {
        parser.reader().read_bytes(data_size)?
    } else {
        Vec::new()
    };
    
    Ok(TrackChange {
        properties,
        author_id,
        timestamp,
        change_type,
        data: change_data,
    })
}

/// Parse TRACK_CHANGE_AUTHOR record (tag 0x0050)
pub fn parse_track_change_author(data: &[u8]) -> Result<TrackChangeAuthor> {
    let mut parser = RecordDataParser::new(data);
    
    let id = parser.reader().read_u16()?;
    let name = parser.read_hwp_string()?;
    
    Ok(TrackChangeAuthor {
        id,
        name,
    })
}

/// Parse MEMO_SHAPE record (tag 0x004C)
pub fn parse_memo_shape(data: &[u8]) -> Result<MemoShape> {
    let mut parser = RecordDataParser::new(data);
    let reader = parser.reader();
    
    let properties = reader.read_u32()?;
    let memo_id = reader.read_u32()?;
    let width = reader.read_i32()?;
    let line_count = reader.read_u16()?;
    let line_spacing = reader.read_i16()?;
    let line_type = reader.read_u8()?;
    let line_color = reader.read_u32()?;
    
    Ok(MemoShape {
        properties,
        memo_id,
        width,
        line_count,
        line_spacing,
        line_type,
        line_color,
    })
}

/// Parse FORBIDDEN_CHAR record (tag 0x004E)
pub fn parse_forbidden_char(data: &[u8]) -> Result<ForbiddenChar> {
    let mut parser = RecordDataParser::new(data);
    
    let forbidden_chars = parser.read_hwp_string()?;
    let allowed_chars = if parser.has_more_data() {
        parser.read_hwp_string()?
    } else {
        String::new()
    };
    
    Ok(ForbiddenChar {
        forbidden_chars,
        allowed_chars,
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
    fn test_parse_char_shape() {
        let mut data = vec![];
        
        // Face name IDs (7 x u16)
        for i in 0..7 {
            data.extend_from_slice(&[i as u8, 0x00]);
        }
        
        // Ratios (7 x u8)
        for i in 0..7 {
            data.push(50 + i as u8);
        }
        
        // Char spaces (7 x i8)
        for i in 0..7 {
            data.push((i as i8).to_le_bytes()[0]);
        }
        
        // Rel sizes (7 x u8)
        for i in 0..7 {
            data.push(100 - i as u8);
        }
        
        // Char offsets (7 x i8)
        for i in 0..7 {
            data.push((-(i as i8)).to_le_bytes()[0]);
        }
        
        // Other fields
        data.extend_from_slice(&[0x00, 0x0A, 0x00, 0x00]); // base_size: 2560
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // properties: 1
        data.push(0x02); // shadow_gap_x: 2
        data.push(0x03); // shadow_gap_y: 3
        data.extend_from_slice(&[0xFF, 0x00, 0x00, 0x00]); // text_color: 0xFF (red)
        data.extend_from_slice(&[0x00, 0xFF, 0x00, 0x00]); // underline_color: 0xFF00 (green)
        data.extend_from_slice(&[0x00, 0x00, 0xFF, 0x00]); // shade_color: 0xFF0000 (blue)
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0xFF]); // shadow_color: 0xFF000000 (black)
        data.extend_from_slice(&[0x05, 0x00]); // border_fill_id: 5
        
        let char_shape = parse_char_shape(&data).unwrap();
        assert_eq!(char_shape.face_name_ids.len(), 7);
        assert_eq!(char_shape.face_name_ids[0], 0);
        assert_eq!(char_shape.face_name_ids[6], 6);
        assert_eq!(char_shape.ratios[0], 50);
        assert_eq!(char_shape.base_size, 2560);
        assert_eq!(char_shape.properties, 1);
        assert_eq!(char_shape.shadow_gap_x, 2);
        assert_eq!(char_shape.shadow_gap_y, 3);
        assert_eq!(char_shape.text_color, 0xFF);
        assert_eq!(char_shape.border_fill_id, Some(5));
    }
    
    #[test]
    fn test_parse_para_shape() {
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // properties1: 1
            0x00, 0x05, 0x00, 0x00, // left_margin: 1280
            0x00, 0x05, 0x00, 0x00, // right_margin: 1280
            0x00, 0x02, 0x00, 0x00, // indent: 512
            0x00, 0x01, 0x00, 0x00, // prev_spacing: 256
            0x00, 0x01, 0x00, 0x00, // next_spacing: 256
            0x00, 0x02, 0x00, 0x00, // line_spacing: 512
            0x00, 0x00, // tab_def_id: 0
            0x00, 0x00, // numbering_id: 0
            0x00, 0x00, // border_fill_id: 0
            0x0A, 0x00, // border_offset_left: 10
            0x0A, 0x00, // border_offset_right: 10
            0x0A, 0x00, // border_offset_top: 10
            0x0A, 0x00, // border_offset_bottom: 10
            0x02, 0x00, 0x00, 0x00, // properties2: 2
            0x03, 0x00, 0x00, 0x00, // properties3: 3
            0x01, 0x00, 0x00, 0x00, // line_spacing_type: 1
        ];
        
        let para_shape = parse_para_shape(&data).unwrap();
        assert_eq!(para_shape.properties1, 1);
        assert_eq!(para_shape.left_margin, 1280);
        assert_eq!(para_shape.right_margin, 1280);
        assert_eq!(para_shape.indent, 512);
        assert_eq!(para_shape.prev_spacing, 256);
        assert_eq!(para_shape.next_spacing, 256);
        assert_eq!(para_shape.line_spacing, 512);
        assert_eq!(para_shape.tab_def_id, 0);
        assert_eq!(para_shape.numbering_id, 0);
        assert_eq!(para_shape.border_fill_id, 0);
        assert_eq!(para_shape.border_offset_left, 10);
        assert_eq!(para_shape.border_offset_right, 10);
        assert_eq!(para_shape.border_offset_top, 10);
        assert_eq!(para_shape.border_offset_bottom, 10);
        assert_eq!(para_shape.properties2, 2);
        assert_eq!(para_shape.properties3, 3);
        assert_eq!(para_shape.line_spacing_type, 1);
    }
    
    #[test]
    fn test_parse_border_fill() {
        let data = vec![
            0x01, 0x00, // properties: 1
            // Left border
            0x01, // line_type: 1
            0x02, // thickness: 2
            0xFF, 0x00, 0x00, 0x00, // color: 0xFF (red)
            // Right border
            0x01, // line_type: 1
            0x02, // thickness: 2
            0x00, 0xFF, 0x00, 0x00, // color: 0xFF00 (green)
            // Top border
            0x01, // line_type: 1
            0x02, // thickness: 2
            0x00, 0x00, 0xFF, 0x00, // color: 0xFF0000 (blue)
            // Bottom border
            0x01, // line_type: 1
            0x02, // thickness: 2
            0x00, 0x00, 0x00, 0xFF, // color: 0xFF000000 (black)
            // Diagonal border
            0x00, // line_type: 0
            0x00, // thickness: 0
            0x00, 0x00, 0x00, 0x00, // color: 0
            0x01, // fill_type: 1
            0xAA, 0xBB, 0xCC, 0xDD, // fill_data
        ];
        
        let border_fill = parse_border_fill(&data).unwrap();
        assert_eq!(border_fill.properties, 1);
        assert_eq!(border_fill.left_border.line_type, 1);
        assert_eq!(border_fill.left_border.thickness, 2);
        assert_eq!(border_fill.left_border.color, 0xFF);
        assert_eq!(border_fill.right_border.color, 0xFF00);
        assert_eq!(border_fill.top_border.color, 0xFF0000);
        assert_eq!(border_fill.bottom_border.color, 0xFF000000);
        assert_eq!(border_fill.diagonal_border.line_type, 0);
        assert_eq!(border_fill.fill_type, 1);
        assert_eq!(border_fill.fill_data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }
    
    #[test]
    fn test_parse_doc_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        
        let doc_data = parse_doc_data(&data).unwrap();
        assert_eq!(doc_data, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
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
    
    #[test]
    fn test_parse_distribute_doc_data() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        
        let distribute = parse_distribute_doc_data(&data).unwrap();
        assert_eq!(distribute.data, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }
    
    #[test]
    fn test_parse_compatible_document() {
        let data = vec![
            0x03, 0x00, 0x00, 0x00, // target_program: 3 (MS Word compatible)
        ];
        
        let compatible = parse_compatible_document(&data).unwrap();
        assert_eq!(compatible.target_program, 3);
    }
    
    #[test]
    fn test_parse_layout_compatibility() {
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // letter_spacing: 1
            0x02, 0x00, 0x00, 0x00, // paragraph_spacing: 2
            0x03, 0x00, 0x00, 0x00, // line_grid: 3
            0x04, 0x00, 0x00, 0x00, // paragraph_grid: 4
            0x01, 0x00, 0x00, 0x00, // snap_to_grid: 1
        ];
        
        let layout = parse_layout_compatibility(&data).unwrap();
        assert_eq!(layout.letter_spacing, 1);
        assert_eq!(layout.paragraph_spacing, 2);
        assert_eq!(layout.line_grid, 3);
        assert_eq!(layout.paragraph_grid, 4);
        assert_eq!(layout.snap_to_grid, 1);
    }
    
    #[test]
    fn test_parse_track_change() {
        // This test also covers CHANGE_TRACKING (0x00F0) which uses the same parser
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // properties: 1
            0x02, 0x00, // author_id: 2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // timestamp: 0x0100000000000000
            0x01, 0x00, // change_type: 1 (insert)
            0xAA, 0xBB, 0xCC, // change data
        ];
        
        let track_change = parse_track_change(&data).unwrap();
        assert_eq!(track_change.properties, 1);
        assert_eq!(track_change.author_id, 2);
        assert_eq!(track_change.timestamp, 0x0100000000000000);
        assert_eq!(track_change.change_type, 1);
        assert_eq!(track_change.data, vec![0xAA, 0xBB, 0xCC]);
    }
    
    #[test]
    fn test_parse_track_change_author() {
        let data = vec![
            0x01, 0x00, // id: 1
            0x04, 0x00, // string length: 4
            0x4A, 0x00, // 'J'
            0x6F, 0x00, // 'o'
            0x68, 0x00, // 'h'
            0x6E, 0x00, // 'n'
        ];
        
        let author = parse_track_change_author(&data).unwrap();
        assert_eq!(author.id, 1);
        assert_eq!(author.name, "John");
    }
    
    #[test]
    fn test_parse_memo_shape() {
        let data = vec![
            0x01, 0x00, 0x00, 0x00, // properties: 1
            0x10, 0x00, 0x00, 0x00, // memo_id: 16
            0x00, 0x05, 0x00, 0x00, // width: 1280
            0x05, 0x00, // line_count: 5
            0x10, 0x00, // line_spacing: 16
            0x01, // line_type: 1 (solid)
            0xFF, 0x00, 0x00, 0x00, // line_color: 0x000000FF (red)
        ];
        
        let memo = parse_memo_shape(&data).unwrap();
        assert_eq!(memo.properties, 1);
        assert_eq!(memo.memo_id, 16);
        assert_eq!(memo.width, 0x0500);
        assert_eq!(memo.line_count, 5);
        assert_eq!(memo.line_spacing, 16);
        assert_eq!(memo.line_type, 1);
        assert_eq!(memo.line_color, 0xFF);
    }
    
    #[test]
    fn test_parse_forbidden_char() {
        let data = vec![
            // Forbidden chars string
            0x03, 0x00, // string length: 3
            0x2C, 0x00, // ','
            0x2E, 0x00, // '.'
            0x3B, 0x00, // ';'
            // Allowed chars string
            0x02, 0x00, // string length: 2
            0x21, 0x00, // '!'
            0x3F, 0x00, // '?'
        ];
        
        let forbidden = parse_forbidden_char(&data).unwrap();
        assert_eq!(forbidden.forbidden_chars, ",.;");
        assert_eq!(forbidden.allowed_chars, "!?");
    }
}