use crate::reader::ByteReader;
use hwp_core::{HwpError, Result};
use hwp_core::models::record::{Record, RecordHeader};

/// Record parser for HWP tag-based format
pub struct RecordParser<'a> {
    reader: ByteReader<'a>,
}

impl<'a> RecordParser<'a> {
    /// Create a new record parser from data
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            reader: ByteReader::new(data),
        }
    }
    
    /// Create a record parser from an existing ByteReader
    pub fn from_reader(reader: ByteReader<'a>) -> Self {
        Self { reader }
    }
    
    /// Parse the next record from the stream
    pub fn parse_next_record(&mut self) -> Result<Option<Record>> {
        if self.reader.is_eof() {
            return Ok(None);
        }
        
        // Read the 4-byte header
        let header_bytes = match self.reader.read_bytes(4) {
            Ok(bytes) => {
                let mut array = [0u8; 4];
                array.copy_from_slice(&bytes);
                array
            }
            Err(HwpError::BufferUnderflow { .. }) => {
                // End of stream
                return Ok(None);
            }
            Err(e) => return Err(e),
        };
        
        let header = RecordHeader::from_bytes(header_bytes);
        
        eprintln!("[DEBUG] Raw header bytes: {:02X?}", header_bytes);
        eprintln!("[DEBUG] Parsed header: tag_id=0x{:04X}, level={}, raw_size={}", 
                 header.tag_id(), header.level(), header.size());
        
        // Determine the actual size
        let size = if header.has_extended_size() {
            // Extended size: next 4 bytes contain the actual size
            let extended_size = self.reader.read_u32()?;
            eprintln!("[DEBUG] Record with extended size: tag_id={:04X}, size={}", header.tag_id(), extended_size);
            extended_size
        } else {
            let normal_size = header.size();
            eprintln!("[DEBUG] Record: tag_id={:04X}, size={}", header.tag_id(), normal_size);
            normal_size
        };
        
        eprintln!("[DEBUG] Available bytes: {}, Requested bytes: {}", self.reader.remaining(), size);
        
        // Read the record data
        let data = if size > 0 {
            if size as usize > self.reader.remaining() {
                eprintln!("[ERROR] Buffer underflow will occur: size={}, remaining={}", size, self.reader.remaining());
            }
            self.reader.read_bytes(size as usize)?
        } else {
            Vec::new()
        };
        
        Ok(Some(Record::new(
            header.tag_id(),
            header.level(),
            size,
            data,
        )))
    }
    
    /// Parse all records from the stream
    pub fn parse_all_records(&mut self) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        
        while let Some(record) = self.parse_next_record()? {
            records.push(record);
        }
        
        Ok(records)
    }
    
    /// Parse records until a specific tag is found
    pub fn parse_until_tag(&mut self, target_tag: u16) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        
        while let Some(record) = self.parse_next_record()? {
            let found_target = record.tag_id == target_tag;
            records.push(record);
            
            if found_target {
                break;
            }
        }
        
        Ok(records)
    }
    
    /// Get the current position in the stream
    pub fn position(&self) -> usize {
        self.reader.position()
    }
    
    /// Get the remaining bytes in the stream
    pub fn remaining(&self) -> usize {
        self.reader.remaining()
    }
    
    /// Check if we've reached the end of the stream
    pub fn is_eof(&self) -> bool {
        self.reader.is_eof()
    }
}

/// Record data parser for specific record types
pub struct RecordDataParser<'a> {
    reader: ByteReader<'a>,
}

impl<'a> RecordDataParser<'a> {
    /// Create a new record data parser
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            reader: ByteReader::new(data),
        }
    }
    
    /// Create from a record
    pub fn from_record(record: &'a Record) -> Self {
        Self::new(&record.data)
    }
    
    /// Get the underlying reader
    pub fn reader(&mut self) -> &mut ByteReader<'a> {
        &mut self.reader
    }
    
    /// Read a variable-length integer (used in some record formats)
    pub fn read_varint(&mut self) -> Result<u32> {
        let mut result = 0u32;
        let mut shift = 0;
        
        loop {
            let byte = self.reader.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            
            if (byte & 0x80) == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 32 {
                return Err(HwpError::ParseError { 
                    offset: 0, 
                    message: "Variable integer too long".to_string() 
                });
            }
        }
        
        Ok(result)
    }
    
    /// Read a HWP string (length-prefixed UTF-16LE)
    pub fn read_hwp_string(&mut self) -> Result<String> {
        let length = self.reader.read_u16()? as usize;
        if length == 0 {
            return Ok(String::new());
        }
        
        self.reader.read_utf16_string_n(length)
    }
    
    /// Read a fixed-size HWP string
    pub fn read_hwp_string_n(&mut self, char_count: usize) -> Result<String> {
        self.reader.read_utf16_string_n(char_count)
    }
    
    /// Read HWP array data (count followed by items)
    pub fn read_hwp_array<T, F>(&mut self, mut reader_fn: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut ByteReader<'a>) -> Result<T>,
    {
        let count = self.reader.read_u16()? as usize;
        let mut items = Vec::with_capacity(count);
        
        for _ in 0..count {
            items.push(reader_fn(&mut self.reader)?);
        }
        
        Ok(items)
    }
    
    /// Check if there's more data to read
    pub fn has_more_data(&self) -> bool {
        !self.reader.is_eof()
    }
    
    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.reader.remaining()
    }
    
    /// Skip bytes
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.reader.skip(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_record() {
        // Create test data: header (tag=0x0010, level=0, size=4) + data
        // Header format: tag_id(10 bits) | level(2 bits) | size(20 bits)
        // 0x0010 = 0b00000_10000, level=0, size=4
        // Combined: 0b0000_0100_0000_0000_00_00_0001_0000 = 0x00004010
        let header_value: u32 = (0x0010) | (0 << 10) | (4 << 12);
        let header_bytes = header_value.to_le_bytes();
        
        let mut data = Vec::new();
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // data
        
        let mut parser = RecordParser::new(&data);
        let record = parser.parse_next_record().unwrap().unwrap();
        
        assert_eq!(record.tag_id, 0x0010);
        assert_eq!(record.level, 0);
        assert_eq!(record.size, 4);
        assert_eq!(record.data, vec![0x01, 0x02, 0x03, 0x04]);
    }
    
    #[test]
    fn test_parse_extended_size_record() {
        // Create test data with extended size
        // Header format: tag_id(10 bits) | level(2 bits) | size(20 bits)
        // For extended size, size field = 0xFFFFF (all 20 bits set)
        let header_value: u32 = (0x0010) | (0 << 10) | (0xFFFFF << 12);
        let header_bytes = header_value.to_le_bytes();
        
        let mut data = Vec::new();
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // actual size: 8 bytes
        data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]); // data
        
        let mut parser = RecordParser::new(&data);
        let record = parser.parse_next_record().unwrap().unwrap();
        
        assert_eq!(record.tag_id, 0x0010);
        assert_eq!(record.level, 0);
        assert_eq!(record.size, 8);
        assert_eq!(record.data, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }
    
    #[test]
    fn test_parse_multiple_records() {
        // First record header: tag=0x0010, level=0, size=2
        let header1_value: u32 = (0x0010) | (0 << 10) | (2 << 12);
        let header1_bytes = header1_value.to_le_bytes();
        
        // Second record header: tag=0x0011, level=0, size=3
        let header2_value: u32 = (0x0011) | (0 << 10) | (3 << 12);
        let header2_bytes = header2_value.to_le_bytes();
        
        let mut data = Vec::new();
        data.extend_from_slice(&header1_bytes);
        data.extend_from_slice(&[0x01, 0x02]); // data for first record
        data.extend_from_slice(&header2_bytes);
        data.extend_from_slice(&[0x03, 0x04, 0x05]); // data for second record
        
        let mut parser = RecordParser::new(&data);
        let records = parser.parse_all_records().unwrap();
        
        assert_eq!(records.len(), 2);
        
        assert_eq!(records[0].tag_id, 0x0010);
        assert_eq!(records[0].data, vec![0x01, 0x02]);
        
        assert_eq!(records[1].tag_id, 0x0011);
        assert_eq!(records[1].data, vec![0x03, 0x04, 0x05]);
    }
    
    #[test]
    fn test_record_data_parser() {
        let record_data = vec![
            0x03, 0x00, // string length: 3 characters
            0x48, 0x00, // 'H'
            0x57, 0x00, // 'W'
            0x50, 0x00, // 'P'
        ];
        
        let mut data_parser = RecordDataParser::new(&record_data);
        let string = data_parser.read_hwp_string().unwrap();
        assert_eq!(string, "HWP");
    }
    
    #[test]
    fn test_varint_parsing() {
        let data = vec![
            0x96, 0x01, // 150 (0x96)
            0x80, 0x02, // 256 (0x80 + 0x02 << 7)
        ];
        
        let mut parser = RecordDataParser::new(&data);
        assert_eq!(parser.read_varint().unwrap(), 150);
        assert_eq!(parser.read_varint().unwrap(), 256);
    }
}