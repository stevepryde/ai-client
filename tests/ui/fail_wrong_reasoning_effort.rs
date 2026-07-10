use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{ExtendedReasoningEffort, Gpt5_4Pro, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4Pro>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .reasoning(ExtendedReasoningEffort::None)
        .build();
}
