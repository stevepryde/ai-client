#![cfg(all(feature = "live-tests", feature = "openai-compatible"))]

use std::num::NonZeroU32;

use ai_client::openai_compatible::{
    chat::{
        ChatFrequencyPenalty, ChatMessage, ChatModality, ChatResponseFormat, ChatRole,
        ChatTemperature, ChatTopP, DynamicChatModel, DynamicChatRequest,
    },
    CompatibleAuth, CustomDialect, OpenAICompatibleClient,
};

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!("{name} is required because an explicitly ignored live-provider test was requested")
    })
}

fn client() -> OpenAICompatibleClient<CustomDialect> {
    OpenAICompatibleClient::builder()
        .base_url(required_env("AI_CLIENT_COMPATIBLE_BASE_URL"))
        .auth(CompatibleAuth::bearer(required_env(
            "AI_CLIENT_COMPATIBLE_API_KEY",
        )))
        .build()
        .expect("compatible-provider environment should build a client")
}

fn model() -> DynamicChatModel {
    DynamicChatModel::new(required_env("AI_CLIENT_COMPATIBLE_MODEL"))
        .expect("AI_CLIENT_COMPATIBLE_MODEL should be a valid model ID")
}

#[tokio::test]
#[ignore = "live provider: requires AI_CLIENT_COMPATIBLE_BASE_URL/API_KEY/MODEL"]
async fn live_compatible_core_chat_completion() {
    let request = DynamicChatRequest::<CustomDialect>::builder(model())
        .messages(vec![ChatMessage::new(
            ChatRole::User,
            "Reply with only OK.",
        )])
        .max_completion_tokens(16)
        .temperature(ChatTemperature::new(0.0).unwrap())
        .top_p(ChatTopP::new(0.9).unwrap())
        .build()
        .unwrap();
    let response = client()
        .chat()
        .create(request)
        .await
        .expect("configured OpenAI-compatible provider should accept a basic request")
        .into_inner();
    assert!(!response.choices().is_empty());
}

#[tokio::test]
#[ignore = "live provider: configured model must support every optional Chat Completions field"]
async fn live_compatible_option_matrix() {
    let request = DynamicChatRequest::<CustomDialect>::builder(model())
        .messages(vec![ChatMessage::new(
            ChatRole::User,
            "Return a JSON object with ok=true.",
        )])
        .max_completion_tokens(64)
        .temperature(ChatTemperature::new(0.0).unwrap())
        .top_p(ChatTopP::new(0.9).unwrap())
        .frequency_penalty(ChatFrequencyPenalty::new(0.0).unwrap())
        .choice_count(NonZeroU32::new(1).unwrap())
        .response_format(ChatResponseFormat::new(serde_json::Map::from_iter([(
            "type".into(),
            serde_json::Value::String("json_object".into()),
        )])))
        .modalities(vec![ChatModality::new("text").unwrap()])
        .reasoning_effort("low")
        .build()
        .unwrap();
    let response = client()
        .chat()
        .create(request)
        .await
        .expect("configured compatible model should accept every advertised dynamic option")
        .into_inner();
    assert!(!response.choices().is_empty());
}

#[cfg(feature = "stream")]
#[tokio::test]
#[ignore = "live provider: requires compatible-provider environment and the stream feature"]
async fn live_compatible_core_streaming() {
    use futures::StreamExt;

    let request = DynamicChatRequest::<CustomDialect>::builder(model())
        .messages(vec![ChatMessage::new(
            ChatRole::User,
            "Reply with only OK.",
        )])
        .max_completion_tokens(16)
        .build()
        .unwrap();
    let response = client()
        .chat()
        .create_stream(request)
        .await
        .expect("compatible-provider streaming handshake should succeed");
    let events = response
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("compatible-provider SSE should decode");
    assert!(!events.is_empty());
}
