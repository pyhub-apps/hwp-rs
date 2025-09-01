use crate::parser::record::RecordParser;
use crate::reader::ByteReader;
use hwp_core::constants::tag_id::section;
use hwp_core::models::paragraph::{ParagraphHeader, CharShapePos, LineSegment};
use hwp_core::models::section::Section;
use hwp_core::models::Paragraph;
use hwp_core::{HwpError, Result};

/// Parse a section from decompressed data
pub fn parse_section(data: &[u8], section_index: usize) -> Result<Section> {
    let mut parser = RecordParser::new(data);
    let mut section = Section::new();
    
    // Parse all records in the section
    while let Some(record) = parser.parse_next_record()? {
        match record.tag_id {
            section::PARA_HEADER => {
                // Start of a new paragraph
                let para_header = parse_para_header(&record.data)?;
                let mut paragraph = Paragraph::new();
                
                // Parse subsequent paragraph-related records
                while let Some(next_record) = parser.parse_next_record()? {
                    match next_record.tag_id {
                        section::PARA_TEXT => {
                            let text = parse_para_text(&next_record.data)?;
                            paragraph.text = text;
                        }
                        section::PARA_CHAR_SHAPE => {
                            let char_shapes = parse_para_char_shapes(&next_record.data, para_header.char_shape_count)?;
                            paragraph.char_shapes = char_shapes;
                        }
                        section::PARA_LINE_SEG => {
                            let line_segments = parse_line_segments(&next_record.data)?;
                            paragraph.line_segments = line_segments;
                        }
                        section::PARA_RANGE_TAG => {
                            // Range tags - skip for now
                        }
                        section::PARA_HEADER => {
                            // Next paragraph starts, put back the record
                            // We need to handle this differently in real implementation
                            break;
                        }
                        _ => {
                            // Other record type, might be control or next section
                            break;
                        }
                    }
                }
                
                section.paragraphs.push(paragraph);
            }
            
            // Section definition records would be here
            // For now, we focus on paragraph parsing
            _ => {
                // Skip unknown records
            }
        }
    }
    
    Ok(section)
}

/// Parse paragraph header
fn parse_para_header(data: &[u8]) -> Result<ParagraphHeader> {
    let mut reader = ByteReader::new(data);
    
    let mut header = ParagraphHeader::default();
    header.text_count = reader.read_u32()?;
    header.control_mask = reader.read_u32()?;
    header.para_shape_id = reader.read_u16()?;
    header.style_id = reader.read_u8()?;
    header.division_type = reader.read_u8()?;
    header.char_shape_count = reader.read_u16()?;
    header.range_tag_count = reader.read_u16()?;
    header.line_align_count = reader.read_u16()?;
    header.instance_id = reader.read_u32()?;
    
    // Optional field based on version
    if reader.remaining() >= 2 {
        header.is_merged_by_track = reader.read_u16()?;
    }
    
    Ok(header)
}

/// Parse paragraph text
fn parse_para_text(data: &[u8]) -> Result<String> {
    // Text is stored as UTF-16LE
    let mut text = String::new();
    let mut i = 0;
    
    while i + 1 < data.len() {
        let ch = u16::from_le_bytes([data[i], data[i + 1]]);
        i += 2;
        
        // Handle special characters
        match ch {
            0x0000 => break, // Null terminator
            0x000A => text.push('\n'), // Line feed
            0x000D => continue, // Carriage return (skip)
            0x0001..=0x001F => {
                // Control characters - these might be inline controls
                // For now, skip them
                continue;
            }
            _ => {
                if let Some(c) = char::from_u32(ch as u32) {
                    text.push(c);
                }
            }
        }
    }
    
    Ok(text)
}

/// Parse character shape positions
fn parse_para_char_shapes(data: &[u8], count: u16) -> Result<Vec<CharShapePos>> {
    let mut reader = ByteReader::new(data);
    let mut shapes = Vec::with_capacity(count as usize);
    
    for _ in 0..count {
        let position = reader.read_u32()?;
        let shape_id = reader.read_u16()?;
        shapes.push(CharShapePos { position, shape_id });
    }
    
    Ok(shapes)
}

/// Parse line segments
fn parse_line_segments(data: &[u8]) -> Result<Vec<LineSegment>> {
    let mut reader = ByteReader::new(data);
    let mut segments = Vec::new();
    
    while reader.remaining() >= 32 { // Each line segment is 32 bytes
        let text_start_pos = reader.read_u32()?;
        let line_height = reader.read_i32()?;
        let text_height = reader.read_i32()?;
        let baseline_gap = reader.read_i32()?;
        let line_spacing = reader.read_i32()?;
        let column_start_pos = reader.read_u32()?;
        let segment_width = reader.read_i32()?;
        let flags = reader.read_u32()?;
        
        segments.push(LineSegment {
            text_start_pos,
            line_height,
            text_height,
            baseline_gap,
            line_spacing,
            column_start_pos,
            segment_width,
            flags,
        });
    }
    
    Ok(segments)
}

/// Parse all sections from a BodyText stream
pub fn parse_body_text(data: &[u8]) -> Result<Vec<Section>> {
    let mut sections = Vec::new();
    let mut section_index = 0;
    
    // For now, parse as a single section
    // In reality, we'd need to handle section breaks
    let section = parse_section(data, section_index)?;
    sections.push(section);
    
    Ok(sections)
}