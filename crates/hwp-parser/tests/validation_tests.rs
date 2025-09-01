use hwp_parser::parser::record::RecordParser;
use hwp_parser::validator::{DefaultRecordValidator, RecordContext};

/// Helper to create a record header
fn create_header(tag_id: u16, level: u16, size: u32) -> Vec<u8> {
    // Ensure values fit in their bit fields
    let tag_id = (tag_id & 0x3FF) as u32; // 10 bits
    let level = (level & 0x3FF) as u32; // 10 bits
    let size = size & 0xFFF; // 12 bits
    let value = tag_id | (level << 10) | (size << 20);
    value.to_le_bytes().to_vec()
}

#[test]
fn test_valid_record_parsing() {
    let mut data = Vec::new();

    // Valid DOCUMENT_PROPERTIES record
    data.extend(create_header(0x0010, 0, 36));
    data.extend(vec![0; 36]); // Dummy data

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let record = parser.parse_next_record().unwrap().unwrap();

    assert_eq!(record.tag_id, 0x0010);
    assert_eq!(record.size, 36);
}

#[test]
fn test_invalid_tag_id_validation() {
    let mut data = Vec::new();

    // Invalid tag ID for DocInfo context
    data.extend(create_header(0x9999, 0, 10));
    data.extend(vec![0; 10]);

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let result = parser.parse_next_record();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid tag ID"));
}

#[test]
fn test_buffer_underflow_validation() {
    let mut data = Vec::new();

    // Record header says 100 bytes but only 10 available
    data.extend(create_header(0x0010, 0, 100));
    data.extend(vec![0; 10]); // Only 10 bytes instead of 100

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let result = parser.parse_next_record();

    assert!(result.is_err());
}

#[test]
fn test_size_validation() {
    let mut data = Vec::new();

    // DOCUMENT_PROPERTIES with size too small (10 bytes, minimum is 22)
    data.extend(create_header(0x0010, 0, 10));
    data.extend(vec![0; 10]);

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let result = parser.parse_next_record();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too small"));
}

#[test]
fn test_extended_size_record() {
    let mut data = Vec::new();

    // Extended size record (size field = 0xFFF)
    data.extend(create_header(0x0010, 0, 0xFFF));
    data.extend(100u32.to_le_bytes()); // Actual size: 100 bytes
    data.extend(vec![0; 100]); // Data

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let record = parser.parse_next_record().unwrap().unwrap();

    assert_eq!(record.tag_id, 0x0010);
    assert_eq!(record.size, 100);
}

#[test]
fn test_recovery_from_corrupted_data() {
    let mut data = Vec::new();

    // First: Corrupted record (invalid tag)
    data.extend(create_header(0x9999, 0, 10));
    data.extend(vec![0xFF; 10]); // Garbage data

    // Second: Valid record that we should recover to
    data.extend(create_header(0x0013, 0, 20)); // FACE_NAME
    data.extend(vec![0; 20]);

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    parser.enable_recovery(true);

    // First parse should fail but recover
    let record = parser.parse_next_record().unwrap();

    // Should recover and find the FACE_NAME record
    if let Some(rec) = record {
        assert_eq!(rec.tag_id, 0x0013);
        assert_eq!(parser.recovery_count(), 1);
    } else {
        panic!("Recovery failed to find valid record");
    }
}

#[test]
fn test_multiple_records_with_corruption() {
    let mut data = Vec::new();

    // First: Valid record
    data.extend(create_header(0x0010, 0, 36));
    data.extend(vec![0; 36]);

    // Second: Corrupted header (invalid size that would cause underflow)
    data.extend(create_header(0x0013, 0, 10000));
    // No data for this corrupted record

    // Third: Another valid record (should not be reached without recovery)
    data.extend(create_header(0x0015, 0, 50)); // CHAR_SHAPE
    data.extend(vec![0; 50]);

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);

    // Parse first record - should succeed
    let rec1 = parser.parse_next_record().unwrap().unwrap();
    assert_eq!(rec1.tag_id, 0x0010);

    // Parse second record - should fail due to buffer underflow
    let rec2 = parser.parse_next_record();
    assert!(rec2.is_err());

    // Now test with recovery enabled
    let mut parser_with_recovery = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    parser_with_recovery.enable_recovery(true);

    // Parse first record
    let rec1 = parser_with_recovery.parse_next_record().unwrap().unwrap();
    assert_eq!(rec1.tag_id, 0x0010);

    // Parse second record - should recover and skip to third
    let rec2 = parser_with_recovery.parse_next_record();
    // Recovery behavior depends on implementation
    assert!(rec2.is_ok());
}

#[test]
fn test_lenient_validator() {
    let mut data = Vec::new();

    // Unknown tag ID (use value that fits in 10 bits)
    data.extend(create_header(0x03FF, 0, 20)); // Max valid tag ID for 10 bits
    data.extend(vec![0; 20]);

    // Create parser with lenient validator
    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    parser.set_validator(Box::new(DefaultRecordValidator::lenient()));

    // Should accept unknown tag with lenient validator
    let record = parser.parse_next_record().unwrap().unwrap();
    assert_eq!(record.tag_id, 0x03FF);
}

#[test]
fn test_boundary_validation() {
    let mut data = Vec::new();

    // Record that extends beyond buffer
    data.extend(create_header(0x0010, 0, 50));
    // Only provide 40 bytes total (4 header + 36 data), but record wants 50
    data.extend(vec![0; 36]);

    let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
    let result = parser.parse_next_record();

    assert!(result.is_err());
}

#[test]
fn test_context_specific_validation() {
    // Test DocInfo context
    {
        let mut data = Vec::new();
        data.extend(create_header(0x0050, 0, 20)); // PARA_HEADER - valid for BodyText, not DocInfo
        data.extend(vec![0; 20]);

        let mut parser = RecordParser::new_with_context(&data, RecordContext::DocInfo);
        let result = parser.parse_next_record();
        assert!(result.is_err()); // Should fail in DocInfo context
    }

    // Test BodyText context
    {
        let mut data = Vec::new();
        data.extend(create_header(0x0050, 0, 20)); // PARA_HEADER - valid for BodyText
        data.extend(vec![0; 20]);

        let mut parser = RecordParser::new_with_context(&data, RecordContext::BodyText);
        let result = parser.parse_next_record();
        assert!(result.is_ok()); // Should succeed in BodyText context
    }
}
