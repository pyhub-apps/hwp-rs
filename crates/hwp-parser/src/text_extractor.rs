use crate::cfb::parse_cfb_bytes;
use crate::parser::section::parse_body_text;
use hwp_core::{HwpDocument, HwpError, Result};
use std::io::Cursor;

/// Text extractor for HWP documents
///
/// Provides functionality to extract plain text from HWP files,
/// handling Korean text encoding, control characters, and document structure.
pub struct TextExtractor;

impl TextExtractor {
    /// Extract text from raw HWP file bytes
    pub fn extract_from_bytes(hwp_data: &[u8]) -> Result<String> {
        // Parse CFB container
        let mut container = parse_cfb_bytes(hwp_data).map_err(|e| HwpError::ParseError {
            offset: 0,
            message: format!("Failed to parse CFB: {}", e),
        })?;

        let mut cursor = Cursor::new(hwp_data);
        let mut full_text = String::new();

        // Process all BodyText sections
        let mut section_index = 0;
        loop {
            let stream_name = format!("BodyText/Section{}", section_index);

            // Check if this section exists
            if !container.has_stream(&stream_name) {
                if section_index == 0 {
                    // No sections at all
                    return Err(HwpError::ParseError {
                        offset: 0,
                        message: "No BodyText sections found".to_string(),
                    });
                }
                break;
            }

            // Read and decompress the section stream
            let stream = container
                .read_stream(&mut cursor, &stream_name)
                .map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to read stream {}: {}", stream_name, e),
                })?;

            let section_data = if stream.is_compressed() {
                stream.decompress().map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to decompress {}: {}", stream_name, e),
                })?
            } else {
                stream.as_bytes().to_vec()
            };

            // Parse the section and extract text
            let sections = parse_body_text(&section_data)?;
            for section in sections {
                for paragraph in &section.paragraphs {
                    if !paragraph.text.is_empty() {
                        full_text.push_str(&paragraph.text);
                        full_text.push('\n');
                    }
                }
            }

            section_index += 1;
        }

        Ok(full_text.trim().to_string())
    }

    /// Extract text from a parsed HWP document
    pub fn extract_from_document(doc: &HwpDocument) -> Result<String> {
        let mut text = String::new();

        for section in &doc.sections {
            for paragraph in &section.paragraphs {
                if !paragraph.text.is_empty() {
                    text.push_str(&paragraph.text);
                    text.push('\n');
                }
            }
        }

        Ok(text.trim().to_string())
    }

    /// Extract text from a single section's raw data
    pub fn extract_from_section(section_data: &[u8]) -> Result<String> {
        let sections = parse_body_text(section_data)?;
        let mut text = String::new();

        for section in sections {
            for paragraph in &section.paragraphs {
                if !paragraph.text.is_empty() {
                    text.push_str(&paragraph.text);
                    text.push('\n');
                }
            }
        }

        Ok(text.trim().to_string())
    }
}

/// Formatted text with paragraph structure preserved
#[derive(Debug, Clone)]
pub struct FormattedText {
    pub paragraphs: Vec<FormattedParagraph>,
}

/// A formatted paragraph with text and metadata
#[derive(Debug, Clone)]
pub struct FormattedParagraph {
    pub text: String,
    pub level: u8, // Heading level, 0 for normal text
    pub is_list_item: bool,
}

impl TextExtractor {
    /// Extract text with formatting information preserved
    pub fn extract_with_formatting(doc: &HwpDocument) -> Result<FormattedText> {
        let mut formatted = FormattedText {
            paragraphs: Vec::new(),
        };

        for section in &doc.sections {
            for paragraph in &section.paragraphs {
                if !paragraph.text.is_empty() {
                    formatted.paragraphs.push(FormattedParagraph {
                        text: paragraph.text.clone(),
                        level: 0,            // TODO: Determine from paragraph properties
                        is_list_item: false, // TODO: Determine from paragraph properties
                    });
                }
            }
        }

        Ok(formatted)
    }
}
