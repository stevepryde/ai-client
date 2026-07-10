use ai_client::openai::{
    responses::{Gpt4oMini, PromptCacheRetention, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt4oMini>::builder()
        .input_text("hello")
        .prompt_cache_retention(PromptCacheRetention::Hours24)
        .build();
}
