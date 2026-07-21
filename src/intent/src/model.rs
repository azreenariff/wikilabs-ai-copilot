//! Intent model — ML classification (future).

pub struct IntentModel;

impl IntentModel {
    pub fn new() -> Self {
        Self
    }

    pub async fn predict(&self, _features: Vec<f32>) -> anyhow::Result<crate::engine::Intent> {
        // Stub: placeholder. Load ML model and classify intent from features.
        anyhow::bail!("Not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_model_new() {
        let model = IntentModel::new();
        // just verify it constructs
    }

    #[test]
    fn test_predict_not_implemented() {
        let model = IntentModel::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(model.predict(vec![0.1, 0.2, 0.3]));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not yet implemented"));
    }
}
