use thiserror::Error;

#[derive(Error, Debug)]
pub enum HwpError {
    #[error("Invalid HWP signature: expected 'HWP Document File'")]
    InvalidSignature,
    
    #[error("Unsupported HWP version: {version}")]
    UnsupportedVersion { version: String },
    
    #[error("Invalid file format: {reason}")]
    InvalidFormat { reason: String },
    
    #[error("Decompression failed: {0}")]
    DecompressionError(String),
    
    #[error("Parse error at offset {offset}: {message}")]
    ParseError { offset: usize, message: String },
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },
    
    #[error("Invalid record: tag={tag}, level={level}, size={size}")]
    InvalidRecord { tag: u16, level: u8, size: u32 },
    
    #[error("Buffer underflow: attempted to read {requested} bytes, but only {available} available")]
    BufferUnderflow { requested: usize, available: usize },
}

pub type Result<T> = std::result::Result<T, HwpError>;