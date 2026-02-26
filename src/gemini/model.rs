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
pub enum GeminiModel {
    Gemini1_0Pro,
    Gemini1_0ProLatest,
    Gemini1_0ProVisionLatest,
    Gemini1_5Pro,
    Gemini1_5Flash,
    Gemini2_0Flash,
    #[default]
    Gemini2_0FlashLite,
    Gemini2_5Flash,
    Gemini2_5FlashLite,
    /// Gemini 2.5 Flash Image (Nano Banana) - fast image generation
    Gemini2_5FlashImage,
    /// Gemini 3 Pro Image (Nano Banana Pro) - high quality image generation
    Gemini3ProImage,
    /// Imagen 4 - dedicated image generation model
    Imagen4,
    /// Imagen 4 Fast - faster image generation
    Imagen4Fast,
}

impl Display for GeminiModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            GeminiModel::Gemini1_0Pro => "gemini-1.0-pro",
            GeminiModel::Gemini1_0ProLatest => "gemini-1.0-pro-latest",
            GeminiModel::Gemini1_0ProVisionLatest => "gemini-1.0-pro-vision-latest",
            GeminiModel::Gemini1_5Pro => "gemini-1.5-pro",
            GeminiModel::Gemini1_5Flash => "gemini-1.5-flash",
            GeminiModel::Gemini2_0Flash => "gemini-2.0-flash",
            GeminiModel::Gemini2_0FlashLite => "gemini-2.0-flash-lite",
            GeminiModel::Gemini2_5Flash => "gemini-2.5-flash",
            GeminiModel::Gemini2_5FlashLite => "gemini-2.5-flash-lite",
            GeminiModel::Gemini2_5FlashImage => "gemini-2.5-flash-image",
            GeminiModel::Gemini3ProImage => "gemini-3-pro-image-preview",
            GeminiModel::Imagen4 => "imagen-4.0-generate-001",
            GeminiModel::Imagen4Fast => "imagen-4.0-fast-generate-001",
        };
        write!(f, "{name}")
    }
}

impl FromStr for GeminiModel {
    type Err = AiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("models/").unwrap_or(s) {
            "gemini-1.0-pro" => Ok(GeminiModel::Gemini1_0Pro),
            "gemini-1.0-pro-latest" => Ok(GeminiModel::Gemini1_0ProLatest),
            "gemini-1.0-pro-vision-latest" => Ok(GeminiModel::Gemini1_0ProVisionLatest),
            "gemini-1.5-pro" => Ok(GeminiModel::Gemini1_5Pro),
            "gemini-1.5-flash" => Ok(GeminiModel::Gemini1_5Flash),
            "gemini-2.0-flash-001" | "gemini-2.0-flash" => Ok(GeminiModel::Gemini2_0Flash),
            "gemini-2.5-flash-preview-05-20" | "gemini-2.5-flash" => {
                Ok(GeminiModel::Gemini2_5Flash)
            }
            "gemini-2.5-flash-lite" => Ok(GeminiModel::Gemini2_5FlashLite),
            "gemini-2.5-flash-image" => Ok(GeminiModel::Gemini2_5FlashImage),
            "gemini-3-pro-image-preview" => Ok(GeminiModel::Gemini3ProImage),
            "imagen-4.0-generate-001" => Ok(GeminiModel::Imagen4),
            "imagen-4.0-fast-generate-001" => Ok(GeminiModel::Imagen4Fast),
            _ => Err(AiError::InvalidModel),
        }
    }
}

impl GeminiModel {
    /// Check if this model supports native image generation output.
    pub fn supports_image_generation(&self) -> bool {
        matches!(
            self,
            GeminiModel::Gemini2_5FlashImage
                | GeminiModel::Gemini3ProImage
                | GeminiModel::Imagen4
                | GeminiModel::Imagen4Fast
        )
    }

    /// Check if this model is a multimodal model (can take image input).
    pub fn supports_image_input(&self) -> bool {
        matches!(
            self,
            GeminiModel::Gemini1_0ProVisionLatest
                | GeminiModel::Gemini1_5Pro
                | GeminiModel::Gemini1_5Flash
                | GeminiModel::Gemini2_0Flash
                | GeminiModel::Gemini2_0FlashLite
                | GeminiModel::Gemini2_5Flash
                | GeminiModel::Gemini2_5FlashLite
                | GeminiModel::Gemini2_5FlashImage
                | GeminiModel::Gemini3ProImage
        )
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
    /// Controls the randomness of the output. Use higher values for more
    /// creative responses, and lower values for more deterministic responses.
    /// Values can range from 0.0 to 2.0.
    pub temperature: f64,
    /// Changes how the model selects tokens for output. Tokens are selected
    /// from the most to least probable until the sum of their probabilities
    /// equals the topP value. The default topP value is 0.95.
    pub top_p: f64,
    /// Changes how the model selects tokens for output. A topK of 1 means the
    /// selected token is the most probable among all the tokens in the model's
    /// vocabulary, while a topK of 3 means that the next token is selected from
    /// among the 3 most probable using the temperature. Tokens are further
    /// filtered based on topP with the final token selected using temperature
    /// sampling.
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
