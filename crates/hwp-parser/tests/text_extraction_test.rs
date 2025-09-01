use hwp_core::HwpDocument;
use hwp_parser::parse;

#[test]
fn test_text_extraction_from_parsed_document() {
    // This test verifies that the text extraction chain works:
    // parse() -> HwpDocument -> get_text()

    // For now, we'll create a minimal valid HWP structure
    // In a real test, we'd use an actual HWP file

    // Create a mock CFB file with minimal HWP v5 structure
    // This would normally come from an actual .hwp file

    // Since we don't have a real HWP file yet, let's at least verify
    // the structure compiles and basic functionality works

    let header = hwp_core::models::header::HwpHeader {
        signature: [
            b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', b'm', b'e', b'n', b't', b' ', b'F',
            b'i', b'l', b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', b'\x1A', b'\x01', b'\x02',
            b'\x03', b'\x04', b'\x05', 0, 0,
        ],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };

    let document = HwpDocument::new(header);

    // Test that get_text() returns empty for empty document
    let text = document.get_text();
    assert_eq!(text, "");

    // Test that page_count returns 0 for empty document
    assert_eq!(document.page_count(), 0);
}

#[test]
fn test_section_text_extraction() {
    use hwp_core::models::{Paragraph, Section};

    let header = hwp_core::models::header::HwpHeader {
        signature: [
            b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', b'm', b'e', b'n', b't', b' ', b'F',
            b'i', b'l', b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', b'\x1A', b'\x01', b'\x02',
            b'\x03', b'\x04', b'\x05', 0, 0,
        ],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };

    let mut document = HwpDocument::new(header);

    // Create a section with paragraphs
    let mut section = Section::new();

    let mut para1 = Paragraph::new();
    para1.text = "Hello, World!".to_string();
    section.paragraphs.push(para1);

    let mut para2 = Paragraph::new();
    para2.text = "This is a test.".to_string();
    section.paragraphs.push(para2);

    document.sections.push(section);

    // Test text extraction
    let text = document.get_text();
    assert!(text.contains("Hello, World!"));
    assert!(text.contains("This is a test."));

    // The text should have paragraphs separated by newlines
    assert_eq!(text.trim(), "Hello, World!\nThis is a test.");
}
