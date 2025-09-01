use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::DeflateDecoder;
use hwp_core::{HwpError, Result};
use std::io::Read;

/// Decompress data using deflate algorithm (legacy function)
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| HwpError::DecompressionError(e.to_string()))?;

    Ok(decompressed)
}

/// Check if data uses HWP compression format
/// HWP format: [4 bytes: uncompressed size][compressed data using raw deflate]
pub fn is_hwp_compressed(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }

    // Read the first 4 bytes as uncompressed size
    let mut cursor = std::io::Cursor::new(data);
    if let Ok(uncompressed_size) = cursor.read_u32::<LittleEndian>() {
        // More lenient sanity checks for HWP format:
        // 1. Uncompressed size should be non-zero
        // 2. Uncompressed size should be reasonable (not gigabytes)
        // 3. There should be compressed data after the 4-byte header
        let compressed_data_size = data.len() - 4;

        // Note: In HWP files, the uncompressed size might be smaller than compressed size
        // for very small data or already compressed content
        uncompressed_size > 0 && compressed_data_size > 0 && uncompressed_size < (100 * 1024 * 1024)
    // Max 100MB reasonable limit
    } else {
        false
    }
}

/// Decompress HWP format data
/// Format: [4 bytes: uncompressed size in little-endian][raw deflate compressed data]
pub fn decompress_hwp(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 8 {
        return Err(HwpError::DecompressionError(format!(
            "Data too small for HWP compression format: {} bytes",
            data.len()
        )));
    }

    // Read 4-byte uncompressed size header
    let mut cursor = std::io::Cursor::new(data);
    let uncompressed_size = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| HwpError::DecompressionError(format!("Failed to read size header: {}", e)))?;

    eprintln!("[DEBUG] HWP Compression Header:");
    eprintln!("  - Total data size: {} bytes", data.len());
    eprintln!(
        "  - Uncompressed size from header: {} bytes",
        uncompressed_size
    );
    eprintln!("  - Compressed data size: {} bytes", data.len() - 4);
    eprintln!("  - First 16 bytes: {:02X?}", &data[..16.min(data.len())]);

    // Validate uncompressed size
    if uncompressed_size == 0 {
        return Err(HwpError::DecompressionError(
            "Invalid uncompressed size: 0".to_string(),
        ));
    }

    if uncompressed_size > 100 * 1024 * 1024 {
        return Err(HwpError::DecompressionError(format!(
            "Uncompressed size too large: {} bytes",
            uncompressed_size
        )));
    }

    // Get compressed data (skip 4-byte header)
    let compressed_data = &data[4..];

    eprintln!("[DEBUG] Attempting raw deflate decompression...");
    eprintln!(
        "[DEBUG] First 8 bytes of compressed data: {:02X?}",
        &compressed_data[..8.min(compressed_data.len())]
    );

    // Decompress using raw deflate (windowBits = -15)
    match decompress_raw_with_size(compressed_data, uncompressed_size as usize) {
        Ok(result) => {
            eprintln!(
                "[DEBUG] Decompression successful, {} bytes decompressed",
                result.len()
            );
            Ok(result)
        }
        Err(e) => {
            eprintln!("[DEBUG] Raw deflate failed, trying with zlib wrapper...");
            // Fallback: Try with zlib wrapper in case the format is different
            decompress_with_zlib_fallback(data, uncompressed_size as usize)
        }
    }
}

/// Fallback decompression attempting different compression formats
fn decompress_with_zlib_fallback(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    // Try interpreting the entire data as zlib-compressed
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(expected_size);

    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => {
            eprintln!(
                "[DEBUG] Zlib decompression successful (fallback), {} bytes",
                decompressed.len()
            );
            Ok(decompressed)
        }
        Err(_) => {
            // Last resort: Try the data after the 4-byte header as zlib
            if data.len() > 4 {
                let mut decoder = ZlibDecoder::new(&data[4..]);
                let mut decompressed = Vec::with_capacity(expected_size);
                decoder.read_to_end(&mut decompressed).map_err(|e| {
                    HwpError::DecompressionError(format!("All decompression methods failed: {}", e))
                })?;
                eprintln!(
                    "[DEBUG] Zlib decompression of data[4..] successful, {} bytes",
                    decompressed.len()
                );
                Ok(decompressed)
            } else {
                Err(HwpError::DecompressionError(
                    "All decompression methods failed".to_string(),
                ))
            }
        }
    }
}

/// Decompress with raw deflate using expected output size
/// Uses windowBits = -15 for raw deflate without header/checksum
pub fn decompress_raw_with_size(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::Decompress;
    use flate2::FlushDecompress;

    // Create raw deflate decompressor (no zlib header)
    let mut decompressor = Decompress::new(false); // false = raw deflate

    // Pre-allocate output buffer with expected size
    let mut output = vec![0u8; expected_size];

    match decompressor.decompress(data, &mut output, FlushDecompress::Finish) {
        Ok(flate2::Status::StreamEnd) => {
            let actual_size = decompressor.total_out() as usize;
            if actual_size != expected_size {
                return Err(HwpError::DecompressionError(format!(
                    "Size mismatch: expected {} bytes, got {} bytes",
                    expected_size, actual_size
                )));
            }
            output.truncate(actual_size);
            Ok(output)
        }
        Ok(flate2::Status::Ok) => {
            // Need more input or output space - shouldn't happen with Finish
            Err(HwpError::DecompressionError(
                "Incomplete decompression".to_string(),
            ))
        }
        Ok(flate2::Status::BufError) => Err(HwpError::DecompressionError(
            "Buffer size error during decompression".to_string(),
        )),
        Err(e) => Err(HwpError::DecompressionError(format!(
            "Raw deflate decompression failed: {}",
            e
        ))),
    }
}

/// Legacy raw deflate function with auto-sizing
pub fn decompress_raw(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Decompress;
    use flate2::FlushDecompress;

    let mut decompressor = Decompress::new(false);
    let mut output = Vec::with_capacity(data.len() * 4);

    // Start with reasonable size estimate
    output.resize(data.len() * 10, 0);

    match decompressor.decompress(data, &mut output, FlushDecompress::Finish) {
        Ok(flate2::Status::Ok) | Ok(flate2::Status::StreamEnd) => {
            let total_out = decompressor.total_out() as usize;
            output.truncate(total_out);
            Ok(output)
        }
        Ok(flate2::Status::BufError) => {
            Err(HwpError::DecompressionError("Buffer too small".to_string()))
        }
        Err(e) => Err(HwpError::DecompressionError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_decompress() {
        let original = b"Hello, HWP World!";

        // Compress the data
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // Decompress and verify
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_hwp_compression_detection() {
        // Test with valid HWP format
        let mut hwp_data = Vec::new();
        hwp_data.write_u32::<LittleEndian>(100).unwrap(); // uncompressed size
        hwp_data.extend_from_slice(&[0x78, 0x9C, 0x01, 0x05]); // some compressed data

        assert!(is_hwp_compressed(&hwp_data));

        // Test with too small data
        assert!(!is_hwp_compressed(&[1, 2, 3]));

        // Test with invalid size (0)
        let mut invalid_data = Vec::new();
        invalid_data.write_u32::<LittleEndian>(0).unwrap();
        invalid_data.extend_from_slice(&[0x78, 0x9C, 0x01, 0x05]);
        assert!(!is_hwp_compressed(&invalid_data));
    }

    #[test]
    fn test_hwp_decompression() {
        let original = b"HWP compressed content test data";

        // Create raw deflate compressed data
        let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed_data = encoder.finish().unwrap();

        // Create HWP format: [4 byte size][compressed data]
        let mut hwp_data = Vec::new();
        hwp_data
            .write_u32::<LittleEndian>(original.len() as u32)
            .unwrap();
        hwp_data.extend_from_slice(&compressed_data);

        // Test decompression
        let decompressed = decompress_hwp(&hwp_data).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_raw_deflate_with_size() {
        let original = b"Raw deflate test data for HWP format";

        // Compress with raw deflate
        let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // Test decompression with correct size
        let decompressed = decompress_raw_with_size(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);

        // Test with wrong size should fail
        assert!(decompress_raw_with_size(&compressed, original.len() + 10).is_err());
    }
}
