# HWP v5.x CFB Compression Handling Solution

## Problem Analysis

The issue you encountered was that HWP v5.x files use CFB (Compound File Binary) format where streams like `DocInfo` and `BodyText/Section*` are compressed, but the compression wasn't being properly detected or handled.

### Key Findings:

1. **Entire streams are compressed**: In HWP v5.x, the DocInfo and BodyText streams are compressed as complete units, not at the record level.

2. **HWP Compression Format**: 
   - **First 4 bytes**: Uncompressed size (little-endian)
   - **Remaining bytes**: Raw deflate compressed data (no zlib header/checksum)

3. **The problematic bytes** `[EC, 57, 3D, 6B, 53, 51, 18, 7E]` were actually compressed data, not a size header followed by data.

## Solution Implementation

### 1. Enhanced Compression Detection (`cfb/stream.rs`)

The compression detection now:
- Assumes DocInfo and BodyText streams are compressed by default in HWP v5.x
- Validates if data looks like uncompressed records (proper tag_id, level, size)
- Checks for HWP compression format (4-byte size header)
- Falls back to other compression formats if needed

### 2. Multi-Method Decompression (`compression/mod.rs`)

The decompression now tries multiple methods:
1. **HWP format**: 4-byte size header + raw deflate
2. **Raw deflate**: Direct deflate without any header
3. **Zlib format**: Standard zlib with header (fallback)

### 3. Improved Error Handling

- More lenient size validation for HWP compression
- Fallback decompression methods when primary method fails
- Better debug output for troubleshooting

## Code Changes Summary

### Stream Compression Detection
```rust
// For DocInfo and BodyText streams, check multiple indicators:
// 1. Valid record header structure (tag_id, level, size)
// 2. HWP compression format (4-byte size header)
// 3. Default to compressed for these critical streams
```

### Decompression Strategy
```rust
// Try multiple decompression methods:
// 1. HWP format (size + raw deflate)
// 2. Raw deflate (entire stream)
// 3. Zlib (with header)
```

## Usage

The compression handling is now automatic:

```rust
// When reading a DocInfo stream
let doc_info_stream = container.read_stream(&mut cursor, "DocInfo")?;

// Compression is automatically detected
if doc_info_stream.is_compressed() {
    // Decompression tries multiple methods automatically
    let decompressed = doc_info_stream.decompress()?;
    // Parse records from decompressed data
    let doc_info = parse_doc_info(&decompressed)?;
}
```

## Testing

Created comprehensive tests in:
- `tests/compression_test.rs`: Basic compression functionality
- `tests/cfb_compression_test.rs`: CFB stream-specific compression

All tests pass, confirming the solution handles:
- HWP format compression
- Raw deflate compression
- Zlib compression
- Mixed formats and edge cases

## Key Takeaways

1. **HWP v5.x streams are compressed at the stream level**, not record level
2. **The compression format is specific**: 4-byte size + raw deflate
3. **Multiple decompression methods** provide robustness
4. **Stream-specific logic** for DocInfo and BodyText ensures proper handling

The implementation now correctly handles the compression in HWP v5.x CFB files, preventing the buffer underflow errors you were experiencing.