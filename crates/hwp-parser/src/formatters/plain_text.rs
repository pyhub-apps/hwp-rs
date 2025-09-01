use super::{OutputFormatter, FormatOptions};
use crate::text_extractor::TextExtractor;
use hwp_core::{HwpDocument, Result};
use hwp_core::models::{Section, Paragraph};
use hwp_core::models::document::DocInfo;

/// Plain text formatter - simple text extraction
pub struct PlainTextFormatter {
    options: FormatOptions,
}

impl PlainTextFormatter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }
    
    fn wrap_text(&self, text: &str) -> String {
        if let Some(width) = self.options.text_width {
            // Simple word wrapping
            let mut result = String::new();
            for line in text.lines() {
                if line.len() <= width {
                    result.push_str(line);
                    result.push('\n');
                } else {
                    let mut current_line = String::new();
                    for word in line.split_whitespace() {
                        if current_line.is_empty() {
                            current_line = word.to_string();
                        } else if current_line.len() + 1 + word.len() <= width {
                            current_line.push(' ');
                            current_line.push_str(word);
                        } else {
                            result.push_str(&current_line);
                            result.push('\n');
                            current_line = word.to_string();
                        }
                    }
                    if !current_line.is_empty() {
                        result.push_str(&current_line);
                        result.push('\n');
                    }
                }
            }
            result
        } else {
            text.to_string()
        }
    }
}

impl OutputFormatter for PlainTextFormatter {
    fn format_document(&self, doc: &HwpDocument) -> Result<String> {
        // Use existing TextExtractor for plain text
        let text = TextExtractor::extract_from_document(doc)?;
        
        // Apply text wrapping if configured
        let formatted = self.wrap_text(&text);
        
        // Add page breaks if configured
        if self.options.text_page_breaks {
            // TODO: Detect and preserve page breaks from the document
            // For now, just return the formatted text
            Ok(formatted)
        } else {
            Ok(formatted)
        }
    }
    
    fn format_metadata(&self, _doc_info: &DocInfo) -> Result<String> {
        // Plain text doesn't include metadata
        Ok(String::new())
    }
    
    fn format_section(&self, section: &Section, _index: usize) -> Result<String> {
        let mut text = String::new();
        
        for paragraph in &section.paragraphs {
            if !paragraph.text.is_empty() {
                text.push_str(&self.wrap_text(&paragraph.text));
                text.push('\n');
            }
        }
        
        Ok(text)
    }
    
    fn format_paragraph(&self, paragraph: &Paragraph, _index: usize) -> Result<String> {
        Ok(self.wrap_text(&paragraph.text))
    }
}