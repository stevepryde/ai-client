use std::sync::Arc;

use crate::openai::{responses::*, OpenAIReasoningEffort};

use super::catalog::*;
use crate::openai::responses::request::OpenAIResponsesWireRequest;

pub struct DynamicResponseRequest;

impl DynamicResponseRequest {
    pub fn builder(model: DynamicOpenAIModel) -> DynamicResponseRequestBuilder {
        DynamicResponseRequestBuilder {
            wire: OpenAIResponsesWireRequest::new(model.0),
            mode: ValidationMode::Off,
            catalog: None,
        }
    }
}

pub struct DynamicResponseRequestBuilder {
    pub(super) wire: OpenAIResponsesWireRequest,
    pub(super) mode: ValidationMode,
    pub(super) catalog: Option<Arc<dyn ResponseModelCapabilitiesCatalog>>,
}

impl DynamicResponseRequestBuilder {
    pub fn metadata(mut self, metadata: OpenAIResponseMetadata) -> Self {
        self.wire.metadata = Some(metadata);
        self
    }
    pub fn top_logprobs(mut self, value: TopLogprobs) -> Self {
        self.wire.top_logprobs = Some(value);
        self
    }
    pub fn instructions(mut self, value: impl Into<String>) -> Self {
        self.wire.instructions = Some(value.into());
        self
    }
    pub fn max_output_tokens(mut self, value: u64) -> Self {
        self.wire.max_output_tokens = Some(value);
        self
    }
    pub fn temperature(mut self, value: Temperature) -> Self {
        self.wire.temperature = Some(value.get());
        self
    }
    pub fn top_p(mut self, value: TopP) -> Self {
        self.wire.top_p = Some(value.get());
        self
    }
    pub fn user(mut self, value: impl Into<String>) -> Self {
        self.wire.user = Some(value.into());
        self
    }
    pub fn safety_identifier(mut self, value: impl Into<String>) -> Self {
        self.wire.safety_identifier = Some(value.into());
        self
    }
    pub fn service_tier(mut self, value: OpenAIServiceTier) -> Self {
        self.wire.service_tier = Some(value);
        self
    }
    pub fn reasoning(mut self, effort: OpenAIReasoningEffort) -> Self {
        self.wire.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(effort),
            summary: None,
            context: None,
            generate_summary: None,
            extra: Default::default(),
        });
        self
    }
    pub fn reasoning_config(mut self, reasoning: OpenAIResponsesReasoning) -> Self {
        self.wire.reasoning = Some(reasoning);
        self
    }
    pub fn prompt_cache_key(mut self, value: impl Into<String>) -> Self {
        self.wire.prompt_cache_key = Some(value.into());
        self
    }
    pub fn prompt_cache_retention(mut self, value: impl Into<String>) -> Self {
        self.wire.prompt_cache_retention = Some(value.into());
        self
    }
    pub fn prompt_cache_options(mut self, value: OpenAIPromptCacheOptions) -> Self {
        self.wire.prompt_cache_options = Some(value);
        self
    }
    pub fn text_format(mut self, format: OpenAIResponsesTextFormat) -> Self {
        self.wire.text = Some(OpenAIResponsesTextConfig {
            format: Some(format),
            verbosity: None,
            extra: Default::default(),
        });
        self
    }
    pub fn text_config(mut self, text: OpenAIResponsesTextConfig) -> Self {
        self.wire.text = Some(text);
        self
    }
    pub fn tool(mut self, tool: OpenAIResponsesTool) -> Self {
        self.wire.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }
    pub fn tool_choice(mut self, value: OpenAIToolChoice) -> Self {
        self.wire.tool_choice = Some(value);
        self
    }
    pub fn prompt(mut self, value: OpenAIPromptTemplate) -> Self {
        self.wire.prompt = Some(value);
        self
    }
    pub fn previous_response_id(mut self, value: ResponseId) -> Self {
        self.wire.previous_response_id = Some(value);
        self
    }
    pub fn store(mut self, value: bool) -> Self {
        self.wire.store = Some(value);
        self
    }
    pub fn background(mut self, value: bool) -> Self {
        self.wire.background = Some(value);
        self
    }
    pub fn max_tool_calls(mut self, value: u64) -> Self {
        self.wire.max_tool_calls = Some(value);
        self
    }
    pub fn truncation(mut self, value: OpenAITruncation) -> Self {
        self.wire.truncation = Some(value);
        self
    }
    pub fn include(mut self, value: ResponseInclude) -> Self {
        self.wire.include.get_or_insert_with(Vec::new).push(value);
        self
    }
    pub fn parallel_tool_calls(mut self, value: bool) -> Self {
        self.wire.parallel_tool_calls = Some(value);
        self
    }
    pub fn moderation(mut self, value: OpenAIModerationConfig) -> Self {
        self.wire.moderation = Some(value);
        self
    }
    pub fn conversation(mut self, value: OpenAIConversationReference) -> Self {
        self.wire.conversation = Some(value);
        self
    }
    pub fn context_management(mut self, value: Vec<OpenAIContextCompaction>) -> Self {
        self.wire.context_management = Some(value);
        self
    }
    pub fn validation(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn catalog(mut self, catalog: impl ResponseModelCapabilitiesCatalog + 'static) -> Self {
        self.catalog = Some(Arc::new(catalog));
        self
    }
    pub fn builtin_catalog(self) -> Self {
        self.catalog(StaticResponseModelCapabilitiesCatalog::builtin())
    }
}
