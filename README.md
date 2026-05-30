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

Both methods return a `Stream` of response chunks that can be processed incrementally.

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
