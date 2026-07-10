use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::{OpenAIResponsesInputContent, OpenAIResponsesInputContentPart};
use crate::openai::responses::{
    output::{OpenAIOutputItemStatus, OpenAIResponseOutputItem},
    tagged::{deserialize_payload, serialize_payload, ExtraFields, RawTaggedValue},
    OpenAIApplyPatchCallStatus, OpenAIApplyPatchOperation, OpenAIApplyPatchOutputStatus,
    OpenAIComputerSafetyCheck, OpenAIComputerScreenshot, OpenAIFunctionShellAction,
    OpenAIFunctionShellEnvironment, OpenAIFunctionShellOutputContent, OpenAIProgramOutputStatus,
    OpenAIResponsesTool, OpenAIToolCallCaller, OpenAIToolCallOutput, OpenAIToolSearchArguments,
    OpenAIToolSearchExecution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIMessageRole {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIEasyInputMessage {
    pub role: OpenAIMessageRole,
    pub content: OpenAIResponsesInputContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInputMessageItem {
    pub role: OpenAIInputMessageRole,
    pub content: Vec<OpenAIResponsesInputContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIInputMessageRole {
    User,
    System,
    Developer,
}

#[derive(Debug, Clone)]
pub enum OpenAIResponseItem {
    InputMessage(OpenAIInputMessageItem),
    Output(Box<OpenAIResponseOutputItem>),
    Unknown(RawTaggedValue),
}

impl Serialize for OpenAIResponseItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::InputMessage(item) => serialize_payload("message", item, serializer),
            Self::Output(item) => item.serialize(serializer),
            Self::Unknown(raw) => raw.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for OpenAIResponseItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(raw) = &value else {
            return Err(D::Error::custom("response item must be an object"));
        };
        let tag = raw
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| D::Error::missing_field("type"))?;
        if tag == "message" {
            let role = raw.get("role").and_then(Value::as_str);
            if role == Some("assistant") {
                return serde_json::from_value(value)
                    .map(Box::new)
                    .map(OpenAIResponseItem::Output)
                    .map_err(D::Error::custom);
            }
            return deserialize_payload(raw.clone()).map(Self::InputMessage);
        }
        let output: OpenAIResponseOutputItem =
            serde_json::from_value(value).map_err(D::Error::custom)?;
        match output {
            OpenAIResponseOutputItem::Unknown(raw) => Ok(Self::Unknown(raw)),
            known => Ok(Self::Output(Box::new(known))),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAICompactionTriggerItem {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIItemReference {
    pub id: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProgramInputItem {
    pub id: String,
    pub call_id: String,
    pub code: String,
    pub fingerprint: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProgramOutputInputItem {
    pub id: String,
    pub call_id: String,
    pub result: String,
    pub status: OpenAIProgramOutputStatus,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallOutputInputItem {
    pub call_id: String,
    pub output: OpenAIToolCallOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComputerCallOutputInputItem {
    pub call_id: String,
    pub output: OpenAIComputerScreenshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub acknowledged_safety_checks: Option<Vec<OpenAIComputerSafetyCheck>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSearchCallInputItem {
    pub arguments: OpenAIToolSearchArguments,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<OpenAIToolSearchExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolSearchOutputInputItem {
    pub tools: Vec<OpenAIResponsesTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<OpenAIToolSearchExecution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIAdditionalToolsInputItem {
    pub role: OpenAIInputMessageRole,
    pub tools: Vec<OpenAIResponsesTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompactionInputItem {
    pub encrypted_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellCallInputItem {
    pub call_id: String,
    pub action: OpenAIFunctionShellAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    pub environment: Option<OpenAIFunctionShellEnvironment>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellCallOutputInputItem {
    pub call_id: String,
    pub output: Vec<OpenAIFunctionShellOutputContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OpenAIOutputItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<u64>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchCallInputItem {
    pub call_id: String,
    pub status: OpenAIApplyPatchCallStatus,
    pub operation: OpenAIApplyPatchOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchCallOutputInputItem {
    pub call_id: String,
    pub status: OpenAIApplyPatchOutputStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    pub output: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpApprovalResponseInputItem {
    pub request_id: String,
    pub approve: bool,
    pub approval_request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub reason: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomToolCallOutputInputItem {
    pub call_id: String,
    pub output: OpenAIToolCallOutput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<OpenAIToolCallCaller>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone)]
pub enum OpenAIResponseInputItem {
    Message(OpenAIEasyInputMessage),
    FunctionCallOutput(OpenAIFunctionCallOutputInputItem),
    ComputerCallOutput(OpenAIComputerCallOutputInputItem),
    ToolSearchCall(OpenAIToolSearchCallInputItem),
    ToolSearchOutput(OpenAIToolSearchOutputInputItem),
    AdditionalTools(OpenAIAdditionalToolsInputItem),
    Compaction(OpenAICompactionInputItem),
    FunctionShellCall(OpenAIFunctionShellCallInputItem),
    FunctionShellCallOutput(OpenAIFunctionShellCallOutputInputItem),
    ApplyPatchCall(OpenAIApplyPatchCallInputItem),
    ApplyPatchCallOutput(OpenAIApplyPatchCallOutputInputItem),
    McpApprovalResponse(OpenAIMcpApprovalResponseInputItem),
    CustomToolCallOutput(OpenAICustomToolCallOutputInputItem),
    CompactionTrigger(OpenAICompactionTriggerItem),
    ItemReference(OpenAIItemReference),
    Program(OpenAIProgramInputItem),
    ProgramOutput(OpenAIProgramOutputInputItem),
    Output(OpenAIResponseOutputItem),
    Unknown(RawTaggedValue),
}

impl Serialize for OpenAIResponseInputItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Message(value) => serialize_payload("message", value, serializer),
            Self::FunctionCallOutput(value) => {
                serialize_payload("function_call_output", value, serializer)
            }
            Self::ComputerCallOutput(value) => {
                serialize_payload("computer_call_output", value, serializer)
            }
            Self::ToolSearchCall(value) => serialize_payload("tool_search_call", value, serializer),
            Self::ToolSearchOutput(value) => {
                serialize_payload("tool_search_output", value, serializer)
            }
            Self::AdditionalTools(value) => {
                serialize_payload("additional_tools", value, serializer)
            }
            Self::Compaction(value) => serialize_payload("compaction", value, serializer),
            Self::FunctionShellCall(value) => serialize_payload("shell_call", value, serializer),
            Self::FunctionShellCallOutput(value) => {
                serialize_payload("shell_call_output", value, serializer)
            }
            Self::ApplyPatchCall(value) => serialize_payload("apply_patch_call", value, serializer),
            Self::ApplyPatchCallOutput(value) => {
                serialize_payload("apply_patch_call_output", value, serializer)
            }
            Self::McpApprovalResponse(value) => {
                serialize_payload("mcp_approval_response", value, serializer)
            }
            Self::CustomToolCallOutput(value) => {
                serialize_payload("custom_tool_call_output", value, serializer)
            }
            Self::CompactionTrigger(value) => {
                serialize_payload("compaction_trigger", value, serializer)
            }
            Self::ItemReference(value) => serialize_payload("item_reference", value, serializer),
            Self::Program(value) => serialize_payload("program", value, serializer),
            Self::ProgramOutput(value) => serialize_payload("program_output", value, serializer),
            Self::Output(value) => value.serialize(serializer),
            Self::Unknown(raw) => raw.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for OpenAIResponseInputItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(raw) = &value else {
            return Err(D::Error::custom("response input item must be an object"));
        };
        let tag = raw.get("type").and_then(Value::as_str);
        if tag.is_none() && raw.contains_key("id") && raw.len() == 1 {
            return deserialize_payload(raw.clone()).map(Self::ItemReference);
        }
        let tag = tag.ok_or_else(|| D::Error::missing_field("type"))?;
        macro_rules! decode {
            ($variant:ident, $ty:ty) => {
                deserialize_payload::<$ty, D::Error>(raw.clone()).map(Self::$variant)
            };
        }
        match tag {
            "message" => decode!(Message, OpenAIEasyInputMessage),
            "function_call_output" => {
                decode!(FunctionCallOutput, OpenAIFunctionCallOutputInputItem)
            }
            "computer_call_output" => {
                decode!(ComputerCallOutput, OpenAIComputerCallOutputInputItem)
            }
            "tool_search_call" => decode!(ToolSearchCall, OpenAIToolSearchCallInputItem),
            "tool_search_output" => decode!(ToolSearchOutput, OpenAIToolSearchOutputInputItem),
            "additional_tools" => decode!(AdditionalTools, OpenAIAdditionalToolsInputItem),
            "compaction" => decode!(Compaction, OpenAICompactionInputItem),
            "shell_call" => decode!(FunctionShellCall, OpenAIFunctionShellCallInputItem),
            "shell_call_output" => decode!(
                FunctionShellCallOutput,
                OpenAIFunctionShellCallOutputInputItem
            ),
            "apply_patch_call" => decode!(ApplyPatchCall, OpenAIApplyPatchCallInputItem),
            "apply_patch_call_output" => {
                decode!(ApplyPatchCallOutput, OpenAIApplyPatchCallOutputInputItem)
            }
            "mcp_approval_response" => {
                decode!(McpApprovalResponse, OpenAIMcpApprovalResponseInputItem)
            }
            "custom_tool_call_output" => {
                decode!(CustomToolCallOutput, OpenAICustomToolCallOutputInputItem)
            }
            "compaction_trigger" => decode!(CompactionTrigger, OpenAICompactionTriggerItem),
            "item_reference" => decode!(ItemReference, OpenAIItemReference),
            "program" => decode!(Program, OpenAIProgramInputItem),
            "program_output" => decode!(ProgramOutput, OpenAIProgramOutputInputItem),
            _ => {
                let output: OpenAIResponseOutputItem =
                    serde_json::from_value(value).map_err(D::Error::custom)?;
                match output {
                    OpenAIResponseOutputItem::Unknown(raw) => Ok(Self::Unknown(raw)),
                    known => Ok(Self::Output(known)),
                }
            }
        }
    }
}

/// Migration alias for the original item name.
pub type OpenAIResponsesInputItem = OpenAIResponseInputItem;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_dispatch_is_role_aware_for_resource_items() {
        let input = serde_json::json!({
            "type":"message", "role":"user",
            "content":[{"type":"input_text","text":"hello"}]
        });
        assert!(matches!(
            serde_json::from_value::<OpenAIResponseItem>(input).unwrap(),
            OpenAIResponseItem::InputMessage(_)
        ));
        let malformed_assistant = serde_json::json!({
            "type":"message", "role":"assistant", "content":[]
        });
        assert!(serde_json::from_value::<OpenAIResponseItem>(malformed_assistant).is_err());
    }

    #[test]
    fn unknown_input_item_round_trips() {
        let future = serde_json::json!({"type":"future_item","secret":"retained"});
        let item: OpenAIResponseInputItem = serde_json::from_value(future.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), future);
    }

    #[test]
    fn closed_tool_output_inputs_decode_typed() {
        let values = [
            serde_json::json!({
                "type":"function_call_output", "call_id":"fc_1",
                "output":[{"type":"input_text","text":"done"}]
            }),
            serde_json::json!({
                "type":"computer_call_output", "call_id":"cu_1",
                "output":{"type":"computer_screenshot","image_url":"https://example.test/a.png"},
                "acknowledged_safety_checks":[{"id":"safe_1"}]
            }),
            serde_json::json!({
                "type":"shell_call_output", "call_id":"sh_1",
                "output":[{"stdout":"ok","stderr":"","outcome":{"type":"exit","exit_code":0}}]
            }),
            serde_json::json!({
                "type":"apply_patch_call", "call_id":"ap_1", "status":"completed",
                "operation":{"type":"create_file","path":"a.txt","diff":"+hello"}
            }),
        ];
        for value in values {
            let item: OpenAIResponseInputItem = serde_json::from_value(value).unwrap();
            assert!(!matches!(item, OpenAIResponseInputItem::Unknown(_)));
        }
    }

    #[test]
    fn malformed_known_nested_input_is_rejected_but_future_nested_tag_round_trips() {
        let malformed = serde_json::json!({
            "type":"apply_patch_call", "call_id":"ap_1", "status":"completed",
            "operation":{"type":"create_file","path":"a.txt"}
        });
        assert!(serde_json::from_value::<OpenAIResponseInputItem>(malformed).is_err());

        let future = serde_json::json!({
            "type":"apply_patch_call", "call_id":"ap_1", "status":"completed",
            "operation":{"type":"rename_file","from":"a","to":"b"}
        });
        let item: OpenAIResponseInputItem = serde_json::from_value(future.clone()).unwrap();
        assert_eq!(serde_json::to_value(item).unwrap(), future);
    }
}
