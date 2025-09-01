use crate::formatters::{FormatOptions, OutputFormatter};
use hwp_core::models::document::DocInfo;
use hwp_core::models::{Paragraph, Section};
use hwp_core::{HwpDocument, HwpError, Result};
use serde_json::json;
use serde_yaml;

/// YAML formatter for HWP documents
pub struct YamlFormatter {
    options: FormatOptions,
}

impl YamlFormatter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    fn build_metadata(&self, doc_info: &DocInfo) -> Result<serde_json::Value> {
        let mut metadata = json!({
            "properties": {
                "section_count": doc_info.properties.section_count,
                "page_start_number": doc_info.properties.page_start_number,
                "page_count": doc_info.properties.total_page_count,
                "character_count": doc_info.properties.total_character_count,
            }
        });

        // Font information
        if !doc_info.face_names.is_empty() {
            let fonts: Vec<serde_json::Value> = doc_info
                .face_names
                .iter()
                .map(|face| {
                    let mut font_obj = json!({
                        "name": face.name,
                        "type": match face.properties & 0x07 {
                            0 => "representative",
                            1 => "truetype",
                            2 => "hft",
                            _ => "unknown",
                        }
                    });

                    if let Some(ref substitute) = face.substitute_font_name {
                        font_obj["substitute"] = json!(substitute);
                    }

                    font_obj
                })
                .collect();

            metadata["fonts"] = json!(fonts);
        }

        // Styles
        if !doc_info.styles.is_empty() {
            let styles: Vec<serde_json::Value> = doc_info
                .styles
                .iter()
                .map(|style| {
                    json!({
                        "name": style.name,
                        "english_name": style.english_name,
                        "type": match style.properties & 0x07 {
                            0 => "paragraph",
                            1 => "character",
                            _ => "unknown",
                        },
                        "char_shape_id": style.char_shape_id,
                        "para_shape_id": style.para_shape_id,
                    })
                })
                .collect();

            metadata["styles"] = json!(styles);
        }

        // Character shapes
        if self.options.include_styles && !doc_info.char_shapes.is_empty() {
            let char_shapes: Vec<serde_json::Value> = doc_info
                .char_shapes
                .iter()
                .take(5)
                .map(|shape| {
                    json!({
                        "font_ids": shape.face_name_ids,
                        "base_size": shape.base_size,
                        "text_color": format!("#{:06X}", shape.text_color & 0xFFFFFF),
                        "bold": (shape.properties & 0x0001) != 0,
                        "italic": (shape.properties & 0x0002) != 0,
                        "underline": (shape.properties & 0x0004) != 0,
                    })
                })
                .collect();

            metadata["character_shapes_sample"] = json!(char_shapes);
        }

        // Paragraph shapes
        if self.options.include_styles && !doc_info.para_shapes.is_empty() {
            let para_shapes: Vec<serde_json::Value> = doc_info
                .para_shapes
                .iter()
                .take(5)
                .map(|shape| {
                    json!({
                        "alignment": match shape.properties1 & 0x0007 {
                            0 => "justify",
                            1 => "left",
                            2 => "right",
                            3 => "center",
                            4 => "distribute",
                            5 => "distribute_space",
                            _ => "unknown",
                        },
                        "left_margin": shape.left_margin,
                        "right_margin": shape.right_margin,
                        "indent": shape.indent,
                        "line_spacing": shape.line_spacing,
                    })
                })
                .collect();

            metadata["paragraph_shapes_sample"] = json!(para_shapes);
        }

        Ok(metadata)
    }
}

impl OutputFormatter for YamlFormatter {
    fn format_document(&self, document: &HwpDocument) -> Result<String> {
        // Build document structure as JSON value first
        let mut doc_value = json!({
            "document": {
                "version": document.header.version.to_string(),
                "properties": {
                    "compressed": document.header.is_compressed(),
                    "has_password": document.header.has_password(),
                    "drm_protected": document.header.is_drm_document(),
                }
            }
        });

        // Add metadata if requested
        if self.options.include_metadata {
            let metadata = self.build_metadata(&document.doc_info)?;
            doc_value["metadata"] = metadata;
        }

        // Format sections
        let sections: Vec<serde_json::Value> = document
            .sections
            .iter()
            .enumerate()
            .map(|(idx, section)| {
                let paragraphs: Vec<serde_json::Value> = section
                    .paragraphs
                    .iter()
                    .map(|para| {
                        json!({
                            "text": para.text,
                        })
                    })
                    .collect();

                json!({
                    "index": idx,
                    "paragraph_count": section.paragraphs.len(),
                    "paragraphs": paragraphs,
                })
            })
            .collect();

        doc_value["sections"] = json!(sections);

        // Add statistics
        let total_paragraphs: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
        let total_characters: usize = document
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .map(|p| p.text.len())
            .sum();

        doc_value["statistics"] = json!({
            "section_count": document.sections.len(),
            "total_paragraphs": total_paragraphs,
            "total_characters": total_characters,
        });

        // Convert to YAML
        let yaml_string =
            serde_yaml::to_string(&doc_value).map_err(|e| HwpError::InvalidFormat {
                reason: e.to_string(),
            })?;

        Ok(yaml_string)
    }

    fn format_metadata(&self, doc_info: &DocInfo) -> Result<String> {
        let metadata = self.build_metadata(doc_info)?;
        let yaml_string =
            serde_yaml::to_string(&metadata).map_err(|e| HwpError::InvalidFormat {
                reason: e.to_string(),
            })?;
        Ok(yaml_string)
    }

    fn format_section(&self, section: &Section, index: usize) -> Result<String> {
        let section_value = json!({
            "section": {
                "index": index,
                "paragraphs": section.paragraphs.iter().map(|p| p.text.clone()).collect::<Vec<_>>(),
            }
        });

        serde_yaml::to_string(&section_value).map_err(|e| HwpError::InvalidFormat {
            reason: e.to_string(),
        })
    }

    fn format_paragraph(&self, paragraph: &Paragraph, index: usize) -> Result<String> {
        let para_value = json!({
            "paragraph": {
                "index": index,
                "text": paragraph.text,
            }
        });

        serde_yaml::to_string(&para_value).map_err(|e| HwpError::InvalidFormat {
            reason: e.to_string(),
        })
    }
}
