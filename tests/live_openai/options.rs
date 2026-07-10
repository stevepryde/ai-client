use ai_client::openai::responses::*;

use super::client;

fn dynamic_request(model: &str) -> DynamicResponseRequestBuilder {
    DynamicResponseRequest::builder(DynamicOpenAIModel::new(model).unwrap())
        .input_text("Reply with only OK.")
        .max_output_tokens(32)
        .store(false)
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and covers every include selector"]
async fn live_openai_option_matrix_accepts_every_include_selector() {
    let client = client();
    let includes = [
        ResponseInclude::FileSearchCallResults,
        ResponseInclude::WebSearchCallResults,
        ResponseInclude::WebSearchCallActionSources,
        ResponseInclude::MessageInputImageUrl,
        ResponseInclude::ComputerCallOutputImageUrl,
        ResponseInclude::CodeInterpreterCallOutputs,
        ResponseInclude::ReasoningEncryptedContent,
        ResponseInclude::MessageOutputTextLogprobs,
    ];
    for include in includes {
        let model = if include == ResponseInclude::ReasoningEncryptedContent {
            "gpt-5.4-mini"
        } else {
            "gpt-4.1"
        };
        let mut builder = dynamic_request(model).include(include);
        if include == ResponseInclude::MessageOutputTextLogprobs {
            builder = builder.top_logprobs(TopLogprobs::new(1).unwrap());
        }
        client
            .responses()
            .create(builder.build().unwrap())
            .await
            .unwrap_or_else(|error| panic!("include selector {include:?} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and covers text format/verbosity enums"]
async fn live_openai_option_matrix_accepts_text_configuration() {
    let client = client();
    let formats = [
        (serde_json::json!({"type": "text"}), "Reply with only OK."),
        (
            serde_json::json!({"type": "json_object"}),
            "Return a JSON object with ok set to true.",
        ),
        (
            serde_json::json!({
                "type": "json_schema",
                "name": "live_result",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }
            }),
            "Return JSON where ok is true.",
        ),
    ];
    for (format, prompt) in formats {
        let format: OpenAIResponsesTextFormat = serde_json::from_value(format).unwrap();
        client
            .responses()
            .create(
                dynamic_request("gpt-4.1")
                    .input_text(prompt)
                    .max_output_tokens(64)
                    .text_format(format)
                    .build()
                    .unwrap(),
            )
            .await
            .expect("text format should be accepted");
    }

    for verbosity in [
        OpenAIResponseVerbosity::Low,
        OpenAIResponseVerbosity::Medium,
        OpenAIResponseVerbosity::High,
    ] {
        let text: OpenAIResponsesTextConfig = serde_json::from_value(serde_json::json!({
            "verbosity": verbosity
        }))
        .unwrap();
        client
            .responses()
            .create(
                dynamic_request("gpt-5.4-mini")
                    .text_config(text)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap_or_else(|error| panic!("text verbosity {verbosity:?} failed: {error}"));
    }
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY; tools are declared but forced off, so hosted tools are not billed"]
async fn live_openai_tool_definition_matrix_is_accepted_without_invocation() {
    let client = client();
    let mut rejected = Vec::new();
    let tools = [
        serde_json::json!({
            "type": "function", "name": "noop", "description": "Do nothing",
            "strict": false,
            "parameters": {"type": "object", "properties": {}}
        }),
        serde_json::json!({"type": "computer"}),
        serde_json::json!({
            "type": "web_search"
        }),
        serde_json::json!({"type": "web_search_2025_08_26"}),
        serde_json::json!({"type": "code_interpreter", "container": {"type": "auto"}}),
        serde_json::json!({"type": "image_generation"}),
        serde_json::json!({"type": "shell"}),
        serde_json::json!({
            "type": "custom", "name": "custom_noop", "description": "Do nothing"
        }),
        serde_json::json!({
            "type": "namespace", "name": "test_namespace", "description": "Live test",
            "tools": [{
                "type": "function", "name": "nested_noop", "strict": false,
                "parameters": {"type": "object", "properties": {}}
            }]
        }),
        serde_json::json!({"type": "web_search_preview"}),
        serde_json::json!({"type": "web_search_preview_2025_03_11"}),
        serde_json::json!({"type": "apply_patch"}),
    ];

    for tool in tools {
        let tag = tool["type"].as_str().unwrap().to_string();
        let tool: OpenAIResponsesTool = serde_json::from_value(tool)
            .unwrap_or_else(|error| panic!("tool definition {tag} is not constructible: {error}"));
        if let Err(error) = client
            .responses()
            .create(
                dynamic_request("gpt-5.4")
                    .tool(tool)
                    .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                    .build()
                    .unwrap(),
            )
            .await
        {
            rejected.push(format!("{tag}: {error}"));
        }
    }
    assert!(
        rejected.is_empty(),
        "tool definitions were rejected:\n{}",
        rejected.join("\n")
    );
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and provisioned OPENAI_VECTOR_STORE_ID"]
async fn live_openai_resource_tool_file_search() {
    let vector_store_id = super::required_env("OPENAI_VECTOR_STORE_ID");
    let tool: OpenAIResponsesTool = serde_json::from_value(serde_json::json!({
        "type": "file_search",
        "vector_store_ids": [vector_store_id],
        "max_num_results": 1
    }))
    .unwrap();
    client()
        .responses()
        .create(
            dynamic_request("gpt-5.4")
                .tool(tool)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .build()
                .unwrap(),
        )
        .await
        .expect("provisioned file-search tool should be accepted");
}

#[tokio::test]
#[ignore = "live provider: requires OPENAI_API_KEY and OPENAI_MCP_SERVER_URL"]
async fn live_openai_resource_tool_mcp() {
    let server_url = super::required_env("OPENAI_MCP_SERVER_URL");
    let tool: OpenAIResponsesTool = serde_json::from_value(serde_json::json!({
        "type": "mcp",
        "server_label": "ai_client_live",
        "server_url": server_url,
        "require_approval": "never"
    }))
    .unwrap();
    client()
        .responses()
        .create(
            dynamic_request("gpt-5.4")
                .tool(tool)
                .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::None))
                .build()
                .unwrap(),
        )
        .await
        .expect("configured MCP tool should be accepted");
}

#[tokio::test]
#[ignore = "EXPENSIVE live provider: invokes web search, code interpreter, and image generation"]
async fn live_openai_expensive_hosted_tool_matrix() {
    let cases = [
        (
            "web_search",
            serde_json::json!({"type": "web_search"}),
            "Use web search to find the OpenAI homepage title.",
        ),
        (
            "code_interpreter",
            serde_json::json!({"type": "code_interpreter", "container": {"type": "auto"}}),
            "Use code interpreter to calculate 2+2.",
        ),
        (
            "image_generation",
            serde_json::json!({"type": "image_generation", "size": "1024x1024", "quality": "low"}),
            "Generate a simple blue circle on white.",
        ),
    ];
    let client = client();
    for (name, tool, prompt) in cases {
        let tool: OpenAIResponsesTool = serde_json::from_value(tool).unwrap();
        let request = DynamicResponseRequest::builder(DynamicOpenAIModel::new("gpt-5.4").unwrap())
            .input_text(prompt)
            .max_output_tokens(256)
            .tool(tool)
            .tool_choice(OpenAIToolChoice::Mode(OpenAIToolChoiceMode::Required))
            .store(false)
            .build()
            .unwrap();
        client
            .responses()
            .create(request)
            .await
            .unwrap_or_else(|error| panic!("hosted tool {name} failed: {error}"));
    }
}
