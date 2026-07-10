mod capability;
mod events;
mod request;
mod response;

pub use capability::*;
pub use events::*;
pub use request::*;
pub use response::*;

use std::{future::Future, marker::PhantomData};

#[cfg(feature = "stream")]
use crate::{
    core::sse,
    error::AiProvider,
    stream::{AiStream, SseJsonEvent},
};
use crate::{
    error::{AiResponse, AiResult, BodySnippet, ProviderApiError},
    openai_compatible::{ChatCompletionsDialect, OpenAICompatibleClient},
};

pub trait ChatCompletionsResource<D: ChatCompletionsDialect> {
    fn create(
        &self,
        request: PreparedChatRequest<D>,
    ) -> impl Future<Output = AiResult<AiResponse<ChatResponse>>> + Send;

    #[cfg(feature = "stream")]
    fn create_stream(
        &self,
        request: PreparedChatRequest<D>,
    ) -> impl Future<Output = AiResult<AiResponse<AiStream<SseJsonEvent<ChatStreamEvent>>>>> + Send;
}

pub struct ChatResource<'a, D: ChatCompletionsDialect> {
    client: &'a OpenAICompatibleClient<D>,
    marker: PhantomData<fn() -> D>,
}

impl<'a, D: ChatCompletionsDialect> ChatResource<'a, D> {
    pub(crate) fn new(client: &'a OpenAICompatibleClient<D>) -> Self {
        Self {
            client,
            marker: PhantomData,
        }
    }

    pub async fn create(
        &self,
        mut request: PreparedChatRequest<D>,
    ) -> AiResult<AiResponse<ChatResponse>> {
        request.set_stream(false);
        let decoder = self.client.decoder.clone();
        self.client
            .transport
            .post_json(
                "compatible.chat.create",
                "chat/completions",
                &request,
                move |bytes, body| decode_error(decoder.as_ref(), bytes, body),
            )
            .await
    }

    #[cfg(feature = "stream")]
    pub async fn create_stream(
        &self,
        mut request: PreparedChatRequest<D>,
    ) -> AiResult<AiResponse<AiStream<SseJsonEvent<ChatStreamEvent>>>> {
        request.set_stream(true);
        let decoder = self.client.decoder.clone();
        let response = self
            .client
            .transport
            .post_json_stream(
                "compatible.chat.stream",
                "chat/completions",
                &request,
                move |bytes, body| decode_error(decoder.as_ref(), bytes, body),
            )
            .await?;
        let (bytes, metadata) = response.into_parts();
        Ok(AiResponse::new(
            sse::json_events(
                bytes,
                AiProvider::OpenAICompatible(D::NAME),
                "compatible.chat.stream",
            ),
            metadata,
        ))
    }
}

impl<D: ChatCompletionsDialect> ChatCompletionsResource<D> for ChatResource<'_, D> {
    fn create(
        &self,
        request: PreparedChatRequest<D>,
    ) -> impl Future<Output = AiResult<AiResponse<ChatResponse>>> + Send {
        ChatResource::create(self, request)
    }

    #[cfg(feature = "stream")]
    fn create_stream(
        &self,
        request: PreparedChatRequest<D>,
    ) -> impl Future<Output = AiResult<AiResponse<AiStream<SseJsonEvent<ChatStreamEvent>>>>> + Send
    {
        ChatResource::create_stream(self, request)
    }
}

fn decode_error(
    decoder: &dyn crate::openai_compatible::CompatibleErrorDecoder,
    bytes: &[u8],
    body: BodySnippet,
) -> ProviderApiError {
    let details = decoder.decode(bytes);
    let (message, code, kind, param) = details.into_parts();
    ProviderApiError::new(message, code, kind, param, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::test_support::{cross_origin_redirect_server, json_response, one_shot_server},
        error::{AiError, AiProvider, ConfigErrorKind},
        openai_compatible::{
            CompatibleAuth, CompatibleErrorDecoder, CompatibleErrorDetails, CustomDialect,
            OpenAICompatibleClient,
        },
    };

    #[derive(Debug)]
    struct TypedModel;

    impl CompatibleChatModel<CustomDialect> for TypedModel {
        const ID: &'static str = "typed-chat-model";
    }

    impl SupportsChatSampling<CustomDialect> for TypedModel {}
    impl SupportsMaxCompletionTokens<CustomDialect> for TypedModel {}

    fn request() -> PreparedChatRequest<CustomDialect> {
        ChatRequest::<CustomDialect, TypedModel>::builder()
            .messages(vec![ChatMessage::new(ChatRole::User, "private prompt")])
            .temperature(ChatTemperature::new(0.7).unwrap())
            .max_completion_tokens(42)
            .build()
            .unwrap()
    }

    #[test]
    fn build_rejects_typed_and_extension_collisions_and_redacts_payload() {
        let mut extra = serde_json::Map::new();
        extra.insert("model".into(), serde_json::json!("replacement"));
        let error = ChatRequest::<CustomDialect, TypedModel>::builder()
            .messages(vec![ChatMessage::new(ChatRole::User, "secret")])
            .extra_body(extra)
            .build()
            .unwrap_err();
        assert_eq!(error, ChatBuildError::ExtraBodyCollision("model".into()));

        let request = request();
        let debug = format!("{request:?}");
        assert!(!debug.contains("private prompt"));
    }

    #[test]
    fn custom_message_preserves_rich_object_and_redacts_debug() {
        let mut object = serde_json::Map::new();
        object.insert("role".into(), serde_json::json!("tool"));
        object.insert("tool_call_id".into(), serde_json::json!("call_secret"));
        object.insert(
            "content".into(),
            serde_json::json!([{"type":"input_text","text":"private nested content"}]),
        );
        let message = ChatMessage::from_object(object);
        assert!(!format!("{message:?}").contains("private nested content"));

        let request = ChatRequest::<CustomDialect, TypedModel>::builder()
            .messages(vec![message])
            .build()
            .unwrap();
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["messages"][0]["role"], "tool");
        assert_eq!(value["messages"][0]["tool_call_id"], "call_secret");
        assert_eq!(
            value["messages"][0]["content"][0]["text"],
            "private nested content"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct CollidingDialect;

    #[derive(Default, serde::Serialize)]
    struct CollidingOptions {
        model: Option<String>,
    }

    impl crate::openai_compatible::OpenAICompatibleDialect for CollidingDialect {
        const NAME: &'static str = "colliding-test";
    }

    impl crate::openai_compatible::ChatCompletionsDialect for CollidingDialect {
        type ChatOptions = CollidingOptions;
        type Message = ChatMessage;
    }

    #[derive(Debug, Clone, Copy)]
    struct InvalidMessageDialect;

    impl crate::openai_compatible::OpenAICompatibleDialect for InvalidMessageDialect {
        const NAME: &'static str = "invalid-message-test";
    }

    impl crate::openai_compatible::ChatCompletionsDialect for InvalidMessageDialect {
        type ChatOptions = crate::openai_compatible::CustomChatOptions;
        type Message = String;
    }

    struct InvalidMessageModel;
    impl CompatibleChatModel<InvalidMessageDialect> for InvalidMessageModel {
        const ID: &'static str = "model";
    }

    #[test]
    fn dialect_message_must_erase_to_an_object() {
        let error = ChatRequest::<InvalidMessageDialect, InvalidMessageModel>::builder()
            .messages(vec!["not-an-object".to_string()])
            .build()
            .unwrap_err();
        assert_eq!(error, ChatBuildError::InvalidMessage(0));
    }

    struct CollidingModel;
    impl CompatibleChatModel<CollidingDialect> for CollidingModel {
        const ID: &'static str = "model";
    }

    #[test]
    fn provider_options_cannot_replace_typed_wire_fields() {
        let error = ChatRequest::<CollidingDialect, CollidingModel>::builder()
            .messages(vec![ChatMessage::new(ChatRole::User, "hello")])
            .provider_options(CollidingOptions {
                model: Some("replacement".into()),
            })
            .build()
            .unwrap_err();
        assert_eq!(
            error,
            ChatBuildError::ProviderOptionCollision("model".into())
        );
    }

    #[test]
    fn auth_is_explicit_redacted_and_cannot_control_http_framing() {
        let auth = CompatibleAuth::header("x-api-key", "secret-value");
        assert!(!format!("{auth:?}").contains("secret-value"));

        for name in [
            "host",
            "connection",
            "content-length",
            "content-type",
            "content-encoding",
            "transfer-encoding",
            "te",
            "trailer",
            "upgrade",
            "expect",
            "keep-alive",
            "proxy-connection",
            "proxy-authenticate",
            "proxy-authorization",
            "forwarded",
            "x-forwarded-host",
            "x-forwarded-proto",
            "user-agent",
        ] {
            let error = OpenAICompatibleClient::<CustomDialect>::builder()
                .base_url("https://example.invalid/v1")
                .auth(CompatibleAuth::header(name, "attacker-value"))
                .build()
                .err()
                .unwrap_or_else(|| panic!("unsafe header {name} must fail"));
            assert!(matches!(
                error,
                AiError::Config {
                    kind: ConfigErrorKind::InvalidHeader,
                    ..
                }
            ));
        }

        for name in ["authorization", "x-api-key", "api-key"] {
            OpenAICompatibleClient::<CustomDialect>::builder()
                .base_url("https://example.invalid/v1")
                .auth(CompatibleAuth::header(name, "safe-secret"))
                .build()
                .unwrap_or_else(|error| panic!("auth header {name} should be allowed: {error}"));
        }
    }

    #[tokio::test]
    async fn custom_dialect_wire_metadata_and_raw_response_are_preserved() {
        let response = json_response(
            "200 OK",
            &[("x-request-id", "req_compatible")],
            r#"{"id":"chat-1","model":"typed-chat-model","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":"hello","refusal":null}}],"usage":{"completion_tokens":1,"prompt_tokens":2,"total_tokens":3},"future_field":{"kept":true}}"#,
        );
        let (base_url, received) = one_shot_server("proxy/v1", response).await;
        let response = OpenAICompatibleClient::<CustomDialect>::builder()
            .base_url(base_url)
            .auth(CompatibleAuth::bearer("wire-secret"))
            .build()
            .unwrap()
            .chat()
            .create(request())
            .await
            .unwrap();

        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_compatible")
        );
        assert_eq!(response.data().raw()["future_field"]["kept"], true);
        let received = received.await.unwrap();
        let (head, body) = received.split_once("\r\n\r\n").unwrap();
        assert!(head.starts_with("POST /proxy/v1/chat/completions HTTP/1.1\r\n"));
        assert!(head
            .to_ascii_lowercase()
            .contains("authorization: bearer wire-secret\r\n"));
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(body["model"], "typed-chat-model");
        assert_eq!(body["messages"][0]["content"], "private prompt");
        assert_eq!(body["temperature"], 0.7);
        assert!(body.get("stream").is_none());
    }

    #[derive(Debug, Clone, Copy)]
    struct CustomDecoder;

    impl CompatibleErrorDecoder for CustomDecoder {
        fn decode(&self, body: &[u8]) -> CompatibleErrorDetails {
            assert_eq!(body, br#"{"problem":"private"}"#);
            CompatibleErrorDetails::new("decoded private error").with_code("custom_code")
        }
    }

    #[tokio::test]
    async fn custom_error_decoder_produces_structured_redacted_error() {
        let body = r#"{"problem":"private"}"#;
        let response = json_response("418 I'm a teapot", &[("x-request-id", "req_error")], body);
        let (base_url, received) = one_shot_server("v1", response).await;
        let error = OpenAICompatibleClient::<CustomDialect>::builder()
            .base_url(base_url)
            .auth(CompatibleAuth::none())
            .error_decoder(CustomDecoder)
            .build()
            .unwrap()
            .chat()
            .create(request())
            .await
            .unwrap_err();
        received.await.unwrap();

        match &error {
            AiError::Api {
                provider,
                operation,
                metadata,
                error,
            } => {
                assert_eq!(*provider, AiProvider::OpenAICompatible("custom"));
                assert_eq!(*operation, "compatible.chat.create");
                assert_eq!(metadata.request_id.as_deref(), Some("req_error"));
                assert_eq!(error.code(), Some("custom_code"));
                assert_eq!(error.message(), "decoded private error");
            }
            other => panic!("expected API error, got {other:?}"),
        }
        assert!(!format!("{error:?}").contains("decoded private error"));
        assert!(!error.to_string().contains("decoded private error"));
    }

    #[tokio::test]
    async fn custom_auth_does_not_follow_cross_origin_redirects() {
        let server = cross_origin_redirect_server("v1").await;
        let _ = OpenAICompatibleClient::<CustomDialect>::builder()
            .base_url(server.base_url)
            .auth(CompatibleAuth::header("x-api-key", "redirect-secret"))
            .build()
            .unwrap()
            .chat()
            .create(request())
            .await;
        let request = server.origin_request.await.unwrap().to_ascii_lowercase();
        assert!(request.contains("x-api-key: redirect-secret\r\n"));
        assert!(tokio::time::timeout(
            std::time::Duration::from_millis(100),
            server.redirect_target.accept()
        )
        .await
        .is_err());
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn stream_owns_wire_mode_and_preserves_raw_sse() {
        use crate::core::test_support::chunked_server;
        use futures::StreamExt;

        let event = br#"data: {"id":"chat-1","model":"typed-chat-model","choices":[{"index":0,"delta":{"role":"assistant","content":"hi"},"finish_reason":null}],"usage":null,"future":"kept"}

"#
        .to_vec();
        let (base_url, received) =
            chunked_server("v1", &[("x-request-id", "req_stream")], vec![event]).await;
        let response = OpenAICompatibleClient::<CustomDialect>::builder()
            .base_url(base_url)
            .auth(CompatibleAuth::none())
            .build()
            .unwrap()
            .chat()
            .create_stream(request())
            .await
            .unwrap();
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_stream")
        );
        let mut stream = response.into_inner();
        let event = stream.next().await.unwrap().unwrap();
        assert_eq!(event.data().choices[0].delta.content.as_deref(), Some("hi"));
        assert_eq!(event.raw()["future"], "kept");
        let received = received.await.unwrap();
        let body: serde_json::Value =
            serde_json::from_str(received.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["stream"], true);
    }
}
