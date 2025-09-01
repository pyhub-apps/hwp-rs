use crate::batch::{BatchProcessor, ErrorStrategy};
use crate::error::CliError;
use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use hwp_core::HwpDocument;
use hwp_parser::parse;
use regex::Regex;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Search result for a single file
#[derive(Debug)]
pub struct SearchMatch {
    pub file: PathBuf,
    pub section: usize,
    pub paragraph: usize,
    pub line: usize,
    pub text: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// Search command arguments
#[derive(Args, Debug)]
pub struct SearchCommand {
    /// Search query (text or regex pattern)
    pub query: String,

    /// Input paths (files or directories)
    pub paths: Vec<PathBuf>,

    /// Use regular expression matching
    #[arg(short = 'e', long)]
    pub regex: bool,

    /// Case-sensitive search
    #[arg(short = 'i', long)]
    pub case_insensitive: bool,

    /// Search recursively in directories
    #[arg(short, long)]
    pub recursive: bool,

    /// Context lines before match
    #[arg(short = 'B', long, default_value = "0")]
    pub before_context: usize,

    /// Context lines after match
    #[arg(short = 'A', long, default_value = "0")]
    pub after_context: usize,

    /// Context lines (before and after)
    #[arg(short = 'C', long)]
    pub context: Option<usize>,

    /// Maximum results to show
    #[arg(long, default_value = "100")]
    pub max_results: usize,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Number of parallel jobs
    #[arg(short = 'j', long, default_value = "4")]
    pub parallel: usize,

    /// Show only file names with matches
    #[arg(short = 'l', long)]
    pub files_with_matches: bool,

    /// Show match count per file
    #[arg(short = 'c', long)]
    pub count: bool,

    /// Invert match (show non-matching lines)
    #[arg(short = 'v', long)]
    pub invert_match: bool,
}

impl SearchCommand {
    pub fn execute(&self) -> Result<()> {
        // Determine context lines
        let before = self.context.unwrap_or(self.before_context);
        let after = self.context.unwrap_or(self.after_context);

        // Create search pattern
        let pattern = self.create_pattern()?;

        // Discover files to search
        let files = self.discover_files()?;
        if files.is_empty() {
            return Err(CliError::NoFilesFound {
                pattern: self
                    .paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            }
            .into());
        }

        eprintln!("Searching {} files for '{}'...", files.len(), self.query);

        // Perform search
        let batch_processor = BatchProcessor::new(self.parallel, ErrorStrategy::Skip);
        let mut all_matches = Vec::new();
        let mut matched_files = 0;

        for file in &files {
            match self.search_file(file, &pattern, before, after) {
                Ok(matches) if !matches.is_empty() => {
                    matched_files += 1;

                    if self.files_with_matches {
                        println!("{}", file.display());
                    } else if self.count {
                        println!("{}:{}", file.display(), matches.len());
                    } else {
                        all_matches.extend(matches);
                    }

                    if all_matches.len() >= self.max_results {
                        break;
                    }
                }
                Ok(_) => {} // No matches
                Err(e) => eprintln!("Error searching {}: {}", file.display(), e),
            }
        }

        // Output results
        if !self.files_with_matches && !self.count {
            self.output_results(&all_matches)?;
        }

        // Summary
        eprintln!(
            "\nFound {} matches in {} files (searched {} files)",
            all_matches.len(),
            matched_files,
            files.len()
        );

        Ok(())
    }

    fn create_pattern(&self) -> Result<Regex> {
        let pattern = if self.regex {
            self.query.clone()
        } else {
            regex::escape(&self.query)
        };

        let pattern = if self.case_insensitive {
            format!("(?i){}", pattern)
        } else {
            pattern
        };

        Regex::new(&pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))
    }

    fn discover_files(&self) -> Result<Vec<PathBuf>> {
        let mut all_files = Vec::new();
        let batch_processor = BatchProcessor::new(self.parallel, ErrorStrategy::Skip);

        for path in &self.paths {
            if path.is_file() {
                if path.extension().map_or(false, |ext| ext == "hwp") {
                    all_files.push(path.clone());
                }
            } else if path.is_dir() {
                let files = batch_processor.discover_files(path, self.recursive)?;
                all_files.extend(files);
            } else {
                // Try as glob pattern
                let files = batch_processor.discover_glob(&path.display().to_string())?;
                all_files.extend(files);
            }
        }

        Ok(all_files)
    }

    fn search_file(
        &self,
        file: &Path,
        pattern: &Regex,
        before: usize,
        after: usize,
    ) -> Result<Vec<SearchMatch>> {
        let hwp_data = fs::read(file)?;
        let document = parse(&hwp_data)?;

        let mut matches = Vec::new();

        for (section_idx, section) in document.sections.iter().enumerate() {
            for (para_idx, paragraph) in section.paragraphs.iter().enumerate() {
                let text = &paragraph.text;
                let lines: Vec<&str> = text.lines().collect();

                for (line_idx, line) in lines.iter().enumerate() {
                    let is_match = if self.invert_match {
                        !pattern.is_match(line)
                    } else {
                        pattern.is_match(line)
                    };

                    if is_match {
                        // Collect context
                        let start = if line_idx >= before {
                            line_idx - before
                        } else {
                            0
                        };
                        let end = std::cmp::min(line_idx + after + 1, lines.len());

                        let context_before = lines[start..line_idx]
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                        let context_after = lines[(line_idx + 1)..end]
                            .iter()
                            .map(|s| s.to_string())
                            .collect();

                        matches.push(SearchMatch {
                            file: file.to_path_buf(),
                            section: section_idx,
                            paragraph: para_idx,
                            line: line_idx,
                            text: line.to_string(),
                            context_before,
                            context_after,
                        });

                        if matches.len() >= self.max_results {
                            return Ok(matches);
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    fn output_results(&self, matches: &[SearchMatch]) -> Result<()> {
        let output = match self.format.as_str() {
            "json" => self.format_json(matches)?,
            _ => self.format_text(matches)?,
        };

        if let Some(output_path) = &self.output {
            let mut file = fs::File::create(output_path)?;
            file.write_all(output.as_bytes())?;
            eprintln!("Results written to: {}", output_path.display());
        } else {
            print!("{}", output);
        }

        Ok(())
    }

    fn format_text(&self, matches: &[SearchMatch]) -> Result<String> {
        let mut output = String::new();
        let mut last_file = None;

        for match_item in matches {
            // Print file header if changed
            if last_file != Some(&match_item.file) {
                output.push_str(&format!(
                    "\n{}\n",
                    match_item.file.display().to_string().blue().bold()
                ));
                last_file = Some(&match_item.file);
            }

            // Print context before
            for line in &match_item.context_before {
                output.push_str(&format!("  {}\n", line.dimmed()));
            }

            // Print matching line with highlighting
            let highlighted = if self.regex {
                let pattern = self.create_pattern().unwrap();
                pattern
                    .replace_all(&match_item.text, |caps: &regex::Captures| {
                        caps[0].red().bold().to_string()
                    })
                    .to_string()
            } else {
                match_item
                    .text
                    .replace(&self.query, &self.query.red().bold().to_string())
            };

            output.push_str(&format!(
                "{}:{}:{}: {}\n",
                match_item.section.to_string().yellow(),
                match_item.paragraph.to_string().yellow(),
                match_item.line.to_string().yellow(),
                highlighted
            ));

            // Print context after
            for line in &match_item.context_after {
                output.push_str(&format!("  {}\n", line.dimmed()));
            }
        }

        Ok(output)
    }

    fn format_json(&self, matches: &[SearchMatch]) -> Result<String> {
        let json_matches: Vec<_> = matches
            .iter()
            .map(|m| {
                json!({
                    "file": m.file.display().to_string(),
                    "section": m.section,
                    "paragraph": m.paragraph,
                    "line": m.line,
                    "text": m.text,
                    "context_before": m.context_before,
                    "context_after": m.context_after,
                })
            })
            .collect();

        let result = json!({
            "query": self.query,
            "total_matches": matches.len(),
            "matches": json_matches,
        });

        Ok(serde_json::to_string_pretty(&result)?)
    }
}
