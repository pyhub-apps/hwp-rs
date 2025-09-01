use clap::{Arg, Command};
use hwp_parser::{parse, FormatOptions, MarkdownFlavor, OutputFormat};
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("hwp-convert")
        .about("Convert HWP files to various output formats")
        .arg(
            Arg::new("input")
                .required(true)
                .help("Input HWP file path")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path (default: stdout)"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .default_value("json")
                .help("Output format: json, text, markdown"),
        )
        // JSON options
        .arg(
            Arg::new("json-pretty")
                .long("json-pretty")
                .action(clap::ArgAction::SetTrue)
                .help("Pretty print JSON with indentation"),
        )
        .arg(
            Arg::new("json-compact")
                .long("json-compact")
                .action(clap::ArgAction::SetTrue)
                .help("Minimize JSON size (opposite of --json-pretty)"),
        )
        .arg(
            Arg::new("json-include-styles")
                .long("json-include-styles")
                .action(clap::ArgAction::SetTrue)
                .help("Include style definitions in JSON output"),
        )
        // Text options
        .arg(
            Arg::new("text-width")
                .long("text-width")
                .value_name("WIDTH")
                .value_parser(clap::value_parser!(usize))
                .help("Line wrap width for plain text"),
        )
        .arg(
            Arg::new("text-page-breaks")
                .long("text-page-breaks")
                .action(clap::ArgAction::SetTrue)
                .help("Preserve page breaks in plain text"),
        )
        // Markdown options
        .arg(
            Arg::new("md-flavor")
                .long("md-flavor")
                .value_name("FLAVOR")
                .default_value("commonmark")
                .help("Markdown flavor: commonmark, gfm, multimarkdown"),
        )
        .arg(
            Arg::new("md-toc")
                .long("md-toc")
                .action(clap::ArgAction::SetTrue)
                .help("Generate table of contents for Markdown"),
        )
        .get_matches();

    // Parse arguments
    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output");
    let format_str = matches.get_one::<String>("format").unwrap();

    // Parse output format
    let output_format = OutputFormat::from_str(format_str)
        .ok_or_else(|| format!("Unknown format: {}", format_str))?;

    // Build format options
    let mut options = FormatOptions::default();

    // JSON options
    if matches.get_flag("json-compact") {
        options.json_pretty = false;
    } else if matches.get_flag("json-pretty") {
        options.json_pretty = true;
    }
    options.json_include_styles = matches.get_flag("json-include-styles");

    // Text options
    if let Some(width) = matches.get_one::<usize>("text-width") {
        options.text_width = Some(*width);
    }
    options.text_page_breaks = matches.get_flag("text-page-breaks");

    // Markdown options
    if let Some(flavor) = matches.get_one::<String>("md-flavor") {
        options.markdown_flavor = match flavor.to_lowercase().as_str() {
            "gfm" | "github" => MarkdownFlavor::GitHubFlavored,
            "multimarkdown" | "mmd" => MarkdownFlavor::MultiMarkdown,
            _ => MarkdownFlavor::CommonMark,
        };
    }
    options.markdown_toc = matches.get_flag("md-toc");

    // Read and parse HWP file
    println!("Reading HWP file: {}", input_path);
    let hwp_data = fs::read(input_path)?;

    println!("Parsing HWP document...");
    let document = parse(&hwp_data)?;

    println!("Converting to {} format...", format_str);

    // Create formatter and convert
    let formatter = output_format.create_formatter(options);
    let output = formatter.format_document(&document)?;

    // Write output
    if let Some(output_file) = output_path {
        println!("Writing output to: {}", output_file);
        let mut file = fs::File::create(output_file)?;
        file.write_all(output.as_bytes())?;
        println!("Conversion complete!");
    } else {
        // Write to stdout
        print!("{}", output);
    }

    Ok(())
}
