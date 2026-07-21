//! OCR engine — Tesseract-based text extraction.

pub struct OCREngine;

impl OCREngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, _image_path: &str) -> anyhow::Result<String> {
        // Stub: placeholder. Implement Tesseract OCR text extraction.
        unimplemented!()
    }
}