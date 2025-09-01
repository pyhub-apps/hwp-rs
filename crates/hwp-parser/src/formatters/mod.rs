pub mod plain_text;
pub mod json;
pub mod markdown;

use hwp_core::{HwpDocument, Result};
use hwp_core::models::{Section, Paragraph};
use hwp_core::models::document::DocInfo;

/// Common trait for different output formatters
pub trait OutputFormatter {
    /// Format the entire document
    fn format_document(&self, doc: &HwpDocument) -> Result<String>;
    
    /// Format document metadata
    fn format_metadata(&self, doc_info: &DocInfo) -> Result<String>;
    
    /// Format a section
    fn format_section(&self, section: &Section, index: usize) -> Result<String>;
    
    /// Format a paragraph
    fn format_paragraph(&self, paragraph: &Paragraph, index: usize) -> Result<String>;
}

/// Options for controlling output formatting
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Pretty print JSON with indentation
    pub json_pretty: bool,
    /// Include style definitions in JSON
    pub json_include_styles: bool,
    /// Line wrap width for plain text
    pub text_width: Option<usize>,
    /// Preserve page breaks in plain text
    pub text_page_breaks: bool,
    /// Markdown flavor (CommonMark, GFM, etc.)
    pub markdown_flavor: MarkdownFlavor,
    /// Generate table of contents for Markdown
    pub markdown_toc: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            json_pretty: true,
            json_include_styles: false,
            text_width: None,
            text_page_breaks: false,
            markdown_flavor: MarkdownFlavor::CommonMark,
            markdown_toc: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFlavor {
    CommonMark,
    GitHubFlavored,
    MultiMarkdown,
}

/// Available output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    PlainText,
    Markdown,
}

impl OutputFormat {
    /// Create a formatter instance for this format
    pub fn create_formatter(&self, options: FormatOptions) -> Box<dyn OutputFormatter> {
        match self {
            OutputFormat::Json => Box::new(json::JsonFormatter::new(options)),
            OutputFormat::PlainText => Box::new(plain_text::PlainTextFormatter::new(options)),
            OutputFormat::Markdown => Box::new(markdown::MarkdownFormatter::new(options)),
        }
    }
    
    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "text" | "txt" | "plain" => Some(OutputFormat::PlainText),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            _ => None,
        }
    }
    
    /// Get file extension for this format
    pub fn file_extension(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::PlainText => "txt",
            OutputFormat::Markdown => "md",
        }
    }
}