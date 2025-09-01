pub mod html;
pub mod json;
pub mod markdown;
pub mod plain_text;
pub mod yaml;

use hwp_core::models::document::DocInfo;
use hwp_core::models::{Paragraph, Section};
use hwp_core::{HwpDocument, Result};

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
    /// Include metadata in output
    pub include_metadata: bool,
    /// Include style information
    pub include_styles: bool,
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
            include_metadata: false,
            include_styles: false,
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
    Html,
    Yaml,
}

impl OutputFormat {
    /// Create a formatter instance for this format
    pub fn create_formatter(&self, options: FormatOptions) -> Box<dyn OutputFormatter> {
        match self {
            OutputFormat::Json => Box::new(json::JsonFormatter::new(options)),
            OutputFormat::PlainText => Box::new(plain_text::PlainTextFormatter::new(options)),
            OutputFormat::Markdown => Box::new(markdown::MarkdownFormatter::new(options)),
            OutputFormat::Html => Box::new(html::HtmlFormatter::new(options)),
            OutputFormat::Yaml => Box::new(yaml::YamlFormatter::new(options)),
        }
    }

    /// Parse format from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "text" | "txt" | "plain" => Some(OutputFormat::PlainText),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            "html" | "htm" => Some(OutputFormat::Html),
            "yaml" | "yml" => Some(OutputFormat::Yaml),
            _ => None,
        }
    }

    /// Get file extension for this format
    pub fn file_extension(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::PlainText => "txt",
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
            OutputFormat::Yaml => "yaml",
        }
    }
}
