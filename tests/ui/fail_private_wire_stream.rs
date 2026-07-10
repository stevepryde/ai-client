use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt4o, ResponseRequest},
};

fn main() {
    let mut request = ResponseRequest::<Gpt4o>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .build();
    request.wire_mut().stream = Some(true);
}
