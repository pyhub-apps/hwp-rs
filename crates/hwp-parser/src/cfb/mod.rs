pub mod header;
pub mod fat;
pub mod directory;
pub mod stream;
pub mod container;

pub use container::{CfbContainer, CfbStream};
pub use header::CfbHeader;
pub use directory::DirectoryEntry;

use hwp_core::Result;
use std::io::{Read, Seek};

/// CFB (Compound File Binary) format constants
pub mod constants {
    /// CFB signature bytes
    pub const CFB_SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    
    /// Standard sector size (512 bytes)
    pub const SECTOR_SIZE_512: u32 = 512;
    
    /// Large sector size (4096 bytes)
    pub const SECTOR_SIZE_4096: u32 = 4096;
    
    /// Mini sector size
    pub const MINI_SECTOR_SIZE: u32 = 64;
    
    /// End of chain marker
    pub const ENDOFCHAIN: u32 = 0xFFFFFFFE;
    
    /// FAT sector marker
    pub const FATSECT: u32 = 0xFFFFFFFD;
    
    /// Free sector marker
    pub const FREESECT: u32 = 0xFFFFFFFF;
    
    /// Directory entry size
    pub const DIR_ENTRY_SIZE: usize = 128;
    
    /// Maximum regular sector ID
    pub const MAXREGSECT: u32 = 0xFFFFFFFA;
}

/// Parse a CFB container from a reader
pub fn parse_cfb<R: Read + Seek>(reader: &mut R) -> Result<CfbContainer> {
    container::CfbContainer::from_reader(reader)
}

/// Parse a CFB container from bytes
pub fn parse_cfb_bytes(data: &[u8]) -> Result<CfbContainer> {
    use std::io::Cursor;
    let mut cursor = Cursor::new(data);
    parse_cfb(&mut cursor)
}