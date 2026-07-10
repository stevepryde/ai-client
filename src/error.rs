use std::{collections::BTreeMap, fmt};

use reqwest::StatusCode;

pub type AiResult<T> = std::result::Result<T, AiError>;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    OpenAI,
    Gemini,
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAI => f.write_str("OpenAI"),
            Self::Gemini => f.write_str("Gemini"),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorKind {
    MissingApiKey,
    InvalidApiKey,
    InvalidBaseUrl,
    InvalidHeader,
    InvalidModel,
    HttpClient,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportErrorKind {
    Connect,
    Body,
    Request,
    Unknown,
}

/// A bounded, lossy view of a provider payload. Content is available only by
/// explicit access and is redacted from `Debug` and all error displays.
#[derive(Clone, PartialEq, Eq)]
pub struct BodySnippet {
    text: String,
    truncated: bool,
}

impl BodySnippet {
    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    pub(crate) fn from_bytes(bytes: &[u8], truncated: bool) -> Self {
        Self {
            text: String::from_utf8_lossy(bytes).into_owned(),
            truncated,
        }
    }
}

impl fmt::Debug for BodySnippet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BodySnippet")
            .field("content", &"[redacted]")
            .field("length", &self.text.len())
            .field("truncated", &self.truncated)
            .finish()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RateLimitMetadata {
    pub limit_requests: Option<String>,
    pub limit_tokens: Option<String>,
    pub remaining_requests: Option<String>,
    pub remaining_tokens: Option<String>,
    pub reset_requests: Option<String>,
    pub reset_tokens: Option<String>,
    /// Additional rate-limit headers, normalized to lowercase names.
    pub other: BTreeMap<String, String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseMetadata {
    pub status: StatusCode,
    pub request_id: Option<String>,
    /// Raw `Retry-After`; providers may use seconds or an HTTP date.
    pub retry_after: Option<String>,
    pub rate_limit: RateLimitMetadata,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiResponse<T> {
    data: T,
    metadata: ResponseMetadata,
}

impl<T> AiResponse<T> {
    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn metadata(&self) -> &ResponseMetadata {
        &self.metadata
    }

    pub fn into_inner(self) -> T {
        self.data
    }

    pub fn into_parts(self) -> (T, ResponseMetadata) {
        (self.data, self.metadata)
    }

    pub(crate) fn new(data: T, metadata: ResponseMetadata) -> Self {
        Self { data, metadata }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderApiError {
    message: String,
    code: Option<String>,
    kind: Option<String>,
    param: Option<String>,
    body: BodySnippet,
}

impl ProviderApiError {
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

    pub fn body(&self) -> &BodySnippet {
        &self.body
    }

    pub(crate) fn new(
        message: impl Into<String>,
        code: Option<String>,
        kind: Option<String>,
        param: Option<String>,
        body: BodySnippet,
    ) -> Self {
        Self {
            message: message.into(),
            code,
            kind,
            param,
            body,
        }
    }
}

impl fmt::Debug for ProviderApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderApiError")
            .field("message", &"[redacted]")
            .field("code", &self.code)
            .field("kind", &self.kind)
            .field("param", &self.param.as_ref().map(|_| "[redacted]"))
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonDecodeError {
    pub category: JsonDecodeErrorCategory,
    pub line: usize,
    pub column: usize,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonDecodeErrorCategory {
    Io,
    Syntax,
    Data,
    Eof,
}

impl From<serde_json::Error> for JsonDecodeError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            category: match value.classify() {
                serde_json::error::Category::Io => JsonDecodeErrorCategory::Io,
                serde_json::error::Category::Syntax => JsonDecodeErrorCategory::Syntax,
                serde_json::error::Category::Data => JsonDecodeErrorCategory::Data,
                serde_json::error::Category::Eof => JsonDecodeErrorCategory::Eof,
            },
            line: value.line(),
            column: value.column(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AiError {
    #[error("invalid client configuration ({kind:?}): {message}")]
    Config {
        kind: ConfigErrorKind,
        message: String,
    },
    #[error("{provider} operation `{operation}` timed out")]
    Timeout {
        provider: AiProvider,
        operation: &'static str,
    },
    #[error("{provider} transport failure during `{operation}` ({kind:?})")]
    Transport {
        provider: AiProvider,
        operation: &'static str,
        kind: TransportErrorKind,
    },
    #[error("{provider} returned malformed JSON during `{operation}` ({status})", status = metadata.status)]
    Decode {
        provider: AiProvider,
        operation: &'static str,
        metadata: Box<ResponseMetadata>,
        error: JsonDecodeError,
    },
    #[error("{provider} API error during `{operation}` ({status})", status = metadata.status)]
    Api {
        provider: AiProvider,
        operation: &'static str,
        metadata: Box<ResponseMetadata>,
        error: Box<ProviderApiError>,
    },
}

impl AiError {
    pub(crate) fn config(kind: ConfigErrorKind, message: impl Into<String>) -> Self {
        Self::Config {
            kind,
            message: message.into(),
        }
    }
}
