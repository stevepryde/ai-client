//! Typed, forward-compatible response output protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    tagged::{lossless_tagged_enum, ExtraFields},
    OpenAIApplyPatchCallStatus, OpenAIApplyPatchOperation, OpenAIApplyPatchOutputStatus,
    OpenAICodeInterpreterOutput, OpenAICodeInterpreterStatus, OpenAIComputerAction,
    OpenAIComputerSafetyCheck, OpenAIComputerScreenshot, OpenAIFileSearchResult,
    OpenAIFileSearchStatus, OpenAIFunctionShellEnvironment, OpenAIFunctionShellOutputContent,
    OpenAIFunctionShellResourceAction, OpenAILocalShellAction, OpenAIMcpListedTool,
    OpenAIProgramOutputStatus, OpenAIResponsesTool, OpenAIToolCallCaller, OpenAIToolCallOutput,
    OpenAIToolSearchExecution, OpenAIWebSearchAction, OpenAIWebSearchStatus,
};
use super::content::OpenAIResponseContentPart;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseMessageItem {
    pub id: String,
    pub role: OpenAIOutputRole,
    pub content: Vec<OpenAIResponseContentPart>,
    pub status: OpenAIOutputItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIOutputRole {
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIOutputItemStatus {
    InProgress,
    Completed,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFileSearchCallItem {
    pub id: String,
    pub status: OpenAIFileSearchStatus,
    pub queries: Vec<String>,
    pub results: Option<Vec<OpenAIFileSearchResult>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallItem {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallOutputItem {
    pub id: String,
    pub status: OpenAIOutputItemStatus,
    pub call_id: String,
    pub output: OpenAIToolCallOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchCallItem {
    pub id: String,
    pub status: OpenAIWebSearchStatus,
    pub action: OpenAIWebSearchAction,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComputerCallItem {
    pub id: String,
    pub call_id: String,
    pub pending_safety_checks: Vec<OpenAIComputerSafetyCheck>,
    pub status: OpenAIOutputItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<OpenAIComputerAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<OpenAIComputerAction>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComputerCallOutputItem {
    pub id: String,
    pub status: OpenAIOutputItemStatus,
    pub call_id: String,
    pub output: OpenAIComputerScreenshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: Option<Vec<OpenAIComputerSafetyCheck>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponseReasoningItem {
    pub id: String,
    pub summary: Vec<OpenAIReasoningSummaryPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<OpenAIResponseContentPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIReasoningSummaryPart {
    SummaryText {
        text: String,
        #[serde(default, flatten)]
        extra: ExtraFields,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProgramItem {
    pub id: String,
    pub call_id: String,
    pub code: String,
    pub fingerprint: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProgramOutputItem {
    pub id: String,
    pub call_id: String,
    pub result: String,
    pub status: OpenAIProgramOutputStatus,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSearchCallItem {
    pub id: String,
    pub call_id: String,
    pub execution: OpenAIToolSearchExecution,
    /// The resource schema intentionally leaves tool-search arguments open.
    pub arguments: Value,
    pub status: OpenAIOutputItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSearchOutputItem {
    pub id: String,
    pub call_id: String,
    pub execution: OpenAIToolSearchExecution,
    pub tools: Vec<OpenAIResponsesTool>,
    pub status: OpenAIOutputItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAdditionalToolsItem {
    pub id: String,
    pub role: String,
    pub tools: Vec<OpenAIResponsesTool>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompactionItem {
    pub id: String,
    pub encrypted_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageGenerationCallItem {
    pub id: String,
    pub status: OpenAIOutputItemStatus,
    pub result: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl OpenAIImageGenerationCallItem {
    pub fn decode_image(&self) -> Result<Option<Vec<u8>>, base64::DecodeError> {
        use base64::Engine as _;
        self.result
            .as_ref()
            .map(|result| base64::engine::general_purpose::STANDARD.decode(result))
            .transpose()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterCallItem {
    pub id: String,
    pub status: OpenAICodeInterpreterStatus,
    pub container_id: String,
    pub code: Option<String>,
    pub outputs: Option<Vec<OpenAICodeInterpreterOutput>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILocalShellCallItem {
    pub id: String,
    pub call_id: String,
    pub action: OpenAILocalShellAction,
    pub status: OpenAIOutputItemStatus,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILocalShellCallOutputItem {
    pub id: String,
    pub call_id: String,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellCallItem {
    pub id: String,
    pub call_id: String,
    pub action: OpenAIFunctionShellResourceAction,
    pub status: OpenAIOutputItemStatus,
    pub environment: Option<OpenAIFunctionShellEnvironment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellCallOutputItem {
    pub id: String,
    pub call_id: String,
    pub status: OpenAIOutputItemStatus,
    pub output: Vec<OpenAIFunctionShellOutputContent>,
    pub max_output_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchCallItem {
    pub id: String,
    pub call_id: String,
    pub status: OpenAIApplyPatchCallStatus,
    pub operation: OpenAIApplyPatchOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchCallOutputItem {
    pub id: String,
    pub call_id: String,
    pub status: OpenAIApplyPatchOutputStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpCallItem {
    pub id: String,
    pub server_label: String,
    pub name: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_request_id: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpListToolsItem {
    pub id: String,
    pub server_label: String,
    pub tools: Vec<OpenAIMcpListedTool>,
    pub error: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpApprovalRequestItem {
    pub id: String,
    pub server_label: String,
    pub name: String,
    pub arguments: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpApprovalResponseItem {
    pub id: String,
    pub request_id: String,
    pub approve: bool,
    pub approval_request_id: String,
    pub reason: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomToolCallItem {
    pub call_id: String,
    pub name: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomToolCallOutputItem {
    pub id: String,
    pub status: OpenAIOutputItemStatus,
    pub call_id: String,
    pub output: OpenAIToolCallOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponseOutputItem {
        Message(OpenAIResponseMessageItem) => "message",
        FileSearchCall(OpenAIFileSearchCallItem) => "file_search_call",
        FunctionCall(OpenAIFunctionCallItem) => "function_call",
        FunctionCallOutput(OpenAIFunctionCallOutputItem) => "function_call_output",
        WebSearchCall(OpenAIWebSearchCallItem) => "web_search_call",
        ComputerCall(OpenAIComputerCallItem) => "computer_call",
        ComputerCallOutput(OpenAIComputerCallOutputItem) => "computer_call_output",
        Reasoning(OpenAIResponseReasoningItem) => "reasoning",
        Program(OpenAIProgramItem) => "program",
        ProgramOutput(OpenAIProgramOutputItem) => "program_output",
        ToolSearchCall(OpenAIToolSearchCallItem) => "tool_search_call",
        ToolSearchOutput(OpenAIToolSearchOutputItem) => "tool_search_output",
        AdditionalTools(OpenAIAdditionalToolsItem) => "additional_tools",
        Compaction(OpenAICompactionItem) => "compaction",
        ImageGenerationCall(OpenAIImageGenerationCallItem) => "image_generation_call",
        CodeInterpreterCall(OpenAICodeInterpreterCallItem) => "code_interpreter_call",
        LocalShellCall(OpenAILocalShellCallItem) => "local_shell_call",
        LocalShellCallOutput(OpenAILocalShellCallOutputItem) => "local_shell_call_output",
        FunctionShellCall(OpenAIFunctionShellCallItem) => "shell_call",
        FunctionShellCallOutput(OpenAIFunctionShellCallOutputItem) => "shell_call_output",
        ApplyPatchCall(OpenAIApplyPatchCallItem) => "apply_patch_call",
        ApplyPatchCallOutput(OpenAIApplyPatchCallOutputItem) => "apply_patch_call_output",
        McpCall(OpenAIMcpCallItem) => "mcp_call",
        McpListTools(OpenAIMcpListToolsItem) => "mcp_list_tools",
        McpApprovalRequest(OpenAIMcpApprovalRequestItem) => "mcp_approval_request",
        McpApprovalResponse(OpenAIMcpApprovalResponseItem) => "mcp_approval_response",
        CustomToolCall(OpenAICustomToolCallItem) => "custom_tool_call",
        CustomToolCallOutput(OpenAICustomToolCallOutputItem) => "custom_tool_call_output",
        @unknown
    }
}

#[cfg(test)]
#[path = "items_tests.rs"]
mod tests;
