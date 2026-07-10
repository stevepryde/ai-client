use std::{fmt, marker::PhantomData};

use serde::{Serialize, Serializer};

use crate::openai_compatible::ChatCompletionsDialect;

use super::{
    ChatModality, ChatResponseFormat, CompatibleChatModel, IntoChatReasoningEffort,
    SupportsChatSampling, SupportsChoiceCount, SupportsFrequencyPenalty,
    SupportsMaxCompletionTokens, SupportsModalities, SupportsReasoning, SupportsStructuredOutput,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct MissingMessages;
#[derive(Debug, Clone, Copy, Default)]
pub struct HasMessages;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    Developer,
    User,
    Assistant,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ChatMessage(serde_json::Map<String, serde_json::Value>);

impl ChatMessage {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        let mut object = serde_json::Map::new();
        object.insert(
            "role".into(),
            serde_json::to_value(role).expect("ChatRole always serializes"),
        );
        object.insert("content".into(), serde_json::Value::String(content.into()));
        Self(object)
    }

    /// Creates a custom-dialect message from a complete JSON object.
    ///
    /// This is the explicit full-fidelity escape hatch for multimodal content,
    /// tool messages, and endpoint-specific message fields.
    pub fn from_object(object: serde_json::Map<String, serde_json::Value>) -> Self {
        Self(object)
    }

    pub fn as_object(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.0
    }
}

impl fmt::Debug for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChatMessage")
            .field("content", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("{setting} must be finite and within {minimum}..={maximum}")]
pub struct ChatSamplingError {
    setting: &'static str,
    minimum: f64,
    maximum: f64,
}

macro_rules! sampling {
    ($name:ident, $setting:literal, $min:literal, $max:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name(f64);
        impl $name {
            pub fn new(value: f64) -> Result<Self, ChatSamplingError> {
                if value.is_finite() && ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(ChatSamplingError {
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
            type Error = ChatSamplingError;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}
sampling!(ChatTemperature, "temperature", 0.0, 2.0);
sampling!(ChatTopP, "top_p", 0.0, 1.0);
sampling!(ChatFrequencyPenalty, "frequency_penalty", -2.0, 2.0);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChatBuildError {
    #[error("chat request requires at least one message")]
    EmptyMessages,
    #[error("invalid dynamic chat model ID")]
    InvalidModelId,
    #[error("chat message at index {0} must serialize to a JSON object")]
    InvalidMessage(usize),
    #[error("provider options must serialize to a JSON object")]
    InvalidProviderOptions,
    #[error("provider option `{0}` collides with a typed field")]
    ProviderOptionCollision(String),
    #[error("extension field `{0}` collides with a typed or provider field")]
    ExtraBodyCollision(String),
}

pub struct ChatRequest<D: ChatCompletionsDialect, M: CompatibleChatModel<D>>(
    PhantomData<fn() -> (D, M)>,
);

impl<D: ChatCompletionsDialect, M: CompatibleChatModel<D>> ChatRequest<D, M> {
    pub fn builder() -> ChatRequestBuilder<D, M, MissingMessages> {
        ChatRequestBuilder {
            core: BuilderCore::new(M::ID.to_string()),
            marker: PhantomData,
        }
    }
}

pub struct ChatRequestBuilder<D: ChatCompletionsDialect, M: CompatibleChatModel<D>, State> {
    core: BuilderCore<D>,
    marker: PhantomData<fn() -> (M, State)>,
}

impl<D: ChatCompletionsDialect, M: CompatibleChatModel<D>, S> ChatRequestBuilder<D, M, S> {
    pub fn provider_options(mut self, options: D::ChatOptions) -> Self {
        self.core.options = options;
        self
    }
    pub fn extra_body(mut self, body: serde_json::Map<String, serde_json::Value>) -> Self {
        self.core.extra = body;
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsMaxCompletionTokens<D>,
{
    pub fn max_completion_tokens(mut self, value: u64) -> Self {
        self.core.wire.max_completion_tokens = Some(value);
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsChatSampling<D>,
{
    pub fn temperature(mut self, value: ChatTemperature) -> Self {
        self.core.wire.temperature = Some(value.get());
        self
    }
    pub fn top_p(mut self, value: ChatTopP) -> Self {
        self.core.wire.top_p = Some(value.get());
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsFrequencyPenalty<D>,
{
    pub fn frequency_penalty(mut self, value: ChatFrequencyPenalty) -> Self {
        self.core.wire.frequency_penalty = Some(value.get());
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsChoiceCount<D>,
{
    pub fn choice_count(mut self, value: std::num::NonZeroU32) -> Self {
        self.core.wire.n = Some(value.get());
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsStructuredOutput<D>,
{
    pub fn response_format(mut self, value: ChatResponseFormat) -> Self {
        self.core.wire.response_format = Some(value);
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsModalities<D>,
{
    pub fn modalities(mut self, value: Vec<ChatModality>) -> Self {
        self.core.wire.modalities = Some(value);
        self
    }
}

impl<D, M, S> ChatRequestBuilder<D, M, S>
where
    D: ChatCompletionsDialect,
    M: CompatibleChatModel<D> + SupportsReasoning<D>,
{
    pub fn reasoning_effort(mut self, value: M::Effort) -> Self {
        self.core.wire.reasoning_effort = Some(value.into_reasoning_effort());
        self
    }
}

impl<D: ChatCompletionsDialect, M: CompatibleChatModel<D>>
    ChatRequestBuilder<D, M, MissingMessages>
{
    pub fn messages(mut self, messages: Vec<D::Message>) -> ChatRequestBuilder<D, M, HasMessages> {
        self.core.messages = messages;
        ChatRequestBuilder {
            core: self.core,
            marker: PhantomData,
        }
    }
}

impl<D: ChatCompletionsDialect, M: CompatibleChatModel<D>> ChatRequestBuilder<D, M, HasMessages> {
    pub fn build(self) -> Result<PreparedChatRequest<D>, ChatBuildError> {
        self.core.build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicChatModel(String);
impl DynamicChatModel {
    pub fn new(id: impl Into<String>) -> Result<Self, ChatBuildError> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 256
            || id
                .chars()
                .any(|value| value.is_whitespace() || value.is_control())
        {
            return Err(ChatBuildError::InvalidModelId);
        }
        Ok(Self(id))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct DynamicChatRequest<D: ChatCompletionsDialect>(PhantomData<fn() -> D>);
impl<D: ChatCompletionsDialect> DynamicChatRequest<D> {
    pub fn builder(model: DynamicChatModel) -> DynamicChatRequestBuilder<D, MissingMessages> {
        DynamicChatRequestBuilder {
            core: BuilderCore::new(model.0),
            marker: PhantomData,
        }
    }
}

pub struct DynamicChatRequestBuilder<D: ChatCompletionsDialect, State> {
    core: BuilderCore<D>,
    marker: PhantomData<State>,
}
impl<D: ChatCompletionsDialect, S> DynamicChatRequestBuilder<D, S> {
    /// Sets a field without claiming that the runtime model supports it.
    pub fn max_completion_tokens(mut self, value: u64) -> Self {
        self.core.wire.max_completion_tokens = Some(value);
        self
    }
    pub fn temperature(mut self, value: ChatTemperature) -> Self {
        self.core.wire.temperature = Some(value.get());
        self
    }
    pub fn top_p(mut self, value: ChatTopP) -> Self {
        self.core.wire.top_p = Some(value.get());
        self
    }
    pub fn frequency_penalty(mut self, value: ChatFrequencyPenalty) -> Self {
        self.core.wire.frequency_penalty = Some(value.get());
        self
    }
    pub fn choice_count(mut self, value: std::num::NonZeroU32) -> Self {
        self.core.wire.n = Some(value.get());
        self
    }
    pub fn response_format(mut self, value: ChatResponseFormat) -> Self {
        self.core.wire.response_format = Some(value);
        self
    }
    pub fn modalities(mut self, value: Vec<ChatModality>) -> Self {
        self.core.wire.modalities = Some(value);
        self
    }
    pub fn reasoning_effort(mut self, value: impl Into<String>) -> Self {
        self.core.wire.reasoning_effort = Some(value.into());
        self
    }
    pub fn provider_options(mut self, options: D::ChatOptions) -> Self {
        self.core.options = options;
        self
    }
    pub fn extra_body(mut self, body: serde_json::Map<String, serde_json::Value>) -> Self {
        self.core.extra = body;
        self
    }
}
impl<D: ChatCompletionsDialect> DynamicChatRequestBuilder<D, MissingMessages> {
    pub fn messages(
        mut self,
        messages: Vec<D::Message>,
    ) -> DynamicChatRequestBuilder<D, HasMessages> {
        self.core.messages = messages;
        DynamicChatRequestBuilder {
            core: self.core,
            marker: PhantomData,
        }
    }
}
impl<D: ChatCompletionsDialect> DynamicChatRequestBuilder<D, HasMessages> {
    pub fn build(self) -> Result<PreparedChatRequest<D>, ChatBuildError> {
        self.core.build()
    }
}

struct BuilderCore<D: ChatCompletionsDialect> {
    wire: ChatWireRequest,
    messages: Vec<D::Message>,
    options: D::ChatOptions,
    extra: serde_json::Map<String, serde_json::Value>,
}
impl<D: ChatCompletionsDialect> BuilderCore<D> {
    fn new(model: String) -> Self {
        Self {
            wire: ChatWireRequest::new(model),
            messages: Vec::new(),
            options: D::ChatOptions::default(),
            extra: Default::default(),
        }
    }
    fn build(self) -> Result<PreparedChatRequest<D>, ChatBuildError> {
        if self.messages.is_empty() {
            return Err(ChatBuildError::EmptyMessages);
        }
        let messages = self
            .messages
            .into_iter()
            .enumerate()
            .map(|(index, message)| {
                let value = serde_json::to_value(message)
                    .map_err(|_| ChatBuildError::InvalidMessage(index))?;
                if value.is_object() {
                    Ok(value)
                } else {
                    Err(ChatBuildError::InvalidMessage(index))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let options = serde_json::to_value(self.options)
            .map_err(|_| ChatBuildError::InvalidProviderOptions)?;
        let serde_json::Value::Object(options) = options else {
            return Err(ChatBuildError::InvalidProviderOptions);
        };
        let mut occupied = self.wire.occupied_keys();
        if let Some(key) = options.keys().find(|key| occupied.contains(*key)) {
            return Err(ChatBuildError::ProviderOptionCollision(key.clone()));
        }
        for key in options.keys() {
            occupied.insert(key.clone());
        }
        if let Some(key) = self.extra.keys().find(|key| occupied.contains(*key)) {
            return Err(ChatBuildError::ExtraBodyCollision(key.clone()));
        }
        let mut wire = self.wire;
        wire.messages = messages;
        Ok(PreparedChatRequest {
            wire,
            options,
            extra: self.extra,
            marker: PhantomData,
        })
    }
}

pub struct PreparedChatRequest<D: ChatCompletionsDialect> {
    wire: ChatWireRequest,
    options: serde_json::Map<String, serde_json::Value>,
    extra: serde_json::Map<String, serde_json::Value>,
    marker: PhantomData<fn() -> D>,
}
impl<D: ChatCompletionsDialect> PreparedChatRequest<D> {
    pub fn model_id(&self) -> &str {
        &self.wire.model
    }
    pub(crate) fn set_stream(&mut self, stream: bool) {
        self.wire.stream = stream.then_some(true);
    }
}
impl<D: ChatCompletionsDialect> fmt::Debug for PreparedChatRequest<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedChatRequest")
            .field("model", &self.wire.model)
            .field("request", &"[redacted]")
            .finish()
    }
}
impl<D: ChatCompletionsDialect> Serialize for PreparedChatRequest<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object =
            match serde_json::to_value(&self.wire).map_err(serde::ser::Error::custom)? {
                serde_json::Value::Object(object) => object,
                _ => unreachable!(),
            };
        object.extend(self.options.clone());
        object.extend(self.extra.clone());
        object.serialize(serializer)
    }
}

#[derive(Serialize)]
struct ChatWireRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modalities: Option<Vec<ChatModality>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}
impl ChatWireRequest {
    fn new(model: String) -> Self {
        Self {
            model,
            messages: Vec::new(),
            max_completion_tokens: None,
            frequency_penalty: None,
            n: None,
            response_format: None,
            modalities: None,
            reasoning_effort: None,
            temperature: None,
            top_p: None,
            stream: None,
        }
    }
    fn occupied_keys(&self) -> std::collections::BTreeSet<String> {
        [
            "model",
            "messages",
            "max_completion_tokens",
            "frequency_penalty",
            "n",
            "response_format",
            "modalities",
            "reasoning_effort",
            "temperature",
            "top_p",
            "stream",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
}
