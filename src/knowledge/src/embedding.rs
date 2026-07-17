//! Local embedding — all-MiniLM-L6-v2 via ONNX Runtime.

pub struct EmbeddingModel;

#[derive(Debug)]
pub struct EmbeddingResult {
    pub vector: Vec<f32>,
    pub dimensions: usize,
}

impl EmbeddingModel {
    pub fn new() -> Self {
        Self
    }

    pub async fn embed(&self, _text: &str) -> anyhow::Result<EmbeddingResult> {
        // TODO: Load ONNX model, run inference
        anyhow::bail!("Not yet implemented")
    }

    pub fn dimensions(&self) -> usize {
        384
    }
}