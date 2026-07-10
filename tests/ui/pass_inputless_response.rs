use ai_client::openai::responses::{Gpt4o, ResponseRequest};

fn main() {
    let _request = ResponseRequest::<Gpt4o>::builder().build();
}
