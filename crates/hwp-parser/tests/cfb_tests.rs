use hwp_parser::cfb::{parse_cfb_bytes, CfbStream};

/// Create a minimal valid CFB file for testing
fn create_test_cfb() -> Vec<u8> {
    // This creates a minimal CFB structure with proper headers and FAT
    // In a real implementation, you'd use a proper CFB writer
    
    let mut data = vec![0u8; 3072]; // Enough for header + FAT + directory + one data sector
    
    // CFB Header (512 bytes)
    // Signature
    data[0..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    
    // CLSID (16 bytes of zeros)
    // Already zeros
    
    // Minor version (0x003E)
    data[0x18] = 0x3E;
    data[0x19] = 0x00;
    
    // Major version (3 for 512-byte sectors)
    data[0x1A] = 0x03;
    data[0x1B] = 0x00;
    
    // Byte order (0xFFFE = little-endian)
    data[0x1C] = 0xFE;
    data[0x1D] = 0xFF;
    
    // Sector shift (9 = 512 bytes)
    data[0x1E] = 0x09;
    data[0x1F] = 0x00;
    
    // Mini sector shift (6 = 64 bytes)
    data[0x20] = 0x06;
    data[0x21] = 0x00;
    
    // Reserved (6 bytes) - already zeros
    
    // Total sectors (0 for version 3)
    // Already zeros at 0x28-0x2B
    
    // FAT sectors (0 for version 3)
    // Already zeros at 0x2C-0x2F
    
    // First directory sector (sector 1)
    data[0x30] = 0x01;
    data[0x31] = 0x00;
    data[0x32] = 0x00;
    data[0x33] = 0x00;
    
    // Transaction signature (0)
    // Already zeros at 0x34-0x37
    
    // Mini stream cutoff size (4096)
    data[0x38] = 0x00;
    data[0x39] = 0x10;
    data[0x3A] = 0x00;
    data[0x3B] = 0x00;
    
    // First mini FAT sector (ENDOFCHAIN = 0xFFFFFFFE)
    data[0x3C] = 0xFE;
    data[0x3D] = 0xFF;
    data[0x3E] = 0xFF;
    data[0x3F] = 0xFF;
    
    // Number of mini FAT sectors (0)
    // Already zeros at 0x40-0x43
    
    // First DIFAT sector (ENDOFCHAIN)
    data[0x44] = 0xFE;
    data[0x45] = 0xFF;
    data[0x46] = 0xFF;
    data[0x47] = 0xFF;
    
    // Number of DIFAT sectors (0)
    // Already zeros at 0x48-0x4B
    
    // DIFAT array (first entry points to FAT sector 0)
    data[0x4C] = 0x00;
    data[0x4D] = 0x00;
    data[0x4E] = 0x00;
    data[0x4F] = 0x00;
    
    // Rest of DIFAT array is FREESECT (0xFFFFFFFF)
    for i in 1..109 {
        let offset = 0x4C + (i * 4);
        data[offset] = 0xFF;
        data[offset + 1] = 0xFF;
        data[offset + 2] = 0xFF;
        data[offset + 3] = 0xFF;
    }
    
    // FAT sector (sector 0, at offset 512)
    // FAT[0] = FATSECT (0xFFFFFFFD) - this sector contains FAT
    data[512] = 0xFD;
    data[513] = 0xFF;
    data[514] = 0xFF;
    data[515] = 0xFF;
    
    // FAT[1] = ENDOFCHAIN (0xFFFFFFFE) - directory sector chain ends
    data[516] = 0xFE;
    data[517] = 0xFF;
    data[518] = 0xFF;
    data[519] = 0xFF;
    
    // FAT[2] = ENDOFCHAIN - for FileHeader stream
    data[520] = 0xFE;
    data[521] = 0xFF;
    data[522] = 0xFF;
    data[523] = 0xFF;
    
    // Rest of FAT is FREESECT
    for i in 3..128 {
        let offset = 512 + (i * 4);
        data[offset] = 0xFF;
        data[offset + 1] = 0xFF;
        data[offset + 2] = 0xFF;
        data[offset + 3] = 0xFF;
    }
    
    // Directory entries (sector 1, at offset 1024)
    // Root entry
    create_directory_entry(&mut data[1024..], "Root Entry", 5, 0xFFFFFFFF, 0xFFFFFFFF, 1, 0xFFFFFFFF, 0);
    
    // FileHeader stream entry (size 4096 to avoid mini FAT)
    create_directory_entry(&mut data[1152..], "FileHeader", 2, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 2, 4096);
    
    // FileHeader stream data (sector 2, at offset 1536)
    // Simple HWP header signature (32 bytes)
    let hwp_sig = b"HWP Document File V5.00 \x1A\x01\x02\x03\x04";
    data[1536..1536 + hwp_sig.len()].copy_from_slice(hwp_sig);
    
    data
}

/// Helper to create a directory entry
fn create_directory_entry(
    data: &mut [u8],
    name: &str,
    object_type: u8,
    left_sibling: u32,
    right_sibling: u32,
    child_did: u32,
    starting_sector: u32,
    stream_size: u32,
) {
    // Name (UTF-16LE)
    let name_utf16: Vec<u16> = name.encode_utf16().collect();
    for (i, ch) in name_utf16.iter().enumerate() {
        if i >= 31 {
            break;
        }
        data[i * 2] = (*ch & 0xFF) as u8;
        data[i * 2 + 1] = (*ch >> 8) as u8;
    }
    
    // Name length (including null terminator)
    let name_len = ((name_utf16.len() + 1) * 2).min(64) as u16;
    data[64] = (name_len & 0xFF) as u8;
    data[65] = (name_len >> 8) as u8;
    
    // Object type
    data[66] = object_type;
    
    // Color flag (black)
    data[67] = 1;
    
    // Left sibling DID
    data[68] = (left_sibling & 0xFF) as u8;
    data[69] = ((left_sibling >> 8) & 0xFF) as u8;
    data[70] = ((left_sibling >> 16) & 0xFF) as u8;
    data[71] = ((left_sibling >> 24) & 0xFF) as u8;
    
    // Right sibling DID
    data[72] = (right_sibling & 0xFF) as u8;
    data[73] = ((right_sibling >> 8) & 0xFF) as u8;
    data[74] = ((right_sibling >> 16) & 0xFF) as u8;
    data[75] = ((right_sibling >> 24) & 0xFF) as u8;
    
    // Child DID
    data[76] = (child_did & 0xFF) as u8;
    data[77] = ((child_did >> 8) & 0xFF) as u8;
    data[78] = ((child_did >> 16) & 0xFF) as u8;
    data[79] = ((child_did >> 24) & 0xFF) as u8;
    
    // CLSID (16 bytes) - zeros
    // State bits - zeros
    // Timestamps - zeros
    
    // Starting sector (offset 116)
    data[116] = (starting_sector & 0xFF) as u8;
    data[117] = ((starting_sector >> 8) & 0xFF) as u8;
    data[118] = ((starting_sector >> 16) & 0xFF) as u8;
    data[119] = ((starting_sector >> 24) & 0xFF) as u8;
    
    // Stream size low (offset 120)
    data[120] = (stream_size & 0xFF) as u8;
    data[121] = ((stream_size >> 8) & 0xFF) as u8;
    data[122] = ((stream_size >> 16) & 0xFF) as u8;
    data[123] = ((stream_size >> 24) & 0xFF) as u8;
    
    // Stream size high - zeros
}

#[test]
fn test_cfb_signature_detection() {
    let cfb_data = create_test_cfb();
    assert_eq!(&cfb_data[0..8], &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
}

#[test]
fn test_cfb_header_parsing() {
    let cfb_data = create_test_cfb();
    let container = parse_cfb_bytes(&cfb_data).unwrap();
    
    assert_eq!(container.header.major_version, 3);
    assert_eq!(container.header.sector_size(), 512);
    assert_eq!(container.header.mini_sector_size(), 64);
    assert!(container.header.is_valid_version());
}

#[test]
fn test_cfb_directory_parsing() {
    let cfb_data = create_test_cfb();
    let container = parse_cfb_bytes(&cfb_data).unwrap();
    
    // Check root entry
    let root = container.root_entry().unwrap();
    assert_eq!(root.name, "Root Entry");
    assert!(root.is_root());
    
    // Check stream list
    let streams = container.list_streams();
    assert!(streams.contains(&"FileHeader".to_string()));
}

#[test]
fn test_cfb_stream_extraction() {
    let cfb_data = create_test_cfb();
    let mut container = parse_cfb_bytes(&cfb_data).unwrap();
    let mut cursor = std::io::Cursor::new(&cfb_data);
    
    // Read FileHeader stream
    let stream = container.read_stream(&mut cursor, "FileHeader").unwrap();
    assert_eq!(stream.name, "FileHeader");
    assert!(stream.as_bytes().len() > 0);
    
    // Check that the stream contains HWP header
    let data = stream.as_bytes();
    assert!(data.starts_with(b"HWP Document File"));
}

#[test]
fn test_compressed_stream_detection() {
    // Test uncompressed stream
    let uncompressed = CfbStream::new("test".to_string(), vec![0x48, 0x57, 0x50]); // "HWP"
    assert!(!uncompressed.compressed);
    
    // Test compressed stream (zlib header)
    let compressed = CfbStream::new("test".to_string(), vec![0x78, 0x9C, 0x00]);
    assert!(compressed.compressed);
}

#[test]
fn test_fat_chain_reading() {
    use hwp_parser::cfb::fat::FatTable;
    
    // Create a simple FAT with a chain
    let fat = FatTable {
        entries: vec![
            1,          // 0 -> 1
            2,          // 1 -> 2
            3,          // 2 -> 3
            0xFFFFFFFE, // 3 -> ENDOFCHAIN
            0xFFFFFFFF, // 4 -> FREESECT
        ],
        sector_size: 512,
    };
    
    let chain = fat.get_chain(0);
    assert_eq!(chain, vec![0, 1, 2, 3]);
    
    let chain = fat.get_chain(2);
    assert_eq!(chain, vec![2, 3]);
    
    assert_eq!(fat.get_next(3), None);
}

#[test]
fn test_directory_tree_traversal() {
    use hwp_parser::cfb::directory::{DirectoryEntry, DirectoryTree, ObjectType, ColorFlag};
    
    // Create test directory entries
    let entries = vec![
        DirectoryEntry {
            name: "Root Entry".to_string(),
            name_len: 22,
            object_type: ObjectType::RootStorage,
            color_flag: ColorFlag::Black,
            left_sibling_did: 0xFFFFFFFF,
            right_sibling_did: 0xFFFFFFFF,
            child_did: 1,
            clsid: [0; 16],
            state_bits: 0,
            creation_time: 0,
            modified_time: 0,
            starting_sector: 0xFFFFFFFF,
            stream_size_low: 0,
            stream_size_high: 0,
        },
        DirectoryEntry {
            name: "FileHeader".to_string(),
            name_len: 22,
            object_type: ObjectType::Stream,
            color_flag: ColorFlag::Black,
            left_sibling_did: 0xFFFFFFFF,
            right_sibling_did: 2,
            child_did: 0xFFFFFFFF,
            clsid: [0; 16],
            state_bits: 0,
            creation_time: 0,
            modified_time: 0,
            starting_sector: 0,
            stream_size_low: 256,
            stream_size_high: 0,
        },
        DirectoryEntry {
            name: "DocInfo".to_string(),
            name_len: 16,
            object_type: ObjectType::Stream,
            color_flag: ColorFlag::Black,
            left_sibling_did: 0xFFFFFFFF,
            right_sibling_did: 0xFFFFFFFF,
            child_did: 0xFFFFFFFF,
            clsid: [0; 16],
            state_bits: 0,
            creation_time: 0,
            modified_time: 0,
            starting_sector: 1,
            stream_size_low: 512,
            stream_size_high: 0,
        },
    ];
    
    let tree = DirectoryTree::new(entries);
    
    // Test finding entries
    assert!(tree.find("FileHeader").is_some());
    assert!(tree.find("DocInfo").is_some());
    assert!(tree.find("NonExistent").is_none());
    
    // Test getting streams
    let streams = tree.streams();
    assert_eq!(streams.len(), 2);
    
    // Test getting root
    let root = tree.root().unwrap();
    assert_eq!(root.name, "Root Entry");
}