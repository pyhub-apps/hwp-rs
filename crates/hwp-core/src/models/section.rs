use crate::models::Paragraph;

/// Section structure representing a document section
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Section {
    /// Section definition
    pub definition: SectionDefinition,

    /// Paragraphs in this section
    pub paragraphs: Vec<Paragraph>,

    /// Page definitions
    pub page_defs: Vec<PageDef>,

    /// Footnote shape
    pub footnote_shape: Option<FootnoteShape>,

    /// Page border fill
    pub page_border_fill: Option<PageBorderFill>,
}

impl Section {
    /// Create a new empty section
    pub fn new() -> Self {
        Self {
            definition: SectionDefinition::default(),
            paragraphs: Vec::new(),
            page_defs: Vec::new(),
            footnote_shape: None,
            page_border_fill: None,
        }
    }

    /// Get the page count for this section
    pub fn page_count(&self) -> usize {
        // Simple estimation based on content
        // In real implementation, this would calculate based on layout
        1
    }

    /// Get all text content from the section
    pub fn get_text(&self) -> String {
        let mut text = String::new();
        for paragraph in &self.paragraphs {
            text.push_str(&paragraph.get_text());
            text.push('\n');
        }
        text
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

/// Section definition information
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SectionDefinition {
    pub properties: u32,
    pub column_count: u16,
    pub column_direction: u16,
    pub column_gap: u16,
    pub starting_page: u16,
    pub starting_page_number: u16,
    pub hide_empty_line: bool,
    pub text_protection: bool,
    pub hide_header: bool,
    pub hide_footer: bool,
    pub hide_master_page: bool,
    pub hide_border: bool,
    pub hide_fill: bool,
    pub hide_page_number: bool,
    pub border_fill_id: u16,
    pub text_vertical_alignment: u8,
    pub header_footer_different_first: bool,
    pub header_footer_different_odd_even: bool,
}

/// Page definition
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageDef {
    pub width: u32,
    pub height: u32,
    pub padding_left: u32,
    pub padding_right: u32,
    pub padding_top: u32,
    pub padding_bottom: u32,
    pub header_padding: u32,
    pub footer_padding: u32,
    pub gutter_padding: u32,
    pub properties: u32,
    pub footnote_shape_id: u16,
}

impl Default for PageDef {
    fn default() -> Self {
        // A4 size defaults (210mm x 297mm in HWPUNIT)
        // 1mm = 7200 HWPUNIT
        Self {
            width: 59528,  // 210mm * 283.465 (approximately)
            height: 84188, // 297mm * 283.465 (approximately)
            padding_left: 8504,
            padding_right: 8504,
            padding_top: 5668,
            padding_bottom: 4252,
            header_padding: 4252,
            footer_padding: 4252,
            gutter_padding: 0,
            properties: 0,
            footnote_shape_id: 0,
        }
    }
}

/// Footnote shape
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FootnoteShape {
    pub properties: u32,
    pub user_symbol: String,
    pub prefix_symbol: String,
    pub suffix_symbol: String,
    pub starting_number: u16,
    pub divider_length: u32,
    pub divider_margin_top: u16,
    pub divider_margin_bottom: u16,
    pub notes_margin_top: u16,
    pub notes_margin_bottom: u16,
    pub divider_type: u8,
    pub divider_thickness: u8,
    pub divider_color: u32,
}

/// Page border fill
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageBorderFill {
    pub properties: u32,
    pub position_criteria: u8,
    pub include_header: bool,
    pub include_footer: bool,
    pub fill_area: u8,
    pub offset_left: i16,
    pub offset_right: i16,
    pub offset_top: i16,
    pub offset_bottom: i16,
    pub border_fill_id: u16,
}
