use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::openai::responses::{
    input::OpenAIResponsesInputContentPart,
    tagged::{lossless_tagged_enum, ExtraFields},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolCallOutput {
    Text(String),
    Content(Vec<OpenAIResponsesInputContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIVectorStoreAttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

pub type OpenAIVectorStoreFileAttributes = BTreeMap<String, OpenAIVectorStoreAttributeValue>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFileSearchResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<OpenAIVectorStoreFileAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIFileSearchStatus {
    InProgress,
    Searching,
    Completed,
    Incomplete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAICodeInterpreterStatus {
    InProgress,
    Completed,
    Incomplete,
    Interpreting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterLogs {
    pub logs: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterImage {
    pub url: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAICodeInterpreterOutput {
        Logs(OpenAICodeInterpreterLogs) => "logs",
        Image(OpenAICodeInterpreterImage) => "image",
        @unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIProgramOutputStatus {
    Completed,
    Incomplete,
}

/// Tool-search input arguments are an intentionally open object in the API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpenAIToolSearchArguments(pub Map<String, Value>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpListedTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema is an intentionally open object boundary.
    pub input_schema: Map<String, Value>,
    /// MCP annotations are provider-defined arbitrary object data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Map<String, Value>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIDirectToolCaller {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProgramToolCaller {
    pub caller_id: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIToolCallCaller {
        Direct(OpenAIDirectToolCaller) => "direct",
        Program(OpenAIProgramToolCaller) => "program",
        @unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter_outputs_are_typed_and_future_tags_round_trip() {
        let logs = serde_json::json!({"type":"logs","logs":"ok"});
        assert!(matches!(
            serde_json::from_value::<OpenAICodeInterpreterOutput>(logs).unwrap(),
            OpenAICodeInterpreterOutput::Logs(_)
        ));
        let future = serde_json::json!({"type":"video","url":"future"});
        let output: OpenAICodeInterpreterOutput = serde_json::from_value(future.clone()).unwrap();
        assert_eq!(serde_json::to_value(output).unwrap(), future);
        assert!(serde_json::from_value::<OpenAICodeInterpreterOutput>(
            serde_json::json!({"type":"logs"})
        )
        .is_err());
    }

    #[test]
    fn tool_call_output_accepts_text_or_typed_content() {
        let text: OpenAIToolCallOutput = serde_json::from_str("\"done\"").unwrap();
        assert!(matches!(text, OpenAIToolCallOutput::Text(_)));
        let parts = serde_json::json!([{"type":"input_text","text":"done"}]);
        let output: OpenAIToolCallOutput = serde_json::from_value(parts).unwrap();
        assert!(matches!(output, OpenAIToolCallOutput::Content(_)));
    }
}
