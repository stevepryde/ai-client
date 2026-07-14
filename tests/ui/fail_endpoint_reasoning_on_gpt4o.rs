use ai_client::openai::responses::{Gpt4o, Gpt5ReasoningEffort};

fn main() {
    let _ = Gpt4o::config().reasoning(Gpt5ReasoningEffort::High);
}
