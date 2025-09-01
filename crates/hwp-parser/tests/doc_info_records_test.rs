use hwp_parser::parser::doc_info_records::*;
use hwp_core::models::document::{TabDef, Numbering, Bullet};

#[test]
fn test_parse_tab_def() {
    let mut data = Vec::new();
    
    // Properties
    data.extend_from_slice(&0x00000001u32.to_le_bytes());
    // Count
    data.extend_from_slice(&2u16.to_le_bytes());
    
    // Tab 1
    data.extend_from_slice(&1000u32.to_le_bytes()); // position
    data.push(0); // tab_type
    data.push(1); // fill_type
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    
    // Tab 2
    data.extend_from_slice(&2000u32.to_le_bytes()); // position
    data.push(1); // tab_type
    data.push(2); // fill_type
    data.extend_from_slice(&0u16.to_le_bytes()); // reserved
    
    let tab_def = parse_tab_def(&data).unwrap();
    
    assert_eq!(tab_def.properties, 1);
    assert_eq!(tab_def.count, 2);
    assert_eq!(tab_def.tabs.len(), 2);
    assert_eq!(tab_def.tabs[0].position, 1000);
    assert_eq!(tab_def.tabs[0].tab_type, 0);
    assert_eq!(tab_def.tabs[0].fill_type, 1);
    assert_eq!(tab_def.tabs[1].position, 2000);
    assert_eq!(tab_def.tabs[1].tab_type, 1);
    assert_eq!(tab_def.tabs[1].fill_type, 2);
}

#[test]
fn test_parse_numbering() {
    let mut data = Vec::new();
    
    // Level 1
    data.extend_from_slice(&0x00000001u32.to_le_bytes()); // paragraph_properties
    data.extend_from_slice(&10u16.to_le_bytes()); // paragraph_style_id
    data.push(1); // number_format
    data.extend_from_slice(&1u16.to_le_bytes()); // start_number
    // format_chars as HWP string (length + UTF-16 chars)
    data.extend_from_slice(&3u16.to_le_bytes()); // length: 3 chars
    data.extend_from_slice(&"1. ".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    
    let numbering = parse_numbering(&data).unwrap();
    
    assert_eq!(numbering.levels.len(), 1);
    assert_eq!(numbering.levels[0].paragraph_properties, 1);
    assert_eq!(numbering.levels[0].paragraph_style_id, 10);
    assert_eq!(numbering.levels[0].number_format, 1);
    assert_eq!(numbering.levels[0].start_number, 1);
    assert_eq!(numbering.levels[0].format_chars, "1. ");
}

#[test]
fn test_parse_bullet() {
    let mut data = Vec::new();
    
    data.extend_from_slice(&0x00000002u32.to_le_bytes()); // paragraph_properties
    data.extend_from_slice(&20u16.to_le_bytes()); // paragraph_style_id
    // bullet_char as HWP string
    data.extend_from_slice(&1u16.to_le_bytes()); // length: 1 char
    data.extend_from_slice(&"•".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    data.push(0); // use_image: false
    
    let bullet = parse_bullet(&data).unwrap();
    
    assert_eq!(bullet.paragraph_properties, 2);
    assert_eq!(bullet.paragraph_style_id, 20);
    assert_eq!(bullet.bullet_char, "•");
    assert_eq!(bullet.use_image, false);
    assert_eq!(bullet.image_id, None);
}

#[test]
fn test_parse_bullet_with_image() {
    let mut data = Vec::new();
    
    data.extend_from_slice(&0x00000003u32.to_le_bytes()); // paragraph_properties
    data.extend_from_slice(&30u16.to_le_bytes()); // paragraph_style_id
    // bullet_char as HWP string (empty when using image)
    data.extend_from_slice(&0u16.to_le_bytes()); // length: 0
    data.push(1); // use_image: true
    data.extend_from_slice(&100u16.to_le_bytes()); // image_id
    
    let bullet = parse_bullet(&data).unwrap();
    
    assert_eq!(bullet.paragraph_properties, 3);
    assert_eq!(bullet.paragraph_style_id, 30);
    assert_eq!(bullet.bullet_char, "");
    assert_eq!(bullet.use_image, true);
    assert_eq!(bullet.image_id, Some(100));
}

#[test]
fn test_parse_id_mappings() {
    let mut data = Vec::new();
    
    data.extend_from_slice(&3u32.to_le_bytes()); // count
    data.extend_from_slice(&100u32.to_le_bytes()); // mapping 1
    data.extend_from_slice(&200u32.to_le_bytes()); // mapping 2
    data.extend_from_slice(&300u32.to_le_bytes()); // mapping 3
    
    let mappings = parse_id_mappings(&data).unwrap();
    
    assert_eq!(mappings.len(), 3);
    assert_eq!(mappings[0], 100);
    assert_eq!(mappings[1], 200);
    assert_eq!(mappings[2], 300);
}

#[test]
fn test_parse_bin_data() {
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    
    let bin_data = parse_bin_data(&data).unwrap();
    
    assert_eq!(bin_data, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
}