use serde::{Deserialize, Serialize};

use super::super::{output::OpenAIReasoningSummaryPart, tagged::ExtraFields};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIReasoningSummaryPartEvent {
    pub item_id: String,
    pub output_index: u64,
    pub summary_index: u64,
    pub part: OpenAIReasoningSummaryPart,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIReasoningSummaryPartDoneEvent {
    pub item_id: String,
    pub output_index: u64,
    pub summary_index: u64,
    pub part: OpenAIReasoningSummaryPart,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIReasoningSummaryDeltaEvent {
    pub item_id: String,
    pub output_index: u64,
    pub summary_index: u64,
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIReasoningSummaryDoneEvent {
    pub item_id: String,
    pub output_index: u64,
    pub summary_index: u64,
    pub text: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}
