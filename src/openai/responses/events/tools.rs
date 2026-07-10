use serde::{Deserialize, Serialize};

use super::super::tagged::ExtraFields;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterCodeDeltaEvent {
    pub output_index: u64,
    pub item_id: String,
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterCodeDoneEvent {
    pub output_index: u64,
    pub item_id: String,
    pub code: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionArgumentsDoneEvent {
    pub item_id: String,
    pub name: String,
    pub output_index: u64,
    pub arguments: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpArgumentsDoneEvent {
    pub output_index: u64,
    pub item_id: String,
    pub arguments: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomToolInputDoneEvent {
    pub output_index: u64,
    pub item_id: String,
    pub input: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}
