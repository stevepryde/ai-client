use ai_client::openai::{
    create_response::{OpenAIImageGenerationTool, OpenAIResponsesInput},
    responses::{
        Gpt4o, Gpt5, Gpt5Nano, Gpt5ReasoningEffort, Gpt5_5,
        Gpt5_5PromptCacheRetention, ResponseRequest, Temperature, TopP,
    },
    OpenAIJsonSchema,
};

fn main() {
    let input = || OpenAIResponsesInput::Text("hello".into());
    let _ = ResponseRequest::<Gpt4o>::builder()
        .input(input())
        .temperature(Temperature::new(0.5).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .build();
    let _ = ResponseRequest::<Gpt5>::builder()
        .input(input())
        .reasoning(Gpt5ReasoningEffort::High)
        .json_schema(OpenAIJsonSchema {
            name: "result".into(),
            description: "result".into(),
            schema: serde_json::json!({"type":"object"}),
            strict: Some(true),
        })
        .build();
    let _ = ResponseRequest::<Gpt5Nano>::builder()
        .input(input())
        .image_generation_tool(OpenAIImageGenerationTool::default())
        .build();
    let _ = ResponseRequest::<Gpt5_5>::builder()
        .input(input())
        .prompt_cache_key("cache-key")
        .prompt_cache_retention(Gpt5_5PromptCacheRetention::Hours24)
        .build();
}
