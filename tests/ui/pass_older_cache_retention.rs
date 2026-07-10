use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt5_4, PromptCacheRetention, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .prompt_cache_retention(PromptCacheRetention::InMemory)
        .build();
}
