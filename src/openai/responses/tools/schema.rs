use super::super::tagged::{lossless_tagged_enum, ExtraFields};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAINetworkPolicyDisabled {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAINetworkPolicyAllowlist {
    pub allowed_domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_secrets: Option<Vec<OpenAINetworkPolicyDomainSecret>>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAINetworkPolicyDomainSecret {
    pub domain: String,
    pub name: String,
    pub value: String,
}

impl std::fmt::Debug for OpenAINetworkPolicyDomainSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenAINetworkPolicyDomainSecret")
            .field("domain", &self.domain)
            .field("name", &self.name)
            .field("value", &"[redacted]")
            .finish()
    }
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIContainerNetworkPolicy {
        Disabled(OpenAINetworkPolicyDisabled) => "disabled",
        Allowlist(OpenAINetworkPolicyAllowlist) => "allowlist",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAISkillReference {
    pub skill_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIInlineSkill {
    pub name: String,
    pub description: String,
    pub source: OpenAIInlineSkillSource,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAIInlineSkillSource {
    #[serde(rename = "type")]
    pub kind: OpenAIInlineSkillSourceType,
    pub media_type: OpenAIInlineSkillMediaType,
    pub data: String,
}

impl std::fmt::Debug for OpenAIInlineSkillSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenAIInlineSkillSource")
            .field("kind", &self.kind)
            .field("media_type", &self.media_type)
            .field("data", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIInlineSkillSourceType {
    Base64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenAIInlineSkillMediaType {
    #[serde(rename = "application/zip")]
    ApplicationZip,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIContainerSkill {
        Reference(OpenAISkillReference) => "skill_reference",
        Inline(OpenAIInlineSkill) => "inline",
        @unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAILocalSkill {
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFileSearchRankingOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hybrid_search: Option<OpenAIHybridSearchOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIHybridSearchOptions {
    pub embedding_weight: f64,
    pub text_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIFileSearchFilter {
    Comparison(OpenAIComparisonFilter),
    Compound(OpenAICompoundFilter),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIComparisonFilter {
    #[serde(rename = "type")]
    pub operator: OpenAIComparisonOperator,
    pub key: String,
    pub value: OpenAIComparisonValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIComparisonOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    Nin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIComparisonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<OpenAIComparisonArrayValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIComparisonArrayValue {
    String(String),
    Number(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompoundFilter {
    #[serde(rename = "type")]
    pub operator: OpenAICompoundOperator,
    pub filters: Vec<OpenAIFileSearchFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAICompoundOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIWebSearchFilters {
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIApproximateLocation {
    #[serde(rename = "type")]
    pub kind: OpenAIApproximateLocationType,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIApproximateLocationType {
    Approximate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMcpToolFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIMcpAllowedTools {
    Names(Vec<String>),
    Filter(OpenAIMcpToolFilter),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIMcpApprovalPolicy {
    Setting(OpenAIMcpApprovalSetting),
    Filter(OpenAIMcpApprovalFilter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIMcpApprovalSetting {
    Always,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenAIMcpApprovalFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always: Option<OpenAIMcpToolFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub never: Option<OpenAIMcpToolFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAICodeInterpreterContainer {
    Id(String),
    Auto(OpenAICodeInterpreterAuto),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICodeInterpreterAuto {
    #[serde(rename = "type")]
    pub kind: OpenAICodeInterpreterAutoType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    pub memory_limit: Option<OpenAIContainerMemoryLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<OpenAIContainerNetworkPolicy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAICodeInterpreterAutoType {
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenAIContainerMemoryLimit {
    #[serde(rename = "1g")]
    G1,
    #[serde(rename = "4g")]
    G4,
    #[serde(rename = "16g")]
    G16,
    #[serde(rename = "64g")]
    G64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIImageMask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIShellContainerAuto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    pub memory_limit: Option<OpenAIContainerMemoryLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<OpenAIContainerNetworkPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<OpenAIContainerSkill>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIShellLocalEnvironment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<OpenAILocalSkill>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIShellContainerReference {
    pub container_id: String,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIShellEnvironment {
        ContainerAuto(OpenAIShellContainerAuto) => "container_auto",
        Local(OpenAIShellLocalEnvironment) => "local",
        ContainerReference(OpenAIShellContainerReference) => "container_reference",
        @unknown
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAICustomTextFormat {
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICustomGrammarFormat {
    pub syntax: OpenAIGrammarSyntax,
    pub definition: String,
    #[serde(default, flatten)]
    pub extra: ExtraFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIGrammarSyntax {
    Lark,
    Regex,
}

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAICustomToolFormat {
        Text(OpenAICustomTextFormat) => "text",
        Grammar(OpenAICustomGrammarFormat) => "grammar",
        @unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIToolSearchExecution {
    Server,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenAICallableToolCaller {
    Direct,
    Programmatic,
}
