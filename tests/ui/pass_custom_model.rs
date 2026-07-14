use ai_client::openai::{
    responses::{
        IntoReasoningEffort, OpenAIResponsesModel, ResponseModelConfig, ResponseRequest,
        SupportsReasoning,
    },
    OpenAIReasoningEffort,
};

struct FineTuned;
impl OpenAIResponsesModel for FineTuned { const ID: &'static str = "ft:gpt-5:local"; }

enum LocalEffort { High }
impl IntoReasoningEffort for LocalEffort {
    fn into_reasoning_effort(self) -> OpenAIReasoningEffort { OpenAIReasoningEffort::High }
}
impl SupportsReasoning for FineTuned { type Effort = LocalEffort; }

fn main() {
    let _ = ResponseRequest::<FineTuned>::builder()
        .input_text("hello")
        .reasoning(LocalEffort::High)
        .build();
    let config = ResponseModelConfig::<FineTuned>::new().reasoning(LocalEffort::High);
    let _ = config.clone();
}
