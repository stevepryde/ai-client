use serde::Deserialize;

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatStreamEvent {
    pub id: String,
    pub model: String,
    #[serde(default)]
    pub choices: Vec<ChatStreamChoice>,
    pub usage: Option<super::ChatUsage>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatStreamChoice {
    pub index: u64,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct ChatDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub refusal: Option<String>,
}
