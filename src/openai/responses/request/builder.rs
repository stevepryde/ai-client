use std::marker::PhantomData;

use crate::openai::{responses::*, OpenAIJsonSchema};

use super::{options::*, wire::*};

pub struct ResponseRequest<M: OpenAIResponsesModel>(PhantomData<fn() -> M>);

impl<M: OpenAIResponsesModel> ResponseRequest<M> {
    pub fn builder() -> ResponseRequestBuilder<M> {
        ResponseRequestBuilder {
            wire: OpenAIResponsesWireRequest::new(M::ID.to_string()),
            model: PhantomData,
        }
    }
}

pub struct ResponseRequestBuilder<M: OpenAIResponsesModel> {
    wire: OpenAIResponsesWireRequest,
    model: PhantomData<fn() -> M>,
}

impl<M: OpenAIResponsesModel> ResponseRequestBuilder<M> {
    pub fn input_text(mut self, input: impl Into<String>) -> Self {
        self.wire.input = Some(OpenAIResponsesInput::Text(input.into()));
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.wire.instructions = Some(instructions.into());
        self
    }

    pub fn max_output_tokens(mut self, max_output_tokens: u64) -> Self {
        self.wire.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn previous_response_id(mut self, id: ResponseId) -> Self {
        self.wire.previous_response_id = Some(id);
        self
    }

    pub fn store(mut self, store: bool) -> Self {
        self.wire.store = Some(store);
        self
    }

    pub fn metadata(mut self, metadata: OpenAIResponseMetadata) -> Self {
        self.wire.metadata = Some(metadata);
        self
    }

    pub fn safety_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.wire.safety_identifier = Some(identifier.into());
        self
    }

    pub fn service_tier(mut self, tier: OpenAIServiceTier) -> Self {
        self.wire.service_tier = Some(tier);
        self
    }

    pub fn background(mut self, background: bool) -> Self {
        self.wire.background = Some(background);
        self
    }

    pub fn max_tool_calls(mut self, maximum: u64) -> Self {
        self.wire.max_tool_calls = Some(maximum);
        self
    }

    pub fn parallel_tool_calls(mut self, parallel: bool) -> Self {
        self.wire.parallel_tool_calls = Some(parallel);
        self
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.wire.include.get_or_insert_with(Vec::new).push(include);
        self
    }

    pub fn truncation(mut self, truncation: OpenAITruncation) -> Self {
        self.wire.truncation = Some(truncation);
        self
    }

    pub fn prompt_cache_options(mut self, options: OpenAIPromptCacheOptions) -> Self {
        self.wire.prompt_cache_options = Some(options);
        self
    }

    pub fn conversation(mut self, conversation: OpenAIConversationReference) -> Self {
        self.wire.conversation = Some(conversation);
        self
    }

    pub fn prompt(mut self, prompt: OpenAIPromptTemplate) -> Self {
        self.wire.prompt = Some(prompt);
        self
    }

    pub fn moderation(mut self, moderation: OpenAIModerationConfig) -> Self {
        self.wire.moderation = Some(moderation);
        self
    }

    pub fn context_management(mut self, entries: Vec<OpenAIContextCompaction>) -> Self {
        self.wire.context_management = Some(entries);
        self
    }
}

impl<M: SupportsItemInput> ResponseRequestBuilder<M> {
    pub fn input_items(mut self, items: Vec<OpenAIResponseInputItem>) -> Self {
        self.wire.input = Some(OpenAIResponsesInput::Items(items));
        self
    }
}

impl<M: SupportsSampling> ResponseRequestBuilder<M> {
    pub fn top_logprobs(mut self, top_logprobs: TopLogprobs) -> Self {
        self.wire.top_logprobs = Some(top_logprobs);
        self
    }
    pub fn temperature(mut self, temperature: Temperature) -> Self {
        self.wire.temperature = Some(temperature.get());
        self
    }

    pub fn top_p(mut self, top_p: TopP) -> Self {
        self.wire.top_p = Some(top_p.get());
        self
    }
}

impl<M: SupportsReasoning> ResponseRequestBuilder<M> {
    pub fn reasoning(mut self, effort: M::Effort) -> Self {
        self.wire.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(effort.into_reasoning_effort()),
            summary: None,
            context: None,
            generate_summary: None,
            extra: Default::default(),
        });
        self
    }

    pub fn reasoning_details(
        mut self,
        effort: M::Effort,
        summary: Option<crate::openai::responses::OpenAIReasoningSummary>,
        context: Option<crate::openai::responses::OpenAIReasoningContext>,
    ) -> Self {
        self.wire.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(effort.into_reasoning_effort()),
            summary,
            context,
            generate_summary: None,
            extra: Default::default(),
        });
        self
    }
}

impl<M: SupportsPromptCacheKey> ResponseRequestBuilder<M> {
    pub fn prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        self.wire.prompt_cache_key = Some(key.into());
        self
    }
}

impl<M: SupportsPromptCacheRetention> ResponseRequestBuilder<M> {
    pub fn prompt_cache_retention(mut self, retention: M::Retention) -> Self {
        self.wire.prompt_cache_retention =
            Some(retention.into_prompt_cache_retention().to_string());
        self
    }
}

impl<M: SupportsStructuredOutput> ResponseRequestBuilder<M> {
    pub fn json_schema(mut self, schema: OpenAIJsonSchema) -> Self {
        self.wire.text = Some(OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::JsonSchema(schema.into())),
            verbosity: None,
            extra: Default::default(),
        });
        self
    }

    pub fn text_config(mut self, text: OpenAIResponsesTextConfig) -> Self {
        self.wire.text = Some(text);
        self
    }
}

impl<M: SupportsImageGenerationTool> ResponseRequestBuilder<M> {
    pub fn image_generation_tool(mut self, tool: OpenAIImageGenerationTool) -> Self {
        self.wire
            .tools
            .get_or_insert_with(Vec::new)
            .push(OpenAIResponsesTool::ImageGeneration(tool));
        self
    }
}

impl<M: OpenAIResponsesModel> ResponseRequestBuilder<M> {
    pub fn build(self) -> PreparedResponseRequest {
        PreparedResponseRequest::new(self.wire, Vec::new())
    }
}
