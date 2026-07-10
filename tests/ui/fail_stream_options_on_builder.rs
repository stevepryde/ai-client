use ai_client::openai::responses::{
    CreateResponseStreamOptions, Gpt4o, ResponseRequest,
};

fn main() {
    let _request = ResponseRequest::<Gpt4o>::builder()
        .stream_options(CreateResponseStreamOptions::new().include_obfuscation(false))
        .build();
}
