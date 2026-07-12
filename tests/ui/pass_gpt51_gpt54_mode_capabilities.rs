use ai_client::openai::responses::{
    Gpt5_1, Gpt5_2, Gpt5_4, OpenAIFunctionTool, PromptCacheRetention, ResponseRequest,
    Temperature, TopP,
};

fn function() -> OpenAIFunctionTool {
    serde_json::from_value(serde_json::json!({
        "name": "noop",
        "strict": false,
        "parameters": {"type": "object", "properties": {}}
    }))
    .unwrap()
}

fn main() {
    let _ = ResponseRequest::<Gpt5_1>::builder()
        .temperature(Temperature::new(0.0).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .tool(function())
        .build();
    let _ = ResponseRequest::<Gpt5_4>::builder()
        .reasoning_none()
        .temperature(Temperature::new(0.0).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .tool(function())
        .build();
    let _ = ResponseRequest::<Gpt5_2>::builder()
        .reasoning_none()
        .temperature(Temperature::new(0.0).unwrap())
        .top_p(TopP::new(0.9).unwrap())
        .prompt_cache_retention(PromptCacheRetention::Hours24)
        .build();
}
