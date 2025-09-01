use anyhow::Result;
use clap::Args;
use hwp_core::HwpDocument;
use hwp_parser::parse;
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

    /// Show only document metadata
    #[arg(long)]
    pub metadata_only: bool,

    /// Analyze document complexity
    #[arg(long)]
    pub analyze_complexity: bool,

    /// Show word frequency analysis
    #[arg(long)]
    pub word_frequency: bool,

    /// Show paragraph statistics
    #[arg(long)]
    pub paragraph_stats: bool,

    /// Show style usage analysis
    #[arg(long)]
    pub style_analysis: bool,
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
        info.push_str(&format!(
            "Size: {} bytes ({:.2} MB)\n",
            file_size,
            file_size as f64 / 1_048_576.0
        ));
        info.push_str(&format!("\n"));

        // Header information
        info.push_str(&format!("=== Header ===\n"));
        info.push_str(&format!("Version: {}\n", document.header.version));
        info.push_str(&format!(
            "Properties: 0x{:08X}\n",
            document.header.properties.to_u32()
        ));
        info.push_str(&format!(
            "Compressed: {}\n",
            if document.header.is_compressed() {
                "Yes"
            } else {
                "No"
            }
        ));
        info.push_str(&format!(
            "Has password: {}\n",
            if document.header.has_password() {
                "Yes"
            } else {
                "No"
            }
        ));
        info.push_str(&format!(
            "DRM protected: {}\n",
            if document.header.is_drm_document() {
                "Yes"
            } else {
                "No"
            }
        ));
        info.push_str(&format!("\n"));

        // Document properties
        info.push_str(&format!("=== Document Properties ===\n"));
        info.push_str(&format!(
            "Section count: {}\n",
            document.doc_info.properties.section_count
        ));
        info.push_str(&format!(
            "Total pages: {}\n",
            document.doc_info.properties.total_page_count
        ));
        info.push_str(&format!(
            "Total characters: {}\n",
            document.doc_info.properties.total_character_count
        ));
        info.push_str(&format!("\n"));

        // DocInfo summary
        info.push_str(&format!("=== DocInfo Summary ===\n"));
        info.push_str(&format!(
            "Character shapes: {}\n",
            document.doc_info.char_shapes.len()
        ));
        info.push_str(&format!(
            "Paragraph shapes: {}\n",
            document.doc_info.para_shapes.len()
        ));
        info.push_str(&format!("Styles: {}\n", document.doc_info.styles.len()));
        info.push_str(&format!(
            "Face names (fonts): {}\n",
            document.doc_info.face_names.len()
        ));
        info.push_str(&format!(
            "Border fills: {}\n",
            document.doc_info.border_fills.len()
        ));
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
                let total_chars: usize = section
                    .paragraphs
                    .iter()
                    .map(|p| p.text.chars().count())
                    .sum();
                info.push_str(&format!(
                    "  Section {}: {} paragraphs, {} characters\n",
                    idx, paragraph_count, total_chars
                ));
            }
        }
        info.push_str(&format!("\n"));

        // Statistics if requested
        if self.stats || self.verbose {
            info.push_str(&format!("=== Statistics ===\n"));

            let total_paragraphs: usize =
                document.sections.iter().map(|s| s.paragraphs.len()).sum();
            let total_text_length = document.get_text().len();
            let total_chars = document.get_text().chars().count();

            // Count Korean characters
            let text = document.get_text();
            let korean_chars = text
                .chars()
                .filter(|c| {
                    (*c >= '\u{AC00}' && *c <= '\u{D7AF}')
                        || (*c >= '\u{1100}' && *c <= '\u{11FF}')
                        || (*c >= '\u{3130}' && *c <= '\u{318F}')
                })
                .count();

            info.push_str(&format!("Total paragraphs: {}\n", total_paragraphs));
            info.push_str(&format!("Total text length: {} bytes\n", total_text_length));
            info.push_str(&format!("Total characters: {}\n", total_chars));
            info.push_str(&format!(
                "Korean characters: {} ({:.1}%)\n",
                korean_chars,
                (korean_chars as f64 / total_chars as f64) * 100.0
            ));

            if total_paragraphs > 0 {
                info.push_str(&format!(
                    "Average paragraph length: {:.1} characters\n",
                    total_chars as f64 / total_paragraphs as f64
                ));
            }
        }

        // Analyze document complexity if requested
        if self.analyze_complexity {
            info.push_str(&self.analyze_document_complexity(document));
        }

        // Show word frequency if requested
        if self.word_frequency {
            info.push_str(&self.show_word_frequency(document));
        }

        // Show paragraph statistics if requested
        if self.paragraph_stats {
            info.push_str(&self.show_paragraph_statistics(document));
        }

        // Show style usage analysis if requested
        if self.style_analysis {
            info.push_str(&self.analyze_style_usage(document));
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
            let fonts: Vec<_> = document
                .doc_info
                .face_names
                .iter()
                .enumerate()
                .map(|(idx, face)| {
                    json!({
                        "index": idx,
                        "name": face.name,
                    })
                })
                .collect();
            info["fonts"] = json!(fonts);
        }

        // Add detailed section info if verbose
        if self.verbose {
            let sections: Vec<_> = document
                .sections
                .iter()
                .enumerate()
                .map(|(idx, section)| {
                    let total_chars: usize = section
                        .paragraphs
                        .iter()
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
            let total_paragraphs: usize =
                document.sections.iter().map(|s| s.paragraphs.len()).sum();
            let text = document.get_text();
            let total_chars = text.chars().count();
            let korean_chars = text
                .chars()
                .filter(|c| {
                    (*c >= '\u{AC00}' && *c <= '\u{D7AF}')
                        || (*c >= '\u{1100}' && *c <= '\u{11FF}')
                        || (*c >= '\u{3130}' && *c <= '\u{318F}')
                })
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

    fn analyze_document_complexity(&self, document: &HwpDocument) -> String {
        let mut info = String::new();
        info.push_str("\n=== Document Complexity Analysis ===\n");

        let total_sections = document.sections.len();
        let total_paragraphs: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
        let total_chars: usize = document
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .map(|p| p.text.len())
            .sum();
        let avg_para_length = if total_paragraphs > 0 {
            total_chars / total_paragraphs
        } else {
            0
        };

        // Calculate complexity score (simple heuristic)
        let complexity_score = (total_sections as f64 * 0.1)
            + (total_paragraphs as f64 * 0.01)
            + (total_chars as f64 * 0.0001);

        info.push_str(&format!("Complexity Score: {:.2}\n", complexity_score));
        info.push_str(&format!(
            "Average paragraph length: {} characters\n",
            avg_para_length
        ));

        // Classify complexity
        let complexity_level = match complexity_score {
            s if s < 10.0 => "Simple",
            s if s < 50.0 => "Moderate",
            s if s < 100.0 => "Complex",
            _ => "Very Complex",
        };
        info.push_str(&format!("Complexity Level: {}\n", complexity_level));

        info
    }

    fn show_word_frequency(&self, document: &HwpDocument) -> String {
        use std::collections::HashMap;

        let mut info = String::new();
        info.push_str("\n=== Word Frequency Analysis ===\n");

        let mut word_count: HashMap<String, usize> = HashMap::new();

        for section in &document.sections {
            for paragraph in &section.paragraphs {
                let words = paragraph.text.split_whitespace();
                for word in words {
                    // Simple normalization (lowercase)
                    let normalized = word.to_lowercase();
                    *word_count.entry(normalized).or_insert(0) += 1;
                }
            }
        }

        // Sort by frequency
        let mut word_vec: Vec<_> = word_count.iter().collect();
        word_vec.sort_by(|a, b| b.1.cmp(a.1));

        // Show top 10 words
        info.push_str("Top 10 most frequent words:\n");
        for (i, (word, count)) in word_vec.iter().take(10).enumerate() {
            info.push_str(&format!("  {}. '{}': {} occurrences\n", i + 1, word, count));
        }

        info.push_str(&format!("Total unique words: {}\n", word_count.len()));

        info
    }

    fn show_paragraph_statistics(&self, document: &HwpDocument) -> String {
        let mut info = String::new();
        info.push_str("\n=== Paragraph Statistics ===\n");

        let mut lengths: Vec<usize> = Vec::new();
        let mut empty_count = 0;

        for section in &document.sections {
            for paragraph in &section.paragraphs {
                if paragraph.text.is_empty() {
                    empty_count += 1;
                } else {
                    lengths.push(paragraph.text.len());
                }
            }
        }

        if lengths.is_empty() {
            info.push_str("No non-empty paragraphs found\n");
            return info;
        }

        lengths.sort();

        let total: usize = lengths.iter().sum();
        let avg = total / lengths.len();
        let median = lengths[lengths.len() / 2];
        let min = lengths[0];
        let max = lengths[lengths.len() - 1];

        info.push_str(&format!("Non-empty paragraphs: {}\n", lengths.len()));
        info.push_str(&format!("Empty paragraphs: {}\n", empty_count));
        info.push_str(&format!("Average length: {} characters\n", avg));
        info.push_str(&format!("Median length: {} characters\n", median));
        info.push_str(&format!("Shortest paragraph: {} characters\n", min));
        info.push_str(&format!("Longest paragraph: {} characters\n", max));

        info
    }

    fn analyze_style_usage(&self, document: &HwpDocument) -> String {
        let mut info = String::new();
        info.push_str("\n=== Style Usage Analysis ===\n");

        // Count unique char shapes and para shapes used
        info.push_str(&format!(
            "Character shapes defined: {}\n",
            document.doc_info.char_shapes.len()
        ));
        info.push_str(&format!(
            "Paragraph shapes defined: {}\n",
            document.doc_info.para_shapes.len()
        ));
        info.push_str(&format!(
            "Styles defined: {}\n",
            document.doc_info.styles.len()
        ));

        // Show style distribution
        if !document.doc_info.styles.is_empty() {
            let para_styles = document
                .doc_info
                .styles
                .iter()
                .filter(|s| s.properties & 0x07 == 0)
                .count();
            let char_styles = document
                .doc_info
                .styles
                .iter()
                .filter(|s| s.properties & 0x07 == 1)
                .count();

            info.push_str(&format!("Paragraph styles: {}\n", para_styles));
            info.push_str(&format!("Character styles: {}\n", char_styles));
        }

        info
    }
}
