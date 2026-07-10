use serde::{Deserialize, Serialize};

use crate::openai::conversations::ConversationId;

use super::super::{
    OpenAIResponseItem, OpenAIResponseUsage, OpenAIResponsesInput, OpenAIResponsesReasoning,
    OpenAIResponsesTextConfig, OpenAIResponsesTool, OpenAIToolChoice,
};
use super::{ResponseId, ResponseItemId};

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIResponseItemList {
    pub object: String,
    pub data: Vec<OpenAIResponseItem>,
    pub has_more: bool,
    pub first_id: ResponseItemId,
    pub last_id: ResponseItemId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIConversationReference {
    Id(ConversationId),
    Object { id: ConversationId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAITruncation {
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIServiceTier {
    Auto,
    Default,
    Flex,
    Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenAIPromptCacheRetention {
    #[serde(rename = "in_memory")]
    InMemory,
    #[serde(rename = "24h")]
    Hours24,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIPromptCacheMode {
    Implicit,
    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenAIPromptCacheTtl {
    #[serde(rename = "30m")]
    Minutes30,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIPromptCacheOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<OpenAIPromptCacheTtl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<OpenAIPromptCacheMode>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OpenAIInputTokenCountRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<OpenAIResponsesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<ResponseId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAIResponsesTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<OpenAIResponsesTextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<OpenAIResponsesReasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<OpenAITruncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<OpenAIConversationReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<OpenAIToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIInputTokenCountResponse {
    pub object: String,
    pub input_tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAICompactRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<OpenAIResponsesInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<ResponseId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<OpenAIPromptCacheRetention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_options: Option<OpenAIPromptCacheOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<OpenAIServiceTier>,
}

impl OpenAICompactRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            previous_response_id: None,
            instructions: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            prompt_cache_options: None,
            service_tier: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAICompactResponse {
    pub id: ResponseId,
    pub object: String,
    pub output: Vec<OpenAIResponseItem>,
    pub created_at: u64,
    pub usage: OpenAIResponseUsage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_payloads_match_pinned_shapes() {
        let count = OpenAIInputTokenCountRequest {
            model: Some("gpt-5".into()),
            input: Some(OpenAIResponsesInput::Text("Tell me a joke.".into())),
            parallel_tool_calls: Some(true),
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(count).unwrap(),
            serde_json::json!({
                "model": "gpt-5",
                "input": "Tell me a joke.",
                "parallel_tool_calls": true
            })
        );

        let compact = OpenAICompactRequest::new("gpt-5.1-codex-max");
        assert_eq!(
            serde_json::to_value(compact).unwrap(),
            serde_json::json!({"model": "gpt-5.1-codex-max"})
        );

        let count: OpenAIInputTokenCountResponse = serde_json::from_value(serde_json::json!({
            "object": "response.input_tokens",
            "input_tokens": 11
        }))
        .unwrap();
        assert_eq!(count.input_tokens, 11);
    }
}
