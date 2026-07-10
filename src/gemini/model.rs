use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{error::ConfigErrorKind, prelude::AiError};

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
    Gemini3_5Flash,
    #[default]
    Gemini3_1FlashLite,
    Gemini3_1ProPreview,
    Gemini3FlashPreview,
    Gemini2_5Flash,
    Gemini2_5FlashLite,
    /// Gemini 3.1 Flash Image (Nano Banana 2).
    Gemini3_1FlashImage,
    /// Gemini 3.1 Flash Lite Image (Nano Banana 2 Lite).
    Gemini3_1FlashLiteImage,
    /// Gemini 3 Pro Image (Nano Banana Pro).
    Gemini3ProImage,
    /// Gemini 2.5 Flash Image (Nano Banana).
    Gemini2_5FlashImage,
}

impl Display for GeminiModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            GeminiModel::Gemini3_5Flash => "gemini-3.5-flash",
            GeminiModel::Gemini3_1FlashLite => "gemini-3.1-flash-lite",
            GeminiModel::Gemini3_1ProPreview => "gemini-3.1-pro-preview",
            GeminiModel::Gemini3FlashPreview => "gemini-3-flash-preview",
            GeminiModel::Gemini2_5Flash => "gemini-2.5-flash",
            GeminiModel::Gemini2_5FlashLite => "gemini-2.5-flash-lite",
            GeminiModel::Gemini3_1FlashImage => "gemini-3.1-flash-image",
            GeminiModel::Gemini3_1FlashLiteImage => "gemini-3.1-flash-lite-image",
            GeminiModel::Gemini3ProImage => "gemini-3-pro-image",
            GeminiModel::Gemini2_5FlashImage => "gemini-2.5-flash-image",
        };
        write!(f, "{name}")
    }
}

impl FromStr for GeminiModel {
    type Err = AiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("models/").unwrap_or(s) {
            "gemini-3.5-flash" => Ok(GeminiModel::Gemini3_5Flash),
            "gemini-3.1-flash-lite" => Ok(GeminiModel::Gemini3_1FlashLite),
            "gemini-3.1-pro-preview" => Ok(GeminiModel::Gemini3_1ProPreview),
            "gemini-3-flash-preview" => Ok(GeminiModel::Gemini3FlashPreview),
            "gemini-2.5-flash-preview-05-20" | "gemini-2.5-flash" => {
                Ok(GeminiModel::Gemini2_5Flash)
            }
            "gemini-2.5-flash-lite" => Ok(GeminiModel::Gemini2_5FlashLite),
            "gemini-3.1-flash-image" => Ok(GeminiModel::Gemini3_1FlashImage),
            "gemini-3.1-flash-lite-image" => Ok(GeminiModel::Gemini3_1FlashLiteImage),
            "gemini-3-pro-image" => Ok(GeminiModel::Gemini3ProImage),
            "gemini-2.5-flash-image" => Ok(GeminiModel::Gemini2_5FlashImage),
            _ => Err(AiError::Config {
                kind: ConfigErrorKind::InvalidModel,
                message: "unknown Gemini model".to_string(),
            }),
        }
    }
}

impl GeminiModel {
    pub const ALL: &'static [Self] = &[
        Self::Gemini3_5Flash,
        Self::Gemini3_1FlashLite,
        Self::Gemini3_1ProPreview,
        Self::Gemini3FlashPreview,
        Self::Gemini2_5Flash,
        Self::Gemini2_5FlashLite,
        Self::Gemini3_1FlashImage,
        Self::Gemini3_1FlashLiteImage,
        Self::Gemini3ProImage,
        Self::Gemini2_5FlashImage,
    ];

    pub const TEXT_GENERATION: &'static [Self] = &[
        Self::Gemini3_5Flash,
        Self::Gemini3_1FlashLite,
        Self::Gemini3_1ProPreview,
        Self::Gemini3FlashPreview,
        Self::Gemini2_5Flash,
        Self::Gemini2_5FlashLite,
    ];

    pub const IMAGE_GENERATION: &'static [Self] = &[
        Self::Gemini3_1FlashImage,
        Self::Gemini3_1FlashLiteImage,
        Self::Gemini3ProImage,
        Self::Gemini2_5FlashImage,
    ];

    /// Check if this model supports native image generation output.
    pub fn supports_image_generation(&self) -> bool {
        matches!(
            self,
            GeminiModel::Gemini3_1FlashImage
                | GeminiModel::Gemini3_1FlashLiteImage
                | GeminiModel::Gemini3ProImage
                | GeminiModel::Gemini2_5FlashImage
        )
    }

    /// Check if this model is a multimodal model (can take image input).
    pub fn supports_image_input(&self) -> bool {
        matches!(
            self,
            GeminiModel::Gemini3_5Flash
                | GeminiModel::Gemini3_1FlashLite
                | GeminiModel::Gemini3_1ProPreview
                | GeminiModel::Gemini3FlashPreview
                | GeminiModel::Gemini2_5Flash
                | GeminiModel::Gemini2_5FlashLite
                | GeminiModel::Gemini3_1FlashImage
                | GeminiModel::Gemini3_1FlashLiteImage
                | GeminiModel::Gemini2_5FlashImage
                | GeminiModel::Gemini3ProImage
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub name: String,
    #[serde(default)]
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
    #[serde(default)]
    pub temperature: f64,
    /// Changes how the model selects tokens for output. Tokens are selected
    /// from the most to least probable until the sum of their probabilities
    /// equals the topP value. The default topP value is 0.95.
    #[serde(default)]
    pub top_p: f64,
    /// Changes how the model selects tokens for output. A topK of 1 means the
    /// selected token is the most probable among all the tokens in the model's
    /// vocabulary, while a topK of 3 means that the next token is selected from
    /// among the 3 most probable using the temperature. Tokens are further
    /// filtered based on topP with the final token selected using temperature
    /// sampling.
    #[serde(default)]
    pub top_k: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GenerationMethod {
    GenerateContent,
    BatchGenerateContent,
    CountTokens,
    CreateCachedContent,
    CreateTunedModel,
    EmbedContent,
    BatchEmbedContents,
    Predict,
    PredictLongRunning,
    #[serde(other)]
    Unknown,
}

impl Display for GenerationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GenerationMethod::GenerateContent => write!(f, "generateContent"),
            GenerationMethod::BatchGenerateContent => write!(f, "batchGenerateContent"),
            GenerationMethod::CountTokens => write!(f, "countTokens"),
            GenerationMethod::CreateCachedContent => write!(f, "createCachedContent"),
            GenerationMethod::CreateTunedModel => write!(f, "createTunedModel"),
            GenerationMethod::EmbedContent => write!(f, "embedContent"),
            GenerationMethod::BatchEmbedContents => write!(f, "batchEmbedContents"),
            GenerationMethod::Predict => write!(f, "predict"),
            GenerationMethod::PredictLongRunning => write!(f, "predictLongRunning"),
            GenerationMethod::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GeminiModel;
    use std::str::FromStr;

    #[test]
    fn every_gemini_model_round_trips_its_wire_id() {
        for model in GeminiModel::ALL {
            let name = model.to_string();
            assert_eq!(GeminiModel::from_str(&name).unwrap(), *model);
            assert_eq!(
                GeminiModel::from_str(&format!("models/{name}")).unwrap(),
                *model
            );
        }
    }
}
