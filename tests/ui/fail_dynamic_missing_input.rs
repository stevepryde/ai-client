use ai_client::openai::responses::{DynamicOpenAIModel, DynamicResponseRequest};

fn main() {
    let model = DynamicOpenAIModel::new("gpt-5").unwrap();
    let _ = DynamicResponseRequest::builder(model).build();
}
