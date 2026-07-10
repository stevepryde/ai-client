use serde::{Deserialize, Serialize};

#[cfg(feature = "chat-completions")]
pub mod create_chat_completion;
pub mod create_response;
pub mod list_models;

#[cfg(feature = "chat-completions")]
pub(crate) fn sanitise_request_params(
    model: &crate::openai::OpenAIModel,
    temperature: &mut Option<f64>,
    reasoning_effort: &mut Option<OpenAIReasoningEffort>,
    cache_key: &mut Option<String>,
    cache_retention: &mut Option<String>,
) {
    use crate::openai::OpenAIModel;
    if !model.allow_temperature() {
        *temperature = None;
    }
    if !model.supports_reasoning() {
        *reasoning_effort = None;
    } else {
        match model {
            OpenAIModel::Gpt5_1 | OpenAIModel::Gpt5 => match reasoning_effort {
                Some(OpenAIReasoningEffort::XHigh) => {
                    *reasoning_effort = Some(OpenAIReasoningEffort::High)
                }
                Some(OpenAIReasoningEffort::Minimal) => {
                    *reasoning_effort = Some(OpenAIReasoningEffort::Low)
                }
                _ => {}
            },
            OpenAIModel::Gpt5_4
            | OpenAIModel::Gpt5_4Mini
            | OpenAIModel::Gpt5_4Nano
            | OpenAIModel::Gpt5_5 => {
                if matches!(reasoning_effort, Some(OpenAIReasoningEffort::Minimal)) {
                    *reasoning_effort = Some(OpenAIReasoningEffort::Low);
                }
            }
            OpenAIModel::Gpt5_4Pro | OpenAIModel::Gpt5_5Pro => {
                if matches!(
                    reasoning_effort,
                    Some(
                        OpenAIReasoningEffort::None
                            | OpenAIReasoningEffort::Minimal
                            | OpenAIReasoningEffort::Low
                    )
                ) {
                    *reasoning_effort = Some(OpenAIReasoningEffort::Medium);
                }
            }
            _ => {}
        }
    }
    if !model.supports_caching() {
        *cache_key = None;
        *cache_retention = None;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OpenAIReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIPrompt {
    pub role: OpenAIRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OpenAIRole {
    Assistant,
    Developer,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
pub struct OpenAIJsonSchema {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
    pub strict: Option<bool>,
}

#[cfg(all(test, feature = "chat-completions"))]
mod tests {
    use super::{sanitise_request_params, OpenAIReasoningEffort};
    use crate::openai::OpenAIModel;

    #[test]
    fn legacy_chat_keeps_gpt_5_4_and_5_5_supported_reasoning_effort() {
        for model in [
            OpenAIModel::Gpt5_4,
            OpenAIModel::Gpt5_4Mini,
            OpenAIModel::Gpt5_4Nano,
            OpenAIModel::Gpt5_5,
        ] {
            let mut temperature = Some(0.7);
            let mut reasoning_effort = Some(OpenAIReasoningEffort::XHigh);
            let mut cache_key = Some("cache-key".to_string());
            let mut cache_retention = Some("24h".to_string());

            sanitise_request_params(
                &model,
                &mut temperature,
                &mut reasoning_effort,
                &mut cache_key,
                &mut cache_retention,
            );

            assert_eq!(temperature, None);
            assert!(matches!(
                reasoning_effort,
                Some(OpenAIReasoningEffort::XHigh)
            ));
            assert_eq!(cache_key.as_deref(), Some("cache-key"));
            assert_eq!(cache_retention.as_deref(), Some("24h"));
        }
    }

    #[test]
    fn legacy_chat_preserves_existing_reasoning_coercion() {
        for model in [
            OpenAIModel::Gpt5_1,
            OpenAIModel::Gpt5,
            OpenAIModel::Gpt5_4,
            OpenAIModel::Gpt5_4Mini,
            OpenAIModel::Gpt5_4Nano,
            OpenAIModel::Gpt5_5,
        ] {
            let mut temperature = Some(0.7);
            let mut reasoning_effort = Some(OpenAIReasoningEffort::Minimal);
            let mut cache_key = Some("cache-key".to_string());
            let mut cache_retention = Some("24h".to_string());

            sanitise_request_params(
                &model,
                &mut temperature,
                &mut reasoning_effort,
                &mut cache_key,
                &mut cache_retention,
            );

            assert_eq!(temperature, None);
            assert!(matches!(reasoning_effort, Some(OpenAIReasoningEffort::Low)));
        }
    }

    #[test]
    fn legacy_chat_preserves_pro_model_sanitisation() {
        for model in [OpenAIModel::Gpt5_4Pro, OpenAIModel::Gpt5_5Pro] {
            let mut temperature = Some(0.7);
            let mut reasoning_effort = Some(OpenAIReasoningEffort::Low);
            let mut cache_key = Some("cache-key".to_string());
            let mut cache_retention = Some("24h".to_string());

            sanitise_request_params(
                &model,
                &mut temperature,
                &mut reasoning_effort,
                &mut cache_key,
                &mut cache_retention,
            );

            assert_eq!(temperature, None);
            assert!(matches!(
                reasoning_effort,
                Some(OpenAIReasoningEffort::Medium)
            ));
            assert_eq!(cache_key, None);
            assert_eq!(cache_retention, None);
        }
    }
}
