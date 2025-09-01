pub mod header;
pub mod doc_info;
pub mod section;

use hwp_core::{HwpDocument, Result, HwpError};
use crate::reader::ByteReader;

/// Parse an HWP document from raw bytes
pub fn parse(data: &[u8]) -> Result<HwpDocument> {
    let mut reader = ByteReader::new(data);
    
    // Parse header
    let header = header::parse_header(&mut reader)?;
    
    // Check if version is supported
    if !header.version.is_supported() {
        return Err(HwpError::UnsupportedVersion {
            version: header.version.to_string(),
        });
    }
    
    // Create document
    let document = HwpDocument::new(header);
    
    // TODO: Parse DocInfo section
    // TODO: Parse body sections
    
    Ok(document)
}