pub mod reader;
pub mod parser;
pub mod compression;
pub mod cfb;
pub mod validator;

use hwp_core::{HwpDocument, Result};

/// Parse an HWP file from raw bytes
pub fn parse(data: &[u8]) -> Result<HwpDocument> {
    parser::parse(data)
}

/// Parse an HWP file from a file path
pub fn parse_file(path: &str) -> Result<HwpDocument> {
    let data = std::fs::read(path)?;
    parse(&data)
}