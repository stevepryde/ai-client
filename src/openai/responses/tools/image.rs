use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpenAIImageModel {
    #[serde(rename = "gpt-image-2")]
    #[default]
    GptImage2,
    #[serde(rename = "gpt-image-1-mini")]
    GptImage1Mini,
    #[serde(rename = "gpt-image-1")]
    GptImage1,
    #[serde(rename = "gpt-image-1.5")]
    GptImage1_5,
}

#[cfg(test)]
mod tests {
    use super::OpenAIImageModel;

    #[test]
    fn current_image_model_decodes() {
        assert_eq!(
            serde_json::from_str::<OpenAIImageModel>(r#""gpt-image-2""#).unwrap(),
            OpenAIImageModel::GptImage2
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpenAIImageSize {
    #[serde(rename = "1024x1024")]
    Square1024,
    #[serde(rename = "1536x1024")]
    Landscape,
    #[serde(rename = "1024x1536")]
    Portrait,
    #[serde(rename = "auto")]
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageQuality {
    Low,
    Medium,
    High,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageBackground {
    Transparent,
    Opaque,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageFormat {
    #[default]
    Png,
    Webp,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageAction {
    #[default]
    Auto,
    Generate,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIImageInputFidelity {
    #[default]
    High,
    Low,
}
