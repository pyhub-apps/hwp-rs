use super::constants::*;
use byteorder::{LittleEndian, ReadBytesExt};
use hwp_core::{HwpError, Result};
use std::io::{Read, Seek, SeekFrom};

/// CFB Header structure (512 bytes)
#[derive(Debug, Clone)]
pub struct CfbHeader {
    /// Signature (0xD0CF11E0A1B11AE1)
    pub signature: [u8; 8],
    /// CLSID (16 bytes, typically zeros)
    pub clsid: [u8; 16],
    /// Minor version
    pub minor_version: u16,
    /// Major version (3 for 512-byte sectors, 4 for 4096-byte sectors)
    pub major_version: u16,
    /// Byte order (0xFFFE = little-endian)
    pub byte_order: u16,
    /// Sector size power (9 = 512 bytes, 12 = 4096 bytes)
    pub sector_shift: u16,
    /// Mini sector size power (typically 6 = 64 bytes)
    pub mini_sector_shift: u16,
    /// Reserved (6 bytes)
    pub reserved: [u8; 6],
    /// Total sectors (0 for version 3)
    pub total_sectors: u32,
    /// FAT sectors (0 for version 3)
    pub fat_sectors: u32,
    /// First directory sector
    pub first_dir_sector: u32,
    /// Transaction signature
    pub transaction_signature: u32,
    /// Mini stream cutoff size (typically 4096)
    pub mini_stream_cutoff_size: u32,
    /// First mini FAT sector
    pub first_mini_fat_sector: u32,
    /// Number of mini FAT sectors
    pub mini_fat_sectors: u32,
    /// First DIFAT sector
    pub first_difat_sector: u32,
    /// Number of DIFAT sectors
    pub difat_sectors: u32,
    /// First 109 FAT sector positions (DIFAT array)
    pub difat: [u32; 109],
}

impl CfbHeader {
    /// Parse CFB header from a reader
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Ensure we're at the beginning
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|e| HwpError::IoError(e))?;

        let mut header = CfbHeader {
            signature: [0; 8],
            clsid: [0; 16],
            minor_version: 0,
            major_version: 0,
            byte_order: 0,
            sector_shift: 0,
            mini_sector_shift: 0,
            reserved: [0; 6],
            total_sectors: 0,
            fat_sectors: 0,
            first_dir_sector: 0,
            transaction_signature: 0,
            mini_stream_cutoff_size: 0,
            first_mini_fat_sector: 0,
            mini_fat_sectors: 0,
            first_difat_sector: 0,
            difat_sectors: 0,
            difat: [0; 109],
        };

        // Read signature
        reader
            .read_exact(&mut header.signature)
            .map_err(|e| HwpError::IoError(e))?;

        // Validate signature
        if header.signature != CFB_SIGNATURE {
            return Err(HwpError::InvalidFormat {
                reason: "Invalid CFB signature".to_string(),
            });
        }

        // Read CLSID
        reader
            .read_exact(&mut header.clsid)
            .map_err(|e| HwpError::IoError(e))?;

        // Read version and byte order
        header.minor_version = reader
            .read_u16::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.major_version = reader
            .read_u16::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.byte_order = reader
            .read_u16::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;

        // Validate byte order
        if header.byte_order != 0xFFFE {
            return Err(HwpError::InvalidFormat {
                reason: "Invalid byte order marker".to_string(),
            });
        }

        // Read sector sizes
        header.sector_shift = reader
            .read_u16::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.mini_sector_shift = reader
            .read_u16::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;

        // Read reserved bytes
        reader
            .read_exact(&mut header.reserved)
            .map_err(|e| HwpError::IoError(e))?;

        // Read sector counts
        header.total_sectors = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.fat_sectors = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;

        // Read directory and mini FAT info
        header.first_dir_sector = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.transaction_signature = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.mini_stream_cutoff_size = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.first_mini_fat_sector = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.mini_fat_sectors = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;

        // Read DIFAT info
        header.first_difat_sector = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;
        header.difat_sectors = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| HwpError::IoError(e))?;

        // Read DIFAT array (first 109 FAT sector positions)
        for i in 0..109 {
            header.difat[i] = reader
                .read_u32::<LittleEndian>()
                .map_err(|e| HwpError::IoError(e))?;
        }

        Ok(header)
    }

    /// Get the sector size in bytes
    pub fn sector_size(&self) -> u32 {
        1 << self.sector_shift
    }

    /// Get the mini sector size in bytes
    pub fn mini_sector_size(&self) -> u32 {
        1 << self.mini_sector_shift
    }

    /// Check if this is a valid version 3 or 4 CFB file
    pub fn is_valid_version(&self) -> bool {
        matches!(self.major_version, 3 | 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfb_header_size() {
        // CFB header should be exactly 512 bytes
        assert_eq!(
            8 + 16 + 2 + 2 + 2 + 2 + 2 + 6 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + (109 * 4),
            512
        );
    }

    #[test]
    fn test_sector_sizes() {
        let mut header = CfbHeader {
            signature: CFB_SIGNATURE,
            clsid: [0; 16],
            minor_version: 0,
            major_version: 3,
            byte_order: 0xFFFE,
            sector_shift: 9,
            mini_sector_shift: 6,
            reserved: [0; 6],
            total_sectors: 0,
            fat_sectors: 0,
            first_dir_sector: 0,
            transaction_signature: 0,
            mini_stream_cutoff_size: 4096,
            first_mini_fat_sector: 0xFFFFFFFE,
            mini_fat_sectors: 0,
            first_difat_sector: 0xFFFFFFFE,
            difat_sectors: 0,
            difat: [0xFFFFFFFE; 109],
        };

        assert_eq!(header.sector_size(), 512);
        assert_eq!(header.mini_sector_size(), 64);

        header.sector_shift = 12;
        assert_eq!(header.sector_size(), 4096);
    }
}
