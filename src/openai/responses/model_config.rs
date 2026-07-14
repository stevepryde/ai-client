use std::{fmt, marker::PhantomData};

use super::{
    DefaultMode, IntoPromptCacheRetention, IntoReasoningEffort, NoReasoningMode,
    OpenAIResponsesModel, OpenAIResponsesReasoning, ReasoningMode, SamplingMode,
    SupportsNoReasoning, SupportsPromptCacheRetention, SupportsReasoning, SupportsSamplingFrom,
    Temperature, TopLogprobs, TopP,
};
use crate::openai::responses::request::OpenAIResponsesWireRequest;

/// Reusable, compile-time checked configuration for one Responses model.
///
/// Request content is deliberately separate. This lets applications build a
/// request once and select a typed model at the call site without duplicating
/// prompts, schemas, or tools in every model branch.
pub struct ResponseModelConfig<M: OpenAIResponsesModel, State = DefaultMode> {
    top_logprobs: Option<TopLogprobs>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    prompt_cache_retention: Option<String>,
    reasoning: Option<OpenAIResponsesReasoning>,
    state: PhantomData<fn() -> (M, State)>,
}

impl<M: OpenAIResponsesModel, State> Clone for ResponseModelConfig<M, State> {
    fn clone(&self) -> Self {
        Self {
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            prompt_cache_retention: self.prompt_cache_retention.clone(),
            reasoning: self.reasoning.clone(),
            state: PhantomData,
        }
    }
}

impl<M: OpenAIResponsesModel> ResponseModelConfig<M, DefaultMode> {
    /// Start configuring a custom or fine-tuned Responses model marker.
    pub fn new() -> Self {
        Self {
            top_logprobs: None,
            temperature: None,
            top_p: None,
            prompt_cache_retention: None,
            reasoning: None,
            state: PhantomData,
        }
    }
}

impl<M: OpenAIResponsesModel> Default for ResponseModelConfig<M, DefaultMode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: OpenAIResponsesModel, State> ResponseModelConfig<M, State> {
    pub fn model_id(&self) -> &'static str {
        M::ID
    }

    pub(crate) fn apply(self, wire: &mut OpenAIResponsesWireRequest) {
        wire.model = M::ID.to_string();
        wire.top_logprobs = self.top_logprobs;
        wire.temperature = self.temperature;
        wire.top_p = self.top_p;
        wire.prompt_cache_retention = self.prompt_cache_retention;
        wire.reasoning = self.reasoning;
    }
}

impl<M: OpenAIResponsesModel, State> fmt::Debug for ResponseModelConfig<M, State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResponseModelConfig")
            .field("model", &M::ID)
            .field("config", &"[redacted]")
            .finish()
    }
}

impl<M, State> ResponseModelConfig<M, State>
where
    M: OpenAIResponsesModel + SupportsSamplingFrom<State>,
{
    fn into_sampling_mode(self) -> ResponseModelConfig<M, SamplingMode> {
        ResponseModelConfig {
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            prompt_cache_retention: self.prompt_cache_retention,
            reasoning: self.reasoning,
            state: PhantomData,
        }
    }

    pub fn top_logprobs(
        mut self,
        top_logprobs: TopLogprobs,
    ) -> ResponseModelConfig<M, SamplingMode> {
        self.top_logprobs = Some(top_logprobs);
        self.into_sampling_mode()
    }

    pub fn temperature(mut self, temperature: Temperature) -> ResponseModelConfig<M, SamplingMode> {
        self.temperature = Some(temperature.get());
        self.into_sampling_mode()
    }

    pub fn top_p(mut self, top_p: TopP) -> ResponseModelConfig<M, SamplingMode> {
        self.top_p = Some(top_p.get());
        self.into_sampling_mode()
    }
}

impl<M: SupportsReasoning> ResponseModelConfig<M, DefaultMode> {
    pub fn reasoning(mut self, effort: M::Effort) -> ResponseModelConfig<M, ReasoningMode> {
        self.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(effort.into_reasoning_effort()),
            summary: None,
            context: None,
            generate_summary: None,
            extra: Default::default(),
        });
        ResponseModelConfig {
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            prompt_cache_retention: self.prompt_cache_retention,
            reasoning: self.reasoning,
            state: PhantomData,
        }
    }

    pub fn reasoning_details(
        mut self,
        effort: M::Effort,
        summary: Option<crate::openai::responses::OpenAIReasoningSummary>,
        context: Option<crate::openai::responses::OpenAIReasoningContext>,
    ) -> ResponseModelConfig<M, ReasoningMode> {
        self.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(effort.into_reasoning_effort()),
            summary,
            context,
            generate_summary: None,
            extra: Default::default(),
        });
        ResponseModelConfig {
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            prompt_cache_retention: self.prompt_cache_retention,
            reasoning: self.reasoning,
            state: PhantomData,
        }
    }
}

impl<M: SupportsNoReasoning> ResponseModelConfig<M, DefaultMode> {
    pub fn reasoning_none(mut self) -> ResponseModelConfig<M, NoReasoningMode> {
        self.reasoning = Some(OpenAIResponsesReasoning {
            mode: None,
            effort: Some(crate::openai::OpenAIReasoningEffort::None),
            summary: None,
            context: None,
            generate_summary: None,
            extra: Default::default(),
        });
        ResponseModelConfig {
            top_logprobs: self.top_logprobs,
            temperature: self.temperature,
            top_p: self.top_p,
            prompt_cache_retention: self.prompt_cache_retention,
            reasoning: self.reasoning,
            state: PhantomData,
        }
    }
}

impl<M: SupportsPromptCacheRetention, State> ResponseModelConfig<M, State> {
    pub fn prompt_cache_retention(mut self, retention: M::Retention) -> Self {
        self.prompt_cache_retention = Some(retention.into_prompt_cache_retention().to_string());
        self
    }
}
