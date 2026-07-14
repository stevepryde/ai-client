use ai_client::openai::responses::{
    CreateResponseRequest, Gpt5_1, OpenAIImageGenerationTool,
};
use ai_client::openai::OpenAIClient;

async fn create(client: &OpenAIClient) {
    let request = CreateResponseRequest::builder()
        .input_text("draw a cat")
        .tool(OpenAIImageGenerationTool::default())
        .build();
    let _ = client.responses().create(Gpt5_1::config(), request).await;
}

fn main() {}
