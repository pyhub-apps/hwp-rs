pub mod header;
pub mod doc_info;
pub mod section;
pub mod record;
pub mod doc_info_records;

use hwp_core::{HwpDocument, Result, HwpError};
use crate::reader::ByteReader;
use crate::cfb::parse_cfb_bytes;
use std::io::Cursor;

/// Parse an HWP document from raw bytes
pub fn parse(data: &[u8]) -> Result<HwpDocument> {
    // Check if this is a CFB file (HWP v5.x)
    if is_cfb_file(data) {
        parse_cfb_hwp(data)
    } else {
        // Legacy format (HWP v3.x or older)
        parse_legacy_hwp(data)
    }
}

/// Check if the data is a CFB file
fn is_cfb_file(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }
    // Check for CFB signature
    data[0..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
}

/// Parse a CFB-based HWP file (v5.x)
fn parse_cfb_hwp(data: &[u8]) -> Result<HwpDocument> {
    // Parse CFB container
    let mut container = parse_cfb_bytes(data)?;
    let mut cursor = Cursor::new(data);
    
    // Read FileHeader stream
    let file_header_stream = container.read_stream(&mut cursor, "FileHeader")?;
    let header_data = if file_header_stream.is_compressed() {
        file_header_stream.decompress()?
    } else {
        file_header_stream.as_bytes().to_vec()
    };
    
    // Parse header from the stream
    let mut reader = ByteReader::new(&header_data);
    let header = header::parse_header(&mut reader)?;
    
    // Check if version is supported
    if !header.version.is_supported() {
        return Err(HwpError::UnsupportedVersion {
            version: header.version.to_string(),
        });
    }
    
    // Create document
    let mut document = HwpDocument::new(header);
    
    if container.has_stream("DocInfo") {
        let doc_info_stream = container.read_stream(&mut cursor, "DocInfo")?;
        let doc_info_data = if doc_info_stream.is_compressed() {
            doc_info_stream.decompress()?
        } else {
            doc_info_stream.as_bytes().to_vec()
        };
        
        // Parse DocInfo records
        document.doc_info = doc_info::parse_doc_info(&doc_info_data)?;
    }
    
    // Parse BodyText sections
    let mut section_idx = 0;
    loop {
        let section_name = format!("BodyText/Section{}", section_idx);
        if !container.has_stream(&section_name) {
            break;
        }
        
        let section_stream = container.read_stream(&mut cursor, &section_name)?;
        let section_data = if section_stream.is_compressed() {
            section_stream.decompress()?
        } else {
            section_stream.as_bytes().to_vec()
        };
        
        // Parse section
        let section = section::parse_section(&section_data, section_idx)?;
        document.sections.push(section);
        
        section_idx += 1;
    }
    
    Ok(document)
}

/// Parse a legacy HWP file (v3.x or older)
fn parse_legacy_hwp(data: &[u8]) -> Result<HwpDocument> {
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