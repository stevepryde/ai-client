use serde::{Deserialize, Serialize};

use crate::openai::responses::tagged::{lossless_tagged_enum, ExtraFields};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIWebSearchStatus {
    InProgress,
    Searching,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchFindAction {
    pub url: String,
    pub pattern: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchOpenPageAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchUrlSource {
    pub url: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIWebSearchSource {
        Url(OpenAIWebSearchUrlSource) => "url",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchSearchAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<OpenAIWebSearchSource>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIWebSearchAction {
        FindInPage(OpenAIWebSearchFindAction) => "find_in_page",
        OpenPage(OpenAIWebSearchOpenPageAction) => "open_page",
        Search(OpenAIWebSearchSearchAction) => "search",
        @unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_actions_are_typed_and_malformed_known_tags_fail() {
        let search = serde_json::json!({
            "type":"search",
            "queries":["rust"],
            "sources":[{"type":"url","url":"https://example.test"}]
        });
        assert!(matches!(
            serde_json::from_value::<OpenAIWebSearchAction>(search).unwrap(),
            OpenAIWebSearchAction::Search(_)
        ));
        assert!(serde_json::from_value::<OpenAIWebSearchAction>(
            serde_json::json!({"type":"find_in_page","url":"https://example.test"})
        )
        .is_err());
    }
}
