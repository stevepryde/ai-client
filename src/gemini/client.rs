use std::fmt::Debug;
use std::time::Duration;

#[cfg(feature = "stream")]
use futures::Stream;
use reqwest::header::USER_AGENT;
#[cfg(feature = "stream")]
use reqwest_streams::error::StreamBodyError;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    prelude::{AiError, AiResult},
    utils::Url,
};

use super::{
    CountTokensRequest, CountTokensResponse, GenerateContentRequest, GenerateContentResponse,
    Model, ModelInfo, ModelsListRequest, ModelsListResponse,
};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";

#[derive(Clone, Default)]
pub struct GeminiClientBuilder {
    api_key: Option<String>,
    timeout: Option<u64>,
}

impl Debug for GeminiClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiClientBuilder")
            .field("api_key", &"*** redacted ***")
            .field(
                "timeout",
                &self
                    .timeout
                    .map(|t| format!("{t} seconds"))
                    .unwrap_or_else(|| "not set".to_string()),
            )
            .finish()
    }
}

impl GeminiClientBuilder {
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> AiResult<GeminiClient> {
        let api_key = self.api_key.ok_or(AiError::MissingApiKey)?;

        // Add default HTTP headers.
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-goog-api-key",
            api_key.parse().map_err(|_| AiError::InvalidApiKey)?,
        );
        headers.insert(
            USER_AGENT,
            env!("CARGO_PKG_NAME")
                .parse()
                .unwrap_or_else(|_| "reqwest".parse().unwrap()),
        );
        let mut builder = reqwest::Client::builder().default_headers(headers);

        // Default timeout.
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(Duration::from_secs(timeout));
        }

        let client = builder
            .build()
            .map_err(|e| AiError::InvalidClient(e.to_string()))?;
        Ok(GeminiClient { api_key, client })
    }
}

pub async fn parse_response<T>(response: reqwest::Response) -> AiResult<T>
where
    T: DeserializeOwned,
{
    if response.status().is_success() {
        // let text = response.text().await.map_err(AiError::Response)?;
        // println!("RESPONSE: {text}");
        // serde_json::from_str(&text).map_err(AiError::Json)
        response.json().await.map_err(AiError::Response)
    } else {
        Err(AiError::ApiError(
            response.status(),
            response.text().await.map_err(AiError::Response)?,
        ))
    }
}

#[non_exhaustive]
pub struct GeminiClient {
    pub api_key: String,
    pub client: reqwest::Client,
}

impl GeminiClient {
    pub fn builder() -> GeminiClientBuilder {
        GeminiClientBuilder::default()
    }

    pub async fn get<T>(&self, url: &str) -> AiResult<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(AiError::Request)?;

        parse_response(response).await
    }

    pub async fn post<Req, Res>(&self, url: &str, request: Req) -> AiResult<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(AiError::Request)?;

        parse_response(response).await
    }

    pub async fn list_models(&self) -> AiResult<ModelsListResponse> {
        self.list_models_with_params(ModelsListRequest::default())
            .await
    }

    pub async fn list_models_with_params(
        &self,
        params: ModelsListRequest,
    ) -> AiResult<ModelsListResponse> {
        let url = Url::new(format!("{BASE_URL}/models"))
            .with_query_from(params)
            .build();

        self.get(&url).await
    }

    pub async fn get_model(&self, model: Model) -> AiResult<ModelInfo> {
        // NOTE: Model serializes with the `models/` prefix.
        let url = Url::new(format!("{BASE_URL}/{model}")).build();
        self.get(&url).await
    }

    pub async fn count_tokens(
        &self,
        model: Model,
        request: CountTokensRequest,
    ) -> AiResult<CountTokensResponse> {
        let url = Url::new(format!("{BASE_URL}/{model}:countTokens")).build();
        self.post(&url, request).await
    }

    pub async fn generate_content(
        &self,
        model: Model,
        request: GenerateContentRequest,
    ) -> AiResult<GenerateContentResponse> {
        let url = Url::new(format!("{BASE_URL}/{model}:generateContent")).build();
        self.post(&url, request).await
    }

    #[cfg(feature = "stream")]
    pub async fn generate_content_streamed(
        &self,
        model: Model,
        request: GenerateContentRequest,
    ) -> AiResult<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>> {
        use reqwest_streams::JsonStreamResponse;

        let url = Url::new(format!("{BASE_URL}/{model}:streamGenerateContent")).build();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(AiError::Request)?;

        let stream = response.json_array_stream::<GenerateContentResponse>(1024);
        Ok(stream)
    }
}
