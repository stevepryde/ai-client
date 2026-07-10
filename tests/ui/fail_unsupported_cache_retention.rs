use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt4oMini, PromptCacheRetention, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt4oMini>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .prompt_cache_retention(PromptCacheRetention::Hours24)
        .build();
}
