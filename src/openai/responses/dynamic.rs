use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    sync::Arc,
};

use crate::openai::{
    create_response::{
        OpenAIResponsesInput, OpenAIResponsesReasoning, OpenAIResponsesTextConfig,
        OpenAIResponsesTextFormat, OpenAIResponsesTool,
    },
    OpenAIReasoningEffort,
};

use super::{
    models::MODEL_EVIDENCE, request::OpenAIResponsesWireRequest, HasInput, MissingInput,
    PreparedResponseRequest, Temperature, TopP,
};

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
pub struct DynamicOpenAIModel(String);

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
    settings: BTreeSet<&'static str>,
    reasoning_efforts: BTreeSet<String>,
    prompt_cache_retentions: BTreeSet<String>,
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

    fn supports(&self, setting: &'static str) -> bool {
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

pub struct DynamicResponseRequest;

impl DynamicResponseRequest {
    pub fn builder(model: DynamicOpenAIModel) -> DynamicResponseRequestBuilder<MissingInput> {
        DynamicResponseRequestBuilder {
            wire: OpenAIResponsesWireRequest::new(model.0),
            mode: ValidationMode::Off,
            catalog: None,
            state: PhantomData,
        }
    }
}

pub struct DynamicResponseRequestBuilder<State> {
    wire: OpenAIResponsesWireRequest,
    mode: ValidationMode,
    catalog: Option<Arc<dyn ResponseModelCapabilitiesCatalog>>,
    state: PhantomData<State>,
}

impl<S> DynamicResponseRequestBuilder<S> {
    pub fn instructions(mut self, value: impl Into<String>) -> Self {
        self.wire.instructions = Some(value.into());
        self
    }
    pub fn max_output_tokens(mut self, value: u64) -> Self {
        self.wire.max_output_tokens = Some(value);
        self
    }
    pub fn temperature(mut self, value: Temperature) -> Self {
        self.wire.temperature = Some(value.get());
        self
    }
    pub fn top_p(mut self, value: TopP) -> Self {
        self.wire.top_p = Some(value.get());
        self
    }
    pub fn reasoning(mut self, effort: OpenAIReasoningEffort) -> Self {
        self.wire.reasoning = Some(OpenAIResponsesReasoning {
            effort: Some(effort),
        });
        self
    }
    pub fn prompt_cache_key(mut self, value: impl Into<String>) -> Self {
        self.wire.prompt_cache_key = Some(value.into());
        self
    }
    pub fn prompt_cache_retention(mut self, value: impl Into<String>) -> Self {
        self.wire.prompt_cache_retention = Some(value.into());
        self
    }
    pub fn text_format(mut self, format: OpenAIResponsesTextFormat) -> Self {
        self.wire.text = Some(OpenAIResponsesTextConfig {
            format: Some(format),
        });
        self
    }
    pub fn tool(mut self, tool: OpenAIResponsesTool) -> Self {
        self.wire.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }
    pub fn previous_response_id(mut self, value: impl Into<String>) -> Self {
        self.wire.previous_response_id = Some(value.into());
        self
    }
    pub fn store(mut self, value: bool) -> Self {
        self.wire.store = Some(value);
        self
    }
    pub fn validation(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn catalog(mut self, catalog: impl ResponseModelCapabilitiesCatalog + 'static) -> Self {
        self.catalog = Some(Arc::new(catalog));
        self
    }
    pub fn builtin_catalog(self) -> Self {
        self.catalog(StaticResponseModelCapabilitiesCatalog::builtin())
    }

    fn configured_settings(&self) -> Vec<&'static str> {
        let mut settings = Vec::new();
        if self.wire.temperature.is_some() {
            settings.push("temperature");
        }
        if self.wire.top_p.is_some() {
            settings.push("top_p");
        }
        if self.wire.reasoning.is_some() {
            settings.push("reasoning");
        }
        if self.wire.prompt_cache_key.is_some() {
            settings.push("prompt_cache_key");
        }
        if self.wire.prompt_cache_retention.is_some() {
            settings.push("prompt_cache_retention");
        }
        if self
            .wire
            .text
            .as_ref()
            .and_then(|text| text.format.as_ref())
            .is_some_and(|format| matches!(format, OpenAIResponsesTextFormat::JsonSchema(_)))
        {
            settings.push("structured_output");
        }
        if self.wire.tools.as_ref().is_some_and(|tools| {
            tools
                .iter()
                .any(|tool| matches!(tool, OpenAIResponsesTool::ImageGeneration(_)))
        }) {
            settings.push("image_generation_tool");
        }
        settings
    }

    fn validate(&self) -> Result<Vec<ValidationWarning>, DynamicRequestError> {
        if self.mode == ValidationMode::Off {
            return Ok(Vec::new());
        }
        let Some(catalog) = self.catalog.as_ref() else {
            return match self.mode {
                ValidationMode::Warn => Ok(vec![ValidationWarning::MissingCatalog]),
                ValidationMode::Strict => Err(DynamicRequestError::MissingCatalog),
                ValidationMode::Off => unreachable!(),
            };
        };
        let Some(capabilities) = catalog.capabilities(&self.wire.model) else {
            return match self.mode {
                ValidationMode::Warn => Ok(vec![ValidationWarning::UnknownModel {
                    model: self.wire.model.clone(),
                }]),
                ValidationMode::Strict => {
                    Err(DynamicRequestError::UnknownModel(self.wire.model.clone()))
                }
                ValidationMode::Off => unreachable!(),
            };
        };
        let mut warnings = Vec::new();
        for setting in self.configured_settings() {
            if !capabilities.supports(setting) {
                let error = DynamicRequestError::UnsupportedSetting {
                    model: self.wire.model.clone(),
                    setting,
                };
                if self.mode == ValidationMode::Strict {
                    return Err(error);
                }
                warnings.push(ValidationWarning::UnsupportedSetting {
                    model: self.wire.model.clone(),
                    setting,
                });
            }
        }
        if capabilities.supports("reasoning") {
            if let Some(effort) = self
                .wire
                .reasoning
                .as_ref()
                .and_then(|reasoning| reasoning.effort.as_ref())
            {
                let effort = serde_json::to_value(effort)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned));
                if effort
                    .as_ref()
                    .is_some_and(|effort| !capabilities.reasoning_efforts.contains(effort))
                {
                    if self.mode == ValidationMode::Strict {
                        return Err(DynamicRequestError::UnsupportedSetting {
                            model: self.wire.model.clone(),
                            setting: "reasoning_effort",
                        });
                    }
                    warnings.push(ValidationWarning::UnsupportedSetting {
                        model: self.wire.model.clone(),
                        setting: "reasoning_effort",
                    });
                }
            }
        }
        if capabilities.supports("prompt_cache_retention")
            && self
                .wire
                .prompt_cache_retention
                .as_ref()
                .is_some_and(|retention| !capabilities.prompt_cache_retentions.contains(retention))
        {
            if self.mode == ValidationMode::Strict {
                return Err(DynamicRequestError::UnsupportedSetting {
                    model: self.wire.model.clone(),
                    setting: "prompt_cache_retention_value",
                });
            }
            warnings.push(ValidationWarning::UnsupportedSetting {
                model: self.wire.model.clone(),
                setting: "prompt_cache_retention_value",
            });
        }
        Ok(warnings)
    }
}

impl DynamicResponseRequestBuilder<MissingInput> {
    pub fn input(
        mut self,
        input: impl Into<OpenAIResponsesInput>,
    ) -> DynamicResponseRequestBuilder<HasInput> {
        self.wire.input = Some(input.into());
        DynamicResponseRequestBuilder {
            wire: self.wire,
            mode: self.mode,
            catalog: self.catalog,
            state: PhantomData,
        }
    }
}

impl DynamicResponseRequestBuilder<HasInput> {
    pub fn build(self) -> Result<PreparedResponseRequest, DynamicRequestError> {
        let warnings = self.validate()?;
        Ok(PreparedResponseRequest::new(self.wire, warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_validation_warns_or_fails_without_coercing() {
        let model = DynamicOpenAIModel::new("gpt-4o").unwrap();
        let warned = DynamicResponseRequest::builder(model.clone())
            .input(OpenAIResponsesInput::Text("secret".into()))
            .reasoning(OpenAIReasoningEffort::High)
            .validation(ValidationMode::Warn)
            .builtin_catalog()
            .build()
            .unwrap();
        assert_eq!(warned.warnings().len(), 1);
        let value = serde_json::to_value(&warned).unwrap();
        assert_eq!(value["reasoning"]["effort"], "high");

        let error = DynamicResponseRequest::builder(model)
            .input(OpenAIResponsesInput::Text("secret".into()))
            .reasoning(OpenAIReasoningEffort::High)
            .validation(ValidationMode::Strict)
            .builtin_catalog()
            .build()
            .unwrap_err();
        assert!(matches!(
            error,
            DynamicRequestError::UnsupportedSetting { .. }
        ));
    }

    #[test]
    fn builtin_catalog_covers_every_checked_in_marker() {
        let catalog = StaticResponseModelCapabilitiesCatalog::builtin();
        for evidence in MODEL_EVIDENCE {
            let capabilities = catalog
                .capabilities(evidence.id)
                .unwrap_or_else(|| panic!("missing {}", evidence.id));
            assert_eq!(
                capabilities.supports("temperature"),
                evidence.sampling,
                "{}",
                evidence.id
            );
            assert_eq!(
                capabilities.supports("top_p"),
                evidence.sampling,
                "{}",
                evidence.id
            );
            assert_eq!(
                capabilities.reasoning_efforts,
                evidence.reasoning.iter().map(ToString::to_string).collect(),
                "{}",
                evidence.id
            );
            assert_eq!(
                capabilities.prompt_cache_retentions,
                evidence
                    .cache_retentions
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                "{}",
                evidence.id
            );
            assert_eq!(
                capabilities.supports("structured_output"),
                evidence.structured_output,
                "{}",
                evidence.id
            );
            assert_eq!(
                capabilities.supports("image_generation_tool"),
                evidence.image_tool,
                "{}",
                evidence.id
            );
        }
    }

    #[test]
    fn strict_rejects_unknown_retention_and_sampling_values_are_bounded() {
        assert!(Temperature::new(f64::NAN).is_err());
        assert!(Temperature::new(2.1).is_err());
        assert!(TopP::new(-0.1).is_err());

        let error = DynamicResponseRequest::builder(DynamicOpenAIModel::new("gpt-5.5").unwrap())
            .input(OpenAIResponsesInput::Text("secret".into()))
            .prompt_cache_retention("in_memory")
            .validation(ValidationMode::Strict)
            .builtin_catalog()
            .build()
            .unwrap_err();
        assert!(matches!(
            error,
            DynamicRequestError::UnsupportedSetting {
                setting: "prompt_cache_retention_value",
                ..
            }
        ));
    }

    #[test]
    fn builtin_catalog_matches_representative_typed_capability_groups() {
        let catalog = StaticResponseModelCapabilitiesCatalog::builtin();
        let gpt4o = catalog.capabilities("gpt-4o").unwrap();
        assert!(gpt4o.supports("temperature"));
        assert!(gpt4o.supports("image_generation_tool"));
        assert!(!gpt4o.supports("reasoning"));
        assert!(catalog
            .capabilities("gpt-5-nano")
            .unwrap()
            .supports("image_generation_tool"));
        assert!(!catalog
            .capabilities("gpt-5.1")
            .unwrap()
            .supports("image_generation_tool"));

        let pro = catalog.capabilities("gpt-5.5-pro").unwrap();
        assert!(pro.reasoning_efforts.contains("xhigh"));
        assert!(!pro.reasoning_efforts.contains("low"));
        assert_eq!(
            pro.prompt_cache_retentions,
            BTreeSet::from(["24h".to_string()])
        );
    }

    #[test]
    fn dynamic_plain_text_is_not_misclassified_as_structured_output() {
        let model = DynamicOpenAIModel::new("gpt-5.4-pro").unwrap();
        DynamicResponseRequest::builder(model.clone())
            .input(OpenAIResponsesInput::Text("hello".into()))
            .text_format(OpenAIResponsesTextFormat::Text)
            .validation(ValidationMode::Strict)
            .builtin_catalog()
            .build()
            .unwrap();

        let error = DynamicResponseRequest::builder(model)
            .input(OpenAIResponsesInput::Text("hello".into()))
            .text_format(OpenAIResponsesTextFormat::JsonSchema(
                crate::openai::OpenAIJsonSchema {
                    name: "result".into(),
                    description: "result".into(),
                    schema: serde_json::json!({"type":"object"}),
                    strict: Some(true),
                },
            ))
            .validation(ValidationMode::Strict)
            .builtin_catalog()
            .build()
            .unwrap_err();
        assert!(matches!(
            error,
            DynamicRequestError::UnsupportedSetting {
                setting: "structured_output",
                ..
            }
        ));
    }
}
