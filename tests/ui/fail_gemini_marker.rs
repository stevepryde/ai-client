use ai_client::{
    gemini::GeminiModel,
    openai::responses::ResponseRequest,
};

fn main() {
    let _ = ResponseRequest::<GeminiModel>::builder();
}
