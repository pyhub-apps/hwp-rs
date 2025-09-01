# CFB (Compound File Binary) Parser for HWP Files

## Overview

This module implements a complete CFB (Compound File Binary) format parser for reading HWP v5.x files. The CFB format, also known as OLE (Object Linking and Embedding) or Microsoft Compound Document format, is used by HWP v5.x and later versions to store document data in a structured container format.

## Architecture

The CFB parser is organized into several modules:

### Core Modules

- **`cfb/mod.rs`** - Main module with public API and constants
- **`cfb/header.rs`** - CFB header parsing (512-byte header)
- **`cfb/fat.rs`** - FAT (File Allocation Table) and Mini FAT management
- **`cfb/directory.rs`** - Directory entry parsing and tree navigation
- **`cfb/stream.rs`** - Stream extraction and decompression
- **`cfb/container.rs`** - High-level container API

### Key Components

#### CfbHeader
- Parses the 512-byte CFB header
- Validates signature (0xD0CF11E0A1B11AE1)
- Extracts sector sizes and FAT information
- Supports both version 3 (512-byte sectors) and version 4 (4096-byte sectors)

#### FatTable
- Manages the File Allocation Table
- Tracks sector chains for streams
- Provides chain traversal and data reading

#### MiniFatTable
- Handles small streams (< 4096 bytes)
- Uses 64-byte mini sectors
- Stored within the root directory stream

#### DirectoryEntry
- Represents files and folders in the container
- Supports red-black tree structure
- Contains stream metadata (size, starting sector)

#### CfbContainer
- High-level API for CFB file access
- Stream extraction with automatic decompression
- Caching for improved performance

## HWP-Specific Streams

HWP v5.x files contain the following standard streams:

### Required Streams
- **`FileHeader`** - HWP file header (signature, version, properties)
- **`DocInfo`** - Document information and metadata
- **`BodyText/Section0`**, `Section1`, ... - Document content sections

### Optional Streams
- **`BinData/`** - Binary data (images, OLE objects)
- **`Scripts/`** - Embedded scripts
- **`DocOptions/`** - Document options and settings
- **`Summary`** - Document summary information

## Usage

### Basic Usage

```rust
use hwp_parser::cfb::{parse_cfb_bytes, CfbContainer};
use std::io::Cursor;

// Read HWP file
let data = std::fs::read("document.hwp")?;

// Parse CFB container
let mut container = parse_cfb_bytes(&data)?;
let mut cursor = Cursor::new(&data);

// Read FileHeader stream
let header_stream = container.read_stream(&mut cursor, "FileHeader")?;

// Check if compressed and decompress
let header_data = if header_stream.is_compressed() {
    header_stream.decompress()?
} else {
    header_stream.as_bytes().to_vec()
};

// List all streams
for stream_name in container.list_streams() {
    println!("Stream: {}", stream_name);
}
```

### Stream Extraction

```rust
// Read a specific section
let section = container.read_stream(&mut cursor, "BodyText/Section0")?;

// Automatic decompression if needed
let content = if section.is_compressed() {
    section.decompress()?
} else {
    section.as_bytes().to_vec()
};
```

### Directory Navigation

```rust
// Get root entry
if let Some(root) = container.root_entry() {
    println!("Root: {}", root.name);
}

// Check if stream exists
if container.has_stream("DocInfo") {
    // Process DocInfo stream
}
```

## Compression

HWP files commonly use zlib compression for streams. The parser automatically:
1. Detects compressed streams (zlib header: 0x789C, 0x78DA, etc.)
2. Provides decompression methods
3. Handles both compressed and uncompressed data transparently

## Error Handling

The parser provides detailed error messages for:
- Invalid CFB signatures
- Unsupported versions
- Corrupted FAT chains
- Missing streams
- Decompression failures

## Example Program

An example CFB parser utility is provided in `examples/cfb_parser.rs`:

```bash
# Show CFB container information
cargo run --example cfb_parser document.hwp info

# List all streams
cargo run --example cfb_parser document.hwp list

# Extract all streams to files
cargo run --example cfb_parser document.hwp extract

# Read a specific stream
cargo run --example cfb_parser document.hwp read FileHeader
```

## Testing

Comprehensive tests are provided in `tests/cfb_tests.rs`:

```bash
# Run CFB parser tests
cargo test --package hwp-parser --test cfb_tests
```

## Technical Details

### CFB Structure
```
+----------------+ 0x000
|   CFB Header   | (512 bytes)
+----------------+ 0x200
|   FAT Sector   | (FAT entries)
+----------------+
| Directory Sect | (Directory entries)
+----------------+
|  Data Sectors  | (Stream data)
+----------------+
```

### Sector Chains
- Regular streams use FAT chains (512 or 4096-byte sectors)
- Small streams use Mini FAT chains (64-byte mini sectors)
- Chains terminated with ENDOFCHAIN (0xFFFFFFFE)

### Directory Structure
- Red-black tree for efficient lookup
- Each entry is 128 bytes
- Contains name, type, timestamps, and stream location

## Performance Considerations

1. **Caching** - Streams are cached after first read
2. **Lazy Loading** - Streams loaded on-demand
3. **Memory Efficiency** - Sector-based reading reduces memory usage
4. **Parallel Processing** - Multiple streams can be read concurrently

## Limitations

- Write support not implemented (read-only)
- Some advanced CFB features not supported (transactions, encryption)
- Large file support depends on available memory

## Future Enhancements

- [ ] Streaming API for large files
- [ ] Write/modification support
- [ ] Encrypted stream support
- [ ] Performance optimizations for large documents
- [ ] Memory-mapped file support

## References

- [MS-CFB]: Microsoft Compound File Binary File Format
- HWP 5.0 File Format Specification
- OLE/COM Structured Storage Documentation