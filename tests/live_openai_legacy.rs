#![cfg(all(feature = "live-tests", feature = "chat-completions"))]
#![allow(deprecated)]

use ai_client::openai::{
    create_chat_completion::{OpenAIGenerateContentRequest, OpenAIResponseFormat},
    OpenAIClient, OpenAIJsonSchema, OpenAIModel, OpenAIPrompt, OpenAIRole,
};

fn client() -> OpenAIClient {
    OpenAIClient::builder()
        .api_key(std::env::var("OPENAI_API_KEY").expect(
            "OPENAI_API_KEY is required because an ignored live-provider test was requested",
        ))
        .build()
        .unwrap()
}

fn request() -> OpenAIGenerateContentRequest {
    OpenAIGenerateContentRequest {
        model: OpenAIModel::Gpt4_1Mini,
        messages: vec![OpenAIPrompt {
            role: OpenAIRole::User,
            content: "Return JSON where ok is true.".into(),
        }],
        frequency_penalty: Some(0.0),
        max_completion_tokens: Some(32),
        n: Some(1),
        modalities: Some(vec!["text".into()]),
        response_format: Some(OpenAIResponseFormat::JsonSchema {
            json_schema: OpenAIJsonSchema {
                name: "live_result".into(),
                description: "Tiny legacy live-test result".into(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }),
                strict: Some(true),
            },
        }),
        temperature: Some(0.0),
        top_p: Some(0.9),
        stream: None,
        reasoning_effort: None,
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; deprecated chat-completions feature only"]
async fn live_openai_legacy_chat_all_request_options() {
    let response = client()
        .generate_content(request())
        .await
        .expect("legacy Chat Completions request should remain provider-valid")
        .into_inner();
    assert!(!response.choices.is_empty());
}

#[cfg(feature = "stream")]
#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY plus chat-completions and stream features"]
async fn live_openai_legacy_chat_streaming() {
    use futures::StreamExt;

    let response = client()
        .generate_content_streamed(request())
        .await
        .expect("legacy Chat Completions stream should connect");
    let events = response
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("legacy Chat Completions SSE should decode");
    assert!(!events.is_empty());
}
