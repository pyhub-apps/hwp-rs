use super::constants::*;
use super::directory::{DirectoryEntry, DirectoryTree};
use super::fat::{FatTable, MiniFatTable};
use super::header::CfbHeader;
use super::stream::Stream;
use hwp_core::{HwpError, Result};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// CFB (Compound File Binary) container
pub struct CfbContainer {
    /// CFB header
    pub header: CfbHeader,
    /// FAT table
    pub fat: FatTable,
    /// Mini FAT table (optional)
    pub mini_fat: Option<MiniFatTable>,
    /// Directory tree
    pub directory: DirectoryTree,
    /// Cached streams
    streams: HashMap<String, Stream>,
}

impl CfbContainer {
    /// Parse a CFB container from a reader
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Parse header
        let header = CfbHeader::from_reader(reader)?;

        // Validate version
        if !header.is_valid_version() {
            return Err(HwpError::InvalidFormat {
                reason: format!("Unsupported CFB version: {}", header.major_version),
            });
        }

        // Parse FAT
        let fat = FatTable::from_reader(reader, &header)?;

        // Parse directory entries
        let directory_entries = Self::read_directory_entries(reader, &header, &fat)?;
        let directory = DirectoryTree::new(directory_entries);

        // Parse Mini FAT if present
        let mini_fat = if let Some(root) = directory.root() {
            if header.mini_fat_sectors > 0 && root.starting_sector != ENDOFCHAIN {
                Some(MiniFatTable::from_reader(
                    reader,
                    &header,
                    &fat,
                    root.starting_sector,
                )?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(CfbContainer {
            header,
            fat,
            mini_fat,
            directory,
            streams: HashMap::new(),
        })
    }

    /// Read all directory entries
    fn read_directory_entries<R: Read + Seek>(
        reader: &mut R,
        header: &CfbHeader,
        fat: &FatTable,
    ) -> Result<Vec<DirectoryEntry>> {
        let mut entries = Vec::new();

        if header.first_dir_sector == ENDOFCHAIN {
            return Ok(entries);
        }

        // Get directory chain
        let dir_chain = fat.get_chain(header.first_dir_sector);
        let entries_per_sector = header.sector_size() as usize / DIR_ENTRY_SIZE;

        // Read all directory entries
        for sector in dir_chain {
            let offset = (sector + 1) * header.sector_size();
            reader
                .seek(SeekFrom::Start(offset as u64))
                .map_err(HwpError::IoError)?;

            let mut sector_data = vec![0u8; header.sector_size() as usize];
            reader
                .read_exact(&mut sector_data)
                .map_err(HwpError::IoError)?;

            for i in 0..entries_per_sector {
                let start = i * DIR_ENTRY_SIZE;
                let end = start + DIR_ENTRY_SIZE;
                let entry = DirectoryEntry::from_bytes(&sector_data[start..end])?;

                // Stop at first invalid entry
                if !entry.is_valid() && entries.is_empty() {
                    continue;
                }

                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get a stream by name
    pub fn get_stream(&self, name: &str) -> Option<&Stream> {
        self.streams.get(name)
    }

    /// Read a stream by name
    pub fn read_stream<R: Read + Seek>(&mut self, reader: &mut R, name: &str) -> Result<&Stream> {
        // Check if already cached
        if self.streams.contains_key(name) {
            return Ok(&self.streams[name]);
        }

        // Find the directory entry
        let entry = self
            .directory
            .find(name)
            .ok_or_else(|| HwpError::InvalidFormat {
                reason: format!("Stream '{}' not found", name),
            })?;

        // Read the stream
        let stream = Stream::from_entry(
            reader,
            entry,
            &self.header,
            &self.fat,
            self.mini_fat.as_ref(),
        )?;

        // Cache and return
        self.streams.insert(name.to_string(), stream);
        Ok(&self.streams[name])
    }

    /// Read a stream by path (e.g., "BodyText/Section0")
    pub fn read_stream_by_path<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        path: &str,
    ) -> Result<&Stream> {
        // For HWP files, paths like "BodyText/Section0" are stored as single entries
        self.read_stream(reader, path)
    }

    /// List all stream names
    pub fn list_streams(&self) -> Vec<String> {
        self.directory
            .streams()
            .into_iter()
            .map(|e| e.name.clone())
            .collect()
    }

    /// List all storage names
    pub fn list_storages(&self) -> Vec<String> {
        self.directory
            .storages()
            .into_iter()
            .map(|e| e.name.clone())
            .collect()
    }

    /// Check if a stream exists
    pub fn has_stream(&self, name: &str) -> bool {
        self.directory
            .find(name)
            .map(|e| e.is_stream())
            .unwrap_or(false)
    }

    /// Get the root directory entry
    pub fn root_entry(&self) -> Option<&DirectoryEntry> {
        self.directory.root()
    }
}

/// A stream extracted from a CFB container
#[derive(Debug, Clone)]
pub struct CfbStream {
    /// Stream name
    pub name: String,
    /// Stream data (possibly compressed)
    pub data: Vec<u8>,
    /// Whether the stream is compressed
    pub compressed: bool,
}

impl CfbStream {
    /// Create a new CFB stream
    pub fn new(name: String, data: Vec<u8>) -> Self {
        let compressed = Self::is_compressed(&data);
        CfbStream {
            name,
            data,
            compressed,
        }
    }

    /// Check if data appears to be compressed
    fn is_compressed(data: &[u8]) -> bool {
        if data.len() >= 2 {
            // Check for zlib header
            let header = u16::from_be_bytes([data[0], data[1]]);
            matches!(header, 0x789C | 0x78DA | 0x7801 | 0x785E | 0x78DE)
        } else {
            false
        }
    }

    /// Get the raw data
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }

    /// Get decompressed data
    pub fn decompressed_data(&self) -> Result<Vec<u8>> {
        if !self.compressed {
            return Ok(self.data.clone());
        }

        // Try HWP format first (4-byte size + raw deflate)
        if crate::compression::is_hwp_compressed(&self.data) {
            return crate::compression::decompress_hwp(&self.data);
        }

        // Fallback to zlib decompression for legacy compatibility
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(&self.data[..]);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| HwpError::DecompressionError(e.to_string()))?;

        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_cfb_header() -> Vec<u8> {
        let mut data = vec![0u8; 512];

        // Signature
        data[0..8].copy_from_slice(&CFB_SIGNATURE);

        // CLSID (zeros)
        // Minor version (0x003E)
        data[0x18] = 0x3E;
        data[0x19] = 0x00;

        // Major version (3)
        data[0x1A] = 0x03;
        data[0x1B] = 0x00;

        // Byte order (0xFFFE)
        data[0x1C] = 0xFE;
        data[0x1D] = 0xFF;

        // Sector shift (9 = 512 bytes)
        data[0x1E] = 0x09;
        data[0x1F] = 0x00;

        // Mini sector shift (6 = 64 bytes)
        data[0x20] = 0x06;
        data[0x21] = 0x00;

        // Set first directory sector to ENDOFCHAIN
        data[0x30] = 0xFE;
        data[0x31] = 0xFF;
        data[0x32] = 0xFF;
        data[0x33] = 0xFF;

        // Mini stream cutoff (4096)
        data[0x38] = 0x00;
        data[0x39] = 0x10;
        data[0x3A] = 0x00;
        data[0x3B] = 0x00;

        // First mini FAT sector (ENDOFCHAIN)
        data[0x3C] = 0xFE;
        data[0x3D] = 0xFF;
        data[0x3E] = 0xFF;
        data[0x3F] = 0xFF;

        // First DIFAT sector (ENDOFCHAIN)
        data[0x44] = 0xFE;
        data[0x45] = 0xFF;
        data[0x46] = 0xFF;
        data[0x47] = 0xFF;

        // Fill DIFAT array with FREESECT
        for i in 0..109 {
            let offset = 0x4C + (i * 4);
            data[offset] = 0xFF;
            data[offset + 1] = 0xFF;
            data[offset + 2] = 0xFF;
            data[offset + 3] = 0xFF;
        }

        data
    }

    #[test]
    fn test_cfb_header_parsing() {
        let data = create_test_cfb_header();
        let mut cursor = Cursor::new(data);

        let header = CfbHeader::from_reader(&mut cursor).unwrap();
        assert_eq!(header.signature, CFB_SIGNATURE);
        assert_eq!(header.major_version, 3);
        assert_eq!(header.sector_size(), 512);
        assert_eq!(header.mini_sector_size(), 64);
        assert!(header.is_valid_version());
    }

    #[test]
    fn test_cfb_stream() {
        let uncompressed = CfbStream::new("test".to_string(), vec![0x00, 0x01, 0x02]);
        assert!(!uncompressed.compressed);

        let compressed = CfbStream::new("test".to_string(), vec![0x78, 0x9C, 0x00, 0x00]);
        assert!(compressed.compressed);
    }
}
