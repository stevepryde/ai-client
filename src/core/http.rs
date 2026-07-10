use std::{collections::BTreeMap, time::Duration};

use reqwest::{
    header::{HeaderMap, HeaderValue, RETRY_AFTER},
    redirect::Policy,
    Method, StatusCode, Url,
};
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "stream")]
use futures::StreamExt;

use crate::error::{
    AiError, AiProvider, AiResponse, AiResult, BodySnippet, ConfigErrorKind, ProviderApiError,
    RateLimitMetadata, ResponseMetadata, TransportErrorKind,
};
#[cfg(feature = "stream")]
use crate::stream::{AiStream, AiStreamError, AiStreamErrorKind};

const MAX_ERROR_BODY_BYTES: usize = 8 * 1024;

pub(crate) struct HttpTransportConfig {
    pub provider: AiProvider,
    pub base_url: String,
    pub headers: HeaderMap,
    pub request_timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
}

#[derive(Clone)]
pub(crate) struct HttpTransport {
    provider: AiProvider,
    base_url: Url,
    headers: HeaderMap,
    client: reqwest::Client,
}

impl HttpTransport {
    pub(crate) fn new(config: HttpTransportConfig) -> AiResult<Self> {
        let base_url = parse_base_url(&config.base_url)?;

        let redirect_origin = base_url.clone();
        let mut builder = reqwest::Client::builder().redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= 10 {
                return attempt.stop();
            }
            if same_origin(attempt.url(), &redirect_origin) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }));
        if let Some(timeout) = config.request_timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(timeout) = config.connect_timeout {
            builder = builder.connect_timeout(timeout);
        }
        let client = builder.build().map_err(|_| {
            AiError::config(ConfigErrorKind::HttpClient, "failed to build HTTP client")
        })?;

        Ok(Self {
            provider: config.provider,
            base_url,
            headers: config.headers,
            client,
        })
    }

    pub(crate) async fn get_json<T, D>(
        &self,
        operation: &'static str,
        path: &str,
        query: &[(String, String)],
        decode_error: D,
    ) -> AiResult<AiResponse<T>>
    where
        T: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let url = self.build_url(path, query)?;
        let request = self
            .client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn get_json_segments<T, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        query: &[(String, String)],
        decode_error: D,
    ) -> AiResult<AiResponse<T>>
    where
        T: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let url = self.build_url_segments(path_segments, query)?;
        let request = self
            .client
            .request(Method::GET, url)
            .headers(self.headers.clone());
        self.send_and_decode(operation, request, decode_error).await
    }

    #[cfg(feature = "stream")]
    pub(crate) async fn get_json_stream_segments<D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        query: &[(String, String)],
        decode_error: D,
    ) -> AiResult<AiResponse<AiStream<Vec<u8>>>>
    where
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.request_segments(Method::GET, path_segments, query)?;
        self.send_stream_handshake(operation, request, decode_error)
            .await
    }

    pub(crate) async fn post_json<Req, Res, D>(
        &self,
        operation: &'static str,
        path: &str,
        request: &Req,
        decode_error: D,
    ) -> AiResult<AiResponse<Res>>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.json_request(Method::POST, path, request)?;
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn post_json_segments<Req, Res, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        request: &Req,
        decode_error: D,
    ) -> AiResult<AiResponse<Res>>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.json_request_segments(Method::POST, path_segments, request)?;
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn post_json_segments_with_query<Req, Res, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        query: &[(String, String)],
        request: &Req,
        decode_error: D,
    ) -> AiResult<AiResponse<Res>>
    where
        Req: Serialize + ?Sized,
        Res: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let url = self.build_url_segments(path_segments, query)?;
        let request = self
            .client
            .request(Method::POST, url)
            .headers(self.headers.clone())
            .json(request);
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn post_empty_segments<Res, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        decode_error: D,
    ) -> AiResult<AiResponse<Res>>
    where
        Res: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.request_segments(Method::POST, path_segments, &[])?;
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn delete_json_segments<Res, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        decode_error: D,
    ) -> AiResult<AiResponse<Res>>
    where
        Res: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.request_segments(Method::DELETE, path_segments, &[])?;
        self.send_and_decode(operation, request, decode_error).await
    }

    pub(crate) async fn delete_empty_segments<D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        decode_error: D,
    ) -> AiResult<AiResponse<()>>
    where
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.request_segments(Method::DELETE, path_segments, &[])?;
        self.send_and_decode_empty(operation, request, decode_error)
            .await
    }

    #[cfg(feature = "stream")]
    pub(crate) async fn post_json_stream<Req, D>(
        &self,
        operation: &'static str,
        path: &str,
        request: &Req,
        decode_error: D,
    ) -> AiResult<AiResponse<AiStream<Vec<u8>>>>
    where
        Req: Serialize + ?Sized,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.json_request(Method::POST, path, request)?;
        self.send_stream_handshake(operation, request, decode_error)
            .await
    }

    #[cfg(feature = "stream")]
    pub(crate) async fn post_json_stream_segments<Req, D>(
        &self,
        operation: &'static str,
        path_segments: &[&str],
        request: &Req,
        decode_error: D,
    ) -> AiResult<AiResponse<AiStream<Vec<u8>>>>
    where
        Req: Serialize + ?Sized,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let request = self.json_request_segments(Method::POST, path_segments, request)?;
        self.send_stream_handshake(operation, request, decode_error)
            .await
    }

    fn json_request<Req: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        request: &Req,
    ) -> AiResult<reqwest::RequestBuilder> {
        let url = self.build_url(path, &[])?;
        Ok(self
            .client
            .request(method, url)
            .headers(self.headers.clone())
            .json(request))
    }

    fn json_request_segments<Req: Serialize + ?Sized>(
        &self,
        method: Method,
        path_segments: &[&str],
        request: &Req,
    ) -> AiResult<reqwest::RequestBuilder> {
        let url = self.build_url_segments(path_segments, &[])?;
        Ok(self
            .client
            .request(method, url)
            .headers(self.headers.clone())
            .json(request))
    }

    fn request_segments(
        &self,
        method: Method,
        path_segments: &[&str],
        query: &[(String, String)],
    ) -> AiResult<reqwest::RequestBuilder> {
        let url = self.build_url_segments(path_segments, query)?;
        Ok(self
            .client
            .request(method, url)
            .headers(self.headers.clone()))
    }

    async fn send_and_decode<T, D>(
        &self,
        operation: &'static str,
        request: reqwest::RequestBuilder,
        decode_error: D,
    ) -> AiResult<AiResponse<T>>
    where
        T: DeserializeOwned,
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let response = request
            .send()
            .await
            .map_err(|error| transport_error(self.provider, operation, error))?;
        let metadata = response_metadata(response.status(), response.headers());

        if metadata.status.is_success() {
            let bytes = response
                .bytes()
                .await
                .map_err(|error| transport_error(self.provider, operation, error))?;
            let data = serde_json::from_slice(&bytes).map_err(|error| AiError::Decode {
                provider: self.provider,
                operation,
                metadata: Box::new(metadata.clone()),
                error: error.into(),
            })?;
            Ok(AiResponse::new(data, metadata))
        } else {
            let (bytes, truncated) = read_bounded_body(response, MAX_ERROR_BODY_BYTES)
                .await
                .map_err(|error| transport_error(self.provider, operation, error))?;
            let body = BodySnippet::from_bytes(&bytes, truncated);
            let error = decode_error(&bytes, body);
            Err(AiError::Api {
                provider: self.provider,
                operation,
                metadata: Box::new(metadata),
                error: Box::new(error),
            })
        }
    }

    async fn send_and_decode_empty<D>(
        &self,
        operation: &'static str,
        request: reqwest::RequestBuilder,
        decode_error: D,
    ) -> AiResult<AiResponse<()>>
    where
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let response = request
            .send()
            .await
            .map_err(|error| transport_error(self.provider, operation, error))?;
        let metadata = response_metadata(response.status(), response.headers());
        if metadata.status.is_success() {
            return Ok(AiResponse::new((), metadata));
        }

        let (bytes, truncated) = read_bounded_body(response, MAX_ERROR_BODY_BYTES)
            .await
            .map_err(|error| transport_error(self.provider, operation, error))?;
        let body = BodySnippet::from_bytes(&bytes, truncated);
        let error = decode_error(&bytes, body);
        Err(AiError::Api {
            provider: self.provider,
            operation,
            metadata: Box::new(metadata),
            error: Box::new(error),
        })
    }

    #[cfg(feature = "stream")]
    async fn send_stream_handshake<D>(
        &self,
        operation: &'static str,
        request: reqwest::RequestBuilder,
        decode_error: D,
    ) -> AiResult<AiResponse<AiStream<Vec<u8>>>>
    where
        D: FnOnce(&[u8], BodySnippet) -> ProviderApiError,
    {
        let response = request
            .send()
            .await
            .map_err(|error| transport_error(self.provider, operation, error))?;
        let metadata = response_metadata(response.status(), response.headers());
        if !metadata.status.is_success() {
            let (bytes, truncated) = read_bounded_body(response, MAX_ERROR_BODY_BYTES)
                .await
                .map_err(|error| transport_error(self.provider, operation, error))?;
            let body = BodySnippet::from_bytes(&bytes, truncated);
            let error = decode_error(&bytes, body);
            return Err(AiError::Api {
                provider: self.provider,
                operation,
                metadata: Box::new(metadata),
                error: Box::new(error),
            });
        }

        let provider = self.provider;
        let stream = response.bytes_stream().map(move |result| {
            result.map(|bytes| bytes.to_vec()).map_err(|error| {
                let kind = if error.is_timeout() {
                    AiStreamErrorKind::Timeout
                } else {
                    AiStreamErrorKind::Transport(classify_transport_error(&error))
                };
                AiStreamError::new(provider, operation, kind)
            })
        });
        Ok(AiResponse::new(AiStream::new(stream), metadata))
    }

    pub(crate) fn build_url(&self, path: &str, query: &[(String, String)]) -> AiResult<Url> {
        validate_relative_path(path)?;
        self.build_url_segments(&path.split('/').collect::<Vec<_>>(), query)
    }

    pub(crate) fn build_url_segments(
        &self,
        path_segments: &[&str],
        query: &[(String, String)],
    ) -> AiResult<Url> {
        if path_segments.is_empty()
            || path_segments
                .iter()
                .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
        {
            return Err(AiError::config(
                ConfigErrorKind::InvalidBaseUrl,
                "endpoint contains an invalid path segment",
            ));
        }
        let mut url = self.base_url.clone();
        {
            let mut segments = url.path_segments_mut().map_err(|_| {
                AiError::config(
                    ConfigErrorKind::InvalidBaseUrl,
                    "base URL cannot contain paths",
                )
            })?;
            segments.pop_if_empty();
            for segment in path_segments {
                segments.push(segment);
            }
        }
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }
}

fn parse_base_url(base_url: &str) -> AiResult<Url> {
    let mut url = Url::parse(base_url)
        .map_err(|_| AiError::config(ConfigErrorKind::InvalidBaseUrl, "base URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AiError::config(
            ConfigErrorKind::InvalidBaseUrl,
            "base URL must be HTTP(S) without credentials, query, or fragment",
        ));
    }
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

fn validate_relative_path(path: &str) -> AiResult<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('?')
        || path.contains('#')
        || path.contains('%')
        || Url::parse(path).is_ok()
        || path.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        return Err(AiError::config(
            ConfigErrorKind::InvalidBaseUrl,
            "endpoint must be a relative path without query or fragment",
        ));
    }
    Ok(())
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn transport_error(
    provider: AiProvider,
    operation: &'static str,
    error: reqwest::Error,
) -> AiError {
    if error.is_timeout() {
        return AiError::Timeout {
            provider,
            operation,
        };
    }
    let kind = classify_transport_error(&error);
    AiError::Transport {
        provider,
        operation,
        kind,
    }
}

fn classify_transport_error(error: &reqwest::Error) -> TransportErrorKind {
    if error.is_connect() {
        TransportErrorKind::Connect
    } else if error.is_body() || error.is_decode() {
        TransportErrorKind::Body
    } else if error.is_builder() || error.is_request() {
        TransportErrorKind::Request
    } else {
        TransportErrorKind::Unknown
    }
}

fn response_metadata(status: StatusCode, headers: &HeaderMap) -> ResponseMetadata {
    let request_id = header_string(headers, "x-request-id")
        .or_else(|| header_string(headers, "x-goog-request-id"))
        .or_else(|| header_string(headers, "x-guploader-uploadid"));
    let retry_after = headers.get(RETRY_AFTER).and_then(header_value_string);

    let known = [
        "x-ratelimit-limit-requests",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-remaining-tokens",
        "x-ratelimit-reset-requests",
        "x-ratelimit-reset-tokens",
    ];
    let mut other = BTreeMap::new();
    for (name, value) in headers {
        let name = name.as_str();
        if name.starts_with("x-ratelimit-") && !known.contains(&name) {
            if let Some(value) = header_value_string(value) {
                other.insert(name.to_string(), value);
            }
        }
    }
    ResponseMetadata {
        status,
        request_id,
        retry_after,
        rate_limit: RateLimitMetadata {
            limit_requests: header_string(headers, known[0]),
            limit_tokens: header_string(headers, known[1]),
            remaining_requests: header_string(headers, known[2]),
            remaining_tokens: header_string(headers, known[3]),
            reset_requests: header_string(headers, known[4]),
            reset_tokens: header_string(headers, known[5]),
            other,
        },
    }
}

fn header_string(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers.get(name).and_then(header_value_string)
}

fn header_value_string(value: &HeaderValue) -> Option<String> {
    value.to_str().ok().map(ToOwned::to_owned)
}

async fn read_bounded_body(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<(Vec<u8>, bool), reqwest::Error> {
    let mut body = Vec::with_capacity(limit.min(1024));
    let mut truncated = false;
    while let Some(chunk) = response.chunk().await? {
        let remaining = limit.saturating_sub(body.len());
        if chunk.len() > remaining {
            body.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            break;
        }
        body.extend_from_slice(&chunk);
        if body.len() == limit {
            if response.chunk().await?.is_some() {
                truncated = true;
            }
            break;
        }
    }
    Ok((body, truncated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn transport(base_url: &str) -> HttpTransport {
        HttpTransport::new(HttpTransportConfig {
            provider: AiProvider::OpenAI,
            base_url: base_url.to_string(),
            headers: HeaderMap::new(),
            request_timeout: None,
            connect_timeout: None,
        })
        .unwrap()
    }

    async fn one_shot_server(response: String) -> (String, tokio::task::JoinHandle<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let read = socket.read(&mut request).await.unwrap();
            socket.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&request[..read]).into_owned()
        });
        (format!("http://{address}/v1"), handle)
    }

    fn test_error(body: BodySnippet) -> ProviderApiError {
        ProviderApiError::new("test API error", None, None, None, body)
    }

    #[test]
    fn preserves_prefix_and_percent_encodes_query() {
        let transport = transport("https://example.com/gateway/v1");
        let url = transport
            .build_url("models", &[("page token".into(), "next/+?& token".into())])
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/gateway/v1/models?page+token=next%2F%2B%3F%26+token"
        );
    }

    #[test]
    fn rejects_authenticated_absolute_and_protocol_relative_paths() {
        let transport = transport("https://api.openai.com/v1");
        for path in [
            "https://attacker.example/steal",
            "//attacker.example/steal",
            "/models",
            "models?redirect=https://attacker.example",
            "models#fragment",
            "../models",
            "a/../../models",
            "%2e%2e/models",
            "%252e%252e/models",
        ] {
            assert!(transport.build_url(path, &[]).is_err(), "accepted {path}");
        }
    }

    #[test]
    fn dynamic_identifier_is_encoded_as_one_path_segment() {
        let transport = transport("https://example.com/gateway/v1");
        let url = transport
            .build_url_segments(&["models", "custom/a ?#%"], &[])
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/gateway/v1/models/custom%2Fa%20%3F%23%25"
        );
    }

    #[test]
    fn rejects_sensitive_or_ambiguous_base_urls() {
        for url in [
            "https://key@example.com/v1",
            "https://example.com/v1?key=secret",
            "https://example.com/v1#fragment",
            "file:///tmp/api",
        ] {
            assert!(HttpTransport::new(HttpTransportConfig {
                provider: AiProvider::OpenAI,
                base_url: url.to_string(),
                headers: HeaderMap::new(),
                request_timeout: None,
                connect_timeout: None,
            })
            .is_err());
        }
    }

    #[test]
    fn extracts_safe_response_metadata() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("req_123"));
        headers.insert(RETRY_AFTER, HeaderValue::from_static("2"));
        headers.insert(
            "x-ratelimit-remaining-requests",
            HeaderValue::from_static("17"),
        );
        headers.insert("x-ratelimit-custom-window", HeaderValue::from_static("60s"));
        let metadata = response_metadata(StatusCode::TOO_MANY_REQUESTS, &headers);
        assert_eq!(metadata.request_id.as_deref(), Some("req_123"));
        assert_eq!(metadata.retry_after.as_deref(), Some("2"));
        assert_eq!(
            metadata.rate_limit.remaining_requests.as_deref(),
            Some("17")
        );
        assert_eq!(
            metadata
                .rate_limit
                .other
                .get("x-ratelimit-custom-window")
                .map(String::as_str),
            Some("60s")
        );
    }

    #[tokio::test]
    async fn malformed_success_is_a_structured_decode_error_without_body() {
        let body = "not-json";
        let response = format!(
            "HTTP/1.1 200 OK\r\nx-request-id: req_malformed\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);
        let error = transport
            .get_json::<serde_json::Value, _>("test.decode", "models", &[], |_, body| {
                ProviderApiError::new("unexpected", None, None, None, body)
            })
            .await
            .unwrap_err();
        request.await.unwrap();
        let debug = format!("{error:?}");
        match &error {
            AiError::Decode {
                operation,
                metadata,
                error,
                ..
            } => {
                assert_eq!(*operation, "test.decode");
                assert_eq!(metadata.request_id.as_deref(), Some("req_malformed"));
                assert_eq!(
                    error.category,
                    crate::error::JsonDecodeErrorCategory::Syntax
                );
            }
            other => panic!("expected decode error, got {other:?}"),
        }
        assert!(!debug.contains(body));
    }

    #[tokio::test]
    async fn crate_client_stops_cross_origin_redirect_before_credentials_can_leak() {
        let sink = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sink_address = sink.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{sink_address}/steal\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let (base_url, origin_request) = one_shot_server(response).await;
        let mut headers = HeaderMap::new();
        let mut credential = HeaderValue::from_static("redirect-secret");
        credential.set_sensitive(true);
        headers.insert("x-goog-api-key", credential);
        let transport = HttpTransport::new(HttpTransportConfig {
            provider: AiProvider::Gemini,
            base_url,
            headers,
            request_timeout: None,
            connect_timeout: None,
        })
        .unwrap();
        let _ = transport
            .get_json::<serde_json::Value, _>("test.redirect", "models", &[], |_, body| {
                ProviderApiError::new("redirect stopped", None, None, None, body)
            })
            .await;
        let request = origin_request.await.unwrap();
        assert!(request.contains("x-goog-api-key: redirect-secret"));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), sink.accept())
                .await
                .is_err(),
            "cross-origin redirect was followed"
        );
    }

    #[tokio::test]
    async fn api_error_body_is_bounded_truncated_and_redacted() {
        let secret = "provider-secret-payload";
        let body = secret.repeat(600);
        assert!(body.len() > MAX_ERROR_BODY_BYTES);
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);
        let error = transport
            .get_json::<serde_json::Value, _>("test.large_error", "models", &[], |_, body| {
                ProviderApiError::new("normalized error", None, None, None, body)
            })
            .await
            .unwrap_err();
        request.await.unwrap();

        let debug = format!("{error:?}");
        let display = error.to_string();
        match &error {
            AiError::Api { error, .. } => {
                assert_eq!(error.body().as_str().len(), MAX_ERROR_BODY_BYTES);
                assert!(error.body().is_truncated());
            }
            other => panic!("expected API error, got {other:?}"),
        }
        assert!(!debug.contains(secret));
        assert!(!display.contains(secret));
    }

    #[tokio::test]
    async fn delete_empty_segments_accepts_an_empty_success_and_encodes_the_id() {
        let response = "HTTP/1.1 200 OK\r\nx-request-id: req_delete\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string();
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);

        let response = transport
            .delete_empty_segments("test.delete", &["responses", "resp/a"], |_, body| {
                test_error(body)
            })
            .await
            .unwrap();

        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_delete")
        );
        let request = request.await.unwrap();
        assert!(request.starts_with("DELETE /v1/responses/resp%2Fa HTTP/1.1\r\n"));
        assert!(request.ends_with("\r\n\r\n"));
    }

    #[tokio::test]
    async fn delete_json_segments_decodes_the_success_body() {
        let body = r#"{"id":"conv_123","deleted":true}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);

        let response = transport
            .delete_json_segments::<serde_json::Value, _>(
                "test.delete_json",
                &["conversations", "conv_123"],
                |_, body| test_error(body),
            )
            .await
            .unwrap();

        assert_eq!(response.data()["deleted"], true);
        assert!(request
            .await
            .unwrap()
            .starts_with("DELETE /v1/conversations/conv_123 HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn post_empty_segments_sends_no_json_body() {
        let body = r#"{"id":"resp_123","status":"cancelled"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);

        let response = transport
            .post_empty_segments::<serde_json::Value, _>(
                "test.cancel",
                &["responses", "resp_123", "cancel"],
                |_, body| test_error(body),
            )
            .await
            .unwrap();

        assert_eq!(response.data()["status"], "cancelled");
        let request = request.await.unwrap();
        assert!(request.starts_with("POST /v1/responses/resp_123/cancel HTTP/1.1\r\n"));
        assert!(request.ends_with("\r\n\r\n"));
        assert!(!request.to_ascii_lowercase().contains("content-type:"));
    }

    #[tokio::test]
    async fn post_json_segments_with_query_encodes_path_query_and_body() {
        let body = r#"{"object":"list","data":[]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);
        let query = vec![
            ("include".into(), "reasoning.encrypted_content".into()),
            ("include".into(), "message.output_text.logprobs".into()),
        ];

        let response = transport
            .post_json_segments_with_query::<_, serde_json::Value, _>(
                "test.items.create",
                &["conversations", "conv/a", "items"],
                &query,
                &serde_json::json!({"items": []}),
                |_, body| test_error(body),
            )
            .await
            .unwrap();
        assert_eq!(response.data()["object"], "list");

        let request = request.await.unwrap();
        assert!(request.starts_with(
            "POST /v1/conversations/conv%2Fa/items?include=reasoning.encrypted_content&include=message.output_text.logprobs HTTP/1.1\r\n"
        ));
        assert!(request.ends_with(r#"{"items":[]}"#));
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn get_stream_segments_preserves_metadata_and_encodes_query() {
        use futures::StreamExt;

        let body = "data: {\"type\":\"response.completed\"}\n\n";
        let response = format!(
            "HTTP/1.1 200 OK\r\nx-request-id: req_stream\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);
        let query = vec![
            ("stream".to_string(), "true".to_string()),
            ("starting_after".to_string(), "item/a".to_string()),
            ("include_obfuscation".to_string(), "false".to_string()),
        ];

        let response = transport
            .get_json_stream_segments(
                "test.retrieve_stream",
                &["responses", "resp/a"],
                &query,
                |_, body| test_error(body),
            )
            .await
            .unwrap();
        assert_eq!(
            response.metadata().request_id.as_deref(),
            Some("req_stream")
        );
        let chunks = response
            .into_inner()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(chunks.concat(), body.as_bytes());

        let request = request.await.unwrap();
        assert!(request.starts_with(
            "GET /v1/responses/resp%2Fa?stream=true&starting_after=item%2Fa&include_obfuscation=false HTTP/1.1\r\n"
        ));
    }

    #[cfg(feature = "stream")]
    #[tokio::test]
    async fn get_stream_segments_decodes_non_success_before_streaming() {
        let body = r#"{"error":{"message":"no such response"}}"#;
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nx-request-id: req_missing\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (base_url, request) = one_shot_server(response).await;
        let transport = transport(&base_url);

        let result = transport
            .get_json_stream_segments(
                "test.retrieve_stream",
                &["responses", "missing"],
                &[],
                |_, body| test_error(body),
            )
            .await;
        let error = match result {
            Ok(_) => panic!("expected API error"),
            Err(error) => error,
        };
        request.await.unwrap();

        match error {
            AiError::Api {
                operation,
                metadata,
                error,
                ..
            } => {
                assert_eq!(operation, "test.retrieve_stream");
                assert_eq!(metadata.status, StatusCode::NOT_FOUND);
                assert_eq!(metadata.request_id.as_deref(), Some("req_missing"));
                assert_eq!(error.body().as_str(), body);
            }
            other => panic!("expected API error, got {other:?}"),
        }
    }
}
