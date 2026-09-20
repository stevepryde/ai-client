//! OpenRouter's Responses API, sharing the Open Responses wire protocol.
//! Chat-completions compatibility remains in `openai_compatible`.
use crate::openai::responses::{OpenAIResponsesModel, ResponseModelConfig, ResponsesResource};
use crate::{
    error::{AiProvider, AiResult},
    openai::{OpenAIClient, OpenAIClientBuilder},
};

pub struct OpenRouterClient(OpenAIClient, bool);

impl OpenRouterClient {
    pub fn builder() -> OpenRouterClientBuilder {
        let mut inner = OpenAIClient::builder().base_url("https://openrouter.ai/api/v1");
        inner.provider = Some(AiProvider::OpenRouter);
        OpenRouterClientBuilder(inner, false)
    }
    pub fn responses(&self) -> ResponsesResource<'_> {
        self.0.responses().with_zdr(self.1)
    }
}

#[derive(Debug, Clone)]
pub struct OpenRouterClientBuilder(OpenAIClientBuilder, bool);
impl OpenRouterClientBuilder {
    pub fn api_key(mut self, key: String) -> Self {
        self.0 = self.0.api_key(key);
        self
    }
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.0 = self.0.base_url(url);
        self
    }
    pub fn request_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.0 = self.0.request_timeout(timeout);
        self
    }
    /// Require Zero Data Retention endpoints for every Responses inference request.
    /// This cannot weaken account-level ZDR enforcement.
    pub fn zdr(mut self, enabled: bool) -> Self {
        self.1 = enabled;
        self
    }
    pub fn build(self) -> AiResult<OpenRouterClient> {
        self.0
            .build()
            .map(|client| OpenRouterClient(client, self.1))
    }
}

macro_rules! model {
    ($name:ident, $id:literal) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl OpenAIResponsesModel for $name {
            const ID: &'static str = $id;
        }
        impl crate::openai::responses::SupportsItemInput for $name {}
        impl $name {
            pub fn config() -> ResponseModelConfig<Self> {
                ResponseModelConfig::new()
            }
        }
    };
}
model!(KimiK3, "moonshotai/kimi-k3");
model!(Euryale70B, "sao10k/l3.3-euryale-70b");

impl crate::openai::responses::SupportsReasoning for KimiK3 {
    type Effort = KimiK3ReasoningEffort;
}
impl crate::openai::responses::SupportsNoReasoning for KimiK3 {}

/// Reasoning efforts advertised by OpenRouter for Kimi K3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KimiK3ReasoningEffort {
    Low,
    High,
    Max,
}
impl crate::openai::responses::IntoReasoningEffort for KimiK3ReasoningEffort {
    fn into_reasoning_effort(self) -> crate::openai::OpenAIReasoningEffort {
        use crate::openai::OpenAIReasoningEffort;
        match self {
            Self::Low => OpenAIReasoningEffort::Low,
            Self::High => OpenAIReasoningEffort::High,
            Self::Max => OpenAIReasoningEffort::Max,
        }
    }
}
