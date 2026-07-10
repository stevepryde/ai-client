use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::Stream;

use crate::error::{AiProvider, JsonDecodeError, TransportErrorKind};

/// Crate-owned asynchronous stream returned by provider streaming APIs.
#[non_exhaustive]
pub struct AiStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<T, AiStreamError>> + Send + 'static>>,
}

impl<T> AiStream<T> {
    pub(crate) fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<T, AiStreamError>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl<T> Stream for AiStream<T> {
    type Item = Result<T, AiStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().inner.as_mut().poll_next(cx)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStreamErrorKind {
    Timeout,
    Transport(TransportErrorKind),
    InvalidUtf8,
    MalformedSse,
    MalformedJson(JsonDecodeError),
    UnexpectedEof,
}

impl fmt::Display for AiStreamErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => f.write_str("timeout"),
            Self::Transport(kind) => write!(f, "transport {kind:?}"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8"),
            Self::MalformedSse => f.write_str("malformed SSE"),
            Self::MalformedJson(_) => f.write_str("malformed JSON"),
            Self::UnexpectedEof => f.write_str("unexpected end of stream"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{provider} stream operation `{operation}` failed ({kind})")]
pub struct AiStreamError {
    provider: AiProvider,
    operation: &'static str,
    kind: AiStreamErrorKind,
}

impl AiStreamError {
    pub fn provider(&self) -> AiProvider {
        self.provider
    }

    pub fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn kind(&self) -> &AiStreamErrorKind {
        &self.kind
    }

    pub(crate) fn new(
        provider: AiProvider,
        operation: &'static str,
        kind: AiStreamErrorKind,
    ) -> Self {
        Self {
            provider,
            operation,
            kind,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SseEventMetadata {
    event: Option<String>,
    id: Option<String>,
    retry: Option<Duration>,
}

impl SseEventMetadata {
    pub fn event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Reconnection delay requested by the server, in milliseconds.
    pub fn retry(&self) -> Option<Duration> {
        self.retry
    }

    pub fn into_parts(self) -> (Option<String>, Option<String>, Option<Duration>) {
        (self.event, self.id, self.retry)
    }

    pub(crate) fn new(event: Option<String>, id: Option<String>, retry: Option<Duration>) -> Self {
        Self { event, id, retry }
    }
}

/// One decoded SSE JSON event with framing metadata and its complete raw value.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct SseJsonEvent<T> {
    data: T,
    raw: serde_json::Value,
    metadata: SseEventMetadata,
}

impl<T> SseJsonEvent<T> {
    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }

    pub fn metadata(&self) -> &SseEventMetadata {
        &self.metadata
    }

    pub fn into_parts(self) -> (T, serde_json::Value, SseEventMetadata) {
        (self.data, self.raw, self.metadata)
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub(crate) fn new(data: T, raw: serde_json::Value, metadata: SseEventMetadata) -> Self {
        Self {
            data,
            raw,
            metadata,
        }
    }
}
