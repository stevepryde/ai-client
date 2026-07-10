use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIModelsListResponse {
    /// Models visible to the authenticated project.
    ///
    /// OpenAI calls this field `data` on the wire. The Rust field keeps its
    /// original name for source compatibility with existing callers.
    #[serde(rename = "data", alias = "models")]
    pub models: Vec<OpenAIModelInfo>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModelInfo {
    pub id: String,
    pub object: String,
    pub owned_by: String,
    pub created: u64,
}
