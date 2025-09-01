use super::constants::*;
use byteorder::{LittleEndian, ReadBytesExt};
use hwp_core::{HwpError, Result};
use std::io::Read;

/// Object type for directory entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// Unknown or unallocated
    Unknown = 0,
    /// Storage object (directory)
    Storage = 1,
    /// Stream object (file)
    Stream = 2,
    /// Root storage
    RootStorage = 5,
}

impl From<u8> for ObjectType {
    fn from(value: u8) -> Self {
        match value {
            1 => ObjectType::Storage,
            2 => ObjectType::Stream,
            5 => ObjectType::RootStorage,
            _ => ObjectType::Unknown,
        }
    }
}

/// Color flag for directory tree nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFlag {
    Red = 0,
    Black = 1,
}

impl From<u8> for ColorFlag {
    fn from(value: u8) -> Self {
        match value {
            0 => ColorFlag::Red,
            _ => ColorFlag::Black,
        }
    }
}

/// Directory entry structure (128 bytes)
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// Entry name (UTF-16LE, up to 32 characters)
    pub name: String,
    /// Name length in bytes (including terminator)
    pub name_len: u16,
    /// Object type
    pub object_type: ObjectType,
    /// Color flag (for red-black tree)
    pub color_flag: ColorFlag,
    /// Left sibling DID
    pub left_sibling_did: u32,
    /// Right sibling DID
    pub right_sibling_did: u32,
    /// Child DID (for storage objects)
    pub child_did: u32,
    /// CLSID (16 bytes)
    pub clsid: [u8; 16],
    /// State bits
    pub state_bits: u32,
    /// Creation time
    pub creation_time: u64,
    /// Modified time
    pub modified_time: u64,
    /// Starting sector (for streams)
    pub starting_sector: u32,
    /// Stream size (low 32 bits)
    pub stream_size_low: u32,
    /// Stream size (high 32 bits, only for version 4)
    pub stream_size_high: u32,
}

impl DirectoryEntry {
    /// Parse a directory entry from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < DIR_ENTRY_SIZE {
            return Err(HwpError::InvalidFormat {
                reason: "Directory entry too small".to_string(),
            });
        }

        let mut cursor = std::io::Cursor::new(data);

        // Read name (64 bytes, UTF-16LE)
        let mut name_bytes = [0u8; 64];
        cursor
            .read_exact(&mut name_bytes)
            .map_err(HwpError::IoError)?;

        // Read name length
        let name_len = cursor
            .read_u16::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        // Convert name from UTF-16LE
        let name = if name_len > 2 {
            let utf16_len = ((name_len - 2) / 2) as usize;
            let mut utf16_chars = Vec::with_capacity(utf16_len);
            for i in 0..utf16_len {
                let ch = u16::from_le_bytes([name_bytes[i * 2], name_bytes[i * 2 + 1]]);
                if ch == 0 {
                    break;
                }
                utf16_chars.push(ch);
            }
            String::from_utf16_lossy(&utf16_chars)
        } else {
            String::new()
        };

        // Read object type
        let object_type = ObjectType::from(cursor.read_u8().map_err(HwpError::IoError)?);

        // Read color flag
        let color_flag = ColorFlag::from(cursor.read_u8().map_err(HwpError::IoError)?);

        // Read DIDs
        let left_sibling_did = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;
        let right_sibling_did = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;
        let child_did = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        // Read CLSID
        let mut clsid = [0u8; 16];
        cursor
            .read_exact(&mut clsid)
            .map_err(HwpError::IoError)?;

        // Read state bits
        let state_bits = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        // Read timestamps
        let creation_time = cursor
            .read_u64::<LittleEndian>()
            .map_err(HwpError::IoError)?;
        let modified_time = cursor
            .read_u64::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        // Read starting sector
        let starting_sector = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        // Read stream size
        let stream_size_low = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;
        let stream_size_high = cursor
            .read_u32::<LittleEndian>()
            .map_err(HwpError::IoError)?;

        Ok(DirectoryEntry {
            name,
            name_len,
            object_type,
            color_flag,
            left_sibling_did,
            right_sibling_did,
            child_did,
            clsid,
            state_bits,
            creation_time,
            modified_time,
            starting_sector,
            stream_size_low,
            stream_size_high,
        })
    }

    /// Get the total stream size
    pub fn stream_size(&self) -> u64 {
        ((self.stream_size_high as u64) << 32) | (self.stream_size_low as u64)
    }

    /// Check if this is a valid entry
    pub fn is_valid(&self) -> bool {
        self.object_type != ObjectType::Unknown
    }

    /// Check if this is the root entry
    pub fn is_root(&self) -> bool {
        self.object_type == ObjectType::RootStorage
    }

    /// Check if this is a storage (directory)
    pub fn is_storage(&self) -> bool {
        matches!(
            self.object_type,
            ObjectType::Storage | ObjectType::RootStorage
        )
    }

    /// Check if this is a stream (file)
    pub fn is_stream(&self) -> bool {
        self.object_type == ObjectType::Stream
    }
}

/// Directory tree for navigating the CFB structure
pub struct DirectoryTree {
    /// All directory entries
    pub entries: Vec<DirectoryEntry>,
}

impl DirectoryTree {
    /// Create a new directory tree from entries
    pub fn new(entries: Vec<DirectoryEntry>) -> Self {
        DirectoryTree { entries }
    }

    /// Find an entry by name
    pub fn find(&self, name: &str) -> Option<&DirectoryEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Find an entry by path (e.g., "BodyText/Section0")
    pub fn find_by_path(&self, path: &str) -> Option<&DirectoryEntry> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.is_empty() {
            return None;
        }

        // For now, just find by the full path as a name
        // In a real implementation, we'd traverse the tree structure
        self.find(path)
    }

    /// Get all stream entries
    pub fn streams(&self) -> Vec<&DirectoryEntry> {
        self.entries.iter().filter(|e| e.is_stream()).collect()
    }

    /// Get all storage entries
    pub fn storages(&self) -> Vec<&DirectoryEntry> {
        self.entries.iter().filter(|e| e.is_storage()).collect()
    }

    /// Get the root entry
    pub fn root(&self) -> Option<&DirectoryEntry> {
        self.entries.iter().find(|e| e.is_root())
    }

    /// Get entry by index (DID)
    pub fn get(&self, did: u32) -> Option<&DirectoryEntry> {
        self.entries.get(did as usize)
    }

    /// Get children of a storage entry
    pub fn get_children(&self, parent: &DirectoryEntry) -> Vec<&DirectoryEntry> {
        if !parent.is_storage() {
            return Vec::new();
        }

        let mut children = Vec::new();
        if parent.child_did != 0xFFFFFFFF {
            self.collect_siblings(parent.child_did, &mut children);
        }
        children
    }

    /// Recursively collect siblings in the red-black tree
    fn collect_siblings<'a>(&'a self, did: u32, result: &mut Vec<&'a DirectoryEntry>) {
        if did == 0xFFFFFFFF {
            return;
        }

        if let Some(entry) = self.get(did) {
            // Traverse left subtree
            if entry.left_sibling_did != 0xFFFFFFFF {
                self.collect_siblings(entry.left_sibling_did, result);
            }

            // Add current node
            result.push(entry);

            // Traverse right subtree
            if entry.right_sibling_did != 0xFFFFFFFF {
                self.collect_siblings(entry.right_sibling_did, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_entry_size() {
        assert_eq!(DIR_ENTRY_SIZE, 128);
    }

    #[test]
    fn test_object_type_conversion() {
        assert_eq!(ObjectType::from(0), ObjectType::Unknown);
        assert_eq!(ObjectType::from(1), ObjectType::Storage);
        assert_eq!(ObjectType::from(2), ObjectType::Stream);
        assert_eq!(ObjectType::from(5), ObjectType::RootStorage);
    }

    #[test]
    fn test_stream_size() {
        let entry = DirectoryEntry {
            name: "test".to_string(),
            name_len: 10,
            object_type: ObjectType::Stream,
            color_flag: ColorFlag::Black,
            left_sibling_did: 0xFFFFFFFF,
            right_sibling_did: 0xFFFFFFFF,
            child_did: 0xFFFFFFFF,
            clsid: [0; 16],
            state_bits: 0,
            creation_time: 0,
            modified_time: 0,
            starting_sector: 0,
            stream_size_low: 0x12345678,
            stream_size_high: 0xABCDEF00,
        };

        assert_eq!(entry.stream_size(), 0xABCDEF0012345678);
    }
}
