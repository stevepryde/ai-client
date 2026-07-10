use ai_client::openai_compatible::{CompatibleAuth, OpenAICompatibleClient, OpenAICompatibleDialect};

struct NoChatDialect;
impl OpenAICompatibleDialect for NoChatDialect {
    const NAME: &'static str = "no-chat";
}

fn main() {
    let client = OpenAICompatibleClient::<NoChatDialect>::builder()
        .base_url("http://localhost:8080/v1")
        .auth(CompatibleAuth::none())
        .build()
        .unwrap();
    let _ = client.chat();
}
