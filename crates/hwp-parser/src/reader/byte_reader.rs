use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::EUC_KR;
use hwp_core::{HwpError, Result};
use std::io::{Cursor, Read, Seek, SeekFrom};

/// A reader for parsing binary HWP data
pub struct ByteReader<'a> {
    cursor: Cursor<&'a [u8]>,
    size: usize,
}

impl<'a> ByteReader<'a> {
    /// Create a new ByteReader from a byte slice
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            size: data.len(),
            cursor: Cursor::new(data),
        }
    }
    
    /// Get the current position in the buffer
    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }
    
    /// Get the remaining bytes available to read
    pub fn remaining(&self) -> usize {
        self.size.saturating_sub(self.position())
    }
    
    /// Check if we've reached the end of the buffer
    pub fn is_eof(&self) -> bool {
        self.remaining() == 0
    }
    
    /// Skip n bytes forward
    pub fn skip(&mut self, n: usize) -> Result<()> {
        if self.remaining() < n {
            return Err(HwpError::BufferUnderflow {
                requested: n,
                available: self.remaining(),
            });
        }
        self.cursor.seek(SeekFrom::Current(n as i64))?;
        Ok(())
    }
    
    /// Seek to an absolute position
    pub fn seek(&mut self, pos: usize) -> Result<()> {
        if pos > self.size {
            return Err(HwpError::BufferUnderflow {
                requested: pos,
                available: self.size,
            });
        }
        self.cursor.seek(SeekFrom::Start(pos as u64))?;
        Ok(())
    }
    
    /// Read a single byte
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.remaining() < 1 {
            return Err(HwpError::BufferUnderflow {
                requested: 1,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_u8()?)
    }
    
    /// Read a signed byte
    pub fn read_i8(&mut self) -> Result<i8> {
        if self.remaining() < 1 {
            return Err(HwpError::BufferUnderflow {
                requested: 1,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_i8()?)
    }
    
    /// Read a 16-bit unsigned integer (little-endian)
    pub fn read_u16(&mut self) -> Result<u16> {
        if self.remaining() < 2 {
            return Err(HwpError::BufferUnderflow {
                requested: 2,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_u16::<LittleEndian>()?)
    }
    
    /// Read a 16-bit signed integer (little-endian)
    pub fn read_i16(&mut self) -> Result<i16> {
        if self.remaining() < 2 {
            return Err(HwpError::BufferUnderflow {
                requested: 2,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_i16::<LittleEndian>()?)
    }
    
    /// Read a 32-bit unsigned integer (little-endian)
    pub fn read_u32(&mut self) -> Result<u32> {
        if self.remaining() < 4 {
            return Err(HwpError::BufferUnderflow {
                requested: 4,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_u32::<LittleEndian>()?)
    }
    
    /// Read a 32-bit signed integer (little-endian)
    pub fn read_i32(&mut self) -> Result<i32> {
        if self.remaining() < 4 {
            return Err(HwpError::BufferUnderflow {
                requested: 4,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_i32::<LittleEndian>()?)
    }
    
    /// Read a 64-bit unsigned integer (little-endian)
    pub fn read_u64(&mut self) -> Result<u64> {
        if self.remaining() < 8 {
            return Err(HwpError::BufferUnderflow {
                requested: 8,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_u64::<LittleEndian>()?)
    }
    
    /// Read a 64-bit signed integer (little-endian)
    pub fn read_i64(&mut self) -> Result<i64> {
        if self.remaining() < 8 {
            return Err(HwpError::BufferUnderflow {
                requested: 8,
                available: self.remaining(),
            });
        }
        Ok(self.cursor.read_i64::<LittleEndian>()?)
    }
    
    /// Read n bytes into a vector
    pub fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        if self.remaining() < n {
            return Err(HwpError::BufferUnderflow {
                requested: n,
                available: self.remaining(),
            });
        }
        let mut buf = vec![0u8; n];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }
    
    /// Read n bytes into an existing buffer
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let n = buf.len();
        if self.remaining() < n {
            return Err(HwpError::BufferUnderflow {
                requested: n,
                available: self.remaining(),
            });
        }
        self.cursor.read_exact(buf)?;
        Ok(())
    }
    
    /// Read a null-terminated UTF-16LE string
    pub fn read_utf16_string(&mut self) -> Result<String> {
        let mut utf16_chars = Vec::new();
        
        loop {
            let ch = self.read_u16()?;
            if ch == 0 {
                break;
            }
            utf16_chars.push(ch);
        }
        
        String::from_utf16(&utf16_chars)
            .map_err(|e| HwpError::EncodingError(e.to_string()))
    }
    
    /// Read a UTF-16LE string with a specified length (in characters)
    pub fn read_utf16_string_n(&mut self, char_count: usize) -> Result<String> {
        let mut utf16_chars = Vec::with_capacity(char_count);
        
        for _ in 0..char_count {
            utf16_chars.push(self.read_u16()?);
        }
        
        // Remove any null terminators
        if let Some(null_pos) = utf16_chars.iter().position(|&c| c == 0) {
            utf16_chars.truncate(null_pos);
        }
        
        String::from_utf16(&utf16_chars)
            .map_err(|e| HwpError::EncodingError(e.to_string()))
    }
    
    /// Read a null-terminated EUC-KR string
    pub fn read_euc_kr_string(&mut self) -> Result<String> {
        let mut bytes = Vec::new();
        
        loop {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        
        let (decoded, _, had_errors) = EUC_KR.decode(&bytes);
        if had_errors {
            return Err(HwpError::EncodingError("Invalid EUC-KR encoding".to_string()));
        }
        
        Ok(decoded.into_owned())
    }
    
    /// Read an EUC-KR string with a specified length (in bytes)
    pub fn read_euc_kr_string_n(&mut self, byte_count: usize) -> Result<String> {
        let bytes = self.read_bytes(byte_count)?;
        
        // Remove any null terminators
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let bytes = &bytes[..end];
        
        let (decoded, _, had_errors) = EUC_KR.decode(bytes);
        if had_errors {
            return Err(HwpError::EncodingError("Invalid EUC-KR encoding".to_string()));
        }
        
        Ok(decoded.into_owned())
    }
    
    /// Read all remaining bytes
    pub fn read_to_end(&mut self) -> Result<Vec<u8>> {
        let remaining = self.remaining();
        self.read_bytes(remaining)
    }
    
    /// Create a sub-reader with a limited size
    pub fn sub_reader(&mut self, size: usize) -> Result<ByteReader<'a>> {
        if self.remaining() < size {
            return Err(HwpError::BufferUnderflow {
                requested: size,
                available: self.remaining(),
            });
        }
        
        let start = self.position();
        let data = self.cursor.get_ref();
        let sub_data = &data[start..start + size];
        
        // Advance the cursor
        self.skip(size)?;
        
        Ok(ByteReader::new(sub_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_basic_types() {
        let data = vec![
            0x01, 0x02,             // u16: 0x0201 (513)
            0x03, 0x04, 0x05, 0x06, // u32: 0x06050403
            0xFF,                   // u8: 255
            0x80,                   // i8: -128
        ];
        
        let mut reader = ByteReader::new(&data);
        
        assert_eq!(reader.read_u16().unwrap(), 0x0201);
        assert_eq!(reader.read_u32().unwrap(), 0x06050403);
        assert_eq!(reader.read_u8().unwrap(), 0xFF);
        assert_eq!(reader.read_i8().unwrap(), -128);
        assert!(reader.is_eof());
    }
    
    #[test]
    fn test_utf16_string() {
        // "한글" in UTF-16LE with null terminator
        let data = vec![
            0x5C, 0xD5, // '한'
            0x00, 0xAE, // '글'
            0x00, 0x00, // null terminator
        ];
        
        let mut reader = ByteReader::new(&data);
        let s = reader.read_utf16_string().unwrap();
        assert_eq!(s, "한글");
    }
    
    #[test]
    fn test_buffer_underflow() {
        let data = vec![0x01, 0x02];
        let mut reader = ByteReader::new(&data);
        
        assert!(reader.read_u32().is_err());
        assert_eq!(reader.read_u16().unwrap(), 0x0201);
        assert!(reader.read_u8().is_err());
    }
}