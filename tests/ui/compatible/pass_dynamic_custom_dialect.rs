use ai_client::openai_compatible::{
    chat::{DynamicChatModel, DynamicChatRequest, ChatMessage, ChatRole},
    CustomDialect,
};

fn main() {
    let model = DynamicChatModel::new("runtime-model").unwrap();
    let _ = DynamicChatRequest::<CustomDialect>::builder(model)
        .messages(vec![ChatMessage::new(ChatRole::User, "hello")])
        .reasoning_effort("provider-specific")
        .build()
        .unwrap();
}
