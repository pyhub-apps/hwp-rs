/// Paragraph structure
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Paragraph {
    /// Paragraph header
    pub header: ParagraphHeader,
    
    /// Text content
    pub text: String,
    
    /// Character shapes
    pub char_shapes: Vec<CharShapePos>,
    
    /// Line segments
    pub line_segments: Vec<LineSegment>,
    
    /// Range tags
    pub range_tags: Vec<RangeTag>,
    
    /// Control characters
    pub controls: Vec<Control>,
}

impl Paragraph {
    /// Create a new empty paragraph
    pub fn new() -> Self {
        Self {
            header: ParagraphHeader::default(),
            text: String::new(),
            char_shapes: Vec::new(),
            line_segments: Vec::new(),
            range_tags: Vec::new(),
            controls: Vec::new(),
        }
    }
    
    /// Get the text content of the paragraph
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Paragraph header information
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParagraphHeader {
    pub text_count: u32,
    pub control_mask: u32,
    pub para_shape_id: u16,
    pub style_id: u8,
    pub division_type: u8,
    pub char_shape_count: u16,
    pub range_tag_count: u16,
    pub line_align_count: u16,
    pub instance_id: u32,
    pub is_merged_by_track: u16,
}

/// Character shape position
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharShapePos {
    pub position: u32,
    pub shape_id: u16,
}

/// Line segment information
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineSegment {
    pub text_start_pos: u32,
    pub line_height: i32,
    pub text_height: i32,
    pub baseline_gap: i32,
    pub line_spacing: i32,
    pub column_start_pos: u32,
    pub segment_width: i32,
    pub flags: u32,
}

/// Range tag for marking ranges in text
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeTag {
    pub start: u32,
    pub end: u32,
    pub tag_data: Vec<u8>,
}

/// Control character in paragraph
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Control {
    pub position: u32,
    pub control_type: ControlType,
    pub data: Vec<u8>,
}

/// Control types in paragraphs
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlType {
    Character(u16),           // Regular control character
    Inline(u32),              // Inline control object
    Extended(ExtendedControl), // Extended control object
}

/// Extended control types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExtendedControl {
    Table,
    GenShapeObject,
    Equation,
    Header,
    Footer,
    Footnote,
    Endnote,
    AutoNum,
    NewNum,
    PageNumPos,
    PageHiding,
    PageOddEvenAdjust,
    PageBreak,
    Field,
    Bookmark,
    IndexMark,
    HiddenComment,
    Other(u32),
}

impl ExtendedControl {
    /// Create from control ID
    pub fn from_ctrl_id(id: u32) -> Self {
        use crate::constants::ctrl_id::CtrlId;
        
        match CtrlId::from_u32(id) {
            Some(CtrlId::Table) => Self::Table,
            Some(CtrlId::GenShapeObject) => Self::GenShapeObject,
            Some(CtrlId::Equation) => Self::Equation,
            Some(CtrlId::Header) => Self::Header,
            Some(CtrlId::Footer) => Self::Footer,
            Some(CtrlId::Footnote) => Self::Footnote,
            Some(CtrlId::Endnote) => Self::Endnote,
            Some(CtrlId::AutoNum) => Self::AutoNum,
            Some(CtrlId::NewNum) => Self::NewNum,
            Some(CtrlId::PageNumPos) => Self::PageNumPos,
            Some(CtrlId::PageHiding) => Self::PageHiding,
            Some(CtrlId::PageOddEvenAdjust) => Self::PageOddEvenAdjust,
            Some(CtrlId::PageBreak) => Self::PageBreak,
            Some(CtrlId::Field) => Self::Field,
            Some(CtrlId::Bookmark) => Self::Bookmark,
            Some(CtrlId::IndexMark) => Self::IndexMark,
            _ => Self::Other(id),
        }
    }
}