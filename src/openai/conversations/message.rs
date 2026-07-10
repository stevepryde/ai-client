use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::openai::responses::{
    OpenAIInputFileContent, OpenAIInputTextContent, OpenAIOutputItemStatus,
    OpenAIOutputTextContent, OpenAIPromptCacheBreakpoint, OpenAIReasoningTextContent,
    OpenAIRefusalContent, OpenAIResponseItem, RawTaggedValue, ResponseItemId,
};

use super::{deserialize_response_item_id, serialize_response_item_id};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationMessageRole {
    Unknown,
    User,
    Assistant,
    System,
    Critic,
    Discriminator,
    Developer,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationMessagePhase {
    Commentary,
    FinalAnswer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationImageDetail {
    Low,
    High,
    Auto,
    Original,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTextContent {
    pub text: String,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInputImageContent {
    pub image_url: Option<String>,
    pub file_id: Option<String>,
    pub detail: ConversationImageDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationComputerScreenshotContent {
    pub image_url: Option<String>,
    pub file_id: Option<String>,
    pub detail: ConversationImageDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_breakpoint: Option<OpenAIPromptCacheBreakpoint>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone)]
pub enum ConversationMessageContent {
    InputText(OpenAIInputTextContent),
    OutputText(OpenAIOutputTextContent),
    Text(ConversationTextContent),
    SummaryText(ConversationTextContent),
    ReasoningText(OpenAIReasoningTextContent),
    Refusal(OpenAIRefusalContent),
    InputImage(ConversationInputImageContent),
    ComputerScreenshot(ConversationComputerScreenshotContent),
    InputFile(OpenAIInputFileContent),
    Unknown(RawTaggedValue),
}

impl Serialize for ConversationMessageContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::InputText(value) => serialize_payload("input_text", value, serializer),
            Self::OutputText(value) => serialize_payload("output_text", value, serializer),
            Self::Text(value) => serialize_payload("text", value, serializer),
            Self::SummaryText(value) => serialize_payload("summary_text", value, serializer),
            Self::ReasoningText(value) => serialize_payload("reasoning_text", value, serializer),
            Self::Refusal(value) => serialize_payload("refusal", value, serializer),
            Self::InputImage(value) => serialize_payload("input_image", value, serializer),
            Self::ComputerScreenshot(value) => {
                serialize_payload("computer_screenshot", value, serializer)
            }
            Self::InputFile(value) => serialize_payload("input_file", value, serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ConversationMessageContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(raw) = value else {
            return Err(D::Error::custom(
                "conversation message content must be an object",
            ));
        };
        let tag = raw
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| D::Error::missing_field("type"))?
            .to_owned();
        match tag.as_str() {
            "input_text" => deserialize_payload(raw).map(Self::InputText),
            "output_text" => deserialize_payload(raw).map(Self::OutputText),
            "text" => deserialize_payload(raw).map(Self::Text),
            "summary_text" => deserialize_payload(raw).map(Self::SummaryText),
            "reasoning_text" => deserialize_payload(raw).map(Self::ReasoningText),
            "refusal" => deserialize_payload(raw).map(Self::Refusal),
            "input_image" => deserialize_payload(raw).map(Self::InputImage),
            "computer_screenshot" => {
                require_keys::<D::Error>(&raw, &["image_url", "file_id", "detail"])?;
                deserialize_payload(raw).map(Self::ComputerScreenshot)
            }
            "input_file" => deserialize_payload(raw).map(Self::InputFile),
            _ => Ok(Self::Unknown(RawTaggedValue::from_map(tag, raw))),
        }
    }
}

fn require_keys<E>(raw: &Map<String, Value>, keys: &[&'static str]) -> Result<(), E>
where
    E: serde::de::Error,
{
    for key in keys {
        if !raw.contains_key(*key) {
            return Err(E::missing_field(key));
        }
    }
    Ok(())
}

fn deserialize_payload<T, E>(mut raw: Map<String, Value>) -> Result<T, E>
where
    T: for<'de> Deserialize<'de>,
    E: serde::de::Error,
{
    raw.remove("type");
    serde_json::from_value(Value::Object(raw)).map_err(E::custom)
}

fn serialize_payload<S, T>(tag: &'static str, payload: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let value = serde_json::to_value(payload).map_err(serde::ser::Error::custom)?;
    let Value::Object(mut object) = value else {
        return Err(serde::ser::Error::custom(
            "tagged payload must serialize as an object",
        ));
    };
    object.insert("type".into(), Value::String(tag.into()));
    object.serialize(serializer)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    #[serde(
        deserialize_with = "deserialize_response_item_id",
        serialize_with = "serialize_response_item_id"
    )]
    pub id: ResponseItemId,
    pub status: OpenAIOutputItemStatus,
    pub role: ConversationMessageRole,
    pub content: Vec<ConversationMessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<ConversationMessagePhase>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone)]
pub enum ConversationItem {
    Message(ConversationMessage),
    Response(Box<OpenAIResponseItem>),
}

impl Serialize for ConversationItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Message(message) => serialize_payload("message", message, serializer),
            Self::Response(item) => item.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ConversationItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(raw) = &value else {
            return Err(D::Error::custom("conversation item must be an object"));
        };
        match raw.get("type").and_then(Value::as_str) {
            Some("message") => deserialize_payload(raw.clone()).map(ConversationItem::Message),
            Some(_) => serde_json::from_value(value)
                .map(Box::new)
                .map(ConversationItem::Response)
                .map_err(D::Error::custom),
            None => Err(D::Error::missing_field("type")),
        }
    }
}
