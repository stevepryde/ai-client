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
  - Full pinned Responses resource operations and typed protocol coverage
  - Conversations state and item operations used by Responses
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
use ai_client::openai::responses::{Gpt5Mini, ResponseRequest};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenAIClient::builder()
    .api_key(std::env::var("OPENAI_API_KEY")?)
    .build()?;
let request = ResponseRequest::<Gpt5Mini>::builder()
    .input_text("Explain typed builders briefly.")
    .build();

let response = client.responses().create(request).await?;
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
the same private `PreparedResponseRequest` before transport. The older
`OpenAIClient::generate_response*` methods remain forwarding methods for
migration; new code should use the borrowed `client.responses()` resource.
See [`specs/migration-0.4.md`](specs/migration-0.4.md) for migration examples.

Stored responses use validated opaque IDs and encoded path segments:

```no_run
use ai_client::openai::OpenAIClient;
use ai_client::openai::responses::ResponseId;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = OpenAIClient::builder()
    .api_key(std::env::var("OPENAI_API_KEY")?)
    .build()?;
let id = ResponseId::new("resp_123")?;
let response = client.responses().retrieve(&id).await?;
println!("status: {:?}", response.data().status);
# Ok(())
# }
```

### Native OpenAI scope

| Resource | Status |
| --- | --- |
| Responses | 7/7 pinned operations, including distinct create/retrieve streaming methods |
| Conversations | 8/8 pinned operations for conversation and nested item state |
| Standalone Images | Next planned native API resource |
| Files, Audio, Realtime, Batches, Videos, administration/control-plane | Deferred and out of the active product scope |

Responses protocol types still represent documented file/audio/tool content and
stream events where the Responses API itself requires them. That does not imply
standalone support for those other API resources.

### Streaming Support

To enable streaming support, add the `stream` feature to your `Cargo.toml`:

```toml
[dependencies]
ai_client = { version = "0.1", features = ["stream"] }
```

Streaming is available via:
- `GeminiClient::generate_content_streamed()` for Gemini
- `OpenAIClient::responses().create_stream()` and `retrieve_stream()` for OpenAI Responses
- `OpenAIClient::generate_response_streamed()` as a migration forwarding method
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

## Product direction

Provider-native APIs remain full fidelity. OpenAI work is intentionally focused
on Responses and standalone Images; OpenAI-compatible Chat Completions remains a
separate typed dialect family rather than a lowest-common-denominator provider
interface.

## Minimum Supported Rust Version

The MSRV for this crate is likely to be close to the latest at least for now.

## Live provider tests

The normal test suite never contacts an AI provider. Real-provider coverage has
two deliberate gates: compile it with the default-off `live-tests` feature, then
select ignored tests with `--ignored`. This keeps routine `cargo test` and
`cargo test --all-features` runs token-free.

```bash
# OpenAI's cheap core operations and tiny prompts
cargo test --all-features --test live_openai live_openai_core \
  -- --ignored --test-threads=1

# Gemini's cheap core operations and tiny prompts
cargo test --all-features --test live_gemini live_gemini_core \
  -- --ignored --test-threads=1
```

The model matrices, entitlement-dependent options, provisioned resources, and
image/hosted-tool tests are separate filters so their cost and prerequisites
are explicit. See [`tests/LIVE_PROVIDERS.md`](tests/LIVE_PROVIDERS.md) for the
environment variables, coverage map, and exact commands. An explicitly
selected live test fails with a clear error when its credential or resource
environment is missing; it does not silently pass without testing anything.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
