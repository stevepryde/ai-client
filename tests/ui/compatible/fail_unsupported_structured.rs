use ai_client::openai_compatible::{
    chat::{ChatRequest, ChatResponseFormat, CompatibleChatModel},
    CustomDialect,
};

struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "model";
}

fn main() {
    let _ = ChatRequest::<CustomDialect, Model>::builder()
        .response_format(ChatResponseFormat::new(serde_json::Map::new()));
}
