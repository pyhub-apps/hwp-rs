pub mod cfb;
pub mod compression;
pub mod formatters;
pub mod parser;
pub mod reader;
pub mod text_extractor;
pub mod validator;

pub use formatters::{FormatOptions, MarkdownFlavor, OutputFormat, OutputFormatter};
use hwp_core::{HwpDocument, Result};
pub use text_extractor::{FormattedParagraph, FormattedText, TextExtractor};

/// Parse an HWP file from raw bytes
pub fn parse(data: &[u8]) -> Result<HwpDocument> {
    parser::parse(data)
}

/// Parse an HWP file from a file path
pub fn parse_file(path: &str) -> Result<HwpDocument> {
    let data = std::fs::read(path)?;
    parse(&data)
}
