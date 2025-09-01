use super::constants::*;
use super::header::CfbHeader;
use byteorder::{LittleEndian, ReadBytesExt};
use hwp_core::{HwpError, Result};
use std::io::{Read, Seek, SeekFrom};

/// FAT (File Allocation Table) manager
pub struct FatTable {
    /// FAT entries
    pub entries: Vec<u32>,
    /// Sector size
    pub sector_size: u32,
}

impl FatTable {
    /// Create a new FAT table from a reader and header
    pub fn from_reader<R: Read + Seek>(reader: &mut R, header: &CfbHeader) -> Result<Self> {
        let sector_size = header.sector_size();
        let entries_per_sector = sector_size / 4; // Each FAT entry is 4 bytes

        // Collect all FAT sector positions from DIFAT
        let mut fat_sectors = Vec::new();

        // First 109 FAT sectors are in the header
        for &sector in header.difat.iter() {
            if sector == FREESECT {
                break;
            }
            fat_sectors.push(sector);
        }

        // If there are additional DIFAT sectors, read them
        if header.difat_sectors > 0 {
            let mut current_difat = header.first_difat_sector;
            for _ in 0..header.difat_sectors {
                if current_difat == ENDOFCHAIN || current_difat == FREESECT {
                    break;
                }

                // Seek to DIFAT sector
                let offset = (current_difat + 1) * sector_size;
                reader
                    .seek(SeekFrom::Start(offset as u64))
                    .map_err(|e| HwpError::IoError(e))?;

                // Read DIFAT entries (last entry points to next DIFAT sector)
                for _ in 0..(entries_per_sector - 1) {
                    let sector = reader
                        .read_u32::<LittleEndian>()
                        .map_err(|e| HwpError::IoError(e))?;
                    if sector != FREESECT {
                        fat_sectors.push(sector);
                    }
                }

                // Read next DIFAT sector pointer
                current_difat = reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| HwpError::IoError(e))?;
            }
        }

        // Read all FAT entries
        let mut entries = Vec::new();
        for &fat_sector in &fat_sectors {
            if fat_sector == FREESECT || fat_sector == ENDOFCHAIN {
                continue;
            }

            // Seek to FAT sector
            let offset = (fat_sector + 1) * sector_size;
            reader
                .seek(SeekFrom::Start(offset as u64))
                .map_err(|e| HwpError::IoError(e))?;

            // Read FAT entries from this sector
            for _ in 0..entries_per_sector {
                let entry = reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| HwpError::IoError(e))?;
                entries.push(entry);
            }
        }

        Ok(FatTable {
            entries,
            sector_size,
        })
    }

    /// Get the next sector in a chain
    pub fn get_next(&self, sector: u32) -> Option<u32> {
        if sector as usize >= self.entries.len() {
            return None;
        }

        let next = self.entries[sector as usize];
        if next == ENDOFCHAIN || next == FREESECT || next == FATSECT {
            None
        } else {
            Some(next)
        }
    }

    /// Get all sectors in a chain starting from the given sector
    pub fn get_chain(&self, start_sector: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start_sector;

        // Limit chain length to prevent infinite loops
        let max_chain_length = self.entries.len();
        let mut count = 0;

        while current != ENDOFCHAIN && current != FREESECT && count < max_chain_length {
            chain.push(current);

            if let Some(next) = self.get_next(current) {
                current = next;
            } else {
                break;
            }
            count += 1;
        }

        chain
    }

    /// Read data from a sector chain
    pub fn read_chain<R: Read + Seek>(&self, reader: &mut R, start_sector: u32) -> Result<Vec<u8>> {
        let chain = self.get_chain(start_sector);
        let mut data = Vec::with_capacity(chain.len() * self.sector_size as usize);

        for sector in chain {
            // Seek to sector (sectors are numbered from -1, so we add 1)
            let offset = (sector + 1) * self.sector_size;
            reader
                .seek(SeekFrom::Start(offset as u64))
                .map_err(|e| HwpError::IoError(e))?;

            // Read sector data
            let mut sector_data = vec![0u8; self.sector_size as usize];
            reader
                .read_exact(&mut sector_data)
                .map_err(|e| HwpError::IoError(e))?;

            data.extend_from_slice(&sector_data);
        }

        Ok(data)
    }
}

/// Mini FAT table for small streams
pub struct MiniFatTable {
    /// Mini FAT entries
    pub entries: Vec<u32>,
    /// Mini stream data
    pub mini_stream: Vec<u8>,
    /// Mini sector size
    mini_sector_size: u32,
}

impl MiniFatTable {
    /// Create a new Mini FAT table from a reader, header, and FAT
    pub fn from_reader<R: Read + Seek>(
        reader: &mut R,
        header: &CfbHeader,
        fat: &FatTable,
        mini_stream_start: u32,
    ) -> Result<Self> {
        let mini_sector_size = header.mini_sector_size();
        let entries_per_sector = header.sector_size() / 4;

        // Read mini FAT entries
        let mut entries = Vec::new();
        if header.mini_fat_sectors > 0 && header.first_mini_fat_sector != ENDOFCHAIN {
            let mini_fat_chain = fat.get_chain(header.first_mini_fat_sector);

            for sector in mini_fat_chain {
                let offset = (sector + 1) * header.sector_size();
                reader
                    .seek(SeekFrom::Start(offset as u64))
                    .map_err(|e| HwpError::IoError(e))?;

                for _ in 0..entries_per_sector {
                    let entry = reader
                        .read_u32::<LittleEndian>()
                        .map_err(|e| HwpError::IoError(e))?;
                    entries.push(entry);
                }
            }
        }

        // Read mini stream data
        let mini_stream = if mini_stream_start != ENDOFCHAIN {
            fat.read_chain(reader, mini_stream_start)?
        } else {
            Vec::new()
        };

        Ok(MiniFatTable {
            entries,
            mini_stream,
            mini_sector_size,
        })
    }

    /// Get the next mini sector in a chain
    pub fn get_next(&self, mini_sector: u32) -> Option<u32> {
        if mini_sector as usize >= self.entries.len() {
            return None;
        }

        let next = self.entries[mini_sector as usize];
        if next == ENDOFCHAIN || next == FREESECT {
            None
        } else {
            Some(next)
        }
    }

    /// Get all mini sectors in a chain
    pub fn get_chain(&self, start_mini_sector: u32) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start_mini_sector;

        // Limit chain length to prevent infinite loops
        let max_chain_length = self.entries.len();
        let mut count = 0;

        while current != ENDOFCHAIN && current != FREESECT && count < max_chain_length {
            chain.push(current);

            if let Some(next) = self.get_next(current) {
                current = next;
            } else {
                break;
            }
            count += 1;
        }

        chain
    }

    /// Read data from a mini sector chain
    pub fn read_chain(&self, start_mini_sector: u32) -> Result<Vec<u8>> {
        let chain = self.get_chain(start_mini_sector);
        let mut data = Vec::with_capacity(chain.len() * self.mini_sector_size as usize);

        for mini_sector in chain {
            let offset = (mini_sector * self.mini_sector_size) as usize;
            let end = offset + self.mini_sector_size as usize;

            if end > self.mini_stream.len() {
                return Err(HwpError::InvalidFormat {
                    reason: "Mini sector offset out of bounds".to_string(),
                });
            }

            data.extend_from_slice(&self.mini_stream[offset..end]);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fat_chain() {
        let fat = FatTable {
            entries: vec![1, 2, 3, ENDOFCHAIN, 5, ENDOFCHAIN],
            sector_size: 512,
        };

        let chain = fat.get_chain(0);
        assert_eq!(chain, vec![0, 1, 2, 3]);

        let chain = fat.get_chain(4);
        assert_eq!(chain, vec![4, 5]);
    }

    #[test]
    fn test_mini_fat_chain() {
        let mini_fat = MiniFatTable {
            entries: vec![1, 2, ENDOFCHAIN, 4, ENDOFCHAIN],
            mini_stream: vec![0; 320], // 5 mini sectors * 64 bytes
            mini_sector_size: 64,
        };

        let chain = mini_fat.get_chain(0);
        assert_eq!(chain, vec![0, 1, 2]);

        let chain = mini_fat.get_chain(3);
        assert_eq!(chain, vec![3, 4]);
    }
}
