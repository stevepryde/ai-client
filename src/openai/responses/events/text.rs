use serde::{Deserialize, Serialize};

use super::super::{
    output::{OpenAILogProb, OpenAIResponseAnnotation, OpenAIResponseContentPart},
    tagged::ExtraFields,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseContentPartEvent {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub part: OpenAIResponseContentPart,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseTextDeltaEvent {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseTextDoneEvent {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub text: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseRefusalDoneEvent {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub refusal: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputTextDelta {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub delta: String,
    pub sequence_number: u64,
    pub logprobs: Vec<OpenAILogProb>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseOutputTextDone {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub text: String,
    pub sequence_number: u64,
    pub logprobs: Vec<OpenAILogProb>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAnnotationAddedEvent {
    pub item_id: String,
    pub output_index: u64,
    pub content_index: u64,
    pub annotation_index: u64,
    pub annotation: OpenAIResponseAnnotation,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}
