use std::{fmt, marker::PhantomData};

use serde::{Serialize, Serializer};

use crate::openai::create_response::{
    OpenAIImageGenerationTool, OpenAIResponsesInput, OpenAIResponsesReasoning,
    OpenAIResponsesTextConfig, OpenAIResponsesTextFormat, OpenAIResponsesTool,
};
use crate::openai::OpenAIJsonSchema;

use super::{
    dynamic::ValidationWarning, IntoPromptCacheRetention, IntoReasoningEffort,
    OpenAIResponsesModel, SupportsImageGenerationTool, SupportsPromptCacheKey,
    SupportsPromptCacheRetention, SupportsReasoning, SupportsSampling, SupportsStructuredOutput,
    Temperature, TopP,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct MissingInput;

#[derive(Debug, Clone, Copy, Default)]
pub struct HasInput;

/// Entry point for a compile-time checked Responses request.
pub struct ResponseRequest<M: OpenAIResponsesModel>(PhantomData<fn() -> M>);

impl<M: OpenAIResponsesModel> ResponseRequest<M> {
    pub fn builder() -> ResponseRequestBuilder<M, MissingInput> {
        ResponseRequestBuilder {
            wire: OpenAIResponsesWireRequest::new(M::ID.to_string()),
            model: PhantomData,
            state: PhantomData,
        }
    }
}

pub struct ResponseRequestBuilder<M: OpenAIResponsesModel, State> {
    wire: OpenAIResponsesWireRequest,
    model: PhantomData<fn() -> M>,
    state: PhantomData<State>,
}

impl<M: OpenAIResponsesModel, S> ResponseRequestBuilder<M, S> {
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.wire.instructions = Some(instructions.into());
        self
    }

    pub fn max_output_tokens(mut self, max_output_tokens: u64) -> Self {
        self.wire.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.wire.previous_response_id = Some(id.into());
        self
    }

    pub fn store(mut self, store: bool) -> Self {
        self.wire.store = Some(store);
        self
    }
}

impl<M: OpenAIResponsesModel> ResponseRequestBuilder<M, MissingInput> {
    pub fn input(
        mut self,
        input: impl Into<OpenAIResponsesInput>,
    ) -> ResponseRequestBuilder<M, HasInput> {
        self.wire.input = Some(input.into());
        ResponseRequestBuilder {
            wire: self.wire,
            model: PhantomData,
            state: PhantomData,
        }
    }
}

impl<M: SupportsSampling, S> ResponseRequestBuilder<M, S> {
    pub fn temperature(mut self, temperature: Temperature) -> Self {
        self.wire.temperature = Some(temperature.get());
        self
    }

    pub fn top_p(mut self, top_p: TopP) -> Self {
        self.wire.top_p = Some(top_p.get());
        self
    }
}

impl<M: SupportsReasoning, S> ResponseRequestBuilder<M, S> {
    pub fn reasoning(mut self, effort: M::Effort) -> Self {
        self.wire.reasoning = Some(OpenAIResponsesReasoning {
            effort: Some(effort.into_reasoning_effort()),
        });
        self
    }
}

impl<M: SupportsPromptCacheKey, S> ResponseRequestBuilder<M, S> {
    pub fn prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        self.wire.prompt_cache_key = Some(key.into());
        self
    }
}

impl<M: SupportsPromptCacheRetention, S> ResponseRequestBuilder<M, S> {
    pub fn prompt_cache_retention(mut self, retention: M::Retention) -> Self {
        self.wire.prompt_cache_retention =
            Some(retention.into_prompt_cache_retention().to_string());
        self
    }
}

impl<M: SupportsStructuredOutput, S> ResponseRequestBuilder<M, S> {
    pub fn json_schema(mut self, schema: OpenAIJsonSchema) -> Self {
        self.wire.text = Some(OpenAIResponsesTextConfig {
            format: Some(OpenAIResponsesTextFormat::JsonSchema(schema)),
        });
        self
    }
}

impl<M: SupportsImageGenerationTool, S> ResponseRequestBuilder<M, S> {
    pub fn image_generation_tool(mut self, tool: OpenAIImageGenerationTool) -> Self {
        self.wire
            .tools
            .get_or_insert_with(Vec::new)
            .push(OpenAIResponsesTool::ImageGeneration(tool));
        self
    }
}

impl<M: OpenAIResponsesModel> ResponseRequestBuilder<M, HasInput> {
    pub fn build(self) -> PreparedResponseRequest {
        PreparedResponseRequest::new(self.wire, Vec::new())
    }
}

/// Non-generic request handed to the shared transport after builder checks.
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
    pub(crate) model: String,
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
    pub(crate) stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_cache_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<OpenAIResponsesTextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning: Option<OpenAIResponsesReasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tools: Option<Vec<OpenAIResponsesTool>>,
}

impl OpenAIResponsesWireRequest {
    pub(crate) fn new(model: String) -> Self {
        Self {
            model,
            input: None,
            instructions: None,
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            stream: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            text: None,
            previous_response_id: None,
            store: None,
            reasoning: None,
            tools: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::responses::{ExtendedReasoningEffort, Gpt5_4};

    #[test]
    fn typed_builder_erases_to_one_redacted_wire_request() {
        let request = ResponseRequest::<Gpt5_4>::builder()
            .input(OpenAIResponsesInput::Text("private prompt".into()))
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
}
