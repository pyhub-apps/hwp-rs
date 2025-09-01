use crate::parser::record::RecordParser;
use crate::reader::ByteReader;
use crate::validator::RecordContext;
use hwp_core::constants::tag_id::section;
use hwp_core::models::paragraph::{CharShapePos, LineSegment, ParagraphHeader};
use hwp_core::models::section::Section;
use hwp_core::models::Paragraph;
use hwp_core::Result;

/// Parse a section from decompressed data
pub fn parse_section(data: &[u8], _section_index: usize) -> Result<Section> {
    let mut parser = RecordParser::new_with_context(data, RecordContext::BodyText);
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
                            let char_shapes = parse_para_char_shapes(
                                &next_record.data,
                                para_header.char_shape_count,
                            )?;
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

    let mut header = ParagraphHeader {
        text_count: reader.read_u32()?,
        control_mask: reader.read_u32()?,
        para_shape_id: reader.read_u16()?,
        style_id: reader.read_u8()?,
        division_type: reader.read_u8()?,
        char_shape_count: reader.read_u16()?,
        range_tag_count: reader.read_u16()?,
        line_align_count: reader.read_u16()?,
        instance_id: reader.read_u32()?,
        ..Default::default()
    };

    // Optional field based on version
    if reader.remaining() >= 2 {
        header.is_merged_by_track = reader.read_u16()?;
    }

    Ok(header)
}

/// Parse paragraph text with proper control character handling
fn parse_para_text(data: &[u8]) -> Result<String> {
    // Text is stored as UTF-16LE
    let mut text = String::new();
    let mut i = 0;

    while i + 1 < data.len() {
        let ch = u16::from_le_bytes([data[i], data[i + 1]]);
        i += 2;

        // Handle special characters and control codes
        match ch {
            0x0000 => break,           // Null terminator
            0x0009 => text.push('\t'), // Tab
            0x000A => text.push('\n'), // Line feed
            0x000D => continue,        // Carriage return (skip in Windows-style line endings)

            // HWP specific control characters
            0x0001 => {
                // Reserved for future use
                continue;
            }
            0x0002 => {
                // Section column definition - marks column break
                // For text extraction, we can treat this as a space or newline
                text.push(' ');
            }
            0x0003 => {
                // Section definition - marks section break
                text.push('\n');
            }
            0x0004..=0x0007 => {
                // Reserved control characters
                continue;
            }
            0x0008 => {
                // Field start - inline control object follows
                // For now, skip the control data
                if i + 5 < data.len() {
                    // Control objects have additional data we need to skip
                    // Format: type(4 bytes) + additional data
                    i += 8; // Skip control ID and basic info
                            // TODO: Parse control objects properly
                }
                continue;
            }
            0x000B => {
                // Drawing object/table - for text extraction, skip
                if i + 5 < data.len() {
                    i += 8; // Skip control data
                }
                continue;
            }
            0x000C => {
                // Form feed / page break
                text.push('\n');
            }
            0x000E..=0x0017 => {
                // Reserved for special controls
                continue;
            }
            0x0018 => {
                // Column break
                text.push('\n');
            }
            0x0019 => {
                // Section break
                text.push('\n');
            }
            0x001A..=0x001D => {
                // Reserved
                continue;
            }
            0x001E => {
                // Hyphen
                text.push('-');
            }
            0x001F => {
                // Non-breaking space
                text.push('\u{00A0}');
            }
            _ => {
                // Regular character
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

    while reader.remaining() >= 32 {
        // Each line segment is 32 bytes
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
    let section_index = 0;

    // For now, parse as a single section
    // In reality, we'd need to handle section breaks
    let section = parse_section(data, section_index)?;
    sections.push(section);

    Ok(sections)
}
