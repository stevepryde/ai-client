use serde::{Deserialize, Serialize};

use crate::openai::OpenAIModel;

pub mod create_chat_completion;
pub mod create_response;
pub mod list_models;

/// Helper function to sanitize request parameters based on model capabilities.
pub fn sanitise_request_params(
    model: &OpenAIModel,
    temperature: &mut Option<f64>,
    reasoning_effort: &mut Option<OpenAIReasoningEffort>,
    cache_key: &mut Option<String>,
    cache_retention: &mut Option<String>,
) {
    if !model.allow_temperature() {
        *temperature = None;
    }
    if !model.supports_reasoning() {
        *reasoning_effort = None;
    } else {
        match model {
            OpenAIModel::Gpt4oMini
            | OpenAIModel::Gpt4o
            | OpenAIModel::Gpt4_1Mini
            | OpenAIModel::Gpt4_1Nano => match reasoning_effort {
                Some(OpenAIReasoningEffort::XHigh) => {
                    *reasoning_effort = Some(OpenAIReasoningEffort::High);
                }
                _ => {}
            },
            OpenAIModel::Gpt5_1
            | OpenAIModel::Gpt5
            | OpenAIModel::Gpt5Mini
            | OpenAIModel::Gpt5Nano => match reasoning_effort {
                Some(OpenAIReasoningEffort::XHigh) => {
                    *reasoning_effort = Some(OpenAIReasoningEffort::High);
                }
                Some(OpenAIReasoningEffort::Minimal) => {
                    *reasoning_effort = Some(OpenAIReasoningEffort::None);
                }
                _ => {}
            },
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
