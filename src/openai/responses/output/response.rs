use super::super::{
    operations::ResponseId, tagged::ExtraFields, OpenAIConversationReference,
    OpenAIModerationConfig, OpenAIPromptCacheOptions, OpenAIPromptTemplate, OpenAIResponseMetadata,
    OpenAIResponsesReasoning, OpenAIResponsesTextConfig, OpenAIResponsesTool, OpenAIServiceTier,
    OpenAIToolChoice, OpenAITruncation, TopLogprobs,
};
use super::OpenAIResponseOutputItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIResponseStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
    Queued,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseUsage {
    pub input_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<OpenAIInputTokensDetails>,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OpenAIOutputTokensDetails>,
    pub total_tokens: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputTokensDetails {
    pub cached_tokens: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIOutputTokensDetails {
    pub reasoning_tokens: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseError {
    pub code: String,
    pub message: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIIncompleteDetails {
    pub reason: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponsesCreateResponse {
    pub metadata: OpenAIResponseMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<TopLogprobs>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<OpenAIServiceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    pub previous_response_id: Option<ResponseId>,
    pub id: ResponseId,
    pub object: String,
    pub status: OpenAIResponseStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<OpenAIResponsesTextConfig>,
    pub error: Option<OpenAIResponseError>,
    pub incomplete_details: Option<OpenAIIncompleteDetails>,
    pub output: Vec<OpenAIResponseOutputItem>,
    pub reasoning: OpenAIResponsesReasoning,
    pub instructions: Option<String>,
    #[serde(default)]
    pub output_text: String,
    pub usage: Option<OpenAIResponseUsage>,
    pub prompt_cache_options: Option<OpenAIPromptCacheOptions>,
    pub moderation: Option<OpenAIModerationConfig>,
    pub parallel_tool_calls: bool,
    pub conversation: Option<OpenAIConversationReference>,
    pub max_output_tokens: Option<u64>,
    pub model: String,
    pub tools: Vec<OpenAIResponsesTool>,
    pub tool_choice: OpenAIToolChoice,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<OpenAIPromptTemplate>,
    pub truncation: OpenAITruncation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

pub type OpenAIResponse = OpenAIResponsesCreateResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_output_item_round_trips_without_losing_fields() {
        let value = serde_json::json!({
            "type": "future_output",
            "id": "x",
            "payload": {"private": true}
        });
        let item: OpenAIResponseOutputItem = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), value);
    }

    #[test]
    fn malformed_known_output_does_not_fall_back_to_unknown() {
        assert!(serde_json::from_value::<OpenAIResponseOutputItem>(
            serde_json::json!({"type":"message","id":"missing-rest"})
        )
        .is_err());
    }

    #[test]
    fn aggregate_output_text_may_be_omitted_by_the_provider() {
        let response: OpenAIResponsesCreateResponse = serde_json::from_value(serde_json::json!({
            "metadata": {}, "top_logprobs": 0, "temperature": 1.0, "top_p": 0.98,
            "model": "gpt-5.4-mini-2026-03-17", "tools": [], "tool_choice": "auto",
            "id": "resp_1", "object": "response", "status": "completed", "created_at": 1,
            "completed_at": 2, "error": null, "incomplete_details": null, "output": [],
            "reasoning": {"context":"current_turn","effort":"none","mode":"standard","summary":null},
            "instructions": null, "usage": null, "moderation": null,
            "parallel_tool_calls": true, "max_output_tokens": 16, "truncation": "disabled",
            "previous_response_id": null
        }))
        .unwrap();
        assert!(response.output_text.is_empty());
    }
}
