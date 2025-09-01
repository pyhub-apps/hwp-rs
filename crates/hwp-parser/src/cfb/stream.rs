use hwp_core::{Result, HwpError};
use std::io::{Read, Seek};
use super::constants::*;
use super::directory::DirectoryEntry;
use super::fat::{FatTable, MiniFatTable};
use super::header::CfbHeader;

/// A stream within a CFB container
#[derive(Debug)]
pub struct Stream {
    /// Stream name
    pub name: String,
    /// Stream data
    pub data: Vec<u8>,
    /// Stream size
    pub size: u64,
}

impl Stream {
    /// Create a new stream
    pub fn new(name: String, data: Vec<u8>) -> Self {
        let size = data.len() as u64;
        Stream { name, data, size }
    }
    
    /// Read stream data from a CFB container
    pub fn from_entry<R: Read + Seek>(
        reader: &mut R,
        entry: &DirectoryEntry,
        header: &CfbHeader,
        fat: &FatTable,
        mini_fat: Option<&MiniFatTable>,
    ) -> Result<Self> {
        if !entry.is_stream() {
            return Err(HwpError::InvalidFormat {
                reason: format!("Entry '{}' is not a stream", entry.name)
            });
        }
        
        let size = entry.stream_size();
        let data = if size == 0 {
            // Empty stream
            Vec::new()
        } else if size < header.mini_stream_cutoff_size as u64 {
            // Mini stream
            if let Some(mini_fat) = mini_fat {
                if entry.starting_sector != ENDOFCHAIN {
                    mini_fat.read_chain(entry.starting_sector)?
                } else {
                    Vec::new()
                }
            } else {
                return Err(HwpError::InvalidFormat {
                    reason: "Mini FAT not available for mini stream".to_string()
                });
            }
        } else {
            // Regular stream
            if entry.starting_sector != ENDOFCHAIN {
                fat.read_chain(reader, entry.starting_sector)?
            } else {
                Vec::new()
            }
        };
        
        // Truncate to actual size (chains are sector-aligned)
        let mut data = data;
        data.truncate(size as usize);
        
        Ok(Stream::new(entry.name.clone(), data))
    }
    
    /// Get stream data as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Check if the stream is compressed
    pub fn is_compressed(&self) -> bool {
        // HWP streams are often compressed with zlib
        // Check for zlib header (0x78 0x9C or 0x78 0xDA)
        if self.data.len() >= 2 {
            let header = u16::from_be_bytes([self.data[0], self.data[1]]);
            matches!(header, 0x789C | 0x78DA | 0x7801 | 0x785E | 0x78DE)
        } else {
            false
        }
    }
    
    /// Decompress the stream if it's compressed
    pub fn decompress(&self) -> Result<Vec<u8>> {
        if !self.is_compressed() {
            return Ok(self.data.clone());
        }
        
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(&self.data[..]);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| HwpError::DecompressionError(e.to_string()))?;
        
        Ok(decompressed)
    }
}

/// Stream reader for reading data from a CFB stream
pub struct StreamReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> StreamReader<'a> {
    /// Create a new stream reader
    pub fn new(data: &'a [u8]) -> Self {
        StreamReader { data, position: 0 }
    }
    
    /// Get the current position
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Get the remaining bytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }
    
    /// Check if we've reached the end
    pub fn is_eof(&self) -> bool {
        self.position >= self.data.len()
    }
    
    /// Skip bytes
    pub fn skip(&mut self, count: usize) -> Result<()> {
        if self.position + count > self.data.len() {
            return Err(HwpError::InvalidFormat {
                reason: "Attempted to skip past end of stream".to_string()
            });
        }
        self.position += count;
        Ok(())
    }
    
    /// Peek at bytes without advancing position
    pub fn peek(&self, count: usize) -> Option<&[u8]> {
        if self.position + count <= self.data.len() {
            Some(&self.data[self.position..self.position + count])
        } else {
            None
        }
    }
}

impl<'a> Read for StreamReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = self.remaining();
        let to_read = buf.len().min(available);
        
        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
            self.position += to_read;
        }
        
        Ok(to_read)
    }
}

impl<'a> Seek for StreamReader<'a> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::Current(offset) => self.position as i64 + offset,
            std::io::SeekFrom::End(offset) => self.data.len() as i64 + offset,
        };
        
        if new_pos < 0 || new_pos > self.data.len() as i64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek position out of bounds",
            ));
        }
        
        self.position = new_pos as usize;
        Ok(self.position as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_compression_detection() {
        let uncompressed = Stream::new("test".to_string(), vec![0x00, 0x01, 0x02]);
        assert!(!uncompressed.is_compressed());
        
        let compressed = Stream::new("test".to_string(), vec![0x78, 0x9C, 0x00, 0x00]);
        assert!(compressed.is_compressed());
    }
    
    #[test]
    fn test_stream_reader() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut reader = StreamReader::new(&data);
        
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), 10);
        assert!(!reader.is_eof());
        
        let mut buf = [0u8; 3];
        assert_eq!(reader.read(&mut buf).unwrap(), 3);
        assert_eq!(buf, [0, 1, 2]);
        assert_eq!(reader.position(), 3);
        
        reader.skip(2).unwrap();
        assert_eq!(reader.position(), 5);
        
        assert_eq!(reader.peek(2), Some(&[5, 6][..]));
        assert_eq!(reader.position(), 5); // Peek doesn't advance
        
        reader.seek(std::io::SeekFrom::End(-2)).unwrap();
        assert_eq!(reader.position(), 8);
        
        reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        assert_eq!(reader.position(), 0);
    }
}