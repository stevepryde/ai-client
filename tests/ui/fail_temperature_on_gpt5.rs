use ai_client::openai::{
    responses::{Gpt5, ResponseRequest, Temperature},
};

fn main() {
    let _ = ResponseRequest::<Gpt5>::builder()
        .input_text("hello")
        .temperature(Temperature::new(0.5).unwrap())
        .build();
}
