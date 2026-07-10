use serde::{Deserialize, Serialize};

use crate::utils::IntoQuery;

use super::{Content, CountTokensGenerateContentRequest, ModelInfo};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListRequest {
    pub page_token: Option<String>,
    pub page_size: Option<i32>,
}

impl IntoQuery for ModelsListRequest {
    fn into_query(self) -> Vec<(String, String)> {
        let mut query = Vec::new();

        if let Some(page_token) = self.page_token {
            query.push(("pageToken".to_string(), page_token));
        }

        if let Some(page_size) = self.page_size {
            query.push(("pageSize".to_string(), page_size.to_string()));
        }

        query
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListResponse {
    pub models: Vec<ModelInfo>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensRequest {
    contents: Option<Vec<Content>>,
    generate_content_request: Option<CountTokensGenerateContentRequest>,
}

impl CountTokensRequest {
    /// Count tokens for the supplied content items.
    pub fn from_contents(contents: Vec<Content>) -> Self {
        Self {
            contents: Some(contents),
            generate_content_request: None,
        }
    }

    /// Count tokens for a complete generation request.
    pub fn from_generate_content(request: CountTokensGenerateContentRequest) -> Self {
        Self {
            contents: None,
            generate_content_request: Some(request),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensResponse {
    total_tokens: u64,
}

impl CountTokensResponse {
    pub fn total_tokens(&self) -> u64 {
        self.total_tokens
    }
}
