use std::{fmt::Debug, time::Duration};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

use crate::{
    core::http::{HttpTransport, HttpTransportConfig},
    error::{
        AiError, AiProvider, AiResponse, AiResult, BodySnippet, ConfigErrorKind, ProviderApiError,
    },
    utils::IntoQuery,
};
#[cfg(feature = "stream")]
use crate::{core::json_array, stream::AiStream};

use super::{
    CountTokensRequest, CountTokensResponse, GeminiModel, GenerateContentRequest,
    GenerateContentResponse, ModelInfo, ModelsListRequest, ModelsListResponse,
};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Default)]
pub struct GeminiClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    request_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    default_headers: HeaderMap,
}

impl Debug for GeminiClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiClientBuilder")
            .field("api_key", &"[redacted]")
            .field("base_url", &self.base_url.as_ref().map(|_| "[configured]"))
            .field("request_timeout", &self.request_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("default_headers", &"[redacted]")
            .finish()
    }
}

impl GeminiClientBuilder {
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Backward-compatible request timeout in seconds.
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.request_timeout = Some(Duration::from_secs(timeout));
        self
    }

    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    pub fn build(self) -> AiResult<GeminiClient> {
        let api_key = self.api_key.ok_or_else(|| {
            AiError::config(ConfigErrorKind::MissingApiKey, "Gemini API key is required")
        })?;
        let mut headers = self.default_headers;
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        let mut api_key = HeaderValue::from_str(&api_key).map_err(|_| {
            AiError::config(ConfigErrorKind::InvalidApiKey, "Gemini API key is invalid")
        })?;
        api_key.set_sensitive(true);
        headers.insert("x-goog-api-key", api_key);

        let transport = HttpTransport::new(HttpTransportConfig {
            provider: AiProvider::Gemini,
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
            headers,
            request_timeout: self.request_timeout,
            connect_timeout: self.connect_timeout,
        })?;
        Ok(GeminiClient { transport })
    }
}

fn decode_gemini_error(bytes: &[u8], body: BodySnippet) -> ProviderApiError {
    #[derive(serde::Deserialize)]
    struct Envelope {
        error: Option<Detail>,
    }
    #[derive(serde::Deserialize)]
    struct Detail {
        message: Option<String>,
        code: Option<serde_json::Value>,
        status: Option<String>,
    }

    let detail = serde_json::from_slice::<Envelope>(bytes)
        .ok()
        .and_then(|envelope| envelope.error);
    ProviderApiError::new(
        detail
            .as_ref()
            .and_then(|detail| detail.message.clone())
            .unwrap_or_else(|| "unrecognized Gemini error response".to_string()),
        detail.as_ref().and_then(|detail| match &detail.code {
            Some(serde_json::Value::Number(value)) => Some(value.to_string()),
            Some(serde_json::Value::String(value)) => Some(value.clone()),
            _ => None,
        }),
        detail.as_ref().and_then(|detail| detail.status.clone()),
        None,
        body,
    )
}

#[non_exhaustive]
pub struct GeminiClient {
    transport: HttpTransport,
}

impl GeminiClient {
    /// Start configuring a Gemini client.
    pub fn builder() -> GeminiClientBuilder {
        GeminiClientBuilder::default()
    }

    /// List Gemini models with default pagination.
    pub async fn list_models(&self) -> AiResult<AiResponse<ModelsListResponse>> {
        self.list_models_with_params(ModelsListRequest::default())
            .await
    }

    /// List Gemini models with explicit pagination parameters.
    pub async fn list_models_with_params(
        &self,
        params: ModelsListRequest,
    ) -> AiResult<AiResponse<ModelsListResponse>> {
        self.transport
            .get_json(
                "models.list",
                "models",
                &params.into_query(),
                decode_gemini_error,
            )
            .await
    }

    /// Retrieve metadata for a known Gemini model.
    pub async fn get_model(&self, model: GeminiModel) -> AiResult<AiResponse<ModelInfo>> {
        let model = model.to_string();
        self.transport
            .get_json_segments(
                "models.retrieve",
                &["models", &model],
                &[],
                decode_gemini_error,
            )
            .await
    }

    /// Count input tokens for a Gemini content request.
    pub async fn count_tokens(
        &self,
        model: GeminiModel,
        request: CountTokensRequest,
    ) -> AiResult<AiResponse<CountTokensResponse>> {
        let model_action = format!("{model}:countTokens");
        self.transport
            .post_json_segments(
                "models.count_tokens",
                &["models", &model_action],
                &request,
                decode_gemini_error,
            )
            .await
    }

    /// Generate content through Gemini's native `generateContent` API.
    pub async fn generate_content(
        &self,
        model: GeminiModel,
        request: GenerateContentRequest,
    ) -> AiResult<AiResponse<GenerateContentResponse>> {
        let model_action = format!("{model}:generateContent");
        self.transport
            .post_json_segments(
                "models.generate_content",
                &["models", &model_action],
                &request,
                decode_gemini_error,
            )
            .await
    }

    #[cfg(feature = "stream")]
    /// Stream Gemini `generateContent` results.
    ///
    /// Streaming support requires the `stream` crate feature.
    pub async fn generate_content_streamed(
        &self,
        model: GeminiModel,
        request: GenerateContentRequest,
    ) -> AiResult<AiResponse<AiStream<GenerateContentResponse>>> {
        let model_action = format!("{model}:streamGenerateContent");
        let response = self
            .transport
            .post_json_stream_segments(
                "models.stream_generate_content",
                &["models", &model_action],
                &request,
                decode_gemini_error,
            )
            .await?;
        let (bytes, metadata) = response.into_parts();
        Ok(AiResponse::new(
            json_array::values(bytes, AiProvider::Gemini, "models.stream_generate_content"),
            metadata,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "stream")]
    use crate::core::test_support::chunked_server;
    use crate::core::test_support::{cross_origin_redirect_server, json_response, one_shot_server};

    #[test]
    fn builder_debug_redacts_credentials_and_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-secret", HeaderValue::from_static("header-secret"));
        let debug = format!(
            "{:?}",
            GeminiClient::builder()
                .api_key("api-secret".into())
                .base_url("https://url-secret@example.com/v1?key=url-secret")
                .default_headers(headers)
        );
        assert!(!debug.contains("api-secret"));
        assert!(!debug.contains("header-secret"));
        assert!(!debug.contains("url-secret"));
    }

    #[test]
    fn decodes_structured_gemini_error_without_debug_leakage() {
        let bytes =
            br#"{"error":{"code":429,"message":"secret prompt","status":"RESOURCE_EXHAUSTED"}}"#;
        let error = decode_gemini_error(bytes, BodySnippet::from_bytes(bytes, false));
        assert_eq!(error.message(), "secret prompt");
        assert_eq!(error.code(), Some("429"));
        assert_eq!(error.kind(), Some("RESOURCE_EXHAUSTED"));
        assert!(!format!("{error:?}").contains("secret prompt"));
    }

    #[tokio::test]
    async fn success_wire_includes_auth_prefix_headers_and_metadata() {
        let response = json_response(
            "200 OK",
            &[
                ("x-goog-request-id", "req_gemini_wire"),
                ("x-ratelimit-remaining-tokens", "123"),
            ],
            r#"{"models":[],"nextPageToken":null}"#,
        );
        let (base_url, request) = one_shot_server("proxy/v1beta", response).await;
        let mut collisions = HeaderMap::new();
        collisions.insert("x-goog-api-key", HeaderValue::from_static("collision-key"));
        collisions.insert(USER_AGENT, HeaderValue::from_static("collision-agent"));
        let response = GeminiClient::builder()
            .api_key("gemini-wire-key".into())
            .base_url(base_url)
            .default_headers(collisions)
            .build()
            .unwrap()
            .list_models()
            .await
            .unwrap();
        let request = request.await.unwrap().to_ascii_lowercase();

        assert!(request.starts_with("get /proxy/v1beta/models http/1.1\r\n"));
        assert!(request.contains("x-goog-api-key: gemini-wire-key\r\n"));
        assert!(request.contains(&format!("user-agent: {USER_AGENT_VALUE}\r\n")));
        assert!(response.data().models.is_empty());
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_gemini_wire")
        );
        assert_eq!(
            response.metadata().rate_limit.remaining_tokens.as_deref(),
            Some("123")
        );
    }

    #[tokio::test]
    async fn structured_json_api_error_survives_public_client_boundary() {
        let body = r#"{"error":{"code":429,"message":"private Gemini detail","status":"RESOURCE_EXHAUSTED"}}"#;
        let response = json_response(
            "429 Too Many Requests",
            &[
                ("x-goog-request-id", "req_gemini_error"),
                ("x-ratelimit-remaining-tokens", "0"),
            ],
            body,
        );
        let (base_url, request) = one_shot_server("v1beta", response).await;
        let error = GeminiClient::builder()
            .api_key("test-key".into())
            .base_url(base_url)
            .build()
            .unwrap()
            .list_models()
            .await
            .unwrap_err();
        request.await.unwrap();

        match &error {
            AiError::Api {
                provider,
                operation,
                metadata,
                error,
            } => {
                assert_eq!(*provider, AiProvider::Gemini);
                assert_eq!(*operation, "models.list");
                assert_eq!(metadata.request_id.as_deref(), Some("req_gemini_error"));
                assert_eq!(metadata.rate_limit.remaining_tokens.as_deref(), Some("0"));
                assert_eq!(error.code(), Some("429"));
                assert_eq!(error.kind(), Some("RESOURCE_EXHAUSTED"));
                assert_eq!(error.body().as_str(), body);
            }
            other => panic!("expected API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("private Gemini detail"));
        assert!(!error.to_string().contains("private Gemini detail"));
    }

    #[tokio::test]
    async fn public_client_stops_cross_origin_redirect_with_auth() {
        let server = cross_origin_redirect_server("v1beta").await;
        let _ = GeminiClient::builder()
            .api_key("redirect-gemini-key".into())
            .base_url(server.base_url)
            .build()
            .unwrap()
            .list_models()
            .await;
        let request = server.origin_request.await.unwrap().to_ascii_lowercase();
        assert!(request.contains("x-goog-api-key: redirect-gemini-key\r\n"));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), server.redirect_target.accept())
                .await
                .is_err()
        );
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn stream_handshake_returns_metadata_wire_path_and_provider_errors() {
        use futures::StreamExt;

        let (base_url, request) = chunked_server(
            "v1beta",
            &[
                ("x-goog-request-id", "req_gemini_stream"),
                ("x-ratelimit-remaining-tokens", "44"),
            ],
            vec![b"[]".to_vec()],
        )
        .await;
        let response = GeminiClient::builder()
            .api_key("key".into())
            .base_url(base_url)
            .build()
            .unwrap()
            .generate_content_streamed(
                GeminiModel::Gemini3_1FlashLite,
                GenerateContentRequest {
                    contents: vec![],
                    safety_settings: None,
                    generation_config: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_gemini_stream")
        );
        assert_eq!(
            response.metadata().rate_limit.remaining_tokens.as_deref(),
            Some("44")
        );
        let mut values = response.into_inner();
        assert!(values.next().await.is_none());
        let request = request.await.unwrap();
        assert!(request.starts_with(
            "POST /v1beta/models/gemini-3.1-flash-lite:streamGenerateContent HTTP/1.1\r\n"
        ));

        let error_body =
            r#"{"error":{"code":429,"message":"private handshake","status":"RESOURCE_EXHAUSTED"}}"#;
        let response = json_response(
            "429 Too Many Requests",
            &[("x-goog-request-id", "req_gemini_stream_error")],
            error_body,
        );
        let (base_url, request) = one_shot_server("v1beta", response).await;
        let result = GeminiClient::builder()
            .api_key("key".into())
            .base_url(base_url)
            .build()
            .unwrap()
            .generate_content_streamed(
                GeminiModel::Gemini3_1FlashLite,
                GenerateContentRequest {
                    contents: vec![],
                    safety_settings: None,
                    generation_config: None,
                },
            )
            .await;
        request.await.unwrap();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected Gemini handshake failure"),
        };
        match &error {
            AiError::Api {
                operation,
                metadata,
                error,
                ..
            } => {
                assert_eq!(*operation, "models.stream_generate_content");
                assert_eq!(
                    metadata.request_id.as_deref(),
                    Some("req_gemini_stream_error")
                );
                assert_eq!(error.code(), Some("429"));
                assert_eq!(error.kind(), Some("RESOURCE_EXHAUSTED"));
            }
            other => panic!("expected handshake API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("private handshake"));

        let non_json = "<html>private Gemini gateway</html>";
        let response = format!(
            "HTTP/1.1 502 Bad Gateway\r\nx-goog-request-id: req_gemini_non_json\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{non_json}",
            non_json.len()
        );
        let (base_url, request) = one_shot_server("v1beta", response).await;
        let result = GeminiClient::builder()
            .api_key("key".into())
            .base_url(base_url)
            .build()
            .unwrap()
            .generate_content_streamed(
                GeminiModel::Gemini3_1FlashLite,
                GenerateContentRequest {
                    contents: vec![],
                    safety_settings: None,
                    generation_config: None,
                },
            )
            .await;
        request.await.unwrap();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("expected non-JSON Gemini stream handshake failure"),
        };
        match &error {
            AiError::Api {
                operation,
                metadata,
                error,
                ..
            } => {
                assert_eq!(*operation, "models.stream_generate_content");
                assert_eq!(metadata.request_id.as_deref(), Some("req_gemini_non_json"));
                assert_eq!(error.body().as_str(), non_json);
            }
            other => panic!("expected non-JSON API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("private Gemini gateway"));
        assert!(!error.to_string().contains("private Gemini gateway"));
    }
}
