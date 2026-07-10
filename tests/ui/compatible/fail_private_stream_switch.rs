use ai_client::openai_compatible::{chat::{ChatMessage, ChatRequest, ChatRole, CompatibleChatModel}, CustomDialect};

struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "model";
}

fn main() {
    let mut request = ChatRequest::<CustomDialect, Model>::builder()
        .messages(vec![ChatMessage::new(ChatRole::User, "hello")])
        .build()
        .unwrap();
    request.set_stream(true);
}
