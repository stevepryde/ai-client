use ai_client::openai::{
    responses::{Gpt4o, ResponseRequest},
};

fn main() {
    let mut request = ResponseRequest::<Gpt4o>::builder()
        .input_text("hello")
        .build();
    request.wire_mut().stream = Some(true);
}
