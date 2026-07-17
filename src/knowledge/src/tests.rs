//! Tests for knowledge module: document types, search, embedding, import, dedup, quality.

use crate::dedup::DeduplicationEngine;
use crate::doc::{KnowledgeChunk, KnowledgeDocument};
use crate::embedding::{EmbeddingModel, EmbeddingResult};
use crate::import::ImportPipeline;
use crate::quality::{QualityScore, QualityScoringEngine};
use crate::search::{SearchEngine, SearchQuery, SearchResult};

mod doc_tests {
    use super::*;

    #[test]
    fn test_knowledge_document_creation() {
        let doc = KnowledgeDocument {
            id: uuid::Uuid::new_v4(),
            title: "Test Doc".to_string(),
            source: "manual".to_string(),
            workspace_id: uuid::Uuid::new_v4(),
            author: "tester".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(doc.title, "Test Doc");
        assert_eq!(doc.source, "manual");
    }

    #[test]
    fn test_knowledge_chunk_creation() {
        let chunk = KnowledgeChunk {
            id: uuid::Uuid::new_v4(),
            document_id: uuid::Uuid::new_v4(),
            content: "chunk content".to_string(),
            vector_id: "vec_001".to_string(),
        };
        assert_eq!(chunk.content, "chunk content");
        assert_eq!(chunk.vector_id, "vec_001");
    }
}

mod search_tests {
    use super::*;

    #[test]
    fn test_search_engine_new() {
        let engine = SearchEngine::new();
        // just verify it constructs
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery {
            text: "find me this".to_string(),
            workspace_id: uuid::Uuid::new_v4(),
        };
        assert_eq!(query.text, "find me this");
    }

    #[test]
    fn test_search_not_implemented() {
        let engine = SearchEngine::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let query = SearchQuery {
            text: "test".to_string(),
            workspace_id: uuid::Uuid::new_v4(),
        };
        let result = rt.block_on(engine.search(&query));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not yet implemented"));
    }
}

mod embedding_tests {
    use super::*;

    #[test]
    fn test_embedding_model_new() {
        let model = EmbeddingModel::new();
        assert_eq!(model.dimensions(), 384);
    }

    #[test]
    fn test_embedding_not_implemented() {
        let model = EmbeddingModel::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(model.embed("test text"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not yet implemented"));
    }

    #[test]
    fn test_embedding_result_defaults() {
        let result = EmbeddingResult {
            vector: vec![0.1, 0.2, 0.3],
            dimensions: 3,
        };
        assert_eq!(result.vector.len(), 3);
        assert_eq!(result.dimensions, 3);
    }
}

mod import_tests {
    use super::*;

    #[test]
    fn test_import_pipeline_new() {
        let pipeline = ImportPipeline::new();
        // just verify it constructs
    }

    #[test]
    fn test_import_file_not_implemented() {
        let pipeline = ImportPipeline::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(pipeline.import_file("/tmp/test.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not yet implemented"));
    }

    #[test]
    fn test_import_text_not_implemented() {
        let pipeline = ImportPipeline::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(pipeline.import_text("Title", "Some content"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not yet implemented"));
    }
}

mod dedup_tests {
    use super::*;

    #[test]
    fn test_dedup_engine_new() {
        let engine = DeduplicationEngine::new();
        // just verify it constructs
    }

    #[test]
    fn test_is_duplicate_not_implemented() {
        let engine = DeduplicationEngine::new();
        let result = engine.is_duplicate("test content");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not yet implemented"));
    }
}

mod quality_tests {
    use super::*;

    #[test]
    fn test_quality_engine_new() {
        let engine = QualityScoringEngine::new();
        // just verify it constructs
    }

    #[test]
    fn test_quality_score_default_values() {
        let score = QualityScore {
            source_authority: 0.5,
            freshness: 0.5,
            usage_signal: 0.5,
            user_feedback: 0.0,
        };
        assert_eq!(score.source_authority, 0.5);
        assert_eq!(score.freshness, 0.5);
        assert_eq!(score.usage_signal, 0.5);
        assert_eq!(score.user_feedback, 0.0);
    }

    #[test]
    fn test_quality_score_bounds() {
        let score = QualityScore {
            source_authority: 1.0,
            freshness: 0.0,
            usage_signal: 0.75,
            user_feedback: -1.0,
        };
        assert!(score.source_authority <= 1.0);
        assert!(score.freshness >= 0.0);
        assert!(score.user_feedback >= -1.0);
    }
}