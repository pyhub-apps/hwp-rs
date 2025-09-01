use crate::models::{HwpHeader, Section};
use std::collections::HashMap;

/// Main HWP document structure
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HwpDocument {
    /// File header
    pub header: HwpHeader,
    
    /// Document information
    pub doc_info: DocInfo,
    
    /// Document sections
    pub sections: Vec<Section>,
    
    /// Binary data storage
    pub bin_data: HashMap<u16, Vec<u8>>,
}

impl HwpDocument {
    /// Create a new empty document
    pub fn new(header: HwpHeader) -> Self {
        Self {
            header,
            doc_info: DocInfo::default(),
            sections: Vec::new(),
            bin_data: HashMap::new(),
        }
    }
    
    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.sections.iter().map(|s| s.page_count()).sum()
    }
    
    /// Get all text content from the document
    pub fn get_text(&self) -> String {
        let mut text = String::new();
        for section in &self.sections {
            text.push_str(&section.get_text());
            text.push('\n');
        }
        text
    }
}

/// Document information container
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocInfo {
    /// Document properties
    pub properties: DocumentProperties,
    
    /// Character shapes
    pub char_shapes: Vec<CharShape>,
    
    /// Paragraph shapes
    pub para_shapes: Vec<ParaShape>,
    
    /// Styles
    pub styles: Vec<Style>,
    
    /// Face names (fonts)
    pub face_names: Vec<FaceName>,
    
    /// Border fills
    pub border_fills: Vec<BorderFill>,
    
    /// ID mappings for internal references
    pub id_mappings: Vec<u32>,
    
    /// Binary data entries (embedded files, images, etc.)
    pub bin_data_entries: Vec<BinDataEntry>,
    
    /// Document-specific data
    pub doc_data: Vec<u8>,
    
    /// Tab definitions
    pub tab_defs: Vec<TabDef>,
    
    /// Numbering definitions
    pub numberings: Vec<Numbering>,
    
    /// Bullet definitions  
    pub bullets: Vec<Bullet>,
    
    /// Document distribution data
    pub distribute_doc_data: Option<DistributeDocData>,
    
    /// Compatible document settings
    pub compatible_document: Option<CompatibleDocument>,
    
    /// Layout compatibility settings
    pub layout_compatibility: Option<LayoutCompatibility>,
    
    /// Track changes
    pub track_changes: Vec<TrackChange>,
    
    /// Track change authors
    pub track_change_authors: Vec<TrackChangeAuthor>,
    
    /// Memo shapes
    pub memo_shapes: Vec<MemoShape>,
    
    /// Forbidden characters
    pub forbidden_chars: Option<ForbiddenChar>,
}

/// Document properties
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentProperties {
    pub section_count: u16,
    pub page_start_number: u16,
    pub footnote_start_number: u16,
    pub endnote_start_number: u16,
    pub picture_start_number: u16,
    pub table_start_number: u16,
    pub equation_start_number: u16,
    pub total_character_count: u32,
    pub total_page_count: u32,
}

/// Character shape information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharShape {
    pub face_name_ids: Vec<u16>,
    pub ratios: Vec<u8>,
    pub char_spaces: Vec<i8>,
    pub rel_sizes: Vec<u8>,
    pub char_offsets: Vec<i8>,
    pub base_size: u32,
    pub properties: u32,
    pub shadow_gap_x: i8,
    pub shadow_gap_y: i8,
    pub text_color: u32,
    pub underline_color: u32,
    pub shade_color: u32,
    pub shadow_color: u32,
    pub border_fill_id: Option<u16>,
}

/// Paragraph shape information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParaShape {
    pub properties1: u32,
    pub left_margin: i32,
    pub right_margin: i32,
    pub indent: i32,
    pub prev_spacing: i32,
    pub next_spacing: i32,
    pub line_spacing: i32,
    pub tab_def_id: u16,
    pub numbering_id: u16,
    pub border_fill_id: u16,
    pub border_offset_left: i16,
    pub border_offset_right: i16,
    pub border_offset_top: i16,
    pub border_offset_bottom: i16,
    pub properties2: u32,
    pub properties3: u32,
    pub line_spacing_type: u32,
}

/// Style information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Style {
    pub name: String,
    pub english_name: String,
    pub properties: u8,
    pub next_style_id: u8,
    pub lang_id: u16,
    pub para_shape_id: u16,
    pub char_shape_id: u16,
}

/// Face name (font) information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FaceName {
    pub properties: u8,
    pub name: String,
    pub substitute_font_type: Option<u8>,
    pub substitute_font_name: Option<String>,
    pub type_info: FaceNameType,
    pub base_font_name: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FaceNameType {
    pub family: u8,
    pub serif: u8,
    pub weight: u8,
    pub proportion: u8,
    pub contrast: u8,
    pub stroke_variation: u8,
    pub arm_style: u8,
    pub letter_form: u8,
    pub midline: u8,
    pub x_height: u8,
}

/// Border and fill information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderFill {
    pub properties: u16,
    pub left_border: BorderLine,
    pub right_border: BorderLine,
    pub top_border: BorderLine,
    pub bottom_border: BorderLine,
    pub diagonal_border: BorderLine,
    pub fill_type: u8,
    pub fill_data: Vec<u8>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderLine {
    pub line_type: u8,
    pub thickness: u8,
    pub color: u32,
}

/// Binary data entry
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinDataEntry {
    pub id: u16,
    pub link_type: u8,
    pub compression_type: u8,
    pub data: Vec<u8>,
}

/// Tab definition
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TabDef {
    pub properties: u32,
    pub count: u32,
    pub tabs: Vec<TabInfo>,
}

/// Individual tab information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TabInfo {
    pub position: i32,
    pub tab_type: u8,
    pub fill_type: u8,
}

/// Numbering definition
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Numbering {
    pub levels: Vec<NumberingLevel>,
}

/// Numbering level information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NumberingLevel {
    pub properties: u32,
    pub paragraph_shape_id: u16,
    pub format: String,
    pub start_number: u16,
}

/// Bullet definition
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bullet {
    pub properties: u32,
    pub paragraph_shape_id: u16,
    pub bullet_char: Option<String>,
    pub image_id: Option<u16>,
}

/// Document distribution data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistributeDocData {
    pub data: Vec<u8>,
}

/// Compatible document settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompatibleDocument {
    pub target_program: u32,
}

/// Layout compatibility settings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayoutCompatibility {
    pub letter_spacing: u32,
    pub paragraph_spacing: u32,
    pub line_grid: u32,
    pub paragraph_grid: u32,
    pub snap_to_grid: u32,
}

/// Track change information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackChange {
    pub properties: u32,
    pub author_id: u16,
    pub timestamp: u64,
    pub change_type: u16,
    pub data: Vec<u8>,
}

/// Track change author
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackChangeAuthor {
    pub id: u16,
    pub name: String,
}

/// Memo shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoShape {
    pub properties: u32,
    pub memo_id: u32,
    pub width: i32,
    pub line_count: u16,
    pub line_spacing: i16,
    pub line_type: u8,
    pub line_color: u32,
}

/// Forbidden characters
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForbiddenChar {
    pub forbidden_chars: String,
    pub allowed_chars: String,
}