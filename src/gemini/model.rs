use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::prelude::AiError;

#[non_exhaustive]
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde_with::DeserializeFromStr,
    serde_with::SerializeDisplay,
)]
pub enum Model {
    Gemini1_0Pro,
    Gemini1_0ProLatest,
    Gemini1_0ProVisionLatest,
    Gemini1_5Pro,
    #[default]
    Gemini1_5Flash,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Model::Gemini1_0Pro => "gemini-1.0-pro",
            Model::Gemini1_0ProLatest => "gemini-1.0-pro-latest",
            Model::Gemini1_0ProVisionLatest => "gemini-1.0-pro-vision-latest",
            Model::Gemini1_5Pro => "gemini-1.5-pro",
            Model::Gemini1_5Flash => "gemini-1.5-flash",
        };
        write!(f, "models/{name}")
    }
}

impl FromStr for Model {
    type Err = AiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("models/").unwrap_or(s) {
            "gemini-1.0-pro" => Ok(Model::Gemini1_0Pro),
            "gemini-1.0-pro-latest" => Ok(Model::Gemini1_0ProLatest),
            "gemini-1.0-pro-vision-latest" => Ok(Model::Gemini1_0ProVisionLatest),
            "gemini-1.5-pro" => Ok(Model::Gemini1_5Pro),
            "gemini-1.5-flash" => Ok(Model::Gemini1_5Flash),
            _ => Err(AiError::InvalidModel),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub name: String,
    pub base_model_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: u64,
    pub output_token_limit: u64,
    pub supported_generation_methods: Vec<GenerationMethod>,
    pub temperature: f64,
    pub top_p: f64,
    pub top_k: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GenerationMethod {
    GenerateContent,
    CountTokens,
    CreateCachedContent,
    CreateTunedModel,
    EmbedContent,
}

impl Display for GenerationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GenerationMethod::GenerateContent => write!(f, "generateContent"),
            GenerationMethod::CountTokens => write!(f, "countTokens"),
            GenerationMethod::CreateCachedContent => write!(f, "createCachedContent"),
            GenerationMethod::CreateTunedModel => write!(f, "createTunedModel"),
            GenerationMethod::EmbedContent => write!(f, "embedContent"),
        }
    }
}
