use anyhow::Result;
use clap::Args;
use hwp_parser::{parse, OutputFormat, FormatOptions};
use hwp_core::HwpDocument;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ExtractCommand {
    /// Input HWP file path
    pub input: PathBuf,
    
    /// Output format (text, markdown, json, html)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Preserve formatting information
    #[arg(long)]
    pub preserve_formatting: bool,
    
    /// Include document metadata
    #[arg(long)]
    pub include_metadata: bool,
    
    /// Extract specific sections (comma-separated, e.g., "1,3,5")
    #[arg(long)]
    pub sections: Option<String>,
    
    /// Search and extract matching content
    #[arg(long)]
    pub search: Option<String>,
    
    /// Context lines around search matches
    #[arg(long, default_value = "0")]
    pub context: usize,
    
    /// Line wrap width for plain text
    #[arg(long)]
    pub text_width: Option<usize>,
    
    /// Generate table of contents for Markdown
    #[arg(long)]
    pub markdown_toc: bool,
    
    /// Pretty print JSON output
    #[arg(long)]
    pub json_pretty: bool,
    
    /// Include styles in JSON output
    #[arg(long)]
    pub json_include_styles: bool,
}

impl ExtractCommand {
    pub fn execute(&self) -> Result<()> {
        // Read and parse the HWP file
        let hwp_data = fs::read(&self.input)?;
        let document = parse(&hwp_data)?;
        
        // Build format options
        let mut options = FormatOptions::default();
        options.text_width = self.text_width;
        options.markdown_toc = self.markdown_toc;
        options.json_pretty = self.json_pretty;
        options.json_include_styles = self.json_include_styles;
        
        // Extract content based on format
        let output = if self.format == "text" || self.format == "txt" {
            // Handle section filtering
            if let Some(sections_str) = &self.sections {
                self.extract_sections(&document, sections_str)?
            } else if let Some(search_query) = &self.search {
                self.search_and_extract(&document, search_query)?
            } else {
                // Use the formatter
                let formatter = OutputFormat::PlainText.create_formatter(options);
                formatter.format_document(&document)?
            }
        } else {
            // Use the appropriate formatter
            let format = match self.format.as_str() {
                "json" => OutputFormat::Json,
                "markdown" | "md" => OutputFormat::Markdown,
                _ => {
                    return Err(anyhow::anyhow!("Unsupported format: {}", self.format));
                }
            };
            
            let formatter = format.create_formatter(options);
            formatter.format_document(&document)?
        };
        
        // Write output
        if let Some(output_path) = &self.output {
            let mut file = fs::File::create(output_path)?;
            file.write_all(output.as_bytes())?;
            eprintln!("Extracted content written to: {}", output_path.display());
        } else {
            print!("{}", output);
        }
        
        Ok(())
    }
    
    fn extract_sections(&self, document: &HwpDocument, sections_str: &str) -> Result<String> {
        let mut result = String::new();
        
        // Parse section numbers
        let section_numbers: Vec<usize> = sections_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        
        for section_num in section_numbers {
            if let Some(section) = document.sections.get(section_num) {
                result.push_str(&format!("=== Section {} ===\n", section_num));
                for paragraph in &section.paragraphs {
                    if !paragraph.text.is_empty() {
                        result.push_str(&paragraph.text);
                        result.push('\n');
                    }
                }
                result.push('\n');
            } else {
                eprintln!("Warning: Section {} not found", section_num);
            }
        }
        
        Ok(result)
    }
    
    fn search_and_extract(&self, document: &HwpDocument, query: &str) -> Result<String> {
        let mut result = String::new();
        let context = self.context;
        
        for (section_idx, section) in document.sections.iter().enumerate() {
            let mut section_matches = Vec::new();
            
            // Find matching paragraphs
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                if paragraph.text.to_lowercase().contains(&query.to_lowercase()) {
                    section_matches.push(para_idx);
                }
            }
            
            // Extract with context
            if !section_matches.is_empty() {
                result.push_str(&format!("=== Section {} ===\n", section_idx));
                
                for &match_idx in &section_matches {
                    // Include context before
                    let start = if match_idx >= context {
                        match_idx - context
                    } else {
                        0
                    };
                    
                    // Include context after
                    let end = std::cmp::min(match_idx + context + 1, section.paragraphs.len());
                    
                    for i in start..end {
                        if let Some(para) = section.paragraphs.get(i) {
                            if i == match_idx {
                                result.push_str(">>> ");
                            }
                            result.push_str(&para.text);
                            result.push('\n');
                        }
                    }
                    result.push('\n');
                }
            }
        }
        
        if result.is_empty() {
            result = format!("No matches found for: {}", query);
        }
        
        Ok(result)
    }
}