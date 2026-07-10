use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::openai::responses::{
    tagged::{lossless_tagged_enum, ExtraFields},
    tools::OpenAILocalSkill,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILocalShellExecAction {
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    pub env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAILocalShellAction {
        Exec(OpenAILocalShellExecAction) => "exec",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellAction {
    pub commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_length: Option<u64>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

/// Stored-response shell actions require both nullable limit fields to be
/// present, unlike request actions where those fields may be omitted.
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIFunctionShellResourceAction {
    pub commands: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub max_output_length: Option<u64>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

impl<'de> Deserialize<'de> for OpenAIFunctionShellResourceAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let raw = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        if !raw.contains_key("timeout_ms") {
            return Err(D::Error::missing_field("timeout_ms"));
        }
        if !raw.contains_key("max_output_length") {
            return Err(D::Error::missing_field("max_output_length"));
        }

        #[derive(Deserialize)]
        struct Wire {
            commands: Vec<String>,
            timeout_ms: Option<u64>,
            max_output_length: Option<u64>,
            #[serde(default, flatten)]
            extra: ExtraFields,
        }

        let wire: Wire =
            serde_json::from_value(serde_json::Value::Object(raw)).map_err(D::Error::custom)?;
        Ok(Self {
            commands: wire.commands,
            timeout_ms: wire.timeout_ms,
            max_output_length: wire.max_output_length,
            extra: wire.extra,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILocalEnvironment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<OpenAILocalSkill>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIContainerReferenceEnvironment {
    pub container_id: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIFunctionShellEnvironment {
        Local(OpenAILocalEnvironment) => "local",
        ContainerReference(OpenAIContainerReferenceEnvironment) => "container_reference",
        @unknown
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIFunctionShellTimeoutOutcome {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellExitOutcome {
    pub exit_code: i64,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIFunctionShellOutcome {
        Timeout(OpenAIFunctionShellTimeoutOutcome) => "timeout",
        Exit(OpenAIFunctionShellExitOutcome) => "exit",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionShellOutputContent {
    pub stdout: String,
    pub stderr: String,
    pub outcome: OpenAIFunctionShellOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIApplyPatchCallStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIApplyPatchOutputStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchCreateFile {
    pub path: String,
    pub diff: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchDeleteFile {
    pub path: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApplyPatchUpdateFile {
    pub path: String,
    pub diff: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIApplyPatchOperation {
        CreateFile(OpenAIApplyPatchCreateFile) => "create_file",
        DeleteFile(OpenAIApplyPatchDeleteFile) => "delete_file",
        UpdateFile(OpenAIApplyPatchUpdateFile) => "update_file",
        @unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_outcomes_and_patch_operations_enforce_known_shapes() {
        let exit = serde_json::json!({"type":"exit","exit_code":0});
        assert!(matches!(
            serde_json::from_value::<OpenAIFunctionShellOutcome>(exit).unwrap(),
            OpenAIFunctionShellOutcome::Exit(_)
        ));
        assert!(serde_json::from_value::<OpenAIFunctionShellOutcome>(
            serde_json::json!({"type":"exit"})
        )
        .is_err());
        assert!(serde_json::from_value::<OpenAIApplyPatchOperation>(
            serde_json::json!({"type":"update_file","path":"a"})
        )
        .is_err());
    }

    #[test]
    fn future_patch_operation_round_trips_losslessly() {
        let future = serde_json::json!({"type":"rename_file","from":"a","to":"b"});
        let operation: OpenAIApplyPatchOperation = serde_json::from_value(future.clone()).unwrap();
        assert_eq!(serde_json::to_value(operation).unwrap(), future);
    }

    #[test]
    fn request_shell_limits_may_be_omitted_but_resource_limits_are_required_nullable() {
        let request: OpenAIFunctionShellAction =
            serde_json::from_value(serde_json::json!({"commands":["pwd"]})).unwrap();
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            serde_json::json!({"commands":["pwd"]})
        );

        assert!(serde_json::from_value::<OpenAIFunctionShellResourceAction>(
            serde_json::json!({"commands":["pwd"],"timeout_ms":null})
        )
        .is_err());
        let resource: OpenAIFunctionShellResourceAction =
            serde_json::from_value(serde_json::json!({
                "commands":["pwd"],
                "timeout_ms":null,
                "max_output_length":null
            }))
            .unwrap();
        assert_eq!(
            serde_json::to_value(resource).unwrap(),
            serde_json::json!({
                "commands":["pwd"],
                "timeout_ms":null,
                "max_output_length":null
            })
        );
    }
}
