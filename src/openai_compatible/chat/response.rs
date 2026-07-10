use serde::{de::Error as _, Deserialize, Deserializer};

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatChoice {
    pub index: u64,
    pub finish_reason: Option<String>,
    pub message: ChatAssistantMessage,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatAssistantMessage {
    pub role: String,
    pub content: Option<String>,
    pub refusal: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ChatUsage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct ChatResponse {
    id: String,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
    raw: serde_json::Value,
}

impl ChatResponse {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn choices(&self) -> &[ChatChoice] {
        &self.choices
    }
    pub fn usage(&self) -> Option<&ChatUsage> {
        self.usage.as_ref()
    }
    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }
    pub fn into_raw(self) -> serde_json::Value {
        self.raw
    }
}

impl<'de> Deserialize<'de> for ChatResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            id: String,
            model: String,
            choices: Vec<ChatChoice>,
            usage: Option<ChatUsage>,
        }
        let raw = serde_json::Value::deserialize(deserializer)?;
        let wire: Wire = serde_json::from_value(raw.clone()).map_err(D::Error::custom)?;
        Ok(Self {
            id: wire.id,
            model: wire.model,
            choices: wire.choices,
            usage: wire.usage,
            raw,
        })
    }
}
