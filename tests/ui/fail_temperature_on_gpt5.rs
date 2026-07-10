use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt5, ResponseRequest, Temperature},
};

fn main() {
    let _ = ResponseRequest::<Gpt5>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .temperature(Temperature::new(0.5).unwrap())
        .build();
}
