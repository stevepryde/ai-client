use ai_client::openai::{
    responses::{Gpt5_5, PromptCacheRetention, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt5_5>::builder()
        .input_text("hello")
        .prompt_cache_retention(PromptCacheRetention::InMemory)
        .build();
}
