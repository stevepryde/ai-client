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
  - Deprecated legacy chat completions (with `chat-completions` feature)

- **OpenAI-compatible APIs**: A separate typed Chat Completions family
  - Explicit custom endpoint and authentication configuration
  - Provider/dialect and model capability bounds
  - Streaming responses (with both `openai-compatible` and `stream`)

## OpenAI Responses

OpenAI Responses is the primary API for new OpenAI integrations. Successful
calls return `AiResponse<T>`, which keeps the provider body and HTTP metadata
together:

```no_run
use ai_client::openai::OpenAIClient;
use ai_client::openai::create_response::OpenAIResponsesInput;
use ai_client::openai::responses::{Gpt5Mini, ResponseRequest};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenAIClient::builder()
    .api_key(std::env::var("OPENAI_API_KEY")?)
    .build()?;
let request = ResponseRequest::<Gpt5Mini>::builder()
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

Known-model builders expose only settings supported by their model marker. Use
`DynamicResponseRequest` with an explicit `ValidationMode` for model IDs loaded
from configuration or released after this crate version. Both paths erase into
the same private `PreparedResponseRequest` before transport.
See [`specs/migration-0.4.md`](specs/migration-0.4.md) for migration examples.

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
default and deprecated since 0.4.0. Enable them only while migrating a downstream
app that intentionally needs the native legacy API:

```toml
[dependencies]
ai_client = { version = "0.1", features = ["chat-completions"] }
```

For OpenAI-shaped third-party endpoints, use the separate
`openai-compatible` feature instead. `CustomDialect` does not claim that an
endpoint implements any particular option: callers provide model markers and
capability implementations for the contract they have verified.

## OpenAI-compatible Chat Completions

The compatibility family preserves a Chat-Completions-shaped protocol without
making it part of native OpenAI. Base URL and authentication are always
explicit, and the prepared request remains bound to its dialect:

```rust,ignore
use ai_client::openai_compatible::{
    chat::{ChatMessage, ChatRole, DynamicChatModel, DynamicChatRequest},
    CompatibleAuth, CustomDialect, OpenAICompatibleClient,
};

let client = OpenAICompatibleClient::<CustomDialect>::builder()
    .base_url("http://localhost:8080/v1")
    .auth(CompatibleAuth::bearer(std::env::var("COMPATIBLE_API_KEY")?))
    .build()?;
let model = DynamicChatModel::new("my-runtime-model")?;
let request = DynamicChatRequest::<CustomDialect>::builder(model)
    .messages(vec![ChatMessage::new(ChatRole::User, "Hello")])
    .build()?;
let response = client.chat().create(request).await?;
println!("{}", response.data().id());
```

The dynamic builder validates only structural safety and intentionally makes no
model-capability guarantees. For compile-time checking, define a
`CompatibleChatModel<CustomDialect>` marker and implement only the relevant
capability traits. `extra_body` is an explicit forward-compatibility escape
hatch; collisions with typed or dialect option fields are rejected. Simple
messages use `ChatMessage::new`; multimodal, tool, and endpoint-specific
messages use the explicit object-preserving `ChatMessage::from_object` path.
Downstream dialects can instead define their own associated message type.

## High level plans (if it ever gets there)

Support various LLMs using a simple interface.

Currently targeting Gemini but it would be cool to add others.

## Minimum Supported Rust Version

The MSRV for this crate is likely to be close to the latest at least for now.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
