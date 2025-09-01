use anyhow::Result;
use clap::Args;
use hwp_parser::parse;
use hwp_core::HwpDocument;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InfoCommand {
    /// Input HWP file path
    pub input: PathBuf,
    
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Verbose output with detailed information
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Show document statistics
    #[arg(long)]
    pub stats: bool,
    
    /// Show font information
    #[arg(long)]
    pub fonts: bool,
    
    /// Show style information
    #[arg(long)]
    pub styles: bool,
    
    /// Check file integrity
    #[arg(long)]
    pub check_integrity: bool,
}

impl InfoCommand {
    pub fn execute(&self) -> Result<()> {
        // Read and parse the HWP file
        let hwp_data = fs::read(&self.input)?;
        let file_size = hwp_data.len();
        let document = parse(&hwp_data)?;
        
        // Generate info based on format
        let output = match self.format.as_str() {
            "json" => self.generate_json_info(&document, file_size)?,
            _ => self.generate_text_info(&document, file_size)?,
        };
        
        // Write output
        if let Some(output_path) = &self.output {
            let mut file = fs::File::create(output_path)?;
            file.write_all(output.as_bytes())?;
            eprintln!("File information written to: {}", output_path.display());
        } else {
            print!("{}", output);
        }
        
        Ok(())
    }
    
    fn generate_text_info(&self, document: &HwpDocument, file_size: usize) -> Result<String> {
        let mut info = String::new();
        
        info.push_str(&format!("=== HWP File Information ===\n"));
        info.push_str(&format!("File: {}\n", self.input.display()));
        info.push_str(&format!("Size: {} bytes ({:.2} MB)\n", file_size, file_size as f64 / 1_048_576.0));
        info.push_str(&format!("\n"));
        
        // Header information
        info.push_str(&format!("=== Header ===\n"));
        info.push_str(&format!("Version: {}\n", document.header.version));
        info.push_str(&format!("Properties: 0x{:08X}\n", document.header.properties.to_u32()));
        info.push_str(&format!("Compressed: {}\n", if document.header.is_compressed() { "Yes" } else { "No" }));
        info.push_str(&format!("Has password: {}\n", if document.header.has_password() { "Yes" } else { "No" }));
        info.push_str(&format!("DRM protected: {}\n", if document.header.is_drm_document() { "Yes" } else { "No" }));
        info.push_str(&format!("\n"));
        
        // Document properties
        info.push_str(&format!("=== Document Properties ===\n"));
        info.push_str(&format!("Section count: {}\n", document.doc_info.properties.section_count));
        info.push_str(&format!("Total pages: {}\n", document.doc_info.properties.total_page_count));
        info.push_str(&format!("Total characters: {}\n", document.doc_info.properties.total_character_count));
        info.push_str(&format!("\n"));
        
        // DocInfo summary
        info.push_str(&format!("=== DocInfo Summary ===\n"));
        info.push_str(&format!("Character shapes: {}\n", document.doc_info.char_shapes.len()));
        info.push_str(&format!("Paragraph shapes: {}\n", document.doc_info.para_shapes.len()));
        info.push_str(&format!("Styles: {}\n", document.doc_info.styles.len()));
        info.push_str(&format!("Face names (fonts): {}\n", document.doc_info.face_names.len()));
        info.push_str(&format!("Border fills: {}\n", document.doc_info.border_fills.len()));
        info.push_str(&format!("\n"));
        
        // Fonts information if requested
        if self.fonts || self.verbose {
            info.push_str(&format!("=== Fonts ===\n"));
            for (idx, face_name) in document.doc_info.face_names.iter().enumerate() {
                info.push_str(&format!("  {}: {}\n", idx, face_name.name));
            }
            info.push_str(&format!("\n"));
        }
        
        // Sections information
        info.push_str(&format!("=== Sections ===\n"));
        info.push_str(&format!("Total sections: {}\n", document.sections.len()));
        
        if self.verbose {
            for (idx, section) in document.sections.iter().enumerate() {
                let paragraph_count = section.paragraphs.len();
                let total_chars: usize = section.paragraphs.iter()
                    .map(|p| p.text.chars().count())
                    .sum();
                info.push_str(&format!("  Section {}: {} paragraphs, {} characters\n", 
                    idx, paragraph_count, total_chars));
            }
        }
        info.push_str(&format!("\n"));
        
        // Statistics if requested
        if self.stats || self.verbose {
            info.push_str(&format!("=== Statistics ===\n"));
            
            let total_paragraphs: usize = document.sections.iter()
                .map(|s| s.paragraphs.len())
                .sum();
            let total_text_length = document.get_text().len();
            let total_chars = document.get_text().chars().count();
            
            // Count Korean characters
            let text = document.get_text();
            let korean_chars = text.chars()
                .filter(|c| (*c >= '\u{AC00}' && *c <= '\u{D7AF}') || 
                           (*c >= '\u{1100}' && *c <= '\u{11FF}') || 
                           (*c >= '\u{3130}' && *c <= '\u{318F}'))
                .count();
            
            info.push_str(&format!("Total paragraphs: {}\n", total_paragraphs));
            info.push_str(&format!("Total text length: {} bytes\n", total_text_length));
            info.push_str(&format!("Total characters: {}\n", total_chars));
            info.push_str(&format!("Korean characters: {} ({:.1}%)\n", 
                korean_chars, 
                (korean_chars as f64 / total_chars as f64) * 100.0));
            
            if total_paragraphs > 0 {
                info.push_str(&format!("Average paragraph length: {:.1} characters\n", 
                    total_chars as f64 / total_paragraphs as f64));
            }
        }
        
        Ok(info)
    }
    
    fn generate_json_info(&self, document: &HwpDocument, file_size: usize) -> Result<String> {
        let mut info = json!({
            "file": {
                "path": self.input.display().to_string(),
                "size_bytes": file_size,
                "size_mb": format!("{:.2}", file_size as f64 / 1_048_576.0),
            },
            "header": {
                "version": document.header.version.to_string(),
                "properties": format!("0x{:08X}", document.header.properties.to_u32()),
                "compressed": document.header.is_compressed(),
                "has_password": document.header.has_password(),
                "drm_protected": document.header.is_drm_document(),
            },
            "document_properties": {
                "section_count": document.doc_info.properties.section_count,
                "total_pages": document.doc_info.properties.total_page_count,
                "total_characters": document.doc_info.properties.total_character_count,
            },
            "doc_info": {
                "character_shapes": document.doc_info.char_shapes.len(),
                "paragraph_shapes": document.doc_info.para_shapes.len(),
                "styles": document.doc_info.styles.len(),
                "fonts": document.doc_info.face_names.len(),
                "border_fills": document.doc_info.border_fills.len(),
            },
            "sections": {
                "count": document.sections.len(),
            }
        });
        
        // Add fonts if requested
        if self.fonts || self.verbose {
            let fonts: Vec<_> = document.doc_info.face_names.iter()
                .enumerate()
                .map(|(idx, face)| json!({
                    "index": idx,
                    "name": face.name,
                }))
                .collect();
            info["fonts"] = json!(fonts);
        }
        
        // Add detailed section info if verbose
        if self.verbose {
            let sections: Vec<_> = document.sections.iter()
                .enumerate()
                .map(|(idx, section)| {
                    let total_chars: usize = section.paragraphs.iter()
                        .map(|p| p.text.chars().count())
                        .sum();
                    json!({
                        "index": idx,
                        "paragraphs": section.paragraphs.len(),
                        "characters": total_chars,
                    })
                })
                .collect();
            info["sections"]["details"] = json!(sections);
        }
        
        // Add statistics if requested
        if self.stats || self.verbose {
            let total_paragraphs: usize = document.sections.iter()
                .map(|s| s.paragraphs.len())
                .sum();
            let text = document.get_text();
            let total_chars = text.chars().count();
            let korean_chars = text.chars()
                .filter(|c| (*c >= '\u{AC00}' && *c <= '\u{D7AF}') || 
                           (*c >= '\u{1100}' && *c <= '\u{11FF}') || 
                           (*c >= '\u{3130}' && *c <= '\u{318F}'))
                .count();
            
            info["statistics"] = json!({
                "total_paragraphs": total_paragraphs,
                "total_text_bytes": text.len(),
                "total_characters": total_chars,
                "korean_characters": korean_chars,
                "korean_percentage": format!("{:.1}", (korean_chars as f64 / total_chars as f64) * 100.0),
                "average_paragraph_length": if total_paragraphs > 0 {
                    format!("{:.1}", total_chars as f64 / total_paragraphs as f64)
                } else {
                    "0".to_string()
                },
            });
        }
        
        if self.verbose {
            Ok(serde_json::to_string_pretty(&info)?)
        } else {
            Ok(serde_json::to_string(&info)?)
        }
    }
}