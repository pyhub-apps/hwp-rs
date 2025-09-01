use flate2::read::DeflateDecoder;
use hwp_core::{HwpError, Result};
use std::io::Read;

/// Decompress data using deflate algorithm
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| HwpError::DecompressionError(e.to_string()))?;
    
    Ok(decompressed)
}

/// Decompress with raw deflate (windowBits = -15)
pub fn decompress_raw(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Decompress;
    use flate2::FlushDecompress;
    
    let mut decompressor = Decompress::new(false);
    let mut output = Vec::with_capacity(data.len() * 2);
    
    // Resize output buffer to a reasonable size
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
        Err(e) => {
            Err(HwpError::DecompressionError(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}