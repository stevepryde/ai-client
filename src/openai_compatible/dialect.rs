use std::fmt;

use serde::Serialize;

pub trait OpenAICompatibleDialect: Send + Sync + 'static {
    const NAME: &'static str;
}

pub trait ChatCompletionsDialect: OpenAICompatibleDialect {
    type ChatOptions: Default + Serialize + Send + Sync + 'static;
    type Message: Serialize + Send + Sync + 'static;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CustomDialect;

impl OpenAICompatibleDialect for CustomDialect {
    const NAME: &'static str = "custom";
}

impl ChatCompletionsDialect for CustomDialect {
    type ChatOptions = CustomChatOptions;
    type Message = crate::openai_compatible::chat::ChatMessage;
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CustomChatOptions {}

#[derive(Clone)]
pub struct CompatibleAuth(CompatibleAuthKind);

#[derive(Clone)]
pub(crate) enum CompatibleAuthKind {
    None,
    Bearer(String),
    Header { name: String, value: String },
}

impl CompatibleAuth {
    pub fn none() -> Self {
        Self(CompatibleAuthKind::None)
    }
    pub fn bearer(secret: impl Into<String>) -> Self {
        Self(CompatibleAuthKind::Bearer(secret.into()))
    }

    pub fn header(name: impl Into<String>, secret: impl Into<String>) -> Self {
        Self(CompatibleAuthKind::Header {
            name: name.into(),
            value: secret.into(),
        })
    }

    pub(crate) fn into_inner(self) -> CompatibleAuthKind {
        self.0
    }
}

impl fmt::Debug for CompatibleAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            CompatibleAuthKind::None => f.write_str("CompatibleAuth::None"),
            CompatibleAuthKind::Bearer(_) => f.write_str("CompatibleAuth::Bearer([redacted])"),
            CompatibleAuthKind::Header { name, .. } => f
                .debug_struct("CompatibleAuth::Header")
                .field("name", name)
                .field("value", &"[redacted]")
                .finish(),
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct CompatibleErrorDetails {
    message: String,
    code: Option<String>,
    kind: Option<String>,
    param: Option<String>,
}

impl CompatibleErrorDetails {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Self::default()
        }
    }
    pub fn with_code(mut self, value: impl Into<String>) -> Self {
        self.code = Some(value.into());
        self
    }
    pub fn with_kind(mut self, value: impl Into<String>) -> Self {
        self.kind = Some(value.into());
        self
    }
    pub fn with_param(mut self, value: impl Into<String>) -> Self {
        self.param = Some(value.into());
        self
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }
    pub fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }
    pub fn param(&self) -> Option<&str> {
        self.param.as_deref()
    }
    pub(crate) fn into_parts(self) -> (String, Option<String>, Option<String>, Option<String>) {
        (self.message, self.code, self.kind, self.param)
    }
}

impl fmt::Debug for CompatibleErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompatibleErrorDetails")
            .field("message", &"[redacted]")
            .field("code", &self.code)
            .field("kind", &self.kind)
            .field("param", &self.param.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

pub trait CompatibleErrorDecoder: Send + Sync + 'static {
    fn decode(&self, body: &[u8]) -> CompatibleErrorDetails;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OpenAICompatibleErrorDecoder;

impl CompatibleErrorDecoder for OpenAICompatibleErrorDecoder {
    fn decode(&self, body: &[u8]) -> CompatibleErrorDetails {
        #[derive(serde::Deserialize)]
        struct Envelope {
            error: Option<Detail>,
        }
        #[derive(serde::Deserialize)]
        struct Detail {
            message: Option<String>,
            code: Option<serde_json::Value>,
            #[serde(rename = "type")]
            kind: Option<String>,
            param: Option<serde_json::Value>,
        }
        fn scalar(value: Option<serde_json::Value>) -> Option<String> {
            match value? {
                serde_json::Value::String(value) => Some(value),
                serde_json::Value::Number(value) => Some(value.to_string()),
                serde_json::Value::Bool(value) => Some(value.to_string()),
                _ => None,
            }
        }
        let detail = serde_json::from_slice::<Envelope>(body)
            .ok()
            .and_then(|value| value.error);
        CompatibleErrorDetails {
            message: detail
                .as_ref()
                .and_then(|value| value.message.clone())
                .unwrap_or_else(|| "unrecognized compatible API error".into()),
            code: detail.as_ref().and_then(|value| scalar(value.code.clone())),
            kind: detail.as_ref().and_then(|value| value.kind.clone()),
            param: detail.and_then(|value| scalar(value.param)),
        }
    }
}
