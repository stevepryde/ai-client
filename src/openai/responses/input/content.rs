//! Typed request input protocol for the Responses API.

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::Map;

use super::super::tagged::ExtraFields;
use super::items::OpenAIResponseInputItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIPromptCacheBreakpoint {
    pub mode: OpenAIPromptCacheBreakpointMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIPromptCacheBreakpointMode {
    Explicit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIResponsesInput {
    Text(String),
    Items(Vec<OpenAIResponseInputItem>),
}

impl From<String> for OpenAIResponsesInput {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for OpenAIResponsesInput {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIResponsesInputContent {
    Text(String),
    Parts(Vec<OpenAIResponsesInputContentPart>),
}

impl From<String> for OpenAIResponsesInputContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for OpenAIResponsesInputContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<Vec<OpenAIResponsesInputContentPart>> for OpenAIResponsesInputContent {
    fn from(value: Vec<OpenAIResponsesInputContentPart>) -> Self {
        Self::Parts(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputTextContent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIInputImageContent {
    pub detail: OpenAIImageDetail,
    #[serde(flatten)]
    pub source: OpenAIImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl<'de> Deserialize<'de> for OpenAIInputImageContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            detail: OpenAIImageDetail,
            image_url: Option<String>,
            file_id: Option<String>,
            prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
            #[serde(default, flatten)]
            extra: ExtraFields,
        }
        let wire = Wire::deserialize(deserializer)?;
        let source = match (wire.image_url, wire.file_id) {
            (Some(image_url), None) => OpenAIImageSource::Url { image_url },
            (None, Some(file_id)) => OpenAIImageSource::File { file_id },
            _ => {
                return Err(D::Error::custom(
                    "input_image requires exactly one of image_url or file_id",
                ))
            }
        };
        Ok(Self {
            detail: wire.detail,
            source,
            prompt_cache_breakpoint: wire.prompt_cache_breakpoint,
            extra: wire.extra,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIInputFileContent {
    #[serde(flatten)]
    pub source: OpenAIFileSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl<'de> Deserialize<'de> for OpenAIInputFileContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            file_id: Option<String>,
            file_data: Option<String>,
            file_url: Option<String>,
            filename: Option<String>,
            detail: Option<String>,
            prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
            #[serde(default, flatten)]
            extra: ExtraFields,
        }
        let wire = Wire::deserialize(deserializer)?;
        let source = match (wire.file_id, wire.file_data, wire.file_url) {
            (Some(file_id), None, None) => OpenAIFileSource::File { file_id },
            (None, Some(file_data), None) => OpenAIFileSource::Data { file_data },
            (None, None, Some(file_url)) => OpenAIFileSource::Url { file_url },
            _ => {
                return Err(D::Error::custom(
                    "input_file requires exactly one of file_id, file_data, or file_url",
                ))
            }
        };
        Ok(Self {
            source,
            filename: wire.filename,
            detail: wire.detail,
            prompt_cache_breakpoint: wire.prompt_cache_breakpoint,
            extra: wire.extra,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIImageSource {
    Url { image_url: String },
    File { file_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIFileSource {
    File { file_id: String },
    Data { file_data: String },
    Url { file_url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIImageDetail {
    Low,
    High,
    Auto,
}

super::super::tagged::lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponsesInputContentPart {
        InputText(OpenAIInputTextContent) => "input_text",
        InputImage(OpenAIInputImageContent) => "input_image",
        InputFile(OpenAIInputFileContent) => "input_file",
        @unknown
    }
}

impl OpenAIResponsesInputContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self::InputText(OpenAIInputTextContent {
            text: text.into(),
            prompt_cache_breakpoint: None,
            extra: Map::new(),
        })
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        Self::InputImage(OpenAIInputImageContent {
            detail: OpenAIImageDetail::Auto,
            source: OpenAIImageSource::Url {
                image_url: url.into(),
            },
            prompt_cache_breakpoint: None,
            extra: Map::new(),
        })
    }

    pub fn image_base64(mime_type: &str, base64_data: &str) -> Self {
        Self::image_url(format!("data:{mime_type};base64,{base64_data}"))
    }

    pub fn image_with_detail(url: impl Into<String>, detail: OpenAIImageDetail) -> Self {
        Self::InputImage(OpenAIInputImageContent {
            detail,
            source: OpenAIImageSource::Url {
                image_url: url.into(),
            },
            prompt_cache_breakpoint: None,
            extra: Map::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_pinned_input_content_tags_decode_typed() {
        let values = [
            serde_json::json!({"type":"input_text","text":"hello"}),
            serde_json::json!({"type":"input_image","detail":"auto","image_url":"https://example.test/a.png"}),
            serde_json::json!({"type":"input_file","file_id":"file_1"}),
        ];
        for value in values {
            let part: OpenAIResponsesInputContentPart = serde_json::from_value(value).unwrap();
            assert!(!matches!(part, OpenAIResponsesInputContentPart::Unknown(_)));
        }
    }

    #[test]
    fn image_and_file_sources_reject_missing_or_multiple_sources() {
        assert!(serde_json::from_value::<OpenAIResponsesInputContentPart>(
            serde_json::json!({"type":"input_image","detail":"auto","image_url":"x","file_id":"y"})
        )
        .is_err());
        assert!(serde_json::from_value::<OpenAIResponsesInputContentPart>(
            serde_json::json!({"type":"input_file"})
        )
        .is_err());
    }

    #[test]
    fn prompt_cache_breakpoint_uses_the_closed_explicit_mode() {
        let value = serde_json::json!({
            "type":"input_text",
            "text":"cached prefix",
            "prompt_cache_breakpoint":{"mode":"explicit"}
        });
        let part: OpenAIResponsesInputContentPart = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(part).unwrap(), value);
        assert!(
            serde_json::from_value::<OpenAIResponsesInputContentPart>(serde_json::json!({
                "type":"input_text",
                "text":"cached prefix",
                "prompt_cache_breakpoint":{"mode":"automatic"}
            }))
            .is_err()
        );
    }
}
