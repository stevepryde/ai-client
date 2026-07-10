use ai_client::openai::responses::{Gpt5_4Pro, ResponseRequest};

fn main() {
    let _ = ResponseRequest::<Gpt5_4Pro>::builder()
        .input_text("hello")
        .reasoning_none()
        .build();
}
