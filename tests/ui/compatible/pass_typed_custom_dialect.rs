use std::num::NonZeroU32;

use ai_client::openai_compatible::{
    chat::{
        ChatFrequencyPenalty, ChatMessage, ChatModality, ChatRequest, ChatResponseFormat, ChatRole,
        ChatTemperature, CompatibleChatModel, IntoChatReasoningEffort, SupportsChatSampling,
        SupportsChoiceCount, SupportsFrequencyPenalty, SupportsMaxCompletionTokens,
        SupportsModalities, SupportsReasoning, SupportsStructuredOutput,
    },
    CompatibleAuth, CustomDialect, OpenAICompatibleClient,
};

struct Model;
impl CompatibleChatModel<CustomDialect> for Model {
    const ID: &'static str = "custom-model";
}
impl SupportsChatSampling<CustomDialect> for Model {}
impl SupportsMaxCompletionTokens<CustomDialect> for Model {}
impl SupportsFrequencyPenalty<CustomDialect> for Model {}
impl SupportsChoiceCount<CustomDialect> for Model {}
impl SupportsStructuredOutput<CustomDialect> for Model {}
impl SupportsModalities<CustomDialect> for Model {}

enum Effort {
    High,
}
impl IntoChatReasoningEffort for Effort {
    fn into_reasoning_effort(self) -> String {
        match self {
            Self::High => "high".into(),
        }
    }
}
impl SupportsReasoning<CustomDialect> for Model {
    type Effort = Effort;
}

fn main() {
    let mut format = serde_json::Map::new();
    format.insert("type".into(), serde_json::json!("json_object"));
    let _request = ChatRequest::<CustomDialect, Model>::builder()
        .messages(vec![ChatMessage::new(ChatRole::User, "hello")])
        .temperature(ChatTemperature::try_from(0.5).unwrap())
        .max_completion_tokens(100)
        .frequency_penalty(ChatFrequencyPenalty::try_from(0.2).unwrap())
        .choice_count(NonZeroU32::new(1).unwrap())
        .response_format(ChatResponseFormat::new(format))
        .modalities(vec![ChatModality::new("text").unwrap()])
        .reasoning_effort(Effort::High)
        .build()
        .unwrap();
    let _client = OpenAICompatibleClient::<CustomDialect>::builder()
        .base_url("http://localhost:8080/v1")
        .auth(CompatibleAuth::none())
        .build()
        .unwrap();
}
