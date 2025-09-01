use crate::reader::ByteReader;
use hwp_core::{
    HwpError, Result,
    HWP_SIGNATURE, HWP_SIGNATURE_LEN, HwpVersion,
};
use hwp_core::models::header::{HwpHeader, HwpProperties};

/// Parse the HWP file header
pub fn parse_header(reader: &mut ByteReader) -> Result<HwpHeader> {
    // Read signature (32 bytes)
    let mut signature = [0u8; 32];
    reader.read_exact(&mut signature)?;
    
    // Verify signature
    if !verify_signature(&signature) {
        return Err(HwpError::InvalidSignature);
    }
    
    // Read version (4 bytes)
    let version_raw = reader.read_u32()?;
    let version = HwpVersion::from_u32(version_raw);
    
    // Read properties (4 bytes)
    let properties_raw = reader.read_u32()?;
    let properties = HwpProperties::from_u32(properties_raw);
    
    // Read reserved bytes (216 bytes)
    let mut reserved = [0u8; 216];
    reader.read_exact(&mut reserved)?;
    
    Ok(HwpHeader {
        signature,
        version,
        properties,
        reserved,
    })
}

/// Verify the HWP file signature
fn verify_signature(signature: &[u8; 32]) -> bool {
    // Check if the signature matches the expected HWP signature
    &signature[..HWP_SIGNATURE_LEN] == HWP_SIGNATURE
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_header() {
        // Create a valid HWP header
        let mut data = Vec::new();
        
        // Add signature
        data.extend_from_slice(HWP_SIGNATURE);
        
        // Add version (5.0.0.0)
        data.extend_from_slice(&0x05000000u32.to_le_bytes());
        
        // Add properties (compressed flag set)
        data.extend_from_slice(&0x00000001u32.to_le_bytes());
        
        // Add reserved bytes
        data.extend_from_slice(&[0u8; 216]);
        
        let mut reader = ByteReader::new(&data);
        let header = parse_header(&mut reader).unwrap();
        
        assert_eq!(header.version.major, 5);
        assert_eq!(header.version.minor, 0);
        assert_eq!(header.version.build, 0);
        assert_eq!(header.version.revision, 0);
        assert!(header.properties.compressed);
        assert!(!header.properties.has_password);
    }
    
    #[test]
    fn test_invalid_signature() {
        let mut data = vec![0u8; 256];
        data[0..4].copy_from_slice(b"FAKE");
        
        let mut reader = ByteReader::new(&data);
        let result = parse_header(&mut reader);
        
        assert!(matches!(result, Err(HwpError::InvalidSignature)));
    }
    
    #[test]
    fn test_header_size() {
        assert_eq!(HwpHeader::SIZE, 256);
    }
}