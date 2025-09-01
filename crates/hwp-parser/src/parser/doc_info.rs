use crate::parser::record::RecordParser;
use crate::parser::doc_info_records::*;
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
                doc_info.properties = parse_document_properties(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse document properties: {}", e) })?;
            }
            
            doc_info::FACE_NAME => {
                let face_name = parse_face_name(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse face name: {}", e) })?;
                doc_info.face_names.push(face_name);
            }
            
            doc_info::CHAR_SHAPE => {
                let char_shape = parse_char_shape(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse character shape: {}", e) })?;
                doc_info.char_shapes.push(char_shape);
            }
            
            doc_info::PARA_SHAPE => {
                let para_shape = parse_para_shape(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse paragraph shape: {}", e) })?;
                doc_info.para_shapes.push(para_shape);
            }
            
            doc_info::STYLE => {
                let style = parse_style(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse style: {}", e) })?;
                doc_info.styles.push(style);
            }
            
            doc_info::BORDER_FILL => {
                let border_fill = parse_border_fill(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse border fill: {}", e) })?;
                doc_info.border_fills.push(border_fill);
            }
            
            doc_info::ID_MAPPINGS => {
                // ID mappings are used internally for reference resolution
                let mappings = parse_id_mappings(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse ID mappings: {}", e) })?;
                doc_info.id_mappings = mappings;
            }
            
            doc_info::BIN_DATA => {
                // Binary data storage - typically images or embedded objects
                let bin_data = parse_bin_data(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse binary data: {}", e) })?;
                doc_info.bin_data_entries.push(bin_data);
            }
            
            doc_info::DOC_DATA => {
                // Document-specific data
                let doc_data = parse_doc_data(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse document data: {}", e) })?;
                doc_info.doc_data = doc_data;
            }
            
            doc_info::TAB_DEF => {
                let tab_def = parse_tab_def(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse tab definition: {}", e) })?;
                doc_info.tab_defs.push(tab_def);
            }
            
            doc_info::NUMBERING => {
                let numbering = parse_numbering(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse numbering: {}", e) })?;
                doc_info.numberings.push(numbering);
            }
            
            doc_info::BULLET => {
                let bullet = parse_bullet(&record.data)
                    .map_err(|e| HwpError::ParseError { offset: 0, message: format!("Failed to parse bullet: {}", e) })?;
                doc_info.bullets.push(bullet);
            }
            
            // Handle other record types
            doc_info::DISTRIBUTE_DOC_DATA | doc_info::COMPATIBLE_DOCUMENT |
            doc_info::LAYOUT_COMPATIBILITY | doc_info::TRACK_CHANGE |
            doc_info::MEMO_SHAPE | doc_info::FORBIDDEN_CHAR |
            doc_info::TRACK_CHANGE_AUTHOR | doc_info::CHANGE_TRACKING => {
                // These records are parsed but not yet fully implemented
                // For now, we skip them with a debug message
                #[cfg(feature = "debug")]
                eprintln!("Skipping unimplemented DocInfo record: tag_id=0x{:04X}, size={}", 
                         record.tag_id, record.size);
            }
            
            _ => {
                // Unknown record type - log and skip
                #[cfg(feature = "debug")]
                eprintln!("Unknown DocInfo record: tag_id=0x{:04X}, size={}", 
                         record.tag_id, record.size);
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