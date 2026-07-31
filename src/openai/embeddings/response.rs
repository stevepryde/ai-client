use serde::Deserialize;

/// One vector in an OpenAI embeddings response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OpenAIEmbedding {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

/// Provider token usage for an embeddings request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct OpenAIEmbeddingUsage {
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

/// Decoded provider body returned by `POST /v1/embeddings`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OpenAIEmbeddingsCreateResponse {
    pub object: String,
    pub data: Vec<OpenAIEmbedding>,
    pub model: String,
    pub usage: OpenAIEmbeddingUsage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_vectors_indices_model_and_usage() {
        let response: OpenAIEmbeddingsCreateResponse = serde_json::from_value(serde_json::json!({
            "object": "list",
            "data": [
                {"object": "embedding", "embedding": [0.25, -0.5], "index": 1},
                {"object": "embedding", "embedding": [1.0, 0.0], "index": 0}
            ],
            "model": "text-embedding-3-large",
            "usage": {"prompt_tokens": 7, "total_tokens": 7}
        }))
        .unwrap();

        assert_eq!(response.model, "text-embedding-3-large");
        assert_eq!(response.data[0].index, 1);
        assert_eq!(response.data[0].embedding, vec![0.25, -0.5]);
        assert_eq!(response.usage.prompt_tokens, 7);
        assert_eq!(response.usage.total_tokens, 7);
    }
}
