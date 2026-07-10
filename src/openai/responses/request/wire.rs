use std::fmt;

use serde::{Serialize, Serializer};

use crate::openai::responses::{
    dynamic::ValidationWarning, CreateResponseStreamOptions, OpenAIContextCompaction,
    OpenAIConversationReference, OpenAIModerationConfig, OpenAIPromptCacheOptions,
    OpenAIPromptTemplate, OpenAIResponseMetadata, OpenAIResponsesInput, OpenAIResponsesReasoning,
    OpenAIResponsesTextConfig, OpenAIResponsesTool, OpenAIServiceTier, OpenAIToolChoice,
    OpenAITruncation, ResponseId, ResponseInclude, TopLogprobs,
};

#[derive(Clone)]
pub struct PreparedResponseRequest {
    wire: OpenAIResponsesWireRequest,
    warnings: Vec<ValidationWarning>,
}

impl PreparedResponseRequest {
    pub fn warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }

    pub fn model_id(&self) -> &str {
        &self.wire.model
    }

    pub(crate) fn new(wire: OpenAIResponsesWireRequest, warnings: Vec<ValidationWarning>) -> Self {
        Self { wire, warnings }
    }

    pub(crate) fn wire_mut(&mut self) -> &mut OpenAIResponsesWireRequest {
        &mut self.wire
    }
}

impl fmt::Debug for PreparedResponseRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedResponseRequest")
            .field("model", &self.wire.model)
            .field("request", &"[redacted]")
            .field("warnings", &self.warnings)
            .finish()
    }
}

impl Serialize for PreparedResponseRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.wire.serialize(serializer)
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct OpenAIResponsesWireRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<OpenAIResponseMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) top_logprobs: Option<TopLogprobs>,
    pub(crate) model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input: Option<OpenAIResponsesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_cache_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_cache_options: Option<OpenAIPromptCacheOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<OpenAIResponsesTextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_response_id: Option<ResponseId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) service_tier: Option<OpenAIServiceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_tool_calls: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning: Option<OpenAIResponsesReasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tools: Option<Vec<OpenAIResponsesTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_choice: Option<OpenAIToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt: Option<OpenAIPromptTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) truncation: Option<OpenAITruncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) include: Option<Vec<ResponseInclude>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) moderation: Option<OpenAIModerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stream_options: Option<CreateResponseStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) conversation: Option<OpenAIConversationReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) context_management: Option<Vec<OpenAIContextCompaction>>,
}

impl OpenAIResponsesWireRequest {
    pub(crate) fn new(model: String) -> Self {
        Self {
            metadata: None,
            top_logprobs: None,
            model,
            input: None,
            instructions: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            user: None,
            safety_identifier: None,
            stream: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            prompt_cache_options: None,
            text: None,
            previous_response_id: None,
            service_tier: None,
            background: None,
            max_tool_calls: None,
            store: None,
            reasoning: None,
            tools: None,
            tool_choice: None,
            prompt: None,
            truncation: None,
            include: None,
            parallel_tool_calls: None,
            moderation: None,
            stream_options: None,
            conversation: None,
            context_management: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::openai::responses::{ExtendedReasoningEffort, Gpt5, Gpt5_4, ResponseRequest};

    #[test]
    fn typed_builder_erases_to_one_redacted_wire_request() {
        let request = ResponseRequest::<Gpt5_4>::builder()
            .input_text("private prompt")
            .reasoning(ExtendedReasoningEffort::XHigh)
            .prompt_cache_key("key")
            .prompt_cache_retention(crate::openai::responses::PromptCacheRetention::Hours24)
            .build();
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["model"], "gpt-5.4");
        assert_eq!(value["reasoning"]["effort"], "xhigh");
        assert_eq!(value["prompt_cache_key"], "key");
        assert!(!format!("{request:?}").contains("private prompt"));
    }

    #[test]
    fn inputless_request_omits_input_instead_of_sending_null() {
        let request = ResponseRequest::<Gpt5>::builder().build();
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["model"], "gpt-5");
        assert!(value.get("input").is_none());
    }
}
