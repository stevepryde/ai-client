use ai_client::openai::responses::{Gpt4oMini, PromptCacheRetention};

fn main() {
    let _ = Gpt4oMini::config().prompt_cache_retention(PromptCacheRetention::Hours24);
}
