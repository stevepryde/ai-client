use std::{fmt::Debug, time::Duration};

#[cfg(feature = "stream")]
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
#[cfg(feature = "stream")]
use reqwest_streams::error::StreamBodyError;

#[cfg(all(feature = "chat-completions", feature = "stream"))]
use crate::openai::create_chat_completion::OpenAIStreamChunk;
#[cfg(feature = "chat-completions")]
use crate::openai::create_chat_completion::{
    OpenAIGenerateContentRequest, OpenAIGenerateContentResponse,
};
#[cfg(feature = "stream")]
use crate::openai::create_response::OpenAIResponsesStreamEvent;
use crate::{
    core::http::{HttpTransport, HttpTransportConfig},
    error::{
        AiError, AiProvider, AiResponse, AiResult, BodySnippet, ConfigErrorKind, ProviderApiError,
    },
    openai::{
        create_response::{OpenAIResponsesCreateRequest, OpenAIResponsesCreateResponse},
        list_models::{OpenAIModelInfo, OpenAIModelsListResponse},
    },
};

use super::OpenAIModel;

const BASE_URL: &str = "https://api.openai.com/v1";
const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Default)]
pub struct OpenAIClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    request_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    default_headers: HeaderMap,
    organization: Option<String>,
    project: Option<String>,
}

impl Debug for OpenAIClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIClientBuilder")
            .field("api_key", &"[redacted]")
            .field("base_url", &self.base_url.as_ref().map(|_| "[configured]"))
            .field("request_timeout", &self.request_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("default_headers", &"[redacted]")
            .field("organization", &"[redacted]")
            .field("project", &"[redacted]")
            .finish()
    }
}

impl OpenAIClientBuilder {
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

    pub fn organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn build(self) -> AiResult<OpenAIClient> {
        let api_key = self.api_key.ok_or_else(|| {
            AiError::config(ConfigErrorKind::MissingApiKey, "OpenAI API key is required")
        })?;
        let mut headers = self.default_headers;
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|_| {
                AiError::config(ConfigErrorKind::InvalidApiKey, "OpenAI API key is invalid")
            })?;
        authorization.set_sensitive(true);
        headers.insert(AUTHORIZATION, authorization);
        insert_optional_header(&mut headers, "openai-organization", self.organization)?;
        insert_optional_header(&mut headers, "openai-project", self.project)?;

        let transport = HttpTransport::new(HttpTransportConfig {
            provider: AiProvider::OpenAI,
            base_url: self.base_url.unwrap_or_else(|| BASE_URL.to_string()),
            headers,
            request_timeout: self.request_timeout,
            connect_timeout: self.connect_timeout,
        })?;
        Ok(OpenAIClient { transport })
    }
}

fn insert_optional_header(
    headers: &mut HeaderMap,
    name: &'static str,
    value: Option<String>,
) -> AiResult<()> {
    if let Some(value) = value {
        let mut value = HeaderValue::from_str(&value).map_err(|_| {
            AiError::config(
                ConfigErrorKind::InvalidHeader,
                format!("invalid {name} header"),
            )
        })?;
        value.set_sensitive(true);
        headers.insert(name, value);
    }
    Ok(())
}

fn decode_openai_error(bytes: &[u8], body: BodySnippet) -> ProviderApiError {
    #[derive(serde::Deserialize)]
    struct Envelope {
        error: Option<Detail>,
    }
    #[derive(serde::Deserialize)]
    struct Detail {
        message: Option<String>,
        code: Option<serde_json::Value>,
        #[serde(rename = "type")]
        kind: Option<String>,
        param: Option<serde_json::Value>,
    }

    let detail = serde_json::from_slice::<Envelope>(bytes)
        .ok()
        .and_then(|envelope| envelope.error);
    ProviderApiError::new(
        detail
            .as_ref()
            .and_then(|detail| detail.message.clone())
            .unwrap_or_else(|| "unrecognized OpenAI error response".to_string()),
        detail.as_ref().and_then(|detail| scalar(&detail.code)),
        detail.as_ref().and_then(|detail| detail.kind.clone()),
        detail.as_ref().and_then(|detail| scalar(&detail.param)),
        body,
    )
}

fn scalar(value: &Option<serde_json::Value>) -> Option<String> {
    match value.as_ref()? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(feature = "stream")]
async fn parse_sse_stream<T>(
    response: reqwest::Response,
) -> impl Stream<Item = Result<T, StreamBodyError>>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    use futures::{stream, StreamExt};

    let byte_stream = response.bytes_stream();
    stream::unfold(
        (byte_stream, String::new(), Vec::<u8>::new()),
        |(mut byte_stream, mut buffer, mut remainder)| async move {
            loop {
                if let Some(pos) = buffer.find("\n\n") {
                    let event = buffer[..pos].to_string();
                    buffer.drain(..pos + 2);
                    for line in event.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" {
                                continue;
                            }
                            return Some((
                                serde_json::from_str::<T>(data).map_err(|_| {
                                    StreamBodyError::new(
                                        reqwest_streams::error::StreamBodyKind::CodecError,
                                        None,
                                        Some("failed to decode SSE JSON".to_string()),
                                    )
                                }),
                                (byte_stream, buffer, remainder),
                            ));
                        }
                    }
                }
                match byte_stream.next().await {
                    Some(Ok(bytes)) => {
                        let combined = if remainder.is_empty() {
                            bytes.to_vec()
                        } else {
                            let mut combined = std::mem::take(&mut remainder);
                            combined.extend_from_slice(&bytes);
                            combined
                        };
                        match std::str::from_utf8(&combined) {
                            Ok(text) => buffer.push_str(text),
                            Err(error) => {
                                let valid_up_to = error.valid_up_to();
                                if let Ok(valid) = std::str::from_utf8(&combined[..valid_up_to]) {
                                    buffer.push_str(valid);
                                }
                                remainder = combined[valid_up_to..].to_vec();
                            }
                        }
                    }
                    Some(Err(error)) => {
                        return Some((
                            Err(StreamBodyError::new(
                                reqwest_streams::error::StreamBodyKind::InputOutputError,
                                Some(Box::new(error)),
                                None,
                            )),
                            (byte_stream, buffer, remainder),
                        ));
                    }
                    None => return None,
                }
            }
        },
    )
}

#[non_exhaustive]
pub struct OpenAIClient {
    transport: HttpTransport,
}

impl OpenAIClient {
    /// Start configuring an OpenAI client.
    pub fn builder() -> OpenAIClientBuilder {
        OpenAIClientBuilder::default()
    }

    /// List models available to the configured OpenAI account.
    pub async fn list_models(&self) -> AiResult<AiResponse<OpenAIModelsListResponse>> {
        self.transport
            .get_json("models.list", "models", &[], decode_openai_error)
            .await
    }

    /// Retrieve metadata for a known OpenAI model.
    pub async fn get_model(&self, model: OpenAIModel) -> AiResult<AiResponse<OpenAIModelInfo>> {
        let model = model.to_string();
        self.transport
            .get_json_segments(
                "models.retrieve",
                &["models", &model],
                &[],
                decode_openai_error,
            )
            .await
    }

    #[cfg(feature = "chat-completions")]
    /// Create a legacy Chat Completions response.
    ///
    /// New OpenAI integrations should use [`Self::generate_response`]. This
    /// migration surface is available only with `chat-completions`.
    pub async fn generate_content(
        &self,
        mut request: OpenAIGenerateContentRequest,
    ) -> AiResult<AiResponse<OpenAIGenerateContentResponse>> {
        request.sanitise();
        self.transport
            .post_json(
                "chat_completions.create",
                "chat/completions",
                &request,
                decode_openai_error,
            )
            .await
    }

    #[cfg(all(feature = "chat-completions", feature = "stream"))]
    /// Stream a legacy Chat Completions response.
    ///
    /// New OpenAI integrations should use [`Self::generate_response_streamed`].
    pub async fn generate_content_streamed(
        &self,
        mut request: OpenAIGenerateContentRequest,
    ) -> AiResult<impl Stream<Item = Result<OpenAIStreamChunk, StreamBodyError>>> {
        request.sanitise();
        let response = self
            .transport
            .post_json_stream("chat_completions.stream", "chat/completions", &request)
            .await?;
        Ok(parse_sse_stream(response).await)
    }

    /// Create a response with OpenAI's primary Responses API.
    ///
    /// The returned [`AiResponse`] includes both the decoded provider body and
    /// request/rate-limit metadata.
    pub async fn generate_response(
        &self,
        mut request: OpenAIResponsesCreateRequest,
    ) -> AiResult<AiResponse<OpenAIResponsesCreateResponse>> {
        request.sanitise();
        self.transport
            .post_json(
                "responses.create",
                "responses",
                &request,
                decode_openai_error,
            )
            .await
    }

    #[cfg(feature = "stream")]
    /// Stream events from OpenAI's Responses API.
    ///
    /// Streaming support requires the `stream` crate feature.
    pub async fn generate_response_streamed(
        &self,
        mut request: OpenAIResponsesCreateRequest,
    ) -> AiResult<impl Stream<Item = Result<OpenAIResponsesStreamEvent, StreamBodyError>>> {
        request.sanitise();
        let response = self
            .transport
            .post_json_stream("responses.stream", "responses", &request)
            .await?;
        Ok(parse_sse_stream(response).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_support::{
        chunked_server, cross_origin_redirect_server, delayed_server, json_response,
        one_shot_server,
    };

    #[test]
    fn builder_debug_redacts_credentials_and_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-secret", HeaderValue::from_static("header-secret"));
        let debug = format!(
            "{:?}",
            OpenAIClient::builder()
                .api_key("api-secret".into())
                .organization("org-secret")
                .project("project-secret")
                .base_url("https://url-secret@example.com/v1?key=url-secret")
                .default_headers(headers)
        );
        for secret in [
            "api-secret",
            "header-secret",
            "org-secret",
            "project-secret",
            "url-secret",
        ] {
            assert!(!debug.contains(secret));
        }
    }

    #[test]
    fn decodes_structured_openai_error_without_debug_leakage() {
        let bytes = br#"{"error":{"message":"secret prompt","type":"invalid_request_error","param":"input","code":"bad_input"}}"#;
        let error = decode_openai_error(bytes, BodySnippet::from_bytes(bytes, false));
        assert_eq!(error.message(), "secret prompt");
        assert_eq!(error.code(), Some("bad_input"));
        assert_eq!(error.kind(), Some("invalid_request_error"));
        assert_eq!(error.param(), Some("input"));
        assert!(!format!("{error:?}").contains("secret prompt"));
    }

    #[tokio::test]
    async fn success_wire_includes_auth_prefix_headers_and_metadata() {
        let response = json_response(
            "200 OK",
            &[
                ("x-request-id", "req_openai_wire"),
                ("x-ratelimit-remaining-requests", "19"),
            ],
            r#"{"models":[],"nextPageToken":null}"#,
        );
        let (base_url, request) = one_shot_server("proxy/v1", response).await;
        let mut collisions = HeaderMap::new();
        collisions.insert(AUTHORIZATION, HeaderValue::from_static("Bearer collision"));
        collisions.insert(USER_AGENT, HeaderValue::from_static("collision-agent"));
        collisions.insert(
            "openai-organization",
            HeaderValue::from_static("org_collision"),
        );
        collisions.insert("openai-project", HeaderValue::from_static("proj_collision"));
        let response = OpenAIClient::builder()
            .api_key("openai-wire-key".into())
            .base_url(base_url)
            .default_headers(collisions)
            .organization("org_wire")
            .project("proj_wire")
            .build()
            .unwrap()
            .list_models()
            .await
            .unwrap();
        let request = request.await.unwrap().to_ascii_lowercase();

        assert!(request.starts_with("get /proxy/v1/models http/1.1\r\n"));
        assert!(request.contains("authorization: bearer openai-wire-key\r\n"));
        assert!(request.contains(&format!("user-agent: {USER_AGENT_VALUE}\r\n")));
        assert!(request.contains("openai-organization: org_wire\r\n"));
        assert!(request.contains("openai-project: proj_wire\r\n"));
        assert!(response.data().models.is_empty());
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_openai_wire")
        );
        assert_eq!(
            response.metadata().rate_limit.remaining_requests.as_deref(),
            Some("19")
        );
    }

    #[tokio::test]
    async fn non_json_api_error_is_bounded_and_structured() {
        let body = "<html>private provider error</html>";
        let response = format!(
            "HTTP/1.1 502 Bad Gateway\r\nx-request-id: req_non_json\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server("v1", response).await;
        let error = OpenAIClient::builder()
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
                operation,
                metadata,
                error,
                ..
            } => {
                assert_eq!(*operation, "models.list");
                assert_eq!(metadata.status, reqwest::StatusCode::BAD_GATEWAY);
                assert_eq!(metadata.request_id.as_deref(), Some("req_non_json"));
                assert_eq!(error.body().as_str(), body);
            }
            other => panic!("expected API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("private provider error"));
        assert!(!error.to_string().contains("private provider error"));
    }

    #[tokio::test]
    async fn structured_json_api_error_survives_public_client_boundary() {
        let body = r#"{"error":{"message":"private OpenAI detail","type":"invalid_request_error","param":"input","code":"bad_input"}}"#;
        let response = json_response(
            "400 Bad Request",
            &[
                ("x-request-id", "req_openai_error"),
                ("x-ratelimit-remaining-requests", "7"),
            ],
            body,
        );
        let (base_url, request) = one_shot_server("v1", response).await;
        let error = OpenAIClient::builder()
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
                assert_eq!(*provider, AiProvider::OpenAI);
                assert_eq!(*operation, "models.list");
                assert_eq!(metadata.request_id.as_deref(), Some("req_openai_error"));
                assert_eq!(metadata.rate_limit.remaining_requests.as_deref(), Some("7"));
                assert_eq!(error.code(), Some("bad_input"));
                assert_eq!(error.kind(), Some("invalid_request_error"));
                assert_eq!(error.param(), Some("input"));
                assert_eq!(error.body().as_str(), body);
            }
            other => panic!("expected API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("private OpenAI detail"));
        assert!(!error.to_string().contains("private OpenAI detail"));
    }

    #[tokio::test]
    async fn request_timeout_is_normalized() {
        let response = json_response("200 OK", &[], r#"{"models":[],"nextPageToken":null}"#);
        let (base_url, server) = delayed_server("v1", response, Duration::from_millis(150)).await;
        let error = OpenAIClient::builder()
            .api_key("test-key".into())
            .base_url(base_url)
            .request_timeout(Duration::from_millis(20))
            .build()
            .unwrap()
            .list_models()
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            AiError::Timeout {
                provider: AiProvider::OpenAI,
                operation: "models.list"
            }
        ));
        server.abort();
    }

    #[tokio::test]
    async fn public_client_stops_cross_origin_redirect_with_auth() {
        let server = cross_origin_redirect_server("v1").await;
        let _ = OpenAIClient::builder()
            .api_key("redirect-openai-key".into())
            .base_url(server.base_url)
            .build()
            .unwrap()
            .list_models()
            .await;
        let request = server.origin_request.await.unwrap().to_ascii_lowercase();
        assert!(request.contains("authorization: bearer redirect-openai-key\r\n"));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), server.redirect_target.accept())
                .await
                .is_err()
        );
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn sse_regression_decodes_basic_json_and_split_utf8() {
        use futures::StreamExt;

        let mut first = br#"data: {"text":"hello"}

data: {"text":"caf"#
            .to_vec();
        first.push(0xC3);
        let mut second = vec![0xA9];
        second.extend_from_slice(
            br#""}

"#,
        );
        let (base_url, request) = chunked_server(vec![first, second]).await;
        let response = reqwest::Client::new()
            .get(format!("{base_url}/events"))
            .send()
            .await
            .unwrap();
        let events = parse_sse_stream::<serde_json::Value>(response).await;
        futures::pin_mut!(events);

        assert_eq!(
            events.next().await.unwrap().unwrap()["text"],
            serde_json::json!("hello")
        );
        assert_eq!(
            events.next().await.unwrap().unwrap()["text"],
            serde_json::json!("café")
        );
        request.await.unwrap();
    }
}
