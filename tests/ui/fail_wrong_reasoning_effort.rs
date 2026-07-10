use ai_client::openai::{
    responses::{ExtendedReasoningEffort, Gpt5_4Pro, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4Pro>::builder()
        .input_text("hello")
        .reasoning(ExtendedReasoningEffort::None)
        .build();
}
