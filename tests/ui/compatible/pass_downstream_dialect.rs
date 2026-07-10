use ai_client::openai_compatible::{
    chat::{ChatRequest, CompatibleChatModel},
    ChatCompletionsDialect, OpenAICompatibleDialect,
};

struct Dialect;
impl OpenAICompatibleDialect for Dialect {
    const NAME: &'static str = "downstream";
}

#[derive(Default, serde::Serialize)]
struct Options {
    vendor_flag: Option<bool>,
}

#[derive(serde::Serialize)]
struct Message {
    role: &'static str,
    content: &'static str,
    vendor_message_field: bool,
}

impl ChatCompletionsDialect for Dialect {
    type ChatOptions = Options;
    type Message = Message;
}

struct Model;
impl CompatibleChatModel<Dialect> for Model {
    const ID: &'static str = "downstream-model";
}

fn main() {
    let _ = ChatRequest::<Dialect, Model>::builder()
        .messages(vec![Message {
            role: "user",
            content: "hello",
            vendor_message_field: true,
        }])
        .provider_options(Options {
            vendor_flag: Some(true),
        })
        .build()
        .unwrap();
}
