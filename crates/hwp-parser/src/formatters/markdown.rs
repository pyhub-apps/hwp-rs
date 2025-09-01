use super::{FormatOptions, OutputFormatter};
use hwp_core::models::document::DocInfo;
use hwp_core::models::{Paragraph, Section};
use hwp_core::{HwpDocument, Result};

/// Markdown formatter - converts HWP to Markdown format
pub struct MarkdownFormatter {
    options: FormatOptions,
}

impl MarkdownFormatter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    /// Escape special Markdown characters
    fn escape_markdown(&self, text: &str) -> String {
        let mut result = String::new();
        for ch in text.chars() {
            match ch {
                '*' | '_' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!' | '`' | '\\' => {
                    result.push('\\');
                    result.push(ch);
                }
                _ => result.push(ch),
            }
        }
        result
    }

    /// Generate table of contents
    fn generate_toc(&self, doc: &HwpDocument) -> String {
        let mut toc = String::from("## Table of Contents\n\n");

        // For now, generate a simple TOC based on sections
        for (index, section) in doc.sections.iter().enumerate() {
            if !section.paragraphs.is_empty() {
                toc.push_str(&format!(
                    "- [Section {}](#section-{})\n",
                    index + 1,
                    index + 1
                ));
            }
        }

        toc.push('\n');
        toc
    }

    /// Convert paragraph to Markdown with basic formatting
    fn format_paragraph_markdown(&self, paragraph: &Paragraph) -> String {
        if paragraph.text.is_empty() {
            return String::new();
        }

        // For now, return plain text
        // TODO: Detect and apply formatting (bold, italic, etc.)
        let text = paragraph.text.trim();

        // Check if it looks like a heading (simple heuristic)
        if text.len() < 100 && !text.contains('\n') {
            // Could be a heading, but we need more info from paragraph properties
            // For now, just return as regular paragraph
            format!("{}\n", text)
        } else {
            format!("{}\n", text)
        }
    }

    /// Check if text looks like a list item
    fn is_list_item(&self, text: &str) -> bool {
        let trimmed = text.trim_start();
        trimmed.starts_with("• ") || 
        trimmed.starts_with("- ") ||
        trimmed.starts_with("* ") ||
        trimmed.starts_with("+ ") ||
        // Check for numbered lists
        trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) &&
        trimmed.chars().nth(1).map_or(false, |c| c == '.' || c == ')')
    }

    /// Format a list item
    fn format_list_item(&self, text: &str) -> String {
        let trimmed = text.trim_start();

        // Handle bullet points (• is 3 bytes in UTF-8)
        if trimmed.starts_with("• ") {
            // Skip the bullet and space (need to handle UTF-8 properly)
            let content = trimmed.chars().skip(2).collect::<String>();
            format!("- {}", content)
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
        {
            format!("- {}", &trimmed[2..])
        }
        // Handle numbered lists
        else if let Some(dot_pos) = trimmed.find(". ") {
            if dot_pos < 3 && trimmed[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
                format!("{}. {}", &trimmed[..dot_pos], &trimmed[dot_pos + 2..])
            } else {
                text.to_string()
            }
        } else {
            text.to_string()
        }
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format_document(&self, doc: &HwpDocument) -> Result<String> {
        let mut markdown = String::new();

        // Add document title if available
        // TODO: Extract from DocInfo when available
        markdown.push_str("# Document\n\n");

        // Add table of contents if requested
        if self.options.markdown_toc {
            markdown.push_str(&self.generate_toc(doc));
        }

        // Convert sections
        for (index, section) in doc.sections.iter().enumerate() {
            if !section.paragraphs.is_empty() {
                // Add section header
                if doc.sections.len() > 1 {
                    markdown.push_str(&format!("## Section {}\n\n", index + 1));
                }

                // Process paragraphs
                let mut in_list = false;
                for paragraph in &section.paragraphs {
                    if paragraph.text.is_empty() {
                        if in_list {
                            markdown.push('\n');
                            in_list = false;
                        }
                        continue;
                    }

                    let text = paragraph.text.trim();

                    // Check if this is a list item
                    if self.is_list_item(text) {
                        markdown.push_str(&self.format_list_item(text));
                        markdown.push('\n');
                        in_list = true;
                    } else {
                        if in_list {
                            markdown.push('\n');
                            in_list = false;
                        }
                        markdown.push_str(&self.format_paragraph_markdown(paragraph));
                        markdown.push('\n');
                    }
                }
            }
        }

        Ok(markdown.trim().to_string())
    }

    fn format_metadata(&self, doc_info: &DocInfo) -> Result<String> {
        // Format as YAML front matter for Markdown
        let mut front_matter = String::from("---\n");

        // TODO: Add actual metadata when DocInfo is more complete
        front_matter.push_str("title: Document\n");
        front_matter.push_str("date: \n");
        front_matter.push_str("author: \n");

        front_matter.push_str("---\n\n");

        Ok(front_matter)
    }

    fn format_section(&self, section: &Section, index: usize) -> Result<String> {
        let mut markdown = String::new();

        // Add section header
        markdown.push_str(&format!("## Section {}\n\n", index + 1));

        // Process paragraphs
        for paragraph in &section.paragraphs {
            if !paragraph.text.is_empty() {
                markdown.push_str(&self.format_paragraph_markdown(paragraph));
                markdown.push('\n');
            }
        }

        Ok(markdown.trim().to_string())
    }

    fn format_paragraph(&self, paragraph: &Paragraph, _index: usize) -> Result<String> {
        Ok(self.format_paragraph_markdown(paragraph))
    }
}
