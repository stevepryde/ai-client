use super::super::tagged::{lossless_tagged_enum, ExtraFields};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIOutputTextContent {
    pub text: String,
    pub annotations: Vec<OpenAIResponseAnnotation>,
    pub logprobs: Vec<OpenAILogProb>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<u8>,
    pub top_logprobs: Vec<OpenAITopLogProb>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITopLogProb {
    pub token: String,
    pub logprob: f64,
    pub bytes: Vec<u8>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRefusalContent {
    pub refusal: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIReasoningTextContent {
    pub text: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponseContentPart {
        OutputText(OpenAIOutputTextContent) => "output_text",
        Refusal(OpenAIRefusalContent) => "refusal",
        ReasoningText(OpenAIReasoningTextContent) => "reasoning_text",
        @unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_content_and_annotation_tags_decode_typed() {
        let content = [
            serde_json::json!({"type":"output_text","text":"x","annotations":[],"logprobs":[]}),
            serde_json::json!({"type":"refusal","refusal":"x"}),
            serde_json::json!({"type":"reasoning_text","text":"x"}),
        ];
        for value in content {
            let part: OpenAIResponseContentPart = serde_json::from_value(value).unwrap();
            assert!(!matches!(part, OpenAIResponseContentPart::Unknown(_)));
        }

        let annotations = [
            serde_json::json!({"type":"file_citation","file_id":"f","index":0,"filename":"a"}),
            serde_json::json!({"type":"url_citation","url":"https://example.test","start_index":0,"end_index":1,"title":"a"}),
            serde_json::json!({"type":"container_file_citation","container_id":"c","file_id":"f","start_index":0,"end_index":1,"filename":"a"}),
            serde_json::json!({"type":"file_path","file_id":"f","index":0}),
        ];
        for value in annotations {
            let annotation: OpenAIResponseAnnotation = serde_json::from_value(value).unwrap();
            assert!(!matches!(annotation, OpenAIResponseAnnotation::Unknown(_)));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFileCitationAnnotation {
    pub file_id: String,
    pub index: u64,
    pub filename: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUrlCitationAnnotation {
    pub url: String,
    pub start_index: u64,
    pub end_index: u64,
    pub title: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIContainerFileCitationAnnotation {
    pub container_id: String,
    pub file_id: String,
    pub start_index: u64,
    pub end_index: u64,
    pub filename: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFilePathAnnotation {
    pub file_id: String,
    pub index: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponseAnnotation {
        FileCitation(OpenAIFileCitationAnnotation) => "file_citation",
        UrlCitation(OpenAIUrlCitationAnnotation) => "url_citation",
        ContainerFileCitation(OpenAIContainerFileCitationAnnotation) => "container_file_citation",
        FilePath(OpenAIFilePathAnnotation) => "file_path",
        @unknown
    }
}
