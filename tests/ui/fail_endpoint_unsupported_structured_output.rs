use ai_client::openai::{
    responses::{CreateResponseRequest, Gpt5_4Pro},
    OpenAIClient, OpenAIJsonSchema,
};

async fn create(client: &OpenAIClient) {
    let request = CreateResponseRequest::builder()
        .input_text("hello")
        .json_schema(OpenAIJsonSchema {
            name: "result".into(),
            description: "result".into(),
            schema: serde_json::json!({"type":"object"}),
            strict: Some(true),
        })
        .build();
    let _ = client
        .responses()
        .create(Gpt5_4Pro::config(), request)
        .await;
}

fn main() {}
