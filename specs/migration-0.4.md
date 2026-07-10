# ai-client 0.3 to 0.4 migration

Status: draft

## OpenAI Responses requests

Direct construction of `OpenAIResponsesCreateRequest` is replaced by a typed
known-model builder. The builder requires input before `build()` and exposes
only settings supported by its model marker:

```rust,ignore
use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt5, Gpt5ReasoningEffort, ResponseRequest},
};

let request = ResponseRequest::<Gpt5>::builder()
    .input(OpenAIResponsesInput::Text("Hello".into()))
    .reasoning(Gpt5ReasoningEffort::High)
    .build();
let response = client.generate_response(request).await?;
```

For a model ID selected at runtime, use the explicit dynamic path. Validation
never deletes or coerces configured fields. The dynamic builder also requires
`input(...)` before its `build()` method becomes available:

```rust,ignore
use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{DynamicOpenAIModel, DynamicResponseRequest, ValidationMode},
};

let model = DynamicOpenAIModel::new(configured_model_id)?;
let request = DynamicResponseRequest::builder(model)
    .input(OpenAIResponsesInput::Text("Hello".into()))
    .validation(ValidationMode::Warn)
    .builtin_catalog()
    .build()?;
for warning in request.warnings() {
    // Surface or record the explicit validation warning.
}
let response = client.generate_response(request).await?;
```

`OpenAIModel` remains available for model retrieval and the default-off legacy
Chat Completions API. It is no longer the native Responses request model type.

## Streaming

The same `PreparedResponseRequest` can be passed to
`generate_response_streamed`. The client owns the wire `stream` switch; callers
do not set it on the request.
