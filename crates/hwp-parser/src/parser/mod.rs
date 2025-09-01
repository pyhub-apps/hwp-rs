pub mod doc_info;
pub mod doc_info_records;
pub mod header;
pub mod record;
pub mod section;

use crate::cfb::parse_cfb_bytes;
use crate::cfb::stream::Stream;
use crate::reader::ByteReader;
use hwp_core::{HwpDocument, HwpError, Result};
use std::io::Cursor;

/// Try to decompress a stream using various methods
fn try_decompress_stream(stream: &Stream) -> Result<Vec<u8>> {
    let data = stream.as_bytes();

    // Try different decompression methods (prefer HWP format first)
    // 1) HWP format: 4-byte size header + raw deflate (most common for HWP v5.x)
    if crate::compression::is_hwp_compressed(data) {
        if let Ok(decompressed) = crate::compression::decompress_hwp(data) {
            eprintln!("[DEBUG] Successfully decompressed with HWP (size + raw deflate)");
            return Ok(decompressed);
        }
    }

    // 2) Raw deflate (some streams may be pure deflate without size header)
    if let Ok(decompressed) = crate::compression::decompress_raw(data) {
        eprintln!("[DEBUG] Successfully decompressed with raw deflate");
        return Ok(decompressed);
    }

    // 3) Zlib (with header) as a last resort
    if data.len() >= 2 {
        let header = u16::from_be_bytes([data[0], data[1]]);
        if matches!(header, 0x789C | 0x78DA | 0x7801 | 0x785E | 0x78DE) {
            if let Ok(decompressed) = decompress_zlib(data) {
                eprintln!("[DEBUG] Successfully decompressed with zlib");
                return Ok(decompressed);
            }
        }
    }

    Err(HwpError::DecompressionError(
        "Failed to decompress stream".to_string(),
    ))
}

/// Decompress data using zlib
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| HwpError::DecompressionError(e.to_string()))?;

    Ok(decompressed)
}

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
        eprintln!("[DEBUG] Reading DocInfo stream...");
        let doc_info_stream = container.read_stream(&mut cursor, "DocInfo")?;
        eprintln!(
            "[DEBUG] DocInfo stream size: {} bytes",
            doc_info_stream.size
        );

        // Try to decompress DocInfo stream - HWP v5.x streams are usually compressed
        let doc_info_data = match try_decompress_stream(&doc_info_stream) {
            Ok(decompressed) => {
                eprintln!(
                    "[DEBUG] DocInfo decompressed successfully: {} bytes",
                    decompressed.len()
                );
                decompressed
            }
            Err(_) => {
                eprintln!("[DEBUG] DocInfo not compressed, using raw data");
                doc_info_stream.as_bytes().to_vec()
            }
        };

        // Parse DocInfo records
        eprintln!("[DEBUG] Parsing DocInfo data...");
        document.doc_info = doc_info::parse_doc_info(&doc_info_data)?;
        eprintln!("[DEBUG] DocInfo parsed successfully");
    }

    // Parse BodyText sections
    let mut section_idx = 0;
    loop {
        let section_name = format!("BodyText/Section{}", section_idx);
        if !container.has_stream(&section_name) {
            break;
        }

        eprintln!("[DEBUG] Reading section: {}", section_name);
        let section_stream = container.read_stream(&mut cursor, &section_name)?;
        eprintln!("[DEBUG] Stream size: {} bytes", section_stream.size);

        // Try to decompress section stream - HWP v5.x sections are usually compressed
        let section_data = match try_decompress_stream(&section_stream) {
            Ok(decompressed) => {
                eprintln!(
                    "[DEBUG] Section decompressed successfully: {} bytes",
                    decompressed.len()
                );
                decompressed
            }
            Err(_) => {
                eprintln!("[DEBUG] Section not compressed, using raw data");
                section_stream.as_bytes().to_vec()
            }
        };

        // Parse section
        eprintln!("[DEBUG] Parsing section data...");
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
