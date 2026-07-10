use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt4o, Gpt5ReasoningEffort, ResponseRequest},
};

fn main() {
    let _ = ResponseRequest::<Gpt4o>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .reasoning(Gpt5ReasoningEffort::High)
        .build();
}
