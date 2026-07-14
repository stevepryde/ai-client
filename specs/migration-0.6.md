# Migrating OpenAI Responses to endpoint model configs

The recommended Responses API now separates reusable request content from the
model-specific configuration passed to the endpoint.

## Basic request

Before:

```rust,ignore
let request = ResponseRequest::<Gpt5_2>::builder()
    .input_text("hello")
    .reasoning(ExtendedReasoningEffort::High)
    .build();
let response = client.responses().create_prepared(request).await?;
```

After:

```rust,ignore
let request = CreateResponseRequest::builder()
    .input_text("hello")
    .build();
let model = Gpt5_2::config().reasoning(ExtendedReasoningEffort::High);
let response = client.responses().create(model, request).await?;
```

`CreateResponseRequest` owns request content and endpoint controls.
`ResponseModelConfig` owns model-specific controls:

- reasoning effort and reasoning details;
- sampling (`temperature`, `top_p`, and `top_logprobs`);
- prompt-cache retention.

Prompt-cache keys, structured output, item input, tools, and tool controls stay
on the request because they describe the call. Their type markers are checked
against the selected model by `create` or `create_stream`.

## Runtime model selection

Build the request once, then select a checked model handle:

```rust,ignore
let request = CreateResponseRequest::builder()
    .instructions(instructions)
    .input_items(input)
    .json_schema(schema)
    .max_output_tokens(2_000)
    .build();

let model: Box<dyn ResponseModelFor<_>> = match model {
    AppModel::Fast => Box::new(Gpt5_4Nano::config().reasoning_none()),
    AppModel::Strong => Box::new(
        Gpt5_2::config().reasoning(ExtendedReasoningEffort::High),
    ),
};

let response = client.responses().create(model, request).await?;
```

`ResponseModelFor<Request>` is object-safe. `Box` or `Arc` can therefore erase
different model markers and reasoning/sampling typestates after the compiler
proves that each config supports this request. Adding an incompatible model
causes its trait-object coercion to fail to compile.

This is also the escape hatch for conditional typestate changes:

```rust,ignore
let model: Box<dyn ResponseModelFor<_>> = if use_reasoning {
    Box::new(Gpt5_2::config().reasoning(ExtendedReasoningEffort::High))
} else {
    Box::new(Gpt5_2::config().temperature(Temperature::new(0.4)?))
};
```

Use a concrete config when no erasure is needed. Pass `&config` to keep a
reusable concrete handle, or use `Arc<dyn ResponseModelFor<_>>` for a shared
dynamic handle.

Do not erase model configs into a string-backed runtime struct. That loses the
type relationship used to reject unsupported model/request combinations before
deployment.

## Custom and fine-tuned models

Known markers expose `GptX::config()`. A downstream marker starts directly:

```rust,ignore
struct FineTuned;

impl OpenAIResponsesModel for FineTuned {
    const ID: &'static str = "ft:gpt-5:local";
}

impl SupportsReasoning for FineTuned {
    type Effort = LocalEffort;
}

let model = ResponseModelConfig::<FineTuned>::new().reasoning(LocalEffort::High);
```

Opt into request capabilities with the existing `SupportsItemInput`,
`SupportsStructuredOutput`, `SupportsPromptCacheKey`, `SupportsTool<T>`, and
`SupportsTools` traits.

## Low-level migration path

The old model-bound `ResponseRequest::<M>::builder()` still produces a
`PreparedResponseRequest`. It can temporarily be sent through
`create_prepared` or `create_prepared_stream`, but new application code should
use endpoint model configs. The prepared methods are hidden from the primary
documentation and are intended to be removed in a later cleanup.
