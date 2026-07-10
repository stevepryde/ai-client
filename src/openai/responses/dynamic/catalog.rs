use std::collections::{BTreeMap, BTreeSet};

use super::super::models::MODEL_EVIDENCE;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DynamicRequestError {
    #[error("model ID must be non-empty, at most 256 bytes, and contain no whitespace or control characters")]
    InvalidModelId,
    #[error("strict validation requires a capabilities catalog")]
    MissingCatalog,
    #[error("model `{0}` is absent from capabilities catalog")]
    UnknownModel(String),
    #[error("model `{model}` does not support configured setting `{setting}`")]
    UnsupportedSetting {
        model: String,
        setting: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    MissingCatalog,
    UnknownModel {
        model: String,
    },
    UnsupportedSetting {
        model: String,
        setting: &'static str,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ValidationMode {
    #[default]
    Off,
    Warn,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicOpenAIModel(pub(super) String);

impl DynamicOpenAIModel {
    pub fn new(id: impl Into<String>) -> Result<Self, DynamicRequestError> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 256
            || id
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(DynamicRequestError::InvalidModelId);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResponseModelCapabilities {
    pub(super) settings: BTreeSet<&'static str>,
    pub(super) reasoning_efforts: BTreeSet<String>,
    pub(super) prompt_cache_retentions: BTreeSet<String>,
}

impl ResponseModelCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn setting(mut self, setting: &'static str) -> Self {
        self.settings.insert(setting);
        self
    }

    pub fn reasoning_efforts(
        mut self,
        efforts: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.settings.insert("reasoning");
        self.reasoning_efforts = efforts.into_iter().map(Into::into).collect();
        self
    }

    pub fn prompt_cache_retentions(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.settings.insert("prompt_cache_retention");
        self.prompt_cache_retentions = values.into_iter().map(Into::into).collect();
        self
    }

    pub(super) fn supports(&self, setting: &'static str) -> bool {
        self.settings.contains(setting)
    }
}

pub trait ResponseModelCapabilitiesCatalog: Send + Sync {
    fn version(&self) -> &str;
    fn capabilities(&self, model: &str) -> Option<&ResponseModelCapabilities>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticResponseModelCapabilitiesCatalog {
    version: String,
    models: BTreeMap<String, ResponseModelCapabilities>,
}

impl StaticResponseModelCapabilitiesCatalog {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            models: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        model: impl Into<String>,
        capabilities: ResponseModelCapabilities,
    ) -> Option<ResponseModelCapabilities> {
        self.models.insert(model.into(), capabilities)
    }

    pub fn builtin() -> Self {
        let mut catalog = Self::new("openai-responses-2026-07-10");
        for evidence in MODEL_EVIDENCE {
            debug_assert!(evidence.url.starts_with("https://developers.openai.com/"));
            debug_assert_eq!(evidence.reviewed, "2026-07-10");
            let mut capabilities = ResponseModelCapabilities::new().setting("prompt_cache_key");
            if evidence.sampling {
                capabilities = capabilities.setting("temperature").setting("top_p");
            }
            if !evidence.reasoning.is_empty() {
                capabilities = capabilities.reasoning_efforts(evidence.reasoning.iter().copied());
            }
            if !evidence.cache_retentions.is_empty() {
                capabilities =
                    capabilities.prompt_cache_retentions(evidence.cache_retentions.iter().copied());
            }
            if evidence.structured_output {
                capabilities = capabilities.setting("structured_output");
            }
            if evidence.image_tool {
                capabilities = capabilities.setting("image_generation_tool");
            }
            catalog.insert(evidence.id, capabilities);
        }
        catalog
    }
}

impl ResponseModelCapabilitiesCatalog for StaticResponseModelCapabilitiesCatalog {
    fn version(&self) -> &str {
        &self.version
    }
    fn capabilities(&self, model: &str) -> Option<&ResponseModelCapabilities> {
        self.models.get(model)
    }
}
