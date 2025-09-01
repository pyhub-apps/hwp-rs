use crate::formatters::{FormatOptions, OutputFormatter};
use hwp_core::models::document::DocInfo;
use hwp_core::models::{Paragraph, Section};
use hwp_core::{HwpDocument, Result};

/// HTML formatter for HWP documents
pub struct HtmlFormatter {
    options: FormatOptions,
}

impl HtmlFormatter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    fn escape_html(text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#39;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
}

impl HtmlFormatter {
    fn get_default_css() -> &'static str {
        r#"
        body {
            font-family: 'Malgun Gothic', '맑은 고딕', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        
        .hwp-content {
            background-color: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        
        .hwp-section {
            margin-bottom: 30px;
        }
        
        .hwp-paragraph {
            margin-bottom: 1em;
            text-align: justify;
        }
        
        .hwp-metadata {
            background-color: #f9f9f9;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 30px;
        }
        
        .hwp-metadata h2 {
            color: #2c3e50;
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        
        .hwp-metadata dt {
            font-weight: bold;
            color: #34495e;
            float: left;
            width: 150px;
            clear: left;
            margin-bottom: 10px;
        }
        
        .hwp-metadata dd {
            margin-left: 160px;
            margin-bottom: 10px;
        }
        
        h1, h2, h3, h4, h5, h6 {
            color: #2c3e50;
            margin-top: 1.5em;
            margin-bottom: 0.5em;
        }
        
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 20px 0;
        }
        
        table, th, td {
            border: 1px solid #ddd;
        }
        
        th, td {
            padding: 12px;
            text-align: left;
        }
        
        th {
            background-color: #f2f2f2;
            font-weight: bold;
        }
        
        @media print {
            body {
                background-color: white;
            }
            
            .hwp-content {
                box-shadow: none;
                padding: 0;
            }
            
            .hwp-metadata {
                page-break-after: always;
            }
        }
        "#
    }
}

impl OutputFormatter for HtmlFormatter {
    fn format_document(&self, document: &HwpDocument) -> Result<String> {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"ko\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("    <title>HWP Document</title>\n");

        // Add CSS styles
        html.push_str("    <style>\n");
        html.push_str(HtmlFormatter::get_default_css());
        html.push_str("    </style>\n");

        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Add document metadata if requested
        if self.options.include_metadata {
            html.push_str(&self.format_metadata(&document.doc_info)?);
        }

        // Main content container
        html.push_str("    <div class=\"hwp-content\">\n");

        // Format sections
        for (idx, section) in document.sections.iter().enumerate() {
            html.push_str(&format!(
                "        <section class=\"hwp-section\" id=\"section-{}\">\n",
                idx
            ));

            // Format paragraphs
            for paragraph in &section.paragraphs {
                if !paragraph.text.is_empty() {
                    let escaped_text = Self::escape_html(&paragraph.text);

                    html.push_str(&format!(
                        "            <p class=\"hwp-paragraph\">{}</p>\n",
                        escaped_text
                    ));
                }
            }

            html.push_str("        </section>\n");
        }

        html.push_str("    </div>\n");

        // HTML footer
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    fn format_metadata(&self, doc_info: &DocInfo) -> Result<String> {
        let mut html = String::new();

        html.push_str("    <div class=\"hwp-metadata\">\n");
        html.push_str("        <h2>Document Information</h2>\n");
        html.push_str("        <dl>\n");

        // Document properties
        html.push_str(&format!(
            "            <dt>Sections</dt><dd>{}</dd>\n",
            doc_info.properties.section_count
        ));
        html.push_str(&format!(
            "            <dt>Pages</dt><dd>{}</dd>\n",
            doc_info.properties.total_page_count
        ));
        html.push_str(&format!(
            "            <dt>Characters</dt><dd>{}</dd>\n",
            doc_info.properties.total_character_count
        ));

        // Font information
        if !doc_info.face_names.is_empty() {
            html.push_str("            <dt>Fonts</dt><dd>\n");
            html.push_str("                <ul>\n");
            for face in &doc_info.face_names {
                html.push_str(&format!(
                    "                    <li>{}</li>\n",
                    Self::escape_html(&face.name)
                ));
            }
            html.push_str("                </ul>\n");
            html.push_str("            </dd>\n");
        }

        html.push_str("        </dl>\n");
        html.push_str("    </div>\n");

        Ok(html)
    }

    fn format_section(&self, section: &Section, index: usize) -> Result<String> {
        let mut html = String::new();
        html.push_str(&format!(
            "<section class=\"hwp-section\" id=\"section-{}\">\n",
            index
        ));

        for paragraph in &section.paragraphs {
            if !paragraph.text.is_empty() {
                let escaped_text = Self::escape_html(&paragraph.text);
                html.push_str(&format!(
                    "    <p class=\"hwp-paragraph\">{}</p>\n",
                    escaped_text
                ));
            }
        }

        html.push_str("</section>\n");
        Ok(html)
    }

    fn format_paragraph(&self, paragraph: &Paragraph, _index: usize) -> Result<String> {
        let escaped_text = Self::escape_html(&paragraph.text);
        Ok(format!("<p class=\"hwp-paragraph\">{}</p>\n", escaped_text))
    }
}
