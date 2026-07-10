use serde::{Deserialize, Serialize};

use super::super::tagged::ExtraFields;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCallStatusEvent {
    pub output_index: u64,
    pub item_id: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIItemDeltaEvent {
    pub output_index: u64,
    pub item_id: String,
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}
