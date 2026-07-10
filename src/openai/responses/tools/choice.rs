use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::openai::responses::tagged::{lossless_tagged_enum, ExtraFields};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpenAIAllowedToolDefinition(std::collections::BTreeMap<String, Value>);

impl OpenAIAllowedToolDefinition {
    pub fn new(values: std::collections::BTreeMap<String, Value>) -> Self {
        Self(values)
    }

    pub fn as_map(&self) -> &std::collections::BTreeMap<String, Value> {
        &self.0
    }

    pub fn into_map(self) -> std::collections::BTreeMap<String, Value> {
        self.0
    }
}

impl From<std::collections::BTreeMap<String, Value>> for OpenAIAllowedToolDefinition {
    fn from(values: std::collections::BTreeMap<String, Value>) -> Self {
        Self::new(values)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIToolChoiceMode {
    None,
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAllowedToolsChoice {
    pub mode: OpenAIAllowedToolsMode,
    pub tools: Vec<OpenAIAllowedToolDefinition>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIAllowedToolsMode {
    Auto,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIHostedToolChoice {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAINamedToolChoice {
    pub name: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpToolChoice {
    pub server_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAITypeOnlyToolChoice {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIObjectToolChoice {
        AllowedTools(OpenAIAllowedToolsChoice) => "allowed_tools",
        FileSearch(OpenAIHostedToolChoice) => "file_search",
        WebSearchPreview(OpenAIHostedToolChoice) => "web_search_preview",
        Computer(OpenAIHostedToolChoice) => "computer",
        ComputerUsePreview(OpenAIHostedToolChoice) => "computer_use_preview",
        ComputerUse(OpenAIHostedToolChoice) => "computer_use",
        WebSearchPreview2025_03_11(OpenAIHostedToolChoice) => "web_search_preview_2025_03_11",
        ImageGeneration(OpenAIHostedToolChoice) => "image_generation",
        CodeInterpreter(OpenAIHostedToolChoice) => "code_interpreter",
        Function(OpenAINamedToolChoice) => "function",
        Mcp(OpenAIMcpToolChoice) => "mcp",
        Custom(OpenAINamedToolChoice) => "custom",
        ProgrammaticToolCalling(OpenAITypeOnlyToolChoice) => "programmatic_tool_calling",
        ApplyPatch(OpenAITypeOnlyToolChoice) => "apply_patch",
        Shell(OpenAITypeOnlyToolChoice) => "shell",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIToolChoice {
    Mode(OpenAIToolChoiceMode),
    Object(OpenAIObjectToolChoice),
}
