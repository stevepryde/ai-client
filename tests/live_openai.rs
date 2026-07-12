#![cfg(feature = "live-tests")]

use std::collections::{BTreeMap, HashSet};

use ai_client::openai::{
    conversations::{
        ConversationItemIncludeOptions, ConversationMetadata, CreateConversationItemsRequest,
        CreateConversationRequest, ListConversationItemsOptions, UpdateConversationRequest,
    },
    responses::*,
    OpenAIClient,
};

#[path = "live_openai/options.rs"]
mod options;

fn client() -> OpenAIClient {
    let mut builder = OpenAIClient::builder().api_key(required_env("OPENAI_API_KEY"));
    if let Ok(organization) = std::env::var("OPENAI_ORGANIZATION") {
        builder = builder.organization(organization);
    }
    if let Ok(project) = std::env::var("OPENAI_PROJECT") {
        builder = builder.project(project);
    }
    builder
        .build()
        .expect("OpenAI credential environment should build a client")
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!("{name} is required because an explicitly ignored live-provider test was requested")
    })
}

fn tiny_request<M: OpenAIResponsesModel>() -> PreparedResponseRequest {
    ResponseRequest::<M>::builder()
        .input_text("Reply with only OK.")
        .max_output_tokens(16)
        .store(false)
        .build()
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; metadata calls do not spend generation tokens"]
async fn live_openai_core_catalog_covers_every_supported_response_model() {
    let client = client();
    let response = client
        .list_models()
        .await
        .expect("OpenAI models.list should decode the provider's data envelope")
        .into_inner();
    let listed: HashSet<_> = response
        .models
        .iter()
        .map(|model| model.id.as_str())
        .collect();
    for model in GENERALLY_AVAILABLE_RESPONSE_MODEL_IDS {
        assert!(
            listed.contains(model),
            "supported OpenAI Responses model {model} was not visible to this project"
        );
    }
}

#[cfg(feature = "stream")]
#[tokio::test]
#[ignore = "live provider: streams a tiny response from GPT-5.1, GPT-5.2, GPT-5.4, and GPT-5.5"]
async fn live_openai_model_matrix_streams_with_representative_families() {
    use futures::StreamExt;

    let client = client();
    let requests = [
        (Gpt5_1::ID, tiny_request::<Gpt5_1>()),
        (Gpt5_2::ID, tiny_request::<Gpt5_2>()),
        (Gpt5_4::ID, tiny_request::<Gpt5_4>()),
        (Gpt5_5::ID, tiny_request::<Gpt5_5>()),
    ];
    for (model, request) in requests {
        let events = client
            .responses()
            .create_stream(request)
            .await
            .unwrap_or_else(|error| panic!("Responses create_stream failed for {model}: {error}"))
            .into_inner()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("Responses stream decode failed for {model}: {error}"));
        let response = events
            .iter()
            .find_map(|event| match event.data() {
                OpenAIResponsesStreamEvent::ResponseDone(done)
                | OpenAIResponsesStreamEvent::ResponseIncomplete(done) => Some(&done.response),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("Responses stream for {model} had no successful terminal event")
            });
        assert!(response.model == model || response.model.starts_with(&format!("{model}-")));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and validates inexpensive create options"]
async fn live_openai_core_create_options_are_accepted_together() {
    let function: OpenAIFunctionTool = serde_json::from_value(serde_json::json!({
        "name": "do_nothing",
        "description": "A function that is deliberately not called.",
        "strict": false,
        "parameters": {"type": "object", "properties": {}, "additionalProperties": false}
    }))
    .unwrap();
    let text: OpenAIResponsesTextConfig = serde_json::from_value(serde_json::json!({
        "format": {
            "type": "json_schema",
            "name": "live_result",
            "description": "A tiny live-test result.",
            "strict": true,
            "schema": {
                "type": "object",
                "properties": {"ok": {"type": "boolean"}},
                "required": ["ok"],
                "additionalProperties": false
            }
        }
    }))
    .unwrap();
    let metadata = OpenAIResponseMetadata::new([("suite", "ai-client-live")]).unwrap();
    let request = ResponseRequest::<Gpt4_1>::builder()
        .input_text("Return JSON where ok is true. Do not call a tool.")
        .instructions("Follow the requested output schema exactly.")
        .metadata(metadata)
        .top_logprobs(TopLogprobs::new(1).unwrap())
        .temperature(Temperature::new(0.0).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .max_output_tokens(64)
        .safety_identifier("ai-client-live-tests")
        .service_tier(OpenAIServiceTier::Auto)
        .prompt_cache_key("ai-client-live-core-options")
        .text_config(text)
        .tool(function)
        .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
        .store(false)
        .background(false)
        .max_tool_calls(1)
        .truncation(OpenAITruncation::Disabled)
        .include(ResponseInclude::MessageOutputTextLogprobs)
        .parallel_tool_calls(false)
        .build();

    let response = client()
        .responses()
        .create(request)
        .await
        .expect("OpenAI should accept the complete inexpensive create option set")
        .into_inner();
    assert!(matches!(response.status, OpenAIResponseStatus::Completed));
    assert_eq!(
        response.safety_identifier.as_deref(),
        Some("ai-client-live-tests")
    );
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and spends one tiny call per reasoning option"]
async fn live_openai_option_matrix_accepts_every_reasoning_effort() {
    let client = client();
    for (label, request) in [
        (
            "gpt-5.1/none",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .store(false)
                .build(),
        ),
        (
            "gpt-5.1/low",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_1ReasoningEffort::Low)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.1/medium",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_1ReasoningEffort::Medium)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.1/high",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_1ReasoningEffort::High)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/none",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/low",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(ExtendedReasoningEffort::Low)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/medium",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(ExtendedReasoningEffort::Medium)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/high",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(ExtendedReasoningEffort::High)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/xhigh",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(ExtendedReasoningEffort::XHigh)
                .store(false)
                .build(),
        ),
    ] {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("reasoning option {label} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and validates every advertised cache retention"]
async fn live_openai_option_matrix_accepts_prompt_cache_settings() {
    let client = client();
    for (label, request) in [
        (
            "gpt-5.1/in_memory",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.1-in-memory")
                .prompt_cache_retention(PromptCacheRetention::InMemory)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.1/24h",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.1-24h")
                .prompt_cache_retention(PromptCacheRetention::Hours24)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.2/in_memory",
            ResponseRequest::<Gpt5_2>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.2-in-memory")
                .prompt_cache_retention(PromptCacheRetention::InMemory)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.2/24h",
            ResponseRequest::<Gpt5_2>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.2-24h")
                .prompt_cache_retention(PromptCacheRetention::Hours24)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/in_memory",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.4-in-memory")
                .prompt_cache_retention(PromptCacheRetention::InMemory)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/24h",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .prompt_cache_key("ai-client-live-gpt-5.4-24h")
                .prompt_cache_retention(PromptCacheRetention::Hours24)
                .store(false)
                .build(),
        ),
    ] {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("cache option {label} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; covers stored response operations and continuation"]
async fn live_openai_core_response_resource_lifecycle() {
    let client = client();
    let count = client
        .responses()
        .count_input_tokens(&OpenAIInputTokenCountRequest {
            model: Some("gpt-5.4-mini".into()),
            input: Some(OpenAIResponsesInput::Text("Reply with only OK.".into())),
            instructions: Some("Be concise.".into()),
            truncation: Some(OpenAITruncation::Disabled),
            parallel_tool_calls: Some(false),
            ..Default::default()
        })
        .await
        .expect("responses.input_tokens.count should succeed")
        .into_inner();
    assert!(count.input_tokens > 0);

    let first = client
        .responses()
        .create(
            ResponseRequest::<Gpt5_4Mini>::builder()
                .input_text("Reply with only ONE.")
                .max_output_tokens(32)
                .store(true)
                .build(),
        )
        .await
        .expect("stored response create should succeed")
        .into_inner();
    let first_id = first.id.clone();
    let retrieved = client
        .responses()
        .retrieve_with_options(
            &first_id,
            &RetrieveResponseOptions::new().include(ResponseInclude::MessageOutputTextLogprobs),
        )
        .await
        .expect("stored response retrieve should succeed")
        .into_inner();
    assert_eq!(retrieved.id, first_id);

    let items = client
        .responses()
        .list_input_items(
            &first_id,
            &ListResponseInputItemsOptions::new()
                .limit(10)
                .unwrap()
                .order(ListOrder::Asc),
        )
        .await
        .expect("stored response input item list should succeed")
        .into_inner();
    assert!(!items.data.is_empty());

    let second = client
        .responses()
        .create(
            ResponseRequest::<Gpt5_4Mini>::builder()
                .input_text("Reply with only TWO.")
                .previous_response_id(first_id.clone())
                .max_output_tokens(32)
                .store(true)
                .build(),
        )
        .await
        .expect("response continuation should succeed")
        .into_inner();
    assert_eq!(second.previous_response_id.as_ref(), Some(&first_id));

    client.responses().delete(&second.id).await.unwrap();
    client.responses().delete(&first_id).await.unwrap();
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; covers all Conversations CRUD operations"]
async fn live_openai_core_conversation_resource_lifecycle() {
    let client = client();
    let input: OpenAIResponseInputItem = serde_json::from_value(serde_json::json!({
        "type": "message",
        "role": "user",
        "content": "Hello"
    }))
    .unwrap();
    let metadata = ConversationMetadata::new(BTreeMap::from([(
        "suite".to_string(),
        "ai-client-live".to_string(),
    )]))
    .unwrap();
    let conversation = client
        .conversations()
        .create(&CreateConversationRequest::new().metadata(metadata))
        .await
        .expect("conversation create should succeed")
        .into_inner();
    let id = conversation.id.clone();

    let retrieved = client
        .conversations()
        .retrieve(&id)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(retrieved.id, id);
    client
        .conversations()
        .update(
            &id,
            &UpdateConversationRequest::new(ConversationMetadata::empty()),
        )
        .await
        .expect("conversation update should succeed");

    let created_items = client
        .conversations()
        .items()
        .create(
            &id,
            &CreateConversationItemsRequest::new(vec![input]).unwrap(),
            &ConversationItemIncludeOptions::new(),
        )
        .await
        .expect("conversation item create should succeed")
        .into_inner();
    let item_id = ResponseItemId::new(
        serde_json::to_value(&created_items.data[0]).unwrap()["id"]
            .as_str()
            .expect("created conversation item should have an id"),
    )
    .unwrap();
    let listed = client
        .conversations()
        .items()
        .list(
            &id,
            &ListConversationItemsOptions::new()
                .limit(10)
                .unwrap()
                .order(ListOrder::Asc),
        )
        .await
        .expect("conversation item list should succeed")
        .into_inner();
    assert!(!listed.data.is_empty());
    client
        .conversations()
        .items()
        .retrieve(&id, &item_id, &ConversationItemIncludeOptions::new())
        .await
        .expect("conversation item retrieve should succeed");
    client
        .conversations()
        .items()
        .delete(&id, &item_id)
        .await
        .expect("conversation item delete should succeed");
    let deleted = client
        .conversations()
        .delete(&id)
        .await
        .expect("conversation delete should succeed")
        .into_inner();
    assert!(deleted.deleted);
}

#[cfg(feature = "stream")]
#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and the stream feature"]
async fn live_openai_core_create_stream_decodes_provider_events() {
    use futures::StreamExt;

    let response = client()
        .responses()
        .create_stream(
            ResponseRequest::<Gpt5_4Mini>::builder()
                .input_text("Reply OK.")
                .build(),
        )
        .await
        .expect("Responses streaming handshake should succeed");
    let events = response
        .into_inner()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Responses SSE body should decode");
    assert!(events
        .iter()
        .any(|event| { matches!(event.data(), OpenAIResponsesStreamEvent::ResponseDone(_)) }));
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; Flex and Priority may require project entitlement"]
async fn live_openai_entitled_service_tier_matrix() {
    let client = client();
    for tier in [
        OpenAIServiceTier::Auto,
        OpenAIServiceTier::Default,
        OpenAIServiceTier::Flex,
        OpenAIServiceTier::Priority,
    ] {
        let request = ResponseRequest::<Gpt5_4Mini>::builder()
            .input_text("Reply OK.")
            .max_output_tokens(16)
            .service_tier(tier)
            .store(false)
            .build();
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("service tier {tier:?} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires an OPENAI_API_KEY project entitled to the GPT-5.6 preview"]
async fn live_openai_entitled_gpt56_model_and_reasoning_matrix() {
    let client = client();
    let listed: HashSet<_> = client
        .list_models()
        .await
        .expect("OpenAI models.list should succeed")
        .into_inner()
        .models
        .into_iter()
        .map(|model| model.id)
        .collect();

    // The unsuffixed ID is a routing alias and is not necessarily returned by
    // models.list. The three concrete preview IDs should be discoverable.
    for model in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        assert!(
            listed.contains(model),
            "preview model {model} was not listed"
        );
    }

    for (model, request) in [
        (Gpt5_6::ID, tiny_request::<Gpt5_6>()),
        (Gpt5_6Sol::ID, tiny_request::<Gpt5_6Sol>()),
        (Gpt5_6Terra::ID, tiny_request::<Gpt5_6Terra>()),
        (Gpt5_6Luna::ID, tiny_request::<Gpt5_6Luna>()),
    ] {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("GPT-5.6 create failed for {model}: {error}"));
    }

    for (effort, request) in [
        (
            "none",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .store(false)
                .build(),
        ),
        (
            "low",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_6ReasoningEffort::Low)
                .store(false)
                .build(),
        ),
        (
            "medium",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_6ReasoningEffort::Medium)
                .store(false)
                .build(),
        ),
        (
            "high",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_6ReasoningEffort::High)
                .store(false)
                .build(),
        ),
        (
            "xhigh",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_6ReasoningEffort::XHigh)
                .store(false)
                .build(),
        ),
        (
            "max",
            ResponseRequest::<Gpt5_6Luna>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning(Gpt5_6ReasoningEffort::Max)
                .store(false)
                .build(),
        ),
    ] {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("GPT-5.6 effort {effort} failed: {error}"));
    }
}
