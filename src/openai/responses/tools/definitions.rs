use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{image::*, schema::*};
use crate::openai::responses::tagged::{lossless_tagged_enum, ExtraFields};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpenAIJsonSchemaObject(std::collections::BTreeMap<String, Value>);

impl OpenAIJsonSchemaObject {
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

impl From<std::collections::BTreeMap<String, Value>> for OpenAIJsonSchemaObject {
    fn from(values: std::collections::BTreeMap<String, Value>) -> Self {
        Self::new(values)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIOptionalJsonSchema {
    Object(OpenAIJsonSchemaObject),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionTool {
    pub name: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub strict: Option<bool>,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub parameters: Option<OpenAIJsonSchemaObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_json_schema",
        skip_serializing_if = "Option::is_none"
    )]
    pub output_schema: Option<OpenAIOptionalJsonSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

fn deserialize_required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

fn deserialize_optional_json_schema<'de, D>(
    deserializer: D,
) -> Result<Option<OpenAIOptionalJsonSchema>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    OpenAIOptionalJsonSchema::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFileSearchTool {
    pub vector_store_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<OpenAIFileSearchRankingOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<OpenAIFileSearchFilter>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIComputerTool {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComputerUsePreviewTool {
    pub environment: String,
    pub display_width: u64,
    pub display_height: u64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<OpenAIWebSearchFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<OpenAIApproximateLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAIMcpTool {
    pub server_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<OpenAIMcpAllowedTools>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<OpenAIMcpApprovalPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl std::fmt::Debug for OpenAIMcpTool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenAIMcpTool")
            .field("server_label", &self.server_label)
            .field(
                "server_url",
                &self.server_url.as_ref().map(|_| "[redacted]"),
            )
            .field("connector_id", &self.connector_id)
            .field("tunnel_id", &self.tunnel_id)
            .field(
                "authorization",
                &self.authorization.as_ref().map(|_| "[redacted]"),
            )
            .field("headers", &self.headers.as_ref().map(|_| "[redacted]"))
            .field("server_description", &self.server_description)
            .field("allowed_tools", &self.allowed_tools)
            .field("allowed_callers", &self.allowed_callers)
            .field("require_approval", &self.require_approval)
            .field("defer_loading", &self.defer_loading)
            .field("extra", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterTool {
    pub container: OpenAICodeInterpreterContainer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIProgrammaticToolCallingTool {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIImageGenerationTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<OpenAIImageModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<OpenAIImageQuality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<OpenAIImageSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<OpenAIImageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<OpenAIImageBackground>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fidelity: Option<OpenAIImageInputFidelity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_image_mask: Option<OpenAIImageMask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<OpenAIImageAction>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAILocalShellTool {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<OpenAIShellEnvironment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OpenAICustomToolFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAINamespaceTool {
    pub name: String,
    pub description: String,
    pub tools: Vec<OpenAIResponsesTool>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSearchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<OpenAIToolSearchExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::BTreeMap<String, Value>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchPreviewTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<OpenAIApproximateLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_content_types: Option<Vec<String>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchTool {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_callers: Option<Vec<OpenAICallableToolCaller>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponsesTool {
        Function(OpenAIFunctionTool) => "function",
        FileSearch(OpenAIFileSearchTool) => "file_search",
        Computer(OpenAIComputerTool) => "computer",
        ComputerUsePreview(OpenAIComputerUsePreviewTool) => "computer_use_preview",
        WebSearch(OpenAIWebSearchTool) => "web_search",
        WebSearch2025_08_26(OpenAIWebSearchTool) => "web_search_2025_08_26",
        Mcp(OpenAIMcpTool) => "mcp",
        CodeInterpreter(OpenAICodeInterpreterTool) => "code_interpreter",
        ProgrammaticToolCalling(OpenAIProgrammaticToolCallingTool) => "programmatic_tool_calling",
        ImageGeneration(OpenAIImageGenerationTool) => "image_generation",
        LocalShell(OpenAILocalShellTool) => "local_shell",
        FunctionShell(OpenAIFunctionShellTool) => "shell",
        Custom(OpenAICustomTool) => "custom",
        Namespace(OpenAINamespaceTool) => "namespace",
        ToolSearch(OpenAIToolSearchTool) => "tool_search",
        WebSearchPreview(OpenAIWebSearchPreviewTool) => "web_search_preview",
        WebSearchPreview2025_03_11(OpenAIWebSearchPreviewTool) => "web_search_preview_2025_03_11",
        ApplyPatch(OpenAIApplyPatchTool) => "apply_patch",
        @unknown
    }
}

impl OpenAIResponsesTool {
    pub fn image_generation() -> Self {
        Self::ImageGeneration(OpenAIImageGenerationTool::default())
    }

    pub fn image_generation_with_model(model: OpenAIImageModel) -> Self {
        Self::ImageGeneration(OpenAIImageGenerationTool {
            model: Some(model),
            ..Default::default()
        })
    }
}

macro_rules! into_responses_tool {
    ($variant:ident: $type:ty) => {
        impl crate::openai::responses::IntoResponsesTool for $type {
            fn into_responses_tool(self) -> OpenAIResponsesTool {
                OpenAIResponsesTool::$variant(self)
            }
        }
    };
}

into_responses_tool!(Function: OpenAIFunctionTool);
into_responses_tool!(FileSearch: OpenAIFileSearchTool);
into_responses_tool!(Computer: OpenAIComputerTool);
into_responses_tool!(ComputerUsePreview: OpenAIComputerUsePreviewTool);
into_responses_tool!(WebSearch: OpenAIWebSearchTool);
into_responses_tool!(Mcp: OpenAIMcpTool);
into_responses_tool!(CodeInterpreter: OpenAICodeInterpreterTool);
into_responses_tool!(ProgrammaticToolCalling: OpenAIProgrammaticToolCallingTool);
into_responses_tool!(ImageGeneration: OpenAIImageGenerationTool);
into_responses_tool!(LocalShell: OpenAILocalShellTool);
into_responses_tool!(FunctionShell: OpenAIFunctionShellTool);
into_responses_tool!(Custom: OpenAICustomTool);
into_responses_tool!(Namespace: OpenAINamespaceTool);
into_responses_tool!(ToolSearch: OpenAIToolSearchTool);
into_responses_tool!(WebSearchPreview: OpenAIWebSearchPreviewTool);
into_responses_tool!(ApplyPatch: OpenAIApplyPatchTool);
