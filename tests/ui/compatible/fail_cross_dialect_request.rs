use ai_client::openai_compatible::{
    chat::{ChatMessage, ChatRequest, ChatRole, CompatibleChatModel},
    ChatCompletionsDialect, CompatibleAuth, CustomDialect, OpenAICompatibleClient,
    OpenAICompatibleDialect,
};

#[derive(Default, serde::Serialize)]
struct OtherOptions {}
struct OtherDialect;
impl OpenAICompatibleDialect for OtherDialect {
    const NAME: &'static str = "other";
}
impl ChatCompletionsDialect for OtherDialect {
    type ChatOptions = OtherOptions;
    type Message = ChatMessage;
}
struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "model";
}

fn main() {
    let request = ChatRequest::<CustomDialect, Model>::builder()
        .messages(vec![ChatMessage::new(ChatRole::User, "hello")])
        .build()
        .unwrap();
    let client = OpenAICompatibleClient::<OtherDialect>::builder()
        .base_url("http://localhost:8080/v1")
        .auth(CompatibleAuth::none())
        .build()
        .unwrap();
    let _ = client.chat().create(request);
}
