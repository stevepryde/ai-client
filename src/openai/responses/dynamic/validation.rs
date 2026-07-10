use crate::openai::responses::{OpenAIResponsesTextFormat, OpenAIResponsesTool};

use super::{
    builder::DynamicResponseRequestBuilder,
    catalog::{DynamicRequestError, ValidationMode, ValidationWarning},
};

impl DynamicResponseRequestBuilder {
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

    pub(super) fn validate(&self) -> Result<Vec<ValidationWarning>, DynamicRequestError> {
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
