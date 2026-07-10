use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::openai::{OpenAIJsonSchema, OpenAIReasoningEffort};

use super::super::tagged::ExtraFields;

#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIResponsesTextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OpenAIResponsesTextFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<OpenAIResponseVerbosity>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAITextFormat {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIJsonObjectFormat {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIJsonSchemaTextFormat {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub schema: Value,
    pub strict: Option<bool>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl From<OpenAIJsonSchema> for OpenAIJsonSchemaTextFormat {
    fn from(value: OpenAIJsonSchema) -> Self {
        Self {
            name: value.name,
            description: Some(value.description),
            schema: value.schema,
            strict: value.strict,
            extra: ExtraFields::new(),
        }
    }
}

super::super::tagged::lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponsesTextFormat {
        Text(OpenAITextFormat) => "text",
        JsonSchema(OpenAIJsonSchemaTextFormat) => "json_schema",
        JsonObject(OpenAIJsonObjectFormat) => "json_object",
        @unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIResponseVerbosity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIReasoningContext {
    Auto,
    CurrentTurn,
    AllTurns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponsesReasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<OpenAIReasoningEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<OpenAIReasoningSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<OpenAIReasoningContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_summary: Option<OpenAIReasoningSummary>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_pinned_text_formats_decode_typed() {
        let values = [
            serde_json::json!({"type":"text"}),
            serde_json::json!({"type":"json_schema","name":"result","schema":{"type":"object"},"strict":true}),
            serde_json::json!({"type":"json_object"}),
        ];
        for value in values {
            let format: OpenAIResponsesTextFormat = serde_json::from_value(value).unwrap();
            assert!(!matches!(format, OpenAIResponsesTextFormat::Unknown(_)));
        }
    }
}
