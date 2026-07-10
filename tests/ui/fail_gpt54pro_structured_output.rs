use ai_client::openai::{
    responses::{Gpt5_4Pro, ResponseRequest},
    OpenAIJsonSchema,
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4Pro>::builder()
        .input_text("hello")
        .json_schema(OpenAIJsonSchema {
            name: "result".into(),
            description: "result".into(),
            schema: serde_json::json!({"type":"object"}),
            strict: Some(true),
        })
        .build();
}
