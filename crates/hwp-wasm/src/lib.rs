use hwp_parser;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct HwpParser {
    document: Option<hwp_core::HwpDocument>,
}

#[wasm_bindgen]
impl HwpParser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> HwpParser {
        // Set panic hook for better error messages in browser console
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        HwpParser { document: None }
    }

    /// Parse HWP file from bytes
    #[wasm_bindgen]
    pub fn parse(&mut self, data: &[u8]) -> Result<(), JsValue> {
        match hwp_parser::parse(data) {
            Ok(doc) => {
                self.document = Some(doc);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
        }
    }

    /// Get document as JSON
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        match &self.document {
            Some(doc) => serde_json::to_string(doc)
                .map_err(|e| JsValue::from_str(&format!("JSON error: {}", e))),
            None => Err(JsValue::from_str("No document parsed yet")),
        }
    }

    /// Get document text content
    #[wasm_bindgen]
    pub fn get_text(&self) -> Result<String, JsValue> {
        match &self.document {
            Some(doc) => Ok(doc.get_text()),
            None => Err(JsValue::from_str("No document parsed yet")),
        }
    }
}
