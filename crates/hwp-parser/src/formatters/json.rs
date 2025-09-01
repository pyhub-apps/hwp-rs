use super::{OutputFormatter, FormatOptions};
use hwp_core::{HwpDocument, Result};
use hwp_core::models::{Section, Paragraph};
use hwp_core::models::document::DocInfo;
use serde::{Serialize, Deserialize};
use serde_json;

/// JSON formatter - structured document representation
pub struct JsonFormatter {
    options: FormatOptions,
}

impl JsonFormatter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }
}

/// JSON representation of an HWP document
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonDocument {
    pub metadata: JsonMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<JsonStyles>,
    pub content: JsonContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    pub version: String,
    pub page_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStyles {
    pub fonts: Vec<JsonFont>,
    pub paragraph_styles: Vec<JsonParagraphStyle>,
    pub character_styles: Vec<JsonCharacterStyle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonFont {
    pub id: u16,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonParagraphStyle {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonCharacterStyle {
    pub id: u16,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonContent {
    pub sections: Vec<JsonSection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonSection {
    pub index: usize,
    pub paragraphs: Vec<JsonParagraph>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonParagraph {
    pub index: usize,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<JsonFormatting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonFormatting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<f32>,
}

impl OutputFormatter for JsonFormatter {
    fn format_document(&self, doc: &HwpDocument) -> Result<String> {
        // Build JSON document structure
        let mut json_doc = JsonDocument {
            metadata: JsonMetadata {
                title: None, // TODO: Extract from DocInfo when available
                author: None, // TODO: Extract from DocInfo when available
                created: None, // TODO: Extract from DocInfo when available
                version: format!("{}", doc.header.version),
                page_count: doc.page_count(),
            },
            styles: None,
            content: JsonContent {
                sections: Vec::new(),
            },
        };
        
        // Add styles if requested
        if self.options.json_include_styles {
            json_doc.styles = Some(self.extract_styles(&doc.doc_info));
        }
        
        // Convert sections
        for (index, section) in doc.sections.iter().enumerate() {
            let mut json_section = JsonSection {
                index,
                paragraphs: Vec::new(),
            };
            
            for (para_index, paragraph) in section.paragraphs.iter().enumerate() {
                if !paragraph.text.is_empty() {
                    json_section.paragraphs.push(JsonParagraph {
                        index: para_index,
                        text: paragraph.text.clone(),
                        style: None, // TODO: Map paragraph style ID to name
                        formatting: None, // TODO: Extract formatting from paragraph
                    });
                }
            }
            
            json_doc.content.sections.push(json_section);
        }
        
        // Serialize to JSON string
        if self.options.json_pretty {
            serde_json::to_string_pretty(&json_doc)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        } else {
            serde_json::to_string(&json_doc)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        }
    }
    
    fn format_metadata(&self, doc_info: &DocInfo) -> Result<String> {
        let metadata = JsonMetadata {
            title: None, // TODO: Extract when DocInfo is more complete
            author: None,
            created: None,
            version: String::new(),
            page_count: 0,
        };
        
        if self.options.json_pretty {
            serde_json::to_string_pretty(&metadata)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        } else {
            serde_json::to_string(&metadata)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        }
    }
    
    fn format_section(&self, section: &Section, index: usize) -> Result<String> {
        let mut json_section = JsonSection {
            index,
            paragraphs: Vec::new(),
        };
        
        for (para_index, paragraph) in section.paragraphs.iter().enumerate() {
            if !paragraph.text.is_empty() {
                json_section.paragraphs.push(JsonParagraph {
                    index: para_index,
                    text: paragraph.text.clone(),
                    style: None,
                    formatting: None,
                });
            }
        }
        
        if self.options.json_pretty {
            serde_json::to_string_pretty(&json_section)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        } else {
            serde_json::to_string(&json_section)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        }
    }
    
    fn format_paragraph(&self, paragraph: &Paragraph, index: usize) -> Result<String> {
        let json_para = JsonParagraph {
            index,
            text: paragraph.text.clone(),
            style: None,
            formatting: None,
        };
        
        if self.options.json_pretty {
            serde_json::to_string_pretty(&json_para)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        } else {
            serde_json::to_string(&json_para)
                .map_err(|e| hwp_core::HwpError::EncodingError(e.to_string()))
        }
    }
}

impl JsonFormatter {
    fn extract_styles(&self, doc_info: &DocInfo) -> JsonStyles {
        let mut styles = JsonStyles {
            fonts: Vec::new(),
            paragraph_styles: Vec::new(),
            character_styles: Vec::new(),
        };
        
        // Extract font information
        for (id, face_name) in doc_info.face_names.iter().enumerate() {
            styles.fonts.push(JsonFont {
                id: id as u16,
                name: face_name.name.clone(),
                english_name: None, // TODO: Add when english_name is available in FaceName
            });
        }
        
        // Extract character styles
        for (id, char_shape) in doc_info.char_shapes.iter().enumerate() {
            styles.character_styles.push(JsonCharacterStyle {
                id: id as u16,
                name: format!("CharStyle{}", id),
                font_size: Some(char_shape.base_size as f32 / 100.0), // Convert from HWPUNIT
                bold: if char_shape.properties & 0x01 != 0 { Some(true) } else { None },
                italic: if char_shape.properties & 0x02 != 0 { Some(true) } else { None },
            });
        }
        
        // Extract paragraph styles
        for (id, para_shape) in doc_info.para_shapes.iter().enumerate() {
            styles.paragraph_styles.push(JsonParagraphStyle {
                id: id as u16,
                name: format!("ParaStyle{}", id),
            });
        }
        
        styles
    }
}