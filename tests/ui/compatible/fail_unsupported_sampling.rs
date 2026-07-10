use ai_client::openai_compatible::{
    chat::{ChatRequest, ChatTemperature, CompatibleChatModel},
    CustomDialect,
};

struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "model";
}

fn main() {
    let _ = ChatRequest::<CustomDialect, Model>::builder()
        .temperature(ChatTemperature::try_from(0.5).unwrap());
}
