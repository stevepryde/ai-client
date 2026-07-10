use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{
        IntoReasoningEffort, OpenAIResponsesModel, ResponseRequest, SupportsReasoning,
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
        .input(OpenAIResponsesInput::Text("hello".into()))
        .reasoning(LocalEffort::High)
        .build();
}
