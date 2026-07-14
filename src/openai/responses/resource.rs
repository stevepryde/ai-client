//! Borrowed resource handle for the OpenAI Responses API.

#[cfg(feature = "stream")]
use crate::{
    core::sse,
    error::AiProvider,
    stream::{AiStream, SseJsonEvent},
};
use crate::{
    error::{AiResponse, AiResult},
    openai::{client::decode_openai_error, OpenAIClient},
};

#[cfg(feature = "stream")]
use super::CreateResponseStreamOptions;
use super::{
    ListResponseInputItemsOptions, OpenAICompactRequest, OpenAICompactResponse,
    OpenAIInputTokenCountRequest, OpenAIInputTokenCountResponse, OpenAIResponseItemList,
    OpenAIResponsesCreateResponse, PreparedResponseRequest, ResponseId, ResponseModelFor,
    RetrieveResponseOptions,
};
#[cfg(feature = "stream")]
use super::{OpenAIResponsesStreamEvent, RetrieveResponseStreamOptions};

#[derive(Clone, Copy)]
pub struct ResponsesResource<'a> {
    client: &'a OpenAIClient,
}

impl<'a> ResponsesResource<'a> {
    pub(crate) fn new(client: &'a OpenAIClient) -> Self {
        Self { client }
    }

    /// Create a response from a typed model config and reusable request.
    pub async fn create<Model, Request>(
        &self,
        model: Model,
        request: Request,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>>
    where
        Model: ResponseModelFor<Request>,
    {
        self.create_prepared(model.prepare(request)).await
    }

    /// Send an already prepared request through the low-level migration path.
    #[doc(hidden)]
    pub async fn create_prepared(
        &self,
        mut request: PreparedResponseRequest,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>> {
        request.wire_mut().stream = None;
        request.wire_mut().stream_options = None;
        self.client
            .transport()
            .post_json(
                "responses.create",
                "responses",
                &request,
                decode_openai_error,
            )
            .await
    }

    /// Create a response and stream its server-sent events.
    #[cfg(feature = "stream")]
    pub async fn create_stream<Model, Request>(
        &self,
        model: Model,
        request: Request,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<OpenAIResponsesStreamEvent>>>>
    where
        Model: ResponseModelFor<Request>,
    {
        self.create_stream_with_options(model, request, &CreateResponseStreamOptions::default())
            .await
    }

    /// Create and stream with transport-owned SSE options.
    #[cfg(feature = "stream")]
    pub async fn create_stream_with_options<Model, Request>(
        &self,
        model: Model,
        request: Request,
        options: &CreateResponseStreamOptions,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<OpenAIResponsesStreamEvent>>>>
    where
        Model: ResponseModelFor<Request>,
    {
        self.create_prepared_stream_with_options(model.prepare(request), options)
            .await
    }

    /// Stream an already prepared request through the low-level migration path.
    #[cfg(feature = "stream")]
    #[doc(hidden)]
    pub async fn create_prepared_stream(
        &self,
        request: PreparedResponseRequest,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<OpenAIResponsesStreamEvent>>>> {
        self.create_prepared_stream_with_options(request, &CreateResponseStreamOptions::default())
            .await
    }

    /// Stream a prepared request with transport-owned SSE options.
    #[cfg(feature = "stream")]
    #[doc(hidden)]
    pub async fn create_prepared_stream_with_options(
        &self,
        mut request: PreparedResponseRequest,
        options: &CreateResponseStreamOptions,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<OpenAIResponsesStreamEvent>>>> {
        request.wire_mut().stream = Some(true);
        request.wire_mut().stream_options = (!options.is_empty()).then(|| options.clone());
        let response = self
            .client
            .transport()
            .post_json_stream(
                "responses.create_stream",
                "responses",
                &request,
                decode_openai_error,
            )
            .await?;
        let (bytes, metadata) = response.into_parts();
        Ok(AiResponse::new(
            sse::json_events(bytes, AiProvider::OpenAI, "responses.create_stream"),
            metadata,
        ))
    }

    /// Retrieve a stored response without additional included fields.
    pub async fn retrieve(
        &self,
        response_id: &ResponseId,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>> {
        self.retrieve_with_options(response_id, &RetrieveResponseOptions::default())
            .await
    }

    /// Retrieve a stored response with explicit include selectors.
    pub async fn retrieve_with_options(
        &self,
        response_id: &ResponseId,
        options: &RetrieveResponseOptions,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>> {
        self.client
            .transport()
            .get_json_segments(
                "responses.retrieve",
                &["responses", response_id.as_str()],
                &options.query(),
                decode_openai_error,
            )
            .await
    }

    /// Retrieve a stored response as a stream, optionally resuming by sequence number.
    #[cfg(feature = "stream")]
    pub async fn retrieve_stream(
        &self,
        response_id: &ResponseId,
        options: &RetrieveResponseStreamOptions,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<OpenAIResponsesStreamEvent>>>> {
        let response = self
            .client
            .transport()
            .get_json_stream_segments(
                "responses.retrieve_stream",
                &["responses", response_id.as_str()],
                &options.query(),
                decode_openai_error,
            )
            .await?;
        let (bytes, metadata) = response.into_parts();
        Ok(AiResponse::new(
            sse::json_events(bytes, AiProvider::OpenAI, "responses.retrieve_stream"),
            metadata,
        ))
    }

    /// Delete a stored response. The pinned operation returns an empty success body.
    pub async fn delete(&self, response_id: &ResponseId) -> AiResult<AiResponse<()>> {
        self.client
            .transport()
            .delete_empty_segments(
                "responses.delete",
                &["responses", response_id.as_str()],
                decode_openai_error,
            )
            .await
    }

    /// Cancel an in-progress background response.
    pub async fn cancel(
        &self,
        response_id: &ResponseId,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>> {
        self.client
            .transport()
            .post_empty_segments(
                "responses.cancel",
                &["responses", response_id.as_str(), "cancel"],
                decode_openai_error,
            )
            .await
    }

    /// List the input items retained for a response.
    pub async fn list_input_items(
        &self,
        response_id: &ResponseId,
        options: &ListResponseInputItemsOptions,
    ) -> AiResult<AiResponse<OpenAIResponseItemList>> {
        self.client
            .transport()
            .get_json_segments(
                "responses.input_items.list",
                &["responses", response_id.as_str(), "input_items"],
                &options.query(),
                decode_openai_error,
            )
            .await
    }

    /// Count input tokens without creating a response.
    pub async fn count_input_tokens(
        &self,
        request: &OpenAIInputTokenCountRequest,
    ) -> AiResult<AiResponse<OpenAIInputTokenCountResponse>> {
        self.client
            .transport()
            .post_json(
                "responses.input_tokens.count",
                "responses/input_tokens",
                request,
                decode_openai_error,
            )
            .await
    }

    /// Compact a conversation context into reusable response items.
    pub async fn compact(
        &self,
        request: &OpenAICompactRequest,
    ) -> AiResult<AiResponse<OpenAICompactResponse>> {
        self.client
            .transport()
            .post_json(
                "responses.compact",
                "responses/compact",
                request,
                decode_openai_error,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::test_support::{json_response, one_shot_server},
        openai::{
            responses::{
                CreateResponseRequest, ExtendedReasoningEffort, Gpt4oMini, Gpt5_2, ListOrder,
                OpenAIResponsesInput, PromptCacheRetention, ResponseInclude, ResponseItemId,
            },
            OpenAIJsonSchema,
        },
    };

    fn error_response() -> String {
        json_response(
            "400 Bad Request",
            &[("x-request-id", "req_wire")],
            r#"{"error":{"message":"wire fixture","type":"invalid_request_error"}}"#,
        )
    }

    fn test_client(base_url: String) -> OpenAIClient {
        OpenAIClient::builder()
            .api_key("test-key".into())
            .base_url(base_url)
            .build()
            .unwrap()
    }

    fn request_json(request: &str) -> serde_json::Value {
        serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap()
    }

    #[tokio::test]
    async fn create_owns_non_stream_wire_mode() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let request = CreateResponseRequest::builder().input_text("hello").build();
        let _ = client
            .responses()
            .create(Gpt4oMini::config(), request)
            .await;
        let wire = wire.await.unwrap();

        assert!(wire.starts_with("POST /v1/responses HTTP/1.1\r\n"));
        let body = request_json(&wire);
        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["input"], "hello");
        assert!(body.get("stream").is_none());
    }

    #[tokio::test]
    async fn create_combines_request_with_typed_model_config() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let request = CreateResponseRequest::builder()
            .input_text("private input")
            .prompt_cache_key("cache-key")
            .json_schema(OpenAIJsonSchema {
                name: "result".into(),
                description: "result".into(),
                schema: serde_json::json!({"type": "object"}),
                strict: Some(true),
            })
            .build();
        let model = Gpt5_2::config()
            .reasoning(ExtendedReasoningEffort::High)
            .prompt_cache_retention(PromptCacheRetention::Hours24);

        let _ = client.responses().create(model, request).await;
        let body = request_json(&wire.await.unwrap());

        assert_eq!(body["model"], "gpt-5.2");
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["prompt_cache_key"], "cache-key");
        assert_eq!(body["prompt_cache_retention"], "24h");
        assert_eq!(body["text"]["format"]["type"], "json_schema");
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn create_stream_owns_stream_wire_mode() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let request = CreateResponseRequest::builder().input_text("hello").build();
        let _ = client
            .responses()
            .create_stream(Gpt4oMini::config(), request)
            .await;
        let wire = wire.await.unwrap();

        assert!(wire.starts_with("POST /v1/responses HTTP/1.1\r\n"));
        let body = request_json(&wire);
        assert_eq!(body["stream"], true);
        assert!(body.get("stream_options").is_none());
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn create_stream_with_options_sends_obfuscation_setting() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let options = CreateResponseStreamOptions::new().include_obfuscation(false);
        let request = CreateResponseRequest::builder().input_text("hello").build();
        let _ = client
            .responses()
            .create_stream_with_options(Gpt4oMini::config(), request, &options)
            .await;
        let body = request_json(&wire.await.unwrap());

        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_obfuscation"], false);
    }

    #[tokio::test]
    async fn retrieve_encodes_id_and_repeated_includes() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let id = ResponseId::new("resp/a?b").unwrap();
        let options = RetrieveResponseOptions::new()
            .include(ResponseInclude::ReasoningEncryptedContent)
            .include(ResponseInclude::MessageOutputTextLogprobs);
        let _ = client
            .responses()
            .retrieve_with_options(&id, &options)
            .await;

        assert!(wire.await.unwrap().starts_with(
            "GET /v1/responses/resp%2Fa%3Fb?include%5B%5D=reasoning.encrypted_content&include%5B%5D=message.output_text.logprobs HTTP/1.1\r\n"
        ));
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn retrieve_stream_encodes_resume_and_obfuscation_options() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let id = ResponseId::new("resp_123").unwrap();
        let options = RetrieveResponseStreamOptions::new()
            .starting_after(7)
            .include_obfuscation(false);
        let _ = client.responses().retrieve_stream(&id, &options).await;

        assert!(wire.await.unwrap().starts_with(
            "GET /v1/responses/resp_123?stream=true&starting_after=7&include_obfuscation=false HTTP/1.1\r\n"
        ));
    }

    #[tokio::test]
    async fn delete_and_cancel_use_bodyless_dynamic_routes() {
        let id = ResponseId::new("resp/a").unwrap();

        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let _ = client.responses().delete(&id).await;
        let wire = wire.await.unwrap();
        assert!(wire.starts_with("DELETE /v1/responses/resp%2Fa HTTP/1.1\r\n"));
        assert!(wire.ends_with("\r\n\r\n"));

        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let _ = client.responses().cancel(&id).await;
        let wire = wire.await.unwrap();
        assert!(wire.starts_with("POST /v1/responses/resp%2Fa/cancel HTTP/1.1\r\n"));
        assert!(wire.ends_with("\r\n\r\n"));
    }

    #[tokio::test]
    async fn input_item_list_encodes_pagination_and_include() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let id = ResponseId::new("resp_123").unwrap();
        let options = ListResponseInputItemsOptions::new()
            .limit(100)
            .unwrap()
            .order(ListOrder::Asc)
            .after(ResponseItemId::new("item/a").unwrap())
            .include(ResponseInclude::FileSearchCallResults);
        let _ = client.responses().list_input_items(&id, &options).await;

        assert!(wire.await.unwrap().starts_with(
            "GET /v1/responses/resp_123/input_items?limit=100&order=asc&after=item%2Fa&include%5B%5D=file_search_call.results HTTP/1.1\r\n"
        ));
    }

    #[tokio::test]
    async fn token_count_and_compact_use_exact_paths_and_bodies() {
        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let count = OpenAIInputTokenCountRequest {
            model: Some("gpt-5".into()),
            input: Some(OpenAIResponsesInput::Text("hello".into())),
            ..Default::default()
        };
        let _ = client.responses().count_input_tokens(&count).await;
        let wire = wire.await.unwrap();
        assert!(wire.starts_with("POST /v1/responses/input_tokens HTTP/1.1\r\n"));
        assert_eq!(
            request_json(&wire),
            serde_json::json!({"model":"gpt-5","input":"hello"})
        );

        let (base_url, wire) = one_shot_server("v1", error_response()).await;
        let client = test_client(base_url);
        let compact = OpenAICompactRequest::new("gpt-5.1-codex-max");
        let _ = client.responses().compact(&compact).await;
        let wire = wire.await.unwrap();
        assert!(wire.starts_with("POST /v1/responses/compact HTTP/1.1\r\n"));
        assert_eq!(
            request_json(&wire),
            serde_json::json!({"model":"gpt-5.1-codex-max"})
        );
    }
}
