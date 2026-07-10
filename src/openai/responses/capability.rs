use crate::openai::OpenAIReasoningEffort;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("{setting} must be finite and within {minimum}..={maximum}")]
pub struct SamplingValueError {
    setting: &'static str,
    minimum: f64,
    maximum: f64,
}

macro_rules! bounded_sampling_value {
    ($name:ident, $setting:literal, $min:literal, $max:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name(f64);

        impl $name {
            pub fn new(value: f64) -> Result<Self, SamplingValueError> {
                if value.is_finite() && ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(SamplingValueError {
                        setting: $setting,
                        minimum: $min,
                        maximum: $max,
                    })
                }
            }

            pub fn get(self) -> f64 {
                self.0
            }
        }

        impl TryFrom<f64> for $name {
            type Error = SamplingValueError;
            fn try_from(value: f64) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

bounded_sampling_value!(Temperature, "temperature", 0.0, 2.0);
bounded_sampling_value!(TopP, "top_p", 0.0, 1.0);

/// A model accepted by OpenAI's native Responses resource.
///
/// This trait is intentionally unsealed so applications can describe native
/// OpenAI fine-tuned models and aliases without waiting for a crate release.
pub trait OpenAIResponsesModel: Send + Sync + 'static {
    const ID: &'static str;
}

/// Converts a model-family-specific effort into the private wire vocabulary.
pub trait IntoReasoningEffort {
    fn into_reasoning_effort(self) -> OpenAIReasoningEffort;
}

pub trait SupportsReasoning: OpenAIResponsesModel {
    type Effort: IntoReasoningEffort;
}

/// Initial request state before a reasoning or sampling mode is selected.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultMode;

/// Request state after explicitly selecting `reasoning.effort = "none"`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoReasoningMode;

/// Request state after selecting a non-`none` reasoning effort.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReasoningMode;

/// Request state after configuring a sampling parameter.
#[derive(Debug, Clone, Copy, Default)]
pub struct SamplingMode;

/// Compile-time gate for models that support sampling from a given request
/// mode. The state parameter prevents combining sampling with an incompatible
/// reasoning effort in either builder-call order.
pub trait SupportsSamplingFrom<State>: OpenAIResponsesModel {}

/// Models that accept `reasoning.effort = "none"`.
pub trait SupportsNoReasoning: SupportsReasoning {}
pub trait SupportsPromptCacheKey: OpenAIResponsesModel {}
pub trait SupportsPromptCacheRetention: OpenAIResponsesModel {
    type Retention: IntoPromptCacheRetention;
}
pub trait SupportsStructuredOutput: OpenAIResponsesModel {}
pub trait SupportsImageGenerationTool: OpenAIResponsesModel {}
pub trait IntoResponsesTool {
    fn into_responses_tool(self) -> crate::openai::responses::OpenAIResponsesTool;
}
pub trait SupportsTool<T: IntoResponsesTool>: OpenAIResponsesModel {}
pub trait SupportsTools: OpenAIResponsesModel {}
/// Opt-in gate for known-model builders that accept the heterogeneous
/// Responses item-input union. Downstream model markers must opt in explicitly;
/// the checked-in OpenAI models are reviewed against their Responses and image
/// input documentation before receiving this capability.
pub trait SupportsItemInput: OpenAIResponsesModel {}

pub trait IntoPromptCacheRetention {
    fn into_prompt_cache_retention(self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCacheRetention {
    InMemory,
    Hours24,
}

impl IntoPromptCacheRetention for PromptCacheRetention {
    fn into_prompt_cache_retention(self) -> &'static str {
        match self {
            Self::InMemory => "in_memory",
            Self::Hours24 => "24h",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gpt5_5PromptCacheRetention {
    Hours24,
}

impl IntoPromptCacheRetention for Gpt5_5PromptCacheRetention {
    fn into_prompt_cache_retention(self) -> &'static str {
        "24h"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gpt5ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gpt5_1ReasoningEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedReasoningEffort {
    Low,
    Medium,
    High,
    XHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProReasoningEffort {
    Medium,
    High,
    XHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexReasoningEffort {
    Low,
    Medium,
    High,
    XHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gpt5ProReasoningEffort {
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gpt5_6ReasoningEffort {
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

macro_rules! effort_conversion {
    ($ty:ty, {$($variant:ident),+ $(,)?}) => {
        impl IntoReasoningEffort for $ty {
            fn into_reasoning_effort(self) -> OpenAIReasoningEffort {
                match self {
                    $(Self::$variant => OpenAIReasoningEffort::$variant),+
                }
            }
        }
    };
}

effort_conversion!(Gpt5ReasoningEffort, { Minimal, Low, Medium, High });
effort_conversion!(Gpt5_1ReasoningEffort, { Low, Medium, High });
effort_conversion!(ExtendedReasoningEffort, { Low, Medium, High, XHigh });
effort_conversion!(ProReasoningEffort, { Medium, High, XHigh });
effort_conversion!(CodexReasoningEffort, { Low, Medium, High, XHigh });
effort_conversion!(Gpt5ProReasoningEffort, { High });
effort_conversion!(Gpt5_6ReasoningEffort, { Low, Medium, High, XHigh, Max });
