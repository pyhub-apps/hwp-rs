use hwp_parser::{TextExtractor, FormattedText};
use hwp_core::models::{Section, Paragraph};
use hwp_core::HwpDocument;

#[test]
fn test_extract_from_empty_document() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let document = HwpDocument::new(header);
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "");
}

#[test]
fn test_extract_simple_text() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para = Paragraph::new();
    para.text = "Hello, World!".to_string();
    section.paragraphs.push(para);
    
    document.sections.push(section);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "Hello, World!");
}

#[test]
fn test_extract_multiple_paragraphs() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para1 = Paragraph::new();
    para1.text = "First paragraph".to_string();
    section.paragraphs.push(para1);
    
    let mut para2 = Paragraph::new();
    para2.text = "Second paragraph".to_string();
    section.paragraphs.push(para2);
    
    let mut para3 = Paragraph::new();
    para3.text = "Third paragraph".to_string();
    section.paragraphs.push(para3);
    
    document.sections.push(section);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "First paragraph\nSecond paragraph\nThird paragraph");
}

#[test]
fn test_extract_korean_text() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para1 = Paragraph::new();
    para1.text = "안녕하세요".to_string();
    section.paragraphs.push(para1);
    
    let mut para2 = Paragraph::new();
    para2.text = "한글 문서입니다".to_string();
    section.paragraphs.push(para2);
    
    document.sections.push(section);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "안녕하세요\n한글 문서입니다");
}

#[test]
fn test_extract_mixed_text() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para = Paragraph::new();
    para.text = "한글과 English가 섞여있는 문서 123".to_string();
    section.paragraphs.push(para);
    
    document.sections.push(section);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "한글과 English가 섞여있는 문서 123");
}

#[test]
fn test_extract_with_formatting() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para1 = Paragraph::new();
    para1.text = "Title".to_string();
    section.paragraphs.push(para1);
    
    let mut para2 = Paragraph::new();
    para2.text = "Content".to_string();
    section.paragraphs.push(para2);
    
    document.sections.push(section);
    
    let formatted = TextExtractor::extract_with_formatting(&document).unwrap();
    assert_eq!(formatted.paragraphs.len(), 2);
    assert_eq!(formatted.paragraphs[0].text, "Title");
    assert_eq!(formatted.paragraphs[1].text, "Content");
}

#[test]
fn test_multiple_sections() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    
    // First section
    let mut section1 = Section::new();
    let mut para1 = Paragraph::new();
    para1.text = "Section 1 text".to_string();
    section1.paragraphs.push(para1);
    document.sections.push(section1);
    
    // Second section
    let mut section2 = Section::new();
    let mut para2 = Paragraph::new();
    para2.text = "Section 2 text".to_string();
    section2.paragraphs.push(para2);
    document.sections.push(section2);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "Section 1 text\nSection 2 text");
}

#[test]
fn test_skip_empty_paragraphs() {
    let header = hwp_core::models::header::HwpHeader {
        signature: [b'H', b'W', b'P', b' ', b'D', b'o', b'c', b'u', 
                    b'm', b'e', b'n', b't', b' ', b'F', b'i', b'l', 
                    b'e', b' ', b'V', b'5', b'.', b'0', b'0', b' ', 
                    b'\x1A', b'\x01', b'\x02', b'\x03', b'\x04', b'\x05', 0, 0],
        version: hwp_core::constants::HwpVersion::new(5, 0, 0, 0),
        properties: hwp_core::models::header::HwpProperties::from_u32(0),
        reserved: [0; 216],
    };
    
    let mut document = HwpDocument::new(header);
    let mut section = Section::new();
    
    let mut para1 = Paragraph::new();
    para1.text = "Text 1".to_string();
    section.paragraphs.push(para1);
    
    // Empty paragraph
    let para2 = Paragraph::new();
    section.paragraphs.push(para2);
    
    let mut para3 = Paragraph::new();
    para3.text = "Text 2".to_string();
    section.paragraphs.push(para3);
    
    document.sections.push(section);
    
    let text = TextExtractor::extract_from_document(&document).unwrap();
    assert_eq!(text, "Text 1\nText 2");
}