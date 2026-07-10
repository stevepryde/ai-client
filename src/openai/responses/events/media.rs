use serde::{Deserialize, Serialize};

use super::super::{operations::ResponseId, tagged::ExtraFields};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAudioDeltaEvent {
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

/// `response_id` is required by the pinned source despite being omitted from
/// that schema's `properties`; retaining it follows the binding requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAudioDoneEvent {
    pub response_id: ResponseId,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAudioTranscriptDeltaEvent {
    pub response_id: ResponseId,
    pub delta: String,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAudioTranscriptDoneEvent {
    pub response_id: ResponseId,
    pub sequence_number: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationPartialEvent {
    pub output_index: u64,
    pub item_id: String,
    pub sequence_number: u64,
    pub partial_image_index: u64,
    pub partial_image_b64: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl OpenAIImageGenerationPartialEvent {
    pub fn decode_image(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.decode(&self.partial_image_b64)
    }
}

pub type OpenAIImageGenerationStatusEvent = super::OpenAIToolCallStatusEvent;
pub type OpenAIImageGenerationCompleteEvent = super::OpenAIToolCallStatusEvent;
