use crate::reader::ByteReader;
use crate::validator::{DefaultRecordValidator, RecordContext, RecordValidator};
use hwp_core::models::record::{Record, RecordHeader};
use hwp_core::{HwpError, Result};
use log::{debug, error, warn};

/// Record parser for HWP tag-based format
pub struct RecordParser<'a> {
    reader: ByteReader<'a>,
    validator: Box<dyn RecordValidator>,
    context: RecordContext,
    /// Whether to attempt recovery on errors
    enable_recovery: bool,
    /// Count of recovered errors
    recovery_count: usize,
}

impl<'a> RecordParser<'a> {
    /// Create a new record parser from data
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            reader: ByteReader::new(data),
            validator: Box::new(DefaultRecordValidator::default()),
            context: RecordContext::Unknown,
            enable_recovery: false,
            recovery_count: 0,
        }
    }

    /// Create a new record parser with context
    pub fn new_with_context(data: &'a [u8], context: RecordContext) -> Self {
        Self {
            reader: ByteReader::new(data),
            validator: Box::new(DefaultRecordValidator::default()),
            context,
            enable_recovery: false,
            recovery_count: 0,
        }
    }

    /// Create a record parser from an existing ByteReader
    pub fn from_reader(reader: ByteReader<'a>) -> Self {
        Self {
            reader,
            validator: Box::new(DefaultRecordValidator::default()),
            context: RecordContext::Unknown,
            enable_recovery: false,
            recovery_count: 0,
        }
    }

    /// Enable error recovery mode
    pub fn enable_recovery(&mut self, enable: bool) {
        self.enable_recovery = enable;
    }

    /// Get the number of recovered errors
    pub fn recovery_count(&self) -> usize {
        self.recovery_count
    }

    /// Set the validation context
    pub fn set_context(&mut self, context: RecordContext) {
        self.context = context;
    }

    /// Set a custom validator
    pub fn set_validator(&mut self, validator: Box<dyn RecordValidator>) {
        self.validator = validator;
    }

    /// Try to recover from a parse error by finding the next valid record
    fn try_recover(&mut self) -> Result<Option<Record>> {
        warn!(
            "Attempting to recover from parse error at position {}",
            self.reader.position()
        );

        // Use the recovery module to find the next valid record
        if let Some((new_pos, _header)) = crate::validator::recovery::find_next_valid_record(
            &mut self.reader,
            self.validator.as_ref(),
            self.context,
        ) {
            warn!("Found potential valid record at position {}", new_pos);
            self.reader.seek(new_pos)?;
            self.recovery_count += 1;

            // Try to parse from the recovered position
            self.parse_next_record_internal()
        } else {
            warn!("No valid record found during recovery");
            Ok(None)
        }
    }

    /// Parse the next record from the stream
    pub fn parse_next_record(&mut self) -> Result<Option<Record>> {
        let result = self.parse_next_record_internal();

        // If error recovery is enabled and we got an error, try to recover
        if self.enable_recovery && result.is_err() {
            warn!("Parse error occurred, attempting recovery: {:?}", result);
            return self.try_recover();
        }

        result
    }

    /// Internal method to parse the next record
    fn parse_next_record_internal(&mut self) -> Result<Option<Record>> {
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

        debug!("Raw header bytes: {:02X?}", header_bytes);
        debug!(
            "Parsed header: tag_id=0x{:04X}, level={}, raw_size={}",
            header.tag_id(),
            header.level(),
            header.size()
        );

        // Validate tag ID for the current context
        if !self
            .validator
            .validate_tag_id(header.tag_id(), self.context)
        {
            warn!(
                "Invalid tag ID 0x{:04X} for context {:?}",
                header.tag_id(),
                self.context
            );
            // In lenient mode, we could try to skip and recover
            // For now, return an error
            return Err(HwpError::ValidationError {
                message: format!(
                    "Invalid tag ID 0x{:04X} for context {:?}",
                    header.tag_id(),
                    self.context
                ),
            });
        }

        // Determine the actual size
        let size = if header.has_extended_size() {
            // Extended size: next 4 bytes contain the actual size
            let extended_size = self.reader.read_u32()?;
            debug!(
                "Record with extended size: tag_id={:04X}, size={}",
                header.tag_id(),
                extended_size
            );
            extended_size
        } else {
            let normal_size = header.size();
            debug!(
                "Record: tag_id={:04X}, size={}",
                header.tag_id(),
                normal_size
            );
            normal_size
        };

        // Validate size
        self.validator.validate_size(size, header.tag_id())?;

        // Validate we have enough data
        self.validator
            .validate_header(&header, self.reader.remaining())?;

        debug!(
            "Available bytes: {}, Requested bytes: {}",
            self.reader.remaining(),
            size
        );

        // Read the record data
        let data = if size > 0 {
            if size as usize > self.reader.remaining() {
                error!(
                    "Buffer underflow will occur: size={}, remaining={}",
                    size,
                    self.reader.remaining()
                );
                return Err(HwpError::BufferUnderflow {
                    requested: size as usize,
                    available: self.reader.remaining(),
                });
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
                    message: "Variable integer too long".to_string(),
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
        // Create test data: header (tag=0x0010, level=0, size=30) + data
        // Header format: tag_id(10 bits) | level(10 bits) | size(12 bits)
        // 0x0010 = DOCUMENT_PROPERTIES, needs minimum 22 bytes
        // Combined: tag_id | (level << 10) | (size << 20)
        let header_value: u32 = 0x0010 | (30 << 20);
        let header_bytes = header_value.to_le_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[0; 30]); // data

        let mut parser =
            RecordParser::new_with_context(&data, crate::validator::RecordContext::DocInfo);
        let record = parser.parse_next_record().unwrap().unwrap();

        assert_eq!(record.tag_id, 0x0010);
        assert_eq!(record.level, 0);
        assert_eq!(record.size, 30);
        assert_eq!(record.data.len(), 30);
    }

    #[test]
    fn test_parse_extended_size_record() {
        // Create test data with extended size
        // Header format: tag_id(10 bits) | level(10 bits) | size(12 bits)
        // For extended size, size field = 0xFFF (all 12 bits set)
        let header_value: u32 = 0x0010 | (0xFFF << 20);
        let header_bytes = header_value.to_le_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[30, 0x00, 0x00, 0x00]); // actual size: 30 bytes (minimum for DOCUMENT_PROPERTIES)
        data.extend_from_slice(&[0; 30]); // data

        let mut parser =
            RecordParser::new_with_context(&data, crate::validator::RecordContext::DocInfo);
        let record = parser.parse_next_record().unwrap().unwrap();

        assert_eq!(record.tag_id, 0x0010);
        assert_eq!(record.level, 0);
        assert_eq!(record.size, 30);
        assert_eq!(record.data.len(), 30);
    }

    #[test]
    fn test_parse_multiple_records() {
        // First record header: tag=0x0010, level=0, size=30 (minimum for DOCUMENT_PROPERTIES)
        let header1_value: u32 = 0x0010 | (30 << 20);
        let header1_bytes = header1_value.to_le_bytes();

        // Second record header: tag=0x0013 (FACE_NAME), level=0, size=10
        let header2_value: u32 = 0x0013 | (10 << 20);
        let header2_bytes = header2_value.to_le_bytes();

        let mut data = Vec::new();
        data.extend_from_slice(&header1_bytes);
        data.extend_from_slice(&[0; 30]); // data for first record
        data.extend_from_slice(&header2_bytes);
        data.extend_from_slice(&[0; 10]); // data for second record

        let mut parser =
            RecordParser::new_with_context(&data, crate::validator::RecordContext::DocInfo);
        let records = parser.parse_all_records().unwrap();

        assert_eq!(records.len(), 2);

        assert_eq!(records[0].tag_id, 0x0010);
        assert_eq!(records[0].data.len(), 30);

        assert_eq!(records[1].tag_id, 0x0013);
        assert_eq!(records[1].data.len(), 10);
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
