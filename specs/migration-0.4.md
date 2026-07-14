# ai-client 0.3 to 0.4 migration

Status: draft

> This document describes the historical 0.4 model-bound builder. For the
> current endpoint model-config API, continue with
> [`migration-0.6.md`](migration-0.6.md).

## OpenAI Responses requests

Direct construction of `OpenAIResponsesCreateRequest` is replaced by a typed
known-model builder. It exposes only settings supported by its model marker;
input is optional so conversation-linked, continuation, and reusable-prompt
requests remain representable:

```rust,ignore
use ai_client::openai::responses::{Gpt5, Gpt5ReasoningEffort, ResponseRequest};

let request = ResponseRequest::<Gpt5>::builder()
    .input_text("Hello")
    .reasoning(Gpt5ReasoningEffort::High)
    .build();
let response = client.responses().create_prepared(request).await?;
```

For a model ID selected at runtime, use the explicit dynamic path. Validation
never deletes or coerces configured fields. Like the known-model builder, the
dynamic builder can build an inputless request when another configured field
supplies the context:

```rust,ignore
use ai_client::openai::responses::{
    DynamicOpenAIModel, DynamicResponseRequest, ValidationMode,
};

let model = DynamicOpenAIModel::new(configured_model_id)?;
let request = DynamicResponseRequest::builder(model)
    .input_text("Hello")
    .validation(ValidationMode::Warn)
    .builtin_catalog()
    .build()?;
for warning in request.warnings() {
    // Surface or record the explicit validation warning.
}
let response = client.responses().create_prepared(request).await?;
```

`OpenAIModel` remains available for model retrieval and the default-off legacy
Chat Completions API. It is no longer the native Responses request model type.

## Streaming

The same `PreparedResponseRequest` can be passed to
`client.responses().create_prepared_stream(...)`. The resource owns the wire `stream`
switch; callers do not set it on the request. The old `generate_response*`
methods remain forwarding methods during migration.

## Native OpenAI Chat Completions

The default-off native OpenAI Chat Completions module, its primary
request/response/event types, and `OpenAIClient::generate_content*` methods are
deprecated since 0.4.0. OpenAI integrations should move to the Responses API
shown above.

Third-party endpoints that implement a Chat-Completions-shaped protocol should
move to the separate `openai-compatible` feature:

```rust,ignore
use ai_client::openai_compatible::{
    chat::{ChatMessage, ChatRole, DynamicChatModel, DynamicChatRequest},
    CompatibleAuth, CustomDialect, OpenAICompatibleClient,
};

let client = OpenAICompatibleClient::<CustomDialect>::builder()
    .base_url(config.compatible_base_url)
    .auth(CompatibleAuth::bearer(config.compatible_api_key))
    .build()?;
let model = DynamicChatModel::new(config.model)?;
let request = DynamicChatRequest::<CustomDialect>::builder(model)
    .messages(vec![ChatMessage::new(ChatRole::User, "Hello")])
    .build()?;
let response = client.chat().create(request).await?;
```

`CustomDialect` is not a conformance claim. The dynamic request path performs
structural validation but cannot promise a runtime model supports any optional
setting. Applications that know their endpoint contract can define typed model
markers and implement the exact capability traits they support; unsupported
builder methods then fail at compile time. Requests built for one dialect
cannot be passed to another dialect's client.

`CustomDialect` supports simple text messages through `ChatMessage::new` and
preserves richer multimodal/tool/provider messages through
`ChatMessage::from_object`. A downstream dialect can define its own associated
serializable message type; it is checked to serialize as an object before the
generic builder erases it into the private wire request.
