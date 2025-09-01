use hwp_core::{HwpVersion, HWP_SIGNATURE};
use hwp_parser;

#[test]
fn test_parse_minimal_header() {
    // Create a minimal valid HWP file structure
    let mut data = Vec::new();

    // Add signature (32 bytes)
    data.extend_from_slice(HWP_SIGNATURE);

    // Add version (5.0.0.0)
    let version = HwpVersion::new(5, 0, 0, 0);
    data.extend_from_slice(&version.to_u32().to_le_bytes());

    // Add properties (no flags set)
    data.extend_from_slice(&0u32.to_le_bytes());

    // Add reserved bytes (216 bytes)
    data.extend_from_slice(&[0u8; 216]);

    // Try to parse
    let result = hwp_parser::parse(&data);

    match result {
        Ok(document) => {
            assert_eq!(document.header.version.major, 5);
            assert_eq!(document.header.version.minor, 0);
            assert!(!document.header.is_compressed());
            assert!(!document.header.has_password());
        }
        Err(e) => {
            panic!("Failed to parse valid HWP header: {}", e);
        }
    }
}

#[test]
fn test_invalid_signature() {
    let mut data = vec![0u8; 256];
    data[0..4].copy_from_slice(b"FAKE");

    let result = hwp_parser::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_unsupported_version() {
    let mut data = Vec::new();

    // Add valid signature
    data.extend_from_slice(HWP_SIGNATURE);

    // Add old version (3.0.0.0)
    let version = HwpVersion::new(3, 0, 0, 0);
    data.extend_from_slice(&version.to_u32().to_le_bytes());

    // Add properties and reserved bytes
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&[0u8; 216]);

    let result = hwp_parser::parse(&data);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("Unsupported"));
    }
}
