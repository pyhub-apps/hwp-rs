/// Record structure for HWP tag-based format
#[derive(Debug)]
pub struct Record {
    /// Tag ID identifying the record type
    pub tag_id: u16,
    
    /// Hierarchy level (0-3)
    pub level: u8,
    
    /// Size of the record data
    pub size: u32,
    
    /// Record data
    pub data: Vec<u8>,
}

impl Record {
    /// Create a new record
    pub fn new(tag_id: u16, level: u8, size: u32, data: Vec<u8>) -> Self {
        Self {
            tag_id,
            level,
            size,
            data,
        }
    }
    
    /// Get the total size including header
    pub fn total_size(&self) -> usize {
        4 + self.size as usize // 4 bytes for header + data size
    }
    
    /// Check if this is a DocInfo record
    pub fn is_doc_info(&self) -> bool {
        use crate::constants::tag_id::doc_info::*;
        matches!(
            self.tag_id,
            DOCUMENT_PROPERTIES
                | ID_MAPPINGS
                | BIN_DATA
                | FACE_NAME
                | BORDER_FILL
                | CHAR_SHAPE
                | TAB_DEF
                | NUMBERING
                | BULLET
                | PARA_SHAPE
                | STYLE
                | DOC_DATA
                | DISTRIBUTE_DOC_DATA
                | COMPATIBLE_DOCUMENT
                | LAYOUT_COMPATIBILITY
                | TRACK_CHANGE
                | MEMO_SHAPE
                | FORBIDDEN_CHAR
                | TRACK_CHANGE_AUTHOR
                | CHANGE_TRACKING
        )
    }
    
    /// Check if this is a Section record
    pub fn is_section(&self) -> bool {
        use crate::constants::tag_id::section::*;
        matches!(
            self.tag_id,
            PARA_HEADER
                | PARA_TEXT
                | PARA_CHAR_SHAPE
                | PARA_LINE_SEG
                | PARA_RANGE_TAG
                | CTRL_HEADER
                | LIST_HEADER
                | PAGE_DEF
                | FOOTNOTE_SHAPE
                | PAGE_BORDER_FILL
                | SHAPE_COMPONENT
                | TABLE
                | SHAPE_COMPONENT_LINE
                | SHAPE_COMPONENT_RECTANGLE
                | SHAPE_COMPONENT_ELLIPSE
                | SHAPE_COMPONENT_ARC
                | SHAPE_COMPONENT_POLYGON
                | SHAPE_COMPONENT_CURVE
                | SHAPE_COMPONENT_OLE
                | SHAPE_COMPONENT_PICTURE
                | SHAPE_COMPONENT_CONTAINER
                | CTRL_DATA
                | EQEDIT
                | SHAPE_COMPONENT_TEXTART
                | FORM_OBJECT
                | MEMO_LIST
                | CHART_DATA
                | VIDEO_DATA
                | SHAPE_COMPONENT_UNKNOWN
        )
    }
}

/// Record header for parsing
#[derive(Debug, Clone, Copy)]
pub struct RecordHeader {
    /// Tag ID (10 bits) + Level (2 bits) + Size (20 bits) packed in 32 bits
    pub value: u32,
}

impl RecordHeader {
    /// Parse header from raw bytes (4 bytes)
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        let value = u32::from_le_bytes(bytes);
        Self { value }
    }
    
    /// Get the tag ID (bits 0-9, 10 bits)
    pub fn tag_id(&self) -> u16 {
        (self.value & 0x3FF) as u16
    }
    
    /// Get the level (bits 10-11, 2 bits)
    pub fn level(&self) -> u8 {
        ((self.value >> 10) & 0x3) as u8
    }
    
    /// Get the size (bits 12-31, 20 bits)
    pub fn size(&self) -> u32 {
        self.value >> 12
    }
    
    /// Check if this record has extended size
    pub fn has_extended_size(&self) -> bool {
        self.size() == 0xFFFFF  // 20 bits all set to 1
    }
}