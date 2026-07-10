use std::num::NonZeroU32;

use ai_client::openai_compatible::{
    chat::{ChatRequest, CompatibleChatModel},
    CustomDialect,
};

struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "model";
}

fn main() {
    let _ = ChatRequest::<CustomDialect, Model>::builder()
        .choice_count(NonZeroU32::new(2).unwrap());
}
