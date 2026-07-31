use crate::{
    error::{AiResponse, AiResult},
    openai::{client::decode_openai_error, OpenAIClient},
};

use super::{CreateEmbeddingRequest, OpenAIEmbeddingModel, OpenAIEmbeddingsCreateResponse};

/// Borrowed resource handle for OpenAI's embeddings API.
#[derive(Clone, Copy)]
pub struct EmbeddingsResource<'a> {
    client: &'a OpenAIClient,
}

impl<'a> EmbeddingsResource<'a> {
    pub(crate) fn new(client: &'a OpenAIClient) -> Self {
        Self { client }
    }

    /// Create one embedding or an ordered batch of embeddings.
    pub async fn create<M: OpenAIEmbeddingModel>(
        &self,
        request: CreateEmbeddingRequest<M>,
    ) -> AiResult<AiResponse<OpenAIEmbeddingsCreateResponse>> {
        self.client
            .transport()
            .post_json(
                "embeddings.create",
                "embeddings",
                &request,
                decode_openai_error,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::test_support::{json_response, one_shot_server},
        openai::{
            embeddings::{
                CreateEmbeddingRequest, EmbeddingDimensions, EmbeddingInput, TextEmbedding3Large,
            },
            OpenAIClient,
        },
    };

    fn request_json(request: &str) -> serde_json::Value {
        let (_, body) = request.split_once("\r\n\r\n").unwrap();
        serde_json::from_str(body).unwrap()
    }

    #[tokio::test]
    async fn create_posts_typed_request_and_preserves_response_metadata() {
        let body = r#"{
            "object":"list",
            "data":[
                {"object":"embedding","embedding":[0.1,0.2],"index":0},
                {"object":"embedding","embedding":[0.3,0.4],"index":1}
            ],
            "model":"text-embedding-3-large",
            "usage":{"prompt_tokens":4,"total_tokens":4}
        }"#;
        let response = json_response("200 OK", &[("x-request-id", "req_embeddings")], body);
        let (base_url, captured) = one_shot_server("v1", response).await;
        let client = OpenAIClient::builder()
            .api_key("test-key".into())
            .base_url(base_url)
            .build()
            .unwrap();
        let request = CreateEmbeddingRequest::<TextEmbedding3Large>::new(
            EmbeddingInput::batch(["first", "second"]).unwrap(),
        )
        .dimensions(EmbeddingDimensions::new(2).unwrap());

        let response = client.embeddings().create(request).await.unwrap();
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_embeddings")
        );
        assert_eq!(response.data().data.len(), 2);
        assert_eq!(response.data().data[1].embedding, vec![0.3, 0.4]);

        let wire = captured.await.unwrap();
        assert!(wire.starts_with("POST /v1/embeddings HTTP/1.1\r\n"));
        assert_eq!(
            request_json(&wire),
            serde_json::json!({
                "model": "text-embedding-3-large",
                "input": ["first", "second"],
                "encoding_format": "float",
                "dimensions": 2
            })
        );
    }
}
