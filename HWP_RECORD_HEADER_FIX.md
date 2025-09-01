# HWP Record Header Parsing Fix

## Problem
The record header parsing was using an incorrect bit layout, causing buffer underflow errors when parsing HWP v5.x files. The issue manifested as:
```
Error: Buffer underflow: attempted to read 12288 bytes, but only 6721 available
```

## Root Cause
The HWP record header uses a 32-bit structure divided as:
- **Correct format** (as per hwp.js and HWP spec):
  - Tag ID: bits 0-9 (10 bits)
  - Level: bits 10-19 (10 bits)
  - Size: bits 20-31 (12 bits)

- **Previous incorrect implementation**:
  - Tag ID: bits 0-9 (10 bits) ✓
  - Level: bits 10-11 (2 bits) ❌
  - Size: bits 12-31 (20 bits) ❌

This incorrect bit layout caused the parser to misinterpret record sizes, leading to buffer underflow errors.

## Solution
Updated the RecordHeader struct methods in `hwp-core/src/models/record.rs`:
- `level()`: Changed from `(value >> 10) & 0x3` to `(value >> 10) & 0x3FF`
- `size()`: Changed from `value >> 12` to `(value >> 20) & 0xFFF`
- `has_extended_size()`: Changed check from `0xFFFFF` to `0xFFF`

Also updated all related code:
- Test cases in record parser tests
- Integration tests
- dump_records example utility

## Testing
- All unit tests now pass
- Integration tests updated to use correct header format
- Ready for testing with actual HWP files

## Impact
This fix resolves GitHub issue #1 and enables proper parsing of HWP v5.x document files.