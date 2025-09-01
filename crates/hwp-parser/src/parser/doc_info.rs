use crate::parser::doc_info_records::*;
use crate::parser::record::RecordParser;
use crate::reader::ByteReader;
use crate::validator::RecordContext;
use hwp_core::constants::tag_id::doc_info;
use hwp_core::models::document::DocInfo;
use hwp_core::{HwpError, Result};

/// Parse the DocInfo section from decompressed data
pub fn parse_doc_info(data: &[u8]) -> Result<DocInfo> {
    let mut parser = RecordParser::new_with_context(data, RecordContext::DocInfo);
    let mut doc_info = DocInfo::default();

    // Parse all records in the DocInfo section
    while let Some(record) = parser.parse_next_record()? {
        match record.tag_id {
            doc_info::DOCUMENT_PROPERTIES => {
                doc_info.properties =
                    parse_document_properties(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse document properties: {}", e),
                    })?;
            }

            doc_info::FACE_NAME => {
                let face_name =
                    parse_face_name(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse face name: {}", e),
                    })?;
                doc_info.face_names.push(face_name);
            }

            doc_info::CHAR_SHAPE => {
                let char_shape =
                    parse_char_shape(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse character shape: {}", e),
                    })?;
                doc_info.char_shapes.push(char_shape);
            }

            doc_info::PARA_SHAPE => {
                let para_shape =
                    parse_para_shape(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse paragraph shape: {}", e),
                    })?;
                doc_info.para_shapes.push(para_shape);
            }

            doc_info::STYLE => {
                let style = parse_style(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse style: {}", e),
                })?;
                doc_info.styles.push(style);
            }

            doc_info::BORDER_FILL => {
                let border_fill =
                    parse_border_fill(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse border fill: {}", e),
                    })?;
                doc_info.border_fills.push(border_fill);
            }

            doc_info::ID_MAPPINGS => {
                // ID mappings are used internally for reference resolution
                let mappings =
                    parse_id_mappings(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse ID mappings: {}", e),
                    })?;
                doc_info.id_mappings = mappings;
            }

            doc_info::BIN_DATA => {
                // Binary data storage - typically images or embedded objects
                let bin_data = parse_bin_data(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse binary data: {}", e),
                })?;
                doc_info.bin_data_entries.push(bin_data);
            }

            doc_info::DOC_DATA => {
                // Document-specific data
                let doc_data = parse_doc_data(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse document data: {}", e),
                })?;
                doc_info.doc_data = doc_data;
            }

            doc_info::TAB_DEF => {
                let tab_def = parse_tab_def(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse tab definition: {}", e),
                })?;
                doc_info.tab_defs.push(tab_def);
            }

            doc_info::NUMBERING => {
                let numbering =
                    parse_numbering(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse numbering: {}", e),
                    })?;
                doc_info.numberings.push(numbering);
            }

            doc_info::BULLET => {
                let bullet = parse_bullet(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse bullet: {}", e),
                })?;
                doc_info.bullets.push(bullet);
            }

            doc_info::DISTRIBUTE_DOC_DATA => {
                let distribute_data =
                    parse_distribute_doc_data(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse distribute doc data: {}", e),
                    })?;
                doc_info.distribute_doc_data = Some(distribute_data);
            }

            doc_info::COMPATIBLE_DOCUMENT => {
                let compatible =
                    parse_compatible_document(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse compatible document: {}", e),
                    })?;
                doc_info.compatible_document = Some(compatible);
            }

            doc_info::LAYOUT_COMPATIBILITY => {
                let layout_compat =
                    parse_layout_compatibility(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse layout compatibility: {}", e),
                    })?;
                doc_info.layout_compatibility = Some(layout_compat);
            }

            doc_info::TRACK_CHANGE => {
                let track_change =
                    parse_track_change(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse track change: {}", e),
                    })?;
                doc_info.track_changes.push(track_change);
            }

            doc_info::TRACK_CHANGE_AUTHOR => {
                let author =
                    parse_track_change_author(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse track change author: {}", e),
                    })?;
                doc_info.track_change_authors.push(author);
            }

            doc_info::MEMO_SHAPE => {
                let memo = parse_memo_shape(&record.data).map_err(|e| HwpError::ParseError {
                    offset: 0,
                    message: format!("Failed to parse memo shape: {}", e),
                })?;
                doc_info.memo_shapes.push(memo);
            }

            doc_info::FORBIDDEN_CHAR => {
                let forbidden =
                    parse_forbidden_char(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse forbidden char: {}", e),
                    })?;
                doc_info.forbidden_chars = Some(forbidden);
            }

            // CHANGE_TRACKING is similar to TRACK_CHANGE, we can reuse the same parser
            doc_info::CHANGE_TRACKING => {
                let track_change =
                    parse_track_change(&record.data).map_err(|e| HwpError::ParseError {
                        offset: 0,
                        message: format!("Failed to parse change tracking: {}", e),
                    })?;
                doc_info.track_changes.push(track_change);
            }

            _ => {
                // Unknown record type - log and skip
                #[cfg(feature = "debug")]
                eprintln!(
                    "Unknown DocInfo record: tag_id=0x{:04X}, size={}",
                    record.tag_id, record.size
                );
            }
        }
    }

    Ok(doc_info)
}

/// Parse DocInfo from ByteReader (legacy interface)
pub fn parse_doc_info_legacy(_reader: &mut ByteReader) -> Result<DocInfo> {
    // This function is kept for backward compatibility
    // In a full implementation, this would read the entire stream
    // and delegate to parse_doc_info()
    Ok(DocInfo::default())
}
