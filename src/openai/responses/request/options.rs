use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateResponseStreamOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    include_obfuscation: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResponseRequestValueError {
    #[error("top_logprobs must be between 0 and 20")]
    InvalidTopLogprobs,
    #[error("response metadata supports at most 16 entries")]
    TooManyMetadataEntries,
    #[error("response metadata keys must be non-empty and at most 64 characters")]
    InvalidMetadataKey,
    #[error("response metadata values must be at most 512 characters")]
    InvalidMetadataValue,
    #[error("context compaction threshold must be at least 1000")]
    InvalidCompactionThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TopLogprobs(u8);

impl TopLogprobs {
    pub fn new(value: u8) -> Result<Self, ResponseRequestValueError> {
        (value <= 20)
            .then_some(Self(value))
            .ok_or(ResponseRequestValueError::InvalidTopLogprobs)
    }
}

impl<'de> Deserialize<'de> for TopLogprobs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct OpenAIResponseMetadata(std::collections::BTreeMap<String, String>);

impl OpenAIResponseMetadata {
    pub fn new(
        entries: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Result<Self, ResponseRequestValueError> {
        let entries: std::collections::BTreeMap<_, _> = entries
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        if entries.len() > 16 {
            return Err(ResponseRequestValueError::TooManyMetadataEntries);
        }
        if entries
            .keys()
            .any(|key| key.is_empty() || key.chars().count() > 64)
        {
            return Err(ResponseRequestValueError::InvalidMetadataKey);
        }
        if entries.values().any(|value| value.chars().count() > 512) {
            return Err(ResponseRequestValueError::InvalidMetadataValue);
        }
        Ok(Self(entries))
    }
}

impl<'de> Deserialize<'de> for OpenAIResponseMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let entries = std::collections::BTreeMap::<String, String>::deserialize(deserializer)?;
        Self::new(entries).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIPromptTemplate {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModerationConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<OpenAIModerationPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModerationPolicy {
    pub input: Option<OpenAIModerationRule>,
    pub output: Option<OpenAIModerationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIModerationRule {
    pub mode: OpenAIModerationMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIModerationMode {
    Score,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIContextCompaction {
    #[serde(rename = "type")]
    kind: OpenAIContextCompactionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    compact_threshold: Option<u64>,
}

impl OpenAIContextCompaction {
    pub fn new(threshold: Option<u64>) -> Result<Self, ResponseRequestValueError> {
        if threshold.is_some_and(|threshold| threshold < 1000) {
            return Err(ResponseRequestValueError::InvalidCompactionThreshold);
        }
        Ok(Self {
            kind: OpenAIContextCompactionType::Compaction,
            compact_threshold: threshold,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OpenAIContextCompactionType {
    Compaction,
}

impl CreateResponseStreamOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_obfuscation(mut self, include: bool) -> Self {
        self.include_obfuscation = Some(include);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.include_obfuscation.is_none()
    }
}
