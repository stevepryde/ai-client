use ai_client::openai_compatible::{chat::{DynamicChatModel, DynamicChatRequest}, CustomDialect};

fn main() {
    let model = DynamicChatModel::new("runtime-model").unwrap();
    let _ = DynamicChatRequest::<CustomDialect>::builder(model).build();
}
