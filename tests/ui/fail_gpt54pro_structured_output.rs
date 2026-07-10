use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{Gpt5_4Pro, ResponseRequest},
    OpenAIJsonSchema,
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4Pro>::builder()
        .input(OpenAIResponsesInput::Text("hello".into()))
        .json_schema(OpenAIJsonSchema {
            name: "result".into(),
            description: "result".into(),
            schema: serde_json::json!({"type":"object"}),
            strict: Some(true),
        })
        .build();
}
