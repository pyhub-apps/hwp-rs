use anyhow::Result;
use clap::Args;
use hwp_parser::parse;
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ValidateCommand {
    /// Input HWP file path
    pub input: PathBuf,

    /// Strict validation mode
    #[arg(long)]
    pub strict: bool,

    /// Check file integrity
    #[arg(long)]
    pub check_integrity: bool,

    /// Verify document structure
    #[arg(long)]
    pub verify_structure: bool,

    /// Show performance metrics
    #[arg(long)]
    pub performance: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl ValidateCommand {
    pub fn execute(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Read the file
        let hwp_data = fs::read(&self.input)?;
        let file_size = hwp_data.len();

        println!("Validating: {}", self.input.display());
        println!(
            "File size: {} bytes ({:.2} MB)",
            file_size,
            file_size as f64 / 1_048_576.0
        );

        // Parse the document
        let parse_start = std::time::Instant::now();
        let document = match parse(&hwp_data) {
            Ok(doc) => {
                println!("✓ File parsing successful");
                doc
            }
            Err(e) => {
                println!("✗ File parsing failed: {}", e);
                if !self.strict {
                    return Err(e.into());
                }
                return Ok(());
            }
        };
        let parse_time = parse_start.elapsed();

        // Basic validation checks
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check header
        if document.header.version.major < 5 {
            warnings.push(format!("Old HWP version: {}", document.header.version));
        }

        if document.header.has_password() {
            warnings.push("Document is password protected".to_string());
        }

        if document.header.is_drm_document() {
            warnings.push("Document has DRM protection".to_string());
        }

        // Check document properties
        if document.doc_info.properties.section_count == 0 {
            errors.push("No sections found in document".to_string());
        }

        if document.doc_info.properties.section_count as usize != document.sections.len() {
            warnings.push(format!(
                "Section count mismatch: header says {}, found {}",
                document.doc_info.properties.section_count,
                document.sections.len()
            ));
        }

        // Check sections
        let mut total_paragraphs = 0;
        let mut empty_sections = 0;

        for (idx, section) in document.sections.iter().enumerate() {
            if section.paragraphs.is_empty() {
                empty_sections += 1;
                if self.verbose {
                    warnings.push(format!("Section {} is empty", idx));
                }
            }
            total_paragraphs += section.paragraphs.len();
        }

        if empty_sections > 0 && !self.verbose {
            warnings.push(format!("{} empty sections found", empty_sections));
        }

        // Check text extraction
        if self.verify_structure || self.strict {
            let text = document.get_text();
            if text.is_empty() && total_paragraphs > 0 {
                warnings.push("No text could be extracted despite having paragraphs".to_string());
            }

            if self.verbose {
                println!("\nDocument Statistics:");
                println!("  Sections: {}", document.sections.len());
                println!("  Paragraphs: {}", total_paragraphs);
                println!("  Text length: {} characters", text.chars().count());
                println!("  Fonts: {}", document.doc_info.face_names.len());
                println!("  Styles: {}", document.doc_info.styles.len());
            }
        }

        // Performance metrics
        if self.performance {
            let total_time = start_time.elapsed();
            println!("\nPerformance Metrics:");
            println!("  Parse time: {:.2}ms", parse_time.as_secs_f64() * 1000.0);
            println!("  Total time: {:.2}ms", total_time.as_secs_f64() * 1000.0);
            println!(
                "  Parse speed: {:.2} MB/s",
                file_size as f64 / 1_048_576.0 / parse_time.as_secs_f64()
            );
        }

        // Report results
        println!("\nValidation Results:");

        if errors.is_empty() && warnings.is_empty() {
            println!("✓ No issues found");
        } else {
            if !errors.is_empty() {
                println!("\nErrors ({}):", errors.len());
                for error in &errors {
                    println!("  ✗ {}", error);
                }
            }

            if !warnings.is_empty() {
                println!("\nWarnings ({}):", warnings.len());
                for warning in &warnings {
                    println!("  ⚠ {}", warning);
                }
            }
        }

        // Return error if strict mode and there are errors
        if self.strict && !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Validation failed with {} errors",
                errors.len()
            ));
        }

        Ok(())
    }
}
