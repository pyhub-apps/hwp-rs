use anyhow::Result;
use clap::Args;
use hwp_parser::{parse, OutputFormat, FormatOptions, MarkdownFlavor};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use glob::glob;

#[derive(Args, Debug)]
pub struct ConvertCommand {
    /// Input HWP file path or pattern (supports wildcards)
    pub input: String,
    
    /// Output format (text, json, markdown)
    #[arg(short = 't', long = "to", default_value = "text")]
    pub format: String,
    
    /// Output file or directory path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Output directory for batch conversion
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    
    /// Process directories recursively
    #[arg(short, long)]
    pub recursive: bool,
    
    // Format-specific options
    
    /// Pretty print JSON
    #[arg(long)]
    pub json_pretty: bool,
    
    /// Include styles in JSON output
    #[arg(long)]
    pub json_include_styles: bool,
    
    /// Line wrap width for text output
    #[arg(long)]
    pub text_width: Option<usize>,
    
    /// Preserve page breaks in text
    #[arg(long)]
    pub text_page_breaks: bool,
    
    /// Markdown flavor (commonmark, gfm, multimarkdown)
    #[arg(long, default_value = "commonmark")]
    pub markdown_flavor: String,
    
    /// Generate table of contents for Markdown
    #[arg(long)]
    pub markdown_toc: bool,
    
    /// Overwrite existing files
    #[arg(long)]
    pub overwrite: bool,
}

impl ConvertCommand {
    pub fn execute(&self) -> Result<()> {
        // Check if input is a pattern or single file
        if self.input.contains('*') || self.input.contains('?') {
            // Batch conversion with glob pattern
            self.batch_convert()?;
        } else {
            // Single file conversion
            let input_path = PathBuf::from(&self.input);
            if input_path.is_dir() {
                // Directory batch conversion
                self.convert_directory(&input_path)?;
            } else {
                // Single file
                self.convert_file(&input_path, self.output.as_ref())?;
            }
        }
        
        Ok(())
    }
    
    fn batch_convert(&self) -> Result<()> {
        let pattern = if self.recursive {
            format!("**/{}", self.input)
        } else {
            self.input.clone()
        };
        
        let mut count = 0;
        for entry in glob(&pattern)? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        let output_path = self.get_output_path(&path)?;
                        if let Err(e) = self.convert_file(&path, Some(&output_path)) {
                            eprintln!("Error converting {}: {}", path.display(), e);
                        } else {
                            count += 1;
                        }
                    }
                }
                Err(e) => eprintln!("Error reading path: {}", e),
            }
        }
        
        eprintln!("Converted {} files", count);
        Ok(())
    }
    
    fn convert_directory(&self, dir: &Path) -> Result<()> {
        let pattern = if self.recursive {
            format!("{}/**/*.hwp", dir.display())
        } else {
            format!("{}/*.hwp", dir.display())
        };
        
        let mut count = 0;
        for entry in glob(&pattern)? {
            match entry {
                Ok(path) => {
                    let output_path = self.get_output_path(&path)?;
                    if let Err(e) = self.convert_file(&path, Some(&output_path)) {
                        eprintln!("Error converting {}: {}", path.display(), e);
                    } else {
                        count += 1;
                    }
                }
                Err(e) => eprintln!("Error reading path: {}", e),
            }
        }
        
        eprintln!("Converted {} files", count);
        Ok(())
    }
    
    fn convert_file(&self, input_path: &Path, output_path: Option<&PathBuf>) -> Result<()> {
        // Check if output file exists and overwrite is not set
        if let Some(out_path) = output_path {
            if out_path.exists() && !self.overwrite {
                eprintln!("Skipping {}: output file exists (use --overwrite to replace)", 
                    input_path.display());
                return Ok(());
            }
        }
        
        eprintln!("Converting: {}", input_path.display());
        
        // Read and parse the HWP file
        let hwp_data = fs::read(input_path)?;
        let document = parse(&hwp_data)?;
        
        // Build format options
        let mut options = FormatOptions::default();
        options.json_pretty = self.json_pretty;
        options.json_include_styles = self.json_include_styles;
        options.text_width = self.text_width;
        options.text_page_breaks = self.text_page_breaks;
        options.markdown_toc = self.markdown_toc;
        options.markdown_flavor = match self.markdown_flavor.to_lowercase().as_str() {
            "gfm" | "github" => MarkdownFlavor::GitHubFlavored,
            "multimarkdown" | "mmd" => MarkdownFlavor::MultiMarkdown,
            _ => MarkdownFlavor::CommonMark,
        };
        
        // Get the output format
        let format = match self.format.to_lowercase().as_str() {
            "text" | "txt" => OutputFormat::PlainText,
            "json" => OutputFormat::Json,
            "markdown" | "md" => OutputFormat::Markdown,
            _ => {
                return Err(anyhow::anyhow!("Unsupported format: {}", self.format));
            }
        };
        
        // Convert the document
        let formatter = format.create_formatter(options);
        let output = formatter.format_document(&document)?;
        
        // Write output
        if let Some(out_path) = output_path {
            // Create parent directory if needed
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let mut file = fs::File::create(out_path)?;
            file.write_all(output.as_bytes())?;
            eprintln!("  -> {}", out_path.display());
        } else {
            print!("{}", output);
        }
        
        Ok(())
    }
    
    fn get_output_path(&self, input_path: &Path) -> Result<PathBuf> {
        // Determine the output extension
        let extension = match self.format.to_lowercase().as_str() {
            "text" | "txt" => "txt",
            "json" => "json",
            "markdown" | "md" => "md",
            _ => "txt",
        };
        
        if let Some(output_dir) = &self.output_dir {
            // Use specified output directory
            let file_stem = input_path.file_stem()
                .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?;
            Ok(output_dir.join(format!("{}.{}", file_stem.to_string_lossy(), extension)))
        } else if let Some(output) = &self.output {
            // Use specified output path
            if output.is_dir() {
                // If output is a directory, generate filename
                let file_stem = input_path.file_stem()
                    .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?;
                Ok(output.join(format!("{}.{}", file_stem.to_string_lossy(), extension)))
            } else {
                Ok(output.clone())
            }
        } else {
            // Generate output path next to input file
            let mut output_path = input_path.to_path_buf();
            output_path.set_extension(extension);
            Ok(output_path)
        }
    }
}