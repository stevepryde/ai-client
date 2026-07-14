use ai_client::openai::{
    responses::{
        CreateResponseRequest, ExtendedReasoningEffort, Gpt5_2, Gpt5_4Nano,
        OpenAIEasyInputMessage, OpenAIImageGenerationTool, OpenAIMessageRole,
        OpenAIResponsesInputContent, OpenAIResponsesInputItem, ResponseModelFor,
    },
    OpenAIClient, OpenAIJsonSchema,
};

enum AppModel {
    Fast,
    Strong,
}

async fn create(client: &OpenAIClient, selected: AppModel) {
    let request = CreateResponseRequest::builder()
        .instructions("Return JSON")
        .input_items(vec![OpenAIResponsesInputItem::Message(
            OpenAIEasyInputMessage {
                role: OpenAIMessageRole::User,
                content: OpenAIResponsesInputContent::Text("hello".into()),
                phase: None,
                extra: Default::default(),
            },
        )])
        .json_schema(OpenAIJsonSchema {
            name: "result".into(),
            description: "result".into(),
            schema: serde_json::json!({"type":"object"}),
            strict: Some(true),
        })
        .build();

    let model: Box<dyn ResponseModelFor<_>> = match selected {
        AppModel::Fast => Box::new(Gpt5_4Nano::config().reasoning_none()),
        AppModel::Strong => Box::new(
            Gpt5_2::config().reasoning(ExtendedReasoningEffort::High),
        ),
    };
    let _ = client.responses().create(model, request).await;
}

async fn create_image(client: &OpenAIClient) {
    let request = CreateResponseRequest::builder()
        .input_text("draw a cat")
        .image_generation_tool(OpenAIImageGenerationTool::default())
        .build();
    let _ = client
        .responses()
        .create(Gpt5_4Nano::config().reasoning_none(), request)
        .await;
}

fn main() {}
