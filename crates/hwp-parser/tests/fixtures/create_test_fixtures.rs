use hwp_core::constants::FILE_HEADER_SIGNATURE;
use hwp_core::models::{HwpHeader, HwpVersion};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Create a minimal valid HWP file with just a header
fn create_minimal_hwp(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Write HWP signature
    file.write_all(&FILE_HEADER_SIGNATURE)?;
    
    // Write version (5.0.0.0)
    let version = HwpVersion { major: 5, minor: 0, build: 0, revision: 0 };
    file.write_all(&[version.major, version.minor, version.build, version.revision])?;
    
    // Write file flags (no compression, no password)
    file.write_all(&[0u8; 4])?;
    
    // Write reserved bytes
    file.write_all(&[0u8; 216])?;
    
    // Write basic CFB structure (minimal)
    // This would need actual CFB container structure
    // For now, we'll create an invalid but parseable structure
    
    Ok(())
}

/// Create a simple HWP file with plain text
fn create_simple_text_hwp(path: &Path, text: &str) -> std::io::Result<()> {
    // This is a placeholder - actual implementation would need
    // proper CFB container, DocInfo, and Section structures
    create_minimal_hwp(path)?;
    
    // In a real implementation, we'd add:
    // 1. CFB container structure
    // 2. DocInfo stream with minimal records
    // 3. Section0 stream with text paragraph
    
    Ok(())
}

/// Create a HWP file with Korean text
fn create_korean_text_hwp(path: &Path) -> std::io::Result<()> {
    create_simple_text_hwp(path, "안녕하세요. 한글 문서입니다.")
}

/// Create a corrupted HWP file (truncated)
fn create_truncated_hwp(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Write only partial header (truncated at 100 bytes instead of 256)
    file.write_all(&FILE_HEADER_SIGNATURE)?;
    file.write_all(&[5u8, 0, 0, 0])?; // version
    file.write_all(&[0u8; 88])?; // truncated!
    
    Ok(())
}

/// Create a HWP file with invalid header
fn create_bad_header_hwp(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Write invalid signature
    file.write_all(b"INVALID_")?;
    
    // Rest of the header
    file.write_all(&[5u8, 0, 0, 0])?; // version
    file.write_all(&[0u8; 244])?;
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Create basic fixtures
    create_minimal_hwp(Path::new("tests/fixtures/basic/empty.hwp"))?;
    create_simple_text_hwp(Path::new("tests/fixtures/basic/simple_text.hwp"), "Hello, World!")?;
    create_simple_text_hwp(Path::new("tests/fixtures/basic/single_para.hwp"), "This is a single paragraph.")?;
    
    // Create encoding fixtures
    create_korean_text_hwp(Path::new("tests/fixtures/encoding/korean_only.hwp"))?;
    create_simple_text_hwp(
        Path::new("tests/fixtures/encoding/mixed_lang.hwp"),
        "English 한글 中文 日本語"
    )?;
    create_simple_text_hwp(
        Path::new("tests/fixtures/encoding/special_chars.hwp"),
        "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?"
    )?;
    
    // Create corrupted fixtures
    create_truncated_hwp(Path::new("tests/fixtures/corrupted/truncated.hwp"))?;
    create_bad_header_hwp(Path::new("tests/fixtures/corrupted/bad_header.hwp"))?;
    
    println!("Test fixtures created successfully!");
    Ok(())
}