use std::fs;
use std::io::Cursor;

use clap::{Arg, ArgAction, Command};
use hwp_parser::cfb::stream::Stream;
use hwp_parser::cfb::{parse_cfb_bytes, CfbContainer};

fn parse_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn dump_records(data: &[u8], start_offset: usize, max_count: usize) {
    let mut offset = start_offset.min(data.len());
    let mut count = 0usize;

    println!("Total bytes: {} | Start offset: {}", data.len(), offset);
    while offset + 4 <= data.len() && count < max_count {
        let header_bytes = &data[offset..offset + 4];
        let header_val = parse_u32_le(header_bytes);

        let tag_id = (header_val & 0x3FF) as u16;
        let level = ((header_val >> 10) & 0x3FF) as u8;
        let size_field = (header_val >> 20) & 0xFFF;

        let mut size = size_field as u32;
        let mut header_len = 4usize;

        if size_field == 0xFFF {
            if offset + 8 > data.len() {
                println!(
                    "{:04} | {:02X?} ... (truncated) -> invalid extended size",
                    offset, &header_bytes
                );
                break;
            }
            let size_bytes = &data[offset + 4..offset + 8];
            size = parse_u32_le(size_bytes);
            header_len = 8;
        }

        println!(
            "{:05} | {:02X?}{} -> tag=0x{:04X}, level={}, size={}",
            offset,
            &header_bytes,
            if header_len == 8 {
                format!(" + {:02X?}", &data[offset + 4..offset + 8])
            } else {
                String::new()
            },
            tag_id,
            level,
            size
        );

        let total = header_len + (size as usize);
        if offset + total > data.len() {
            println!(
                "  \u{2514}\u{2500} Data truncated: need {}, have {}",
                total,
                data.len() - offset
            );
            break;
        }

        offset += total;
        count += 1;
    }

    println!("\nParsed {} record headers. Next offset: {}", count, offset);
}

fn read_stream_data(
    container: &mut CfbContainer,
    cursor: &mut Cursor<&[u8]>,
    name: &str,
    raw: bool,
) -> anyhow::Result<Vec<u8>> {
    let stream = container.read_stream(cursor, name)?;
    if raw {
        Ok(stream.as_bytes().to_vec())
    } else if stream.is_compressed() {
        Ok(stream.decompress()?)
    } else {
        Ok(stream.as_bytes().to_vec())
    }
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("dump_records")
        .about("Dump HWP record headers from a CFB stream (DocInfo/BodyText)")
        .arg(Arg::new("file").required(true).help("Path to .hwp file"))
        .arg(
            Arg::new("stream")
                .short('s')
                .long("stream")
                .value_name("NAME")
                .default_value("DocInfo")
                .help("Stream name (e.g., DocInfo, BodyText/Section0)"),
        )
        .arg(
            Arg::new("count")
                .short('n')
                .long("count")
                .value_parser(clap::value_parser!(usize))
                .default_value("10")
                .help("Max number of records to dump"),
        )
        .arg(
            Arg::new("offset")
                .short('o')
                .long("offset")
                .value_parser(clap::value_parser!(usize))
                .default_value("0")
                .help("Start offset (in bytes) within decompressed stream"),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .action(ArgAction::SetTrue)
                .help("List available streams and exit"),
        )
        .arg(
            Arg::new("raw")
                .long("raw")
                .action(ArgAction::SetTrue)
                .help("Do not decompress stream; dump from raw bytes"),
        )
        .get_matches();

    let path = matches.get_one::<String>("file").unwrap();
    let stream_name = matches.get_one::<String>("stream").unwrap();
    let count = *matches.get_one::<usize>("count").unwrap();
    let start_offset = *matches.get_one::<usize>("offset").unwrap();
    let list_only = matches.get_flag("list");
    let raw = matches.get_flag("raw");

    let bytes = fs::read(path)?;

    // Parse CFB container
    let mut container = parse_cfb_bytes(&bytes)?;
    let mut cursor = Cursor::new(bytes.as_slice());

    if list_only {
        println!("Available streams:");
        for name in container.list_streams() {
            println!("  - {}", name);
        }
        return Ok(());
    }

    if !container.has_stream(stream_name) {
        anyhow::bail!("Stream '{}' not found", stream_name);
    }

    let data = read_stream_data(&mut container, &mut cursor, stream_name, raw)?;

    println!(
        "Dumping first {} record headers from stream '{}'{}:",
        count,
        stream_name,
        if raw { " (raw)" } else { "" }
    );
    dump_records(&data, start_offset, count);

    Ok(())
}
