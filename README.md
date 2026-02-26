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
  - Text generation (chat completions)
  - Alternative response generation methods
  - Model listing
  - Streaming responses (with `stream` feature)

### Streaming Support

To enable streaming support, add the `stream` feature to your `Cargo.toml`:

```toml
[dependencies]
ai_client = { version = "0.1", features = ["stream"] }
```

Streaming is available via:
- `GeminiClient::generate_content_streamed()` for Gemini
- `OpenAIClient::generate_content_streamed()` for OpenAI chat completions
- `OpenAIClient::generate_response_streamed()` for OpenAI (alternative method)

Both methods return a `Stream` of response chunks that can be processed incrementally.

## High level plans (if it ever gets there)

Support various LLMs using a simple interface.

Currently targeting Gemini but it would be cool to add others.

## Minimum Supported Rust Version

The MSRV for this crate is likely to be close to the latest at least for now.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
