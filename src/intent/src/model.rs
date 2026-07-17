//! Intent model — ML classification (future).

pub struct IntentModel;

impl IntentModel {
    pub fn new() -> Self {
        Self
    }

    pub async fn predict(&self, _features: Vec<f32>) -> anyhow::Result<crate::engine::Intent> {
        // TODO: Load ML model, predict intent
        anyhow::bail!("Not yet implemented")
    }
}