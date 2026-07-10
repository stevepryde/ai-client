# ai-client
Client API for accessing various AI services

This is highly experimental while I figure out a nice API.

It also needs a catchy name.

## Features

- **Gemini API**: Support for Google's Gemini API
  - Text generation
  - Token counting
  - Model listing
  - Streaming responses (with `stream` feature)

- **OpenAI API**: Support for OpenAI's API
  - Response generation via the Responses API
  - Model listing
  - Streaming responses (with `stream` feature)
  - Legacy chat completions (with `chat-completions` feature)

## OpenAI Responses

OpenAI Responses is the primary API for new OpenAI integrations. Successful
calls return `AiResponse<T>`, which keeps the provider body and HTTP metadata
together:

```no_run
use ai_client::openai::{OpenAIClient, OpenAIModel};
use ai_client::openai::create_response::{
    OpenAIResponsesCreateRequest, OpenAIResponsesInput,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenAIClient::builder()
    .api_key(std::env::var("OPENAI_API_KEY")?)
    .build()?;
let request = OpenAIResponsesCreateRequest::builder()
    .model(OpenAIModel::Gpt5Mini)
    .input(OpenAIResponsesInput::Text("Explain typed builders briefly.".into()))
    .build();

let response = client.generate_response(request).await?;
println!("response id: {}", response.data().id);
println!("request id: {:?}", response.metadata().request_id);

let response_body = response.into_inner();
println!("status: {:?}", response_body.status);
# Ok(())
# }
```

### Streaming Support

To enable streaming support, add the `stream` feature to your `Cargo.toml`:

```toml
[dependencies]
ai_client = { version = "0.1", features = ["stream"] }
```

Streaming is available via:
- `GeminiClient::generate_content_streamed()` for Gemini
- `OpenAIClient::generate_response_streamed()` for OpenAI Responses
- `OpenAIClient::generate_content_streamed()` for legacy OpenAI chat completions when both `stream` and `chat-completions` are enabled

Streaming methods return `AiResponse<AiStream<_>>`. The outer response exposes
request and rate-limit metadata from the successful HTTP handshake; its inner
`AiStream` yields crate-owned `AiStreamError` values. OpenAI SSE items are
`SseJsonEvent<T>`, preserving event metadata and the complete raw JSON value
alongside typed provider data. Read `response.metadata()` before calling
`response.into_inner()` to obtain and poll the stream.

### Legacy OpenAI Chat Completions

OpenAI recommends the Responses API for new work, so chat completions are disabled by
default. Enable them only when a downstream app intentionally needs the legacy API:

```toml
[dependencies]
ai_client = { version = "0.1", features = ["chat-completions"] }
```

## High level plans (if it ever gets there)

Support various LLMs using a simple interface.

Currently targeting Gemini but it would be cool to add others.

## Minimum Supported Rust Version

The MSRV for this crate is likely to be close to the latest at least for now.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
