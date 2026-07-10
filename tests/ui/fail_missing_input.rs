use ai_client::openai::responses::{Gpt5, ResponseRequest};

fn main() {
    let _ = ResponseRequest::<Gpt5>::builder().build();
}
