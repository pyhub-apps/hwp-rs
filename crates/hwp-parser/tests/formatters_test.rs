use hwp_parser::formatters::{OutputFormat, FormatOptions, MarkdownFlavor};
use hwp_parser::OutputFormatter;
use hwp_core::models::{Section, Paragraph};
use hwp_core::HwpDocument;

fn create_test_document() -> HwpDocument {
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
    
    // Add test paragraphs
    let mut para1 = Paragraph::new();
    para1.text = "Test Document Title".to_string();
    section.paragraphs.push(para1);
    
    let mut para2 = Paragraph::new();
    para2.text = "This is the first paragraph with some content.".to_string();
    section.paragraphs.push(para2);
    
    let mut para3 = Paragraph::new();
    para3.text = "• First list item".to_string();
    section.paragraphs.push(para3);
    
    let mut para4 = Paragraph::new();
    para4.text = "• Second list item".to_string();
    section.paragraphs.push(para4);
    
    let mut para5 = Paragraph::new();
    para5.text = "한글 텍스트도 포함되어 있습니다.".to_string();
    section.paragraphs.push(para5);
    
    document.sections.push(section);
    document
}

#[test]
fn test_plain_text_formatter() {
    let doc = create_test_document();
    let options = FormatOptions::default();
    let formatter = OutputFormat::PlainText.create_formatter(options);
    
    let result = formatter.format_document(&doc).unwrap();
    
    assert!(result.contains("Test Document Title"));
    assert!(result.contains("This is the first paragraph"));
    assert!(result.contains("First list item"));
    assert!(result.contains("Second list item"));
    assert!(result.contains("한글 텍스트도 포함되어 있습니다"));
}

#[test]
fn test_plain_text_with_wrapping() {
    let doc = create_test_document();
    let mut options = FormatOptions::default();
    options.text_width = Some(30);
    
    let formatter = OutputFormat::PlainText.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    // Check that lines are wrapped
    for line in result.lines() {
        assert!(line.len() <= 30 || line.contains("한글")); // Korean text might exceed due to character width
    }
}

#[test]
fn test_json_formatter() {
    let doc = create_test_document();
    let mut options = FormatOptions::default();
    options.json_pretty = false;
    
    let formatter = OutputFormat::Json.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    // Parse JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    
    assert!(json["metadata"].is_object());
    assert!(json["content"].is_object());
    assert!(json["content"]["sections"].is_array());
    
    let sections = json["content"]["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 1);
    
    let paragraphs = sections[0]["paragraphs"].as_array().unwrap();
    assert_eq!(paragraphs.len(), 5);
    assert_eq!(paragraphs[0]["text"], "Test Document Title");
    assert_eq!(paragraphs[4]["text"], "한글 텍스트도 포함되어 있습니다.");
}

#[test]
fn test_json_formatter_pretty() {
    let doc = create_test_document();
    let mut options = FormatOptions::default();
    options.json_pretty = true;
    
    let formatter = OutputFormat::Json.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    // Pretty JSON should have indentation
    assert!(result.contains("\n  "));
    assert!(result.contains("{\n"));
}

#[test]
fn test_json_formatter_with_styles() {
    let doc = create_test_document();
    let mut options = FormatOptions::default();
    options.json_include_styles = true;
    
    let formatter = OutputFormat::Json.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(json["styles"].is_object());
    assert!(json["styles"]["fonts"].is_array());
    assert!(json["styles"]["paragraph_styles"].is_array());
    assert!(json["styles"]["character_styles"].is_array());
}

#[test]
fn test_markdown_formatter() {
    let doc = create_test_document();
    let options = FormatOptions::default();
    let formatter = OutputFormat::Markdown.create_formatter(options);
    
    let result = formatter.format_document(&doc).unwrap();
    
    assert!(result.contains("# Document"));
    assert!(result.contains("Test Document Title"));
    assert!(result.contains("This is the first paragraph"));
    assert!(result.contains("- First list item"));
    assert!(result.contains("- Second list item"));
    assert!(result.contains("한글 텍스트도 포함되어 있습니다"));
}

#[test]
fn test_markdown_formatter_with_toc() {
    let doc = create_test_document();
    let mut options = FormatOptions::default();
    options.markdown_toc = true;
    
    let formatter = OutputFormat::Markdown.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    assert!(result.contains("## Table of Contents"));
    assert!(result.contains("[Section 1]"));
}

#[test]
fn test_markdown_list_detection() {
    let mut doc = create_test_document();
    
    // Add numbered list items
    if let Some(section) = doc.sections.get_mut(0) {
        let mut para = Paragraph::new();
        para.text = "1. Numbered item one".to_string();
        section.paragraphs.push(para);
        
        let mut para = Paragraph::new();
        para.text = "2. Numbered item two".to_string();
        section.paragraphs.push(para);
    }
    
    let options = FormatOptions::default();
    let formatter = OutputFormat::Markdown.create_formatter(options);
    let result = formatter.format_document(&doc).unwrap();
    
    assert!(result.contains("1. Numbered item one"));
    assert!(result.contains("2. Numbered item two"));
}

#[test]
fn test_output_format_parsing() {
    assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
    assert_eq!(OutputFormat::from_str("JSON"), Some(OutputFormat::Json));
    assert_eq!(OutputFormat::from_str("text"), Some(OutputFormat::PlainText));
    assert_eq!(OutputFormat::from_str("txt"), Some(OutputFormat::PlainText));
    assert_eq!(OutputFormat::from_str("plain"), Some(OutputFormat::PlainText));
    assert_eq!(OutputFormat::from_str("markdown"), Some(OutputFormat::Markdown));
    assert_eq!(OutputFormat::from_str("md"), Some(OutputFormat::Markdown));
    assert_eq!(OutputFormat::from_str("unknown"), None);
}

#[test]
fn test_file_extensions() {
    assert_eq!(OutputFormat::Json.file_extension(), "json");
    assert_eq!(OutputFormat::PlainText.file_extension(), "txt");
    assert_eq!(OutputFormat::Markdown.file_extension(), "md");
}

#[test]
fn test_format_single_paragraph() {
    let mut para = Paragraph::new();
    para.text = "Single paragraph test".to_string();
    
    let options = FormatOptions::default();
    
    // Test plain text
    let formatter = OutputFormat::PlainText.create_formatter(options.clone());
    let result = formatter.format_paragraph(&para, 0).unwrap();
    assert_eq!(result, "Single paragraph test");
    
    // Test JSON
    let formatter = OutputFormat::Json.create_formatter(options.clone());
    let result = formatter.format_paragraph(&para, 0).unwrap();
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(json["text"], "Single paragraph test");
    assert_eq!(json["index"], 0);
    
    // Test Markdown
    let formatter = OutputFormat::Markdown.create_formatter(options);
    let result = formatter.format_paragraph(&para, 0).unwrap();
    assert!(result.contains("Single paragraph test"));
}