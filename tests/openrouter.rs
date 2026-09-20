//! Fixture from https://openrouter.ai/openapi.json, POST /responses, 2026-09-20.
//! Exercises HTTP/SSE decoding against the provider's example (nullable metadata,
//! omitted reasoning/truncation/logprobs), rather than our own response serializer.
use ai_client::{
    openai::responses::CreateResponseRequest,
    openrouter::{Euryale70B, OpenRouterClient},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn server(body: String, content_type: &str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = Vec::new();
        loop {
            let mut chunk = [0; 4096];
            let count = socket.read(&mut chunk).await.unwrap();
            assert!(count > 0);
            bytes.extend_from_slice(&chunk[..count]);
            if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                let length: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if bytes.len() >= end + 4 + length {
                    break;
                }
            }
        }
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8(bytes).unwrap()
    });
    (format!("http://{address}/api/v1"), task)
}

#[tokio::test]
async fn decodes_documented_response_and_sends_responses_request() {
    let (url, received) = server(
        include_str!("fixtures/openrouter-response.json").into(),
        "application/json",
    )
    .await;
    let client = OpenRouterClient::builder()
        .api_key("test-router-key".into())
        .base_url(url)
        .zdr(true)
        .build()
        .unwrap();
    let response = client
        .responses()
        .create(
            Euryale70B::config(),
            CreateResponseRequest::builder()
                .input_text("Tell me a joke")
                .build(),
        )
        .await
        .unwrap()
        .into_inner();
    assert_eq!(response.usage.unwrap().output_tokens, 20);
    assert_eq!(response.output.len(), 1);
    let received = received.await.unwrap();
    let body: serde_json::Value =
        serde_json::from_str(received.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    // OpenRouter ResponsesRequest.provider uses ProviderPreferences.zdr.
    assert_eq!(body["provider"]["zdr"], true);
    assert!(received.starts_with("POST /api/v1/responses HTTP/1.1"));
    assert!(received
        .to_lowercase()
        .contains("authorization: bearer test-router-key"));
}

#[cfg(feature = "stream")]
#[tokio::test]
async fn decodes_router_text_and_completion_over_sse() {
    use ai_client::openai::responses::OpenAIResponsesStreamEvent;
    use futures::TryStreamExt;
    let response: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/openrouter-response.json")).unwrap();
    let complete =
        serde_json::json!({"type":"response.completed","sequence_number":5,"response":response});
    let body = format!("data: {{\"type\":\"response.output_text.delta\",\"sequence_number\":4,\"item_id\":\"item-1\",\"output_index\":0,\"content_index\":0,\"delta\":\"Hello\",\"logprobs\":[]}}\n\ndata: {complete}\n\ndata: [DONE]\n\n");
    let (url, received) = server(body, "text/event-stream").await;
    let client = OpenRouterClient::builder()
        .api_key("test-router-key".into())
        .base_url(url)
        .zdr(true)
        .build()
        .unwrap();
    let events: Vec<_> = client
        .responses()
        .create_stream(
            Euryale70B::config(),
            CreateResponseRequest::builder().input_text("Hello").build(),
        )
        .await
        .unwrap()
        .into_inner()
        .try_collect()
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert!(
        matches!(events[0].data(), OpenAIResponsesStreamEvent::OutputTextDelta(event) if event.delta == "Hello")
    );
    assert!(
        matches!(events[1].data(), OpenAIResponsesStreamEvent::ResponseDone(event) if event.response.usage.as_ref().unwrap().input_tokens == 10)
    );
    let body: serde_json::Value =
        serde_json::from_str(received.await.unwrap().split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["stream"], true);
    assert_eq!(body["provider"]["zdr"], true);
}
