use ai_client::openai::{responses::*, OpenAIJsonSchema};

use super::client;

fn noop_function() -> OpenAIFunctionTool {
    serde_json::from_value(serde_json::json!({
        "name": "noop",
        "description": "Do nothing",
        "strict": false,
        "parameters": {"type": "object", "properties": {}, "additionalProperties": false}
    }))
    .unwrap()
}

fn schema() -> OpenAIJsonSchema {
    OpenAIJsonSchema {
        name: "live_result".into(),
        description: "A tiny live-test result.".into(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"],
            "additionalProperties": false
        }),
        strict: Some(true),
    }
}

#[tokio::test]
#[ignore = "live provider: tiny GPT-5.1/GPT-5.2/GPT-5.4 calls verify sampling is accepted only in no-reasoning typestates"]
async fn live_openai_gpt51_gpt52_and_gpt54_sampling_in_no_reasoning_modes() {
    let client = client();
    let requests = [
        (
            "gpt-5.1/default-none",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .top_logprobs(TopLogprobs::new(1).unwrap())
                .store(false)
                .build(),
        ),
        (
            "gpt-5.1/explicit-none",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .store(false)
                .build(),
        ),
        (
            "gpt-5.2/default-none",
            ResponseRequest::<Gpt5_2>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .top_logprobs(TopLogprobs::new(1).unwrap())
                .store(false)
                .build(),
        ),
        (
            "gpt-5.2/explicit-none",
            ResponseRequest::<Gpt5_2>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/default-none",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .top_logprobs(TopLogprobs::new(1).unwrap())
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4/explicit-none",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .reasoning_none()
                .temperature(Temperature::new(0.0).unwrap())
                .top_p(TopP::new(0.9).unwrap())
                .store(false)
                .build(),
        ),
    ];
    for (label, request) in requests {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("sampling combination {label} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: tiny GPT-5.1/GPT-5.4 calls cover documented structured output, function calling, and common request fields"]
async fn live_openai_gpt51_and_gpt54_documented_core_features() {
    let metadata = OpenAIResponseMetadata::new([("suite", "ai-client-live")]).unwrap();
    let requests = [
        (
            "gpt-5.1",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("Return JSON with ok=true. Do not call a tool.")
                .instructions("Follow the schema exactly.")
                .metadata(metadata.clone())
                .max_output_tokens(32)
                .safety_identifier("ai-client-live-tests")
                .service_tier(OpenAIServiceTier::Auto)
                .prompt_cache_key("ai-client-live-gpt-5.1-core")
                .json_schema(schema())
                .tool(noop_function())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .background(false)
                .max_tool_calls(1)
                .truncation(OpenAITruncation::Disabled)
                .parallel_tool_calls(false)
                .build(),
        ),
        (
            "gpt-5.4",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("Return JSON with ok=true. Do not call a tool.")
                .instructions("Follow the schema exactly.")
                .metadata(metadata)
                .max_output_tokens(32)
                .safety_identifier("ai-client-live-tests")
                .service_tier(OpenAIServiceTier::Auto)
                .prompt_cache_key("ai-client-live-gpt-5.4-core")
                .json_schema(schema())
                .tool(noop_function())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .background(false)
                .max_tool_calls(1)
                .truncation(OpenAITruncation::Disabled)
                .parallel_tool_calls(false)
                .build(),
        ),
    ];
    let client = client();
    for (model, request) in requests {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("documented core features failed for {model}: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: tiny GPT-5.1/GPT-5.4 calls require one local function call"]
async fn live_openai_gpt51_and_gpt54_function_calling() {
    let requests = [
        (
            "gpt-5.1",
            ResponseRequest::<Gpt5_1>::builder()
                .input_text("Call noop now.")
                .max_output_tokens(32)
                .tool(noop_function())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("Call noop now.")
                .max_output_tokens(32)
                .tool(noop_function())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
                .store(false)
                .build(),
        ),
    ];
    let client = client();
    for (model, request) in requests {
        let response = client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("function calling failed for {model}: {error}"))
            .into_inner();
        assert!(
            response
                .output
                .iter()
                .any(|item| matches!(item, OpenAIResponseOutputItem::FunctionCall(_))),
            "{model} did not produce the required function call"
        );
    }
}

#[tokio::test]
#[ignore = "live provider: tiny GPT-5.1/GPT-5.4 calls verify documented image input"]
async fn live_openai_gpt51_and_gpt54_image_input() {
    const TINY_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAJklEQVR4nO3NMQ0AAAwDoPo33arYsQQMkB6LQCAQCAQCgUAg+BIMi1X0pjxKe0gAAAAASUVORK5CYII=";
    let item: OpenAIResponseInputItem = serde_json::from_value(serde_json::json!({
        "type": "message",
        "role": "user",
        "content": [
            {"type": "input_text", "text": "Reply only OK."},
            {
                "type": "input_image",
                "detail": "low",
                "image_url": format!("data:image/png;base64,{TINY_PNG}")
            }
        ]
    }))
    .unwrap();
    let requests = [
        (
            "gpt-5.1",
            ResponseRequest::<Gpt5_1>::builder()
                .input_items(vec![item.clone()])
                .max_output_tokens(16)
                .store(false)
                .build(),
        ),
        (
            "gpt-5.4",
            ResponseRequest::<Gpt5_4>::builder()
                .input_items(vec![item])
                .max_output_tokens(16)
                .store(false)
                .build(),
        ),
    ];
    let client = client();
    for (model, request) in requests {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("documented image input failed for {model}: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: GPT-5.4 documented tool definitions are declared but forced off, so hosted tools are not billed"]
async fn live_openai_gpt54_documented_tool_definitions() {
    let computer: OpenAIComputerTool = serde_json::from_value(serde_json::json!({})).unwrap();
    let web: OpenAIWebSearchTool = serde_json::from_value(serde_json::json!({})).unwrap();
    let code: OpenAICodeInterpreterTool =
        serde_json::from_value(serde_json::json!({"container": {"type": "auto"}})).unwrap();
    let shell: OpenAIFunctionShellTool = serde_json::from_value(serde_json::json!({})).unwrap();
    let search: OpenAIToolSearchTool = serde_json::from_value(serde_json::json!({})).unwrap();
    let patch: OpenAIApplyPatchTool = serde_json::from_value(serde_json::json!({})).unwrap();

    let deferred_function: OpenAIFunctionTool = serde_json::from_value(serde_json::json!({
        "name": "deferred_noop",
        "description": "Do nothing",
        "strict": false,
        "parameters": {"type": "object", "properties": {}},
        "defer_loading": true
    }))
    .unwrap();
    let requests = [
        (
            "function",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(noop_function())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "computer",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(computer)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "web_search",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(web)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "code_interpreter",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(code)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "image_generation",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(OpenAIImageGenerationTool::default())
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "shell",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(shell)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "apply_patch",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(patch)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
        (
            "tool_search",
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(deferred_function)
                .tool(search)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        ),
    ];
    let client = client();
    for (name, request) in requests {
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("GPT-5.4 tool {name} was rejected: {error:?}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_VECTOR_STORE_ID and verifies GPT-5.4 file search declaration"]
async fn live_openai_resource_tool_file_search() {
    let vector_store_id = super::required_env("OPENAI_VECTOR_STORE_ID");
    let tool: OpenAIFileSearchTool = serde_json::from_value(serde_json::json!({
        "vector_store_ids": [vector_store_id],
        "max_num_results": 1
    }))
    .unwrap();
    client()
        .responses()
        .create(
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(tool)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        )
        .await
        .expect("provisioned GPT-5.4 file-search tool should be accepted");
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_MCP_SERVER_URL and verifies GPT-5.4 MCP declaration"]
async fn live_openai_resource_tool_mcp() {
    let server_url = super::required_env("OPENAI_MCP_SERVER_URL");
    let tool: OpenAIMcpTool = serde_json::from_value(serde_json::json!({
        "server_label": "ai_client_live",
        "server_url": server_url,
        "require_approval": "never"
    }))
    .unwrap();
    client()
        .responses()
        .create(
            ResponseRequest::<Gpt5_4>::builder()
                .input_text("OK")
                .max_output_tokens(16)
                .tool(tool)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .store(false)
                .build(),
        )
        .await
        .expect("configured GPT-5.4 MCP tool should be accepted");
}

#[tokio::test]
#[ignore = "EXPENSIVE live provider: invokes GPT-5.4 web search, code interpreter, and image generation"]
async fn live_openai_expensive_hosted_tool_matrix() {
    let client = client();
    let cases = [
        ResponseRequest::<Gpt5_4>::builder()
            .input_text("Use web search to find the OpenAI homepage title.")
            .max_output_tokens(64)
            .tool(serde_json::from_value::<OpenAIWebSearchTool>(serde_json::json!({})).unwrap())
            .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
            .store(false)
            .build(),
        ResponseRequest::<Gpt5_4>::builder()
            .input_text("Use code interpreter to calculate 2+2.")
            .max_output_tokens(64)
            .tool(
                serde_json::from_value::<OpenAICodeInterpreterTool>(
                    serde_json::json!({"container": {"type": "auto"}}),
                )
                .unwrap(),
            )
            .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
            .store(false)
            .build(),
        ResponseRequest::<Gpt5_4>::builder()
            .input_text("Generate a simple blue circle on white.")
            .max_output_tokens(64)
            .tool(OpenAIImageGenerationTool::default())
            .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
            .store(false)
            .build(),
    ];
    for request in cases {
        client
            .responses()
            .create(request)
            .await
            .expect("GPT-5.4 hosted tool invocation should succeed");
    }
}
