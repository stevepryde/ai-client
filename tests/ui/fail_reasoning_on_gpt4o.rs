use ai_client::openai::{
    responses::{Gpt4o, Gpt5ReasoningEffort, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt4o>::builder()
        .input_text("hello")
        .reasoning(Gpt5ReasoningEffort::High)
        .build();
}

trait SupportsReasoning {}

fn _uses_trait_name<T: SupportsReasoning>() {}
