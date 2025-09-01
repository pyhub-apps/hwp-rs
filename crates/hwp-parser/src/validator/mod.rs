use hwp_core::models::record::RecordHeader;
use hwp_core::{HwpError, Result};

/// Context for record validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordContext {
    /// DocInfo stream context
    DocInfo,
    /// BodyText/Section stream context
    BodyText,
    /// Unknown or generic context
    Unknown,
}

/// Record validation trait
pub trait RecordValidator {
    /// Validate record header against available data
    fn validate_header(&self, header: &RecordHeader, available: usize) -> Result<()>;

    /// Validate if tag ID is valid for the given context
    fn validate_tag_id(&self, tag_id: u16, context: RecordContext) -> bool;

    /// Validate if size is reasonable for the given tag
    fn validate_size(&self, size: u32, tag_id: u16) -> Result<()>;

    /// Validate record boundaries and alignment
    fn validate_boundaries(
        &self,
        header: &RecordHeader,
        position: usize,
        total_size: usize,
    ) -> Result<()>;
}

/// Default implementation of RecordValidator
pub struct DefaultRecordValidator {
    /// Maximum allowed record size (default: 100MB)
    max_record_size: u32,
    /// Whether to allow unknown tag IDs
    allow_unknown_tags: bool,
}

impl Default for DefaultRecordValidator {
    fn default() -> Self {
        Self {
            max_record_size: 100 * 1024 * 1024, // 100MB
            allow_unknown_tags: false,
        }
    }
}

impl DefaultRecordValidator {
    /// Create a new validator with custom settings
    pub fn new(max_record_size: u32, allow_unknown_tags: bool) -> Self {
        Self {
            max_record_size,
            allow_unknown_tags,
        }
    }

    /// Create a lenient validator that allows unknown tags
    pub fn lenient() -> Self {
        Self {
            max_record_size: 100 * 1024 * 1024,
            allow_unknown_tags: true,
        }
    }
}

impl RecordValidator for DefaultRecordValidator {
    fn validate_header(&self, header: &RecordHeader, available: usize) -> Result<()> {
        // Check if we have enough bytes for the record data
        let required_size = if header.has_extended_size() {
            // Extended size requires 4 additional bytes
            4
        } else {
            header.size() as usize
        };

        if required_size > available {
            return Err(HwpError::ValidationError {
                message: format!(
                    "Insufficient data for record: need {} bytes, have {} bytes",
                    required_size, available
                ),
            });
        }

        Ok(())
    }

    fn validate_tag_id(&self, tag_id: u16, context: RecordContext) -> bool {
        use hwp_core::constants::tag_id::{doc_info, section};

        match context {
            RecordContext::DocInfo => {
                matches!(
                    tag_id,
                    doc_info::DOCUMENT_PROPERTIES
                        | doc_info::ID_MAPPINGS
                        | doc_info::BIN_DATA
                        | doc_info::FACE_NAME
                        | doc_info::BORDER_FILL
                        | doc_info::CHAR_SHAPE
                        | doc_info::TAB_DEF
                        | doc_info::NUMBERING
                        | doc_info::BULLET
                        | doc_info::PARA_SHAPE
                        | doc_info::STYLE
                        | doc_info::DOC_DATA
                        | doc_info::DISTRIBUTE_DOC_DATA
                        | doc_info::COMPATIBLE_DOCUMENT
                        | doc_info::LAYOUT_COMPATIBILITY
                        | doc_info::TRACK_CHANGE
                        | doc_info::MEMO_SHAPE
                        | doc_info::FORBIDDEN_CHAR
                        // Note: TRACK_CHANGE_AUTHOR (0x0050) conflicts with PARA_HEADER
                        // So we don't include it here
                        | doc_info::CHANGE_TRACKING
                ) || self.allow_unknown_tags
            }
            RecordContext::BodyText => {
                matches!(
                    tag_id,
                    section::PARA_HEADER
                        | section::PARA_TEXT
                        | section::PARA_CHAR_SHAPE
                        | section::PARA_LINE_SEG
                        | section::PARA_RANGE_TAG
                        | section::CTRL_HEADER
                        | section::LIST_HEADER
                        | section::PAGE_DEF
                        | section::FOOTNOTE_SHAPE
                        | section::PAGE_BORDER_FILL
                        | section::SHAPE_COMPONENT
                        | section::TABLE
                        | section::SHAPE_COMPONENT_LINE
                        | section::SHAPE_COMPONENT_RECTANGLE
                        | section::SHAPE_COMPONENT_ELLIPSE
                        | section::SHAPE_COMPONENT_ARC
                        | section::SHAPE_COMPONENT_POLYGON
                        | section::SHAPE_COMPONENT_CURVE
                        | section::SHAPE_COMPONENT_OLE
                        | section::SHAPE_COMPONENT_PICTURE
                        | section::SHAPE_COMPONENT_CONTAINER
                        | section::CTRL_DATA
                        | section::EQEDIT
                        | section::SHAPE_COMPONENT_TEXTART
                        | section::FORM_OBJECT
                        | section::MEMO_LIST
                        | section::CHART_DATA
                        | section::VIDEO_DATA
                        | section::SHAPE_COMPONENT_UNKNOWN
                ) || self.allow_unknown_tags
            }
            RecordContext::Unknown => self.allow_unknown_tags,
        }
    }

    fn validate_size(&self, size: u32, tag_id: u16) -> Result<()> {
        // Global maximum size check
        if size > self.max_record_size {
            return Err(HwpError::ValidationError {
                message: format!(
                    "Record size {} exceeds maximum allowed size {} for tag 0x{:04X}",
                    size, self.max_record_size, tag_id
                ),
            });
        }

        // Tag-specific size validation
        use hwp_core::constants::tag_id::doc_info;

        match tag_id {
            doc_info::DOCUMENT_PROPERTIES => {
                // Document properties should be at least 22 bytes
                if size < 22 {
                    return Err(HwpError::ValidationError {
                        message: format!(
                            "Document properties record too small: {} bytes (minimum 22)",
                            size
                        ),
                    });
                }
            }
            doc_info::FACE_NAME => {
                // Face name should be at least 3 bytes (properties + length)
                if size < 3 {
                    return Err(HwpError::ValidationError {
                        message: format!("Face name record too small: {} bytes (minimum 3)", size),
                    });
                }
            }
            _ => {
                // No specific validation for other tags yet
            }
        }

        Ok(())
    }

    fn validate_boundaries(
        &self,
        header: &RecordHeader,
        position: usize,
        total_size: usize,
    ) -> Result<()> {
        let record_end = position + 4 + header.size() as usize;

        if record_end > total_size {
            return Err(HwpError::ValidationError {
                message: format!(
                    "Record at position {} extends beyond stream boundary (ends at {}, stream size {})",
                    position, record_end, total_size
                ),
            });
        }

        Ok(())
    }
}

/// Record recovery utilities
pub mod recovery {
    use super::*;
    use crate::reader::ByteReader;

    /// Try to find the next valid record header after an error
    pub fn find_next_valid_record(
        reader: &mut ByteReader,
        validator: &dyn RecordValidator,
        context: RecordContext,
    ) -> Option<(usize, RecordHeader)> {
        let start_pos = reader.position();
        let mut search_pos = start_pos;

        // Scan byte by byte looking for a valid header
        while search_pos + 4 <= reader.len() {
            if let Ok(()) = reader.seek(search_pos) {
                if let Ok(header_bytes) = reader.peek_bytes(4) {
                    let mut array = [0u8; 4];
                    array.copy_from_slice(&header_bytes);
                    let header = RecordHeader::from_bytes(array);

                    // Check if this could be a valid record
                    if validator.validate_tag_id(header.tag_id(), context) {
                        let remaining = reader.len() - search_pos - 4;
                        if validator.validate_header(&header, remaining).is_ok() {
                            // Found a potentially valid record
                            return Some((search_pos, header));
                        }
                    }
                }
            }

            search_pos += 1;
        }

        None
    }

    /// Skip to the next record boundary
    pub fn skip_to_next_record(
        reader: &mut ByteReader,
        current_header: &RecordHeader,
    ) -> Result<()> {
        let skip_bytes = if current_header.has_extended_size() {
            // Skip extended size (4 bytes) + actual data
            4 // We'll read the actual size and skip that amount
        } else {
            current_header.size() as usize
        };

        reader.skip(skip_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_header_sufficient_data() {
        let validator = DefaultRecordValidator::default();
        // Create header with correct bit layout: tag(10) | level(10) | size(12)
        let value = (0x10_u32) | (0_u32 << 10) | (4_u32 << 20);
        let header = RecordHeader::from_bytes(value.to_le_bytes());

        assert!(validator.validate_header(&header, 100).is_ok());
        assert!(validator.validate_header(&header, 4).is_ok());
        assert!(validator.validate_header(&header, 3).is_err());
    }

    #[test]
    fn test_validate_tag_id() {
        let validator = DefaultRecordValidator::default();

        // Valid DocInfo tags
        assert!(validator.validate_tag_id(0x0010, RecordContext::DocInfo)); // DOCUMENT_PROPERTIES
        assert!(validator.validate_tag_id(0x0013, RecordContext::DocInfo)); // FACE_NAME

        // Invalid tag for DocInfo
        assert!(!validator.validate_tag_id(0x9999, RecordContext::DocInfo));

        // Valid BodyText tags
        assert!(validator.validate_tag_id(0x0050, RecordContext::BodyText)); // PARA_HEADER
        assert!(validator.validate_tag_id(0x0051, RecordContext::BodyText)); // PARA_TEXT

        // Test lenient validator
        let lenient = DefaultRecordValidator::lenient();
        assert!(lenient.validate_tag_id(0x9999, RecordContext::DocInfo));
    }

    #[test]
    fn test_validate_size() {
        let validator = DefaultRecordValidator::default();

        // Normal size
        assert!(validator.validate_size(1000, 0x0010).is_ok());

        // Too large
        assert!(validator.validate_size(200 * 1024 * 1024, 0x0010).is_err());

        // Document properties specific validation
        assert!(validator.validate_size(30, 0x0010).is_ok());
        assert!(validator.validate_size(10, 0x0010).is_err()); // Too small
    }

    #[test]
    fn test_validate_boundaries() {
        let validator = DefaultRecordValidator::default();
        // Create header with correct bit layout: tag(10) | level(10) | size(12)
        let value = (0x10_u32) | (0_u32 << 10) | (4_u32 << 20);
        let header = RecordHeader::from_bytes(value.to_le_bytes());

        // Valid boundaries
        assert!(validator.validate_boundaries(&header, 0, 100).is_ok());
        assert!(validator.validate_boundaries(&header, 10, 18).is_ok()); // 10 + 4 + 4 = 18

        // Invalid boundaries
        assert!(validator.validate_boundaries(&header, 10, 17).is_err()); // Not enough space
    }
}
