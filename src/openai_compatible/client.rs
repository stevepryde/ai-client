use std::{fmt, marker::PhantomData, sync::Arc, time::Duration};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, USER_AGENT};

use crate::{
    core::http::{HttpTransport, HttpTransportConfig},
    error::{AiError, AiProvider, AiResult, ConfigErrorKind},
};

use super::{
    chat::ChatResource, ChatCompletionsDialect, CompatibleAuth, CompatibleErrorDecoder,
    OpenAICompatibleDialect, OpenAICompatibleErrorDecoder,
};

const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct OpenAICompatibleClientBuilder<D> {
    base_url: Option<String>,
    auth: Option<CompatibleAuth>,
    decoder: Arc<dyn CompatibleErrorDecoder>,
    request_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    marker: PhantomData<fn() -> D>,
}

impl<D> Default for OpenAICompatibleClientBuilder<D> {
    fn default() -> Self {
        Self {
            base_url: None,
            auth: None,
            decoder: Arc::new(OpenAICompatibleErrorDecoder),
            request_timeout: None,
            connect_timeout: None,
            marker: PhantomData,
        }
    }
}

impl<D> fmt::Debug for OpenAICompatibleClientBuilder<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAICompatibleClientBuilder")
            .field("base_url", &self.base_url.as_ref().map(|_| "[configured]"))
            .field("auth", &self.auth)
            .field("error_decoder", &"[configured]")
            .field("request_timeout", &self.request_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .finish()
    }
}

impl<D: OpenAICompatibleDialect> OpenAICompatibleClientBuilder<D> {
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn auth(mut self, auth: CompatibleAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn error_decoder(mut self, decoder: impl CompatibleErrorDecoder) -> Self {
        self.decoder = Arc::new(decoder);
        self
    }

    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> AiResult<OpenAICompatibleClient<D>> {
        let base_url = self.base_url.ok_or_else(|| {
            AiError::config(
                ConfigErrorKind::InvalidBaseUrl,
                "compatible base URL is required",
            )
        })?;
        let auth = self.auth.ok_or_else(|| {
            AiError::config(
                ConfigErrorKind::MissingApiKey,
                "compatible authentication is required",
            )
        })?;
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        match auth.into_inner() {
            super::dialect::CompatibleAuthKind::None => {}
            super::dialect::CompatibleAuthKind::Bearer(secret) => {
                let mut value =
                    HeaderValue::from_str(&format!("Bearer {secret}")).map_err(|_| {
                        AiError::config(ConfigErrorKind::InvalidApiKey, "invalid bearer credential")
                    })?;
                value.set_sensitive(true);
                headers.insert(AUTHORIZATION, value);
            }
            super::dialect::CompatibleAuthKind::Header { name, value } => {
                let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
                    AiError::config(
                        ConfigErrorKind::InvalidHeader,
                        "invalid authentication header name",
                    )
                })?;
                if is_unsafe_auth_header(&name) {
                    return Err(AiError::config(
                        ConfigErrorKind::InvalidHeader,
                        "authentication header cannot alter HTTP routing or framing",
                    ));
                }
                let mut value = HeaderValue::from_str(&value).map_err(|_| {
                    AiError::config(
                        ConfigErrorKind::InvalidApiKey,
                        "invalid authentication header value",
                    )
                })?;
                value.set_sensitive(true);
                headers.insert(name, value);
            }
        }
        let transport = HttpTransport::new(HttpTransportConfig {
            provider: AiProvider::OpenAICompatible(D::NAME),
            base_url,
            headers,
            request_timeout: self.request_timeout,
            connect_timeout: self.connect_timeout,
        })?;
        Ok(OpenAICompatibleClient {
            transport,
            decoder: self.decoder,
            marker: PhantomData,
        })
    }
}

fn is_unsafe_auth_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "host"
            | "connection"
            | "content-length"
            | "content-type"
            | "content-encoding"
            | "transfer-encoding"
            | "te"
            | "trailer"
            | "upgrade"
            | "expect"
            | "keep-alive"
            | "proxy-connection"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "forwarded"
            | "x-forwarded-host"
            | "x-forwarded-proto"
            | "user-agent"
    )
}

pub struct OpenAICompatibleClient<D> {
    pub(crate) transport: HttpTransport,
    pub(crate) decoder: Arc<dyn CompatibleErrorDecoder>,
    marker: PhantomData<fn() -> D>,
}

impl<D: OpenAICompatibleDialect> OpenAICompatibleClient<D> {
    pub fn builder() -> OpenAICompatibleClientBuilder<D> {
        OpenAICompatibleClientBuilder::default()
    }
}

impl<D: ChatCompletionsDialect> OpenAICompatibleClient<D> {
    pub fn chat(&self) -> ChatResource<'_, D> {
        ChatResource::new(self)
    }
}
