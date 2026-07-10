use serde::Serialize;

use crate::openai_compatible::ChatCompletionsDialect;

pub trait CompatibleChatModel<D: ChatCompletionsDialect>: Send + Sync + 'static {
    const ID: &'static str;
}

pub trait SupportsChatSampling<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

pub trait SupportsMaxCompletionTokens<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

pub trait SupportsFrequencyPenalty<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

pub trait SupportsChoiceCount<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

pub trait SupportsStructuredOutput<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

pub trait SupportsModalities<D: ChatCompletionsDialect>: CompatibleChatModel<D> {}

/// Converts a dialect/model-specific reasoning level to its wire value.
pub trait IntoChatReasoningEffort: Send + Sync + 'static {
    fn into_reasoning_effort(self) -> String;
}

impl IntoChatReasoningEffort for String {
    fn into_reasoning_effort(self) -> String {
        self
    }
}

impl IntoChatReasoningEffort for &'static str {
    fn into_reasoning_effort(self) -> String {
        self.to_owned()
    }
}

pub trait SupportsReasoning<D: ChatCompletionsDialect>: CompatibleChatModel<D> {
    type Effort: IntoChatReasoningEffort;
}

/// A structured-output object sent as the Chat Completions `response_format`.
///
/// Compatibility dialects differ in the exact accepted shape, so the model
/// capability is compile-time checked while the object itself remains
/// lossless and dialect-defined.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ChatResponseFormat(serde_json::Map<String, serde_json::Value>);

impl ChatResponseFormat {
    pub fn new(value: serde_json::Map<String, serde_json::Value>) -> Self {
        Self(value)
    }

    pub fn as_object(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.0
    }
}

/// A provider-defined output modality identifier such as `text` or `audio`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ChatModality(String);

impl ChatModality {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty()
            && value.len() <= 64
            && value
                .chars()
                .all(|character| !character.is_control() && !character.is_whitespace()))
        .then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
