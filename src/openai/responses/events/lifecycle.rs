use serde::{Deserialize, Serialize};

use super::super::{
    output::{OpenAIResponseOutputItem, OpenAIResponsesCreateResponse},
    tagged::ExtraFields,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseEvent {
    pub response: OpenAIResponsesCreateResponse,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputItemEvent {
    pub output_index: u64,
    pub item: OpenAIResponseOutputItem,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamError {
    pub code: Option<String>,
    pub message: String,
    pub param: Option<String>,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}
