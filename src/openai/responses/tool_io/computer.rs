use serde::{Deserialize, Serialize};

use crate::openai::responses::tagged::{lossless_tagged_enum, ExtraFields};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIClickButton {
    Left,
    Right,
    Wheel,
    Back,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICoordinate {
    pub x: i64,
    pub y: i64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIClickAction {
    pub button: OpenAIClickButton,
    pub x: i64,
    pub y: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDoubleClickAction {
    pub x: i64,
    pub y: i64,
    pub keys: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDragAction {
    pub path: Vec<OpenAICoordinate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIKeyPressAction {
    pub keys: Vec<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMoveAction {
    pub x: i64,
    pub y: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIScreenshotAction {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIScrollAction {
    pub x: i64,
    pub y: i64,
    pub scroll_x: i64,
    pub scroll_y: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITypeAction {
    pub text: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIWaitAction {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIComputerAction {
        Click(OpenAIClickAction) => "click",
        DoubleClick(OpenAIDoubleClickAction) => "double_click",
        Drag(OpenAIDragAction) => "drag",
        KeyPress(OpenAIKeyPressAction) => "keypress",
        Move(OpenAIMoveAction) => "move",
        Screenshot(OpenAIScreenshotAction) => "screenshot",
        Scroll(OpenAIScrollAction) => "scroll",
        Type(OpenAITypeAction) => "type",
        Wait(OpenAIWaitAction) => "wait",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComputerSafetyCheck {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone)]
pub struct OpenAIComputerScreenshot {
    pub image_url: Option<String>,
    pub file_id: Option<String>,
    pub extra: ExtraFields,
}

impl serde::Serialize for OpenAIComputerScreenshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(1 + self.extra.len()))?;
        map.serialize_entry("type", "computer_screenshot")?;
        if let Some(image_url) = &self.image_url {
            map.serialize_entry("image_url", image_url)?;
        }
        if let Some(file_id) = &self.file_id {
            map.serialize_entry("file_id", file_id)?;
        }
        for (key, value) in &self.extra {
            if key != "type" && key != "image_url" && key != "file_id" {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for OpenAIComputerScreenshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let mut raw = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        match raw
            .remove("type")
            .and_then(|value| value.as_str().map(str::to_owned))
        {
            Some(tag) if tag == "computer_screenshot" => {}
            Some(tag) => {
                return Err(D::Error::custom(format!(
                    "unknown computer screenshot type `{tag}`"
                )))
            }
            None => return Err(D::Error::missing_field("type")),
        }
        let image_url = take_optional_string::<D::Error>(&mut raw, "image_url")?;
        let file_id = take_optional_string::<D::Error>(&mut raw, "file_id")?;
        Ok(Self {
            image_url,
            file_id,
            extra: raw,
        })
    }
}

fn take_optional_string<E: serde::de::Error>(
    raw: &mut serde_json::Map<String, serde_json::Value>,
    field: &'static str,
) -> Result<Option<String>, E> {
    match raw.remove(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(E::custom(format!("`{field}` must be a string or null"))),
    }
}
