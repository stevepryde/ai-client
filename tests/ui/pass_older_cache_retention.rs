use ai_client::openai::{
    responses::{Gpt5_4, PromptCacheRetention, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4>::builder()
        .input_text("hello")
        .prompt_cache_retention(PromptCacheRetention::InMemory)
        .build();
}
