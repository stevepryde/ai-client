use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

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
pub enum OpenAIModel {
    /// https://platform.openai.com/docs/models/gpt-4o-mini
    #[default]
    Gpt4oMini,
    Gpt4o,
    Gpt4_1,
    Gpt4_1Mini,
    Gpt4_1Nano,
    Gpt5_1,
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
}

impl OpenAIModel {
    pub(crate) fn allow_temperature(&self) -> bool {
        match self {
            OpenAIModel::Gpt4oMini => true,
            OpenAIModel::Gpt4o => true,
            OpenAIModel::Gpt4_1 => true,
            OpenAIModel::Gpt4_1Mini => true,
            OpenAIModel::Gpt4_1Nano => true,
            OpenAIModel::Gpt5_1 => false,
            OpenAIModel::Gpt5 => false,
            OpenAIModel::Gpt5Mini => false,
            OpenAIModel::Gpt5Nano => false,
        }
    }

    pub(crate) fn supports_reasoning(&self) -> bool {
        match self {
            OpenAIModel::Gpt4oMini => false,
            OpenAIModel::Gpt4o => false,
            OpenAIModel::Gpt4_1 => true,
            OpenAIModel::Gpt4_1Mini => false,
            OpenAIModel::Gpt4_1Nano => false,
            OpenAIModel::Gpt5_1 => true,
            OpenAIModel::Gpt5 => true,
            OpenAIModel::Gpt5Mini => false,
            OpenAIModel::Gpt5Nano => false,
        }
    }

    pub(crate) fn supports_caching(&self) -> bool {
        match self {
            OpenAIModel::Gpt4oMini => false,
            OpenAIModel::Gpt4o => false,
            OpenAIModel::Gpt4_1 => true,
            OpenAIModel::Gpt4_1Mini => false,
            OpenAIModel::Gpt4_1Nano => false,
            OpenAIModel::Gpt5_1 => true,
            OpenAIModel::Gpt5 => true,
            OpenAIModel::Gpt5Mini => false,
            OpenAIModel::Gpt5Nano => false,
        }
    }
}

impl Display for OpenAIModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OpenAIModel::Gpt4oMini => "gpt-4o-mini",
            OpenAIModel::Gpt4o => "gpt-4o",
            OpenAIModel::Gpt4_1 => "gpt-4.1",
            OpenAIModel::Gpt4_1Mini => "gpt-4.1-mini",
            OpenAIModel::Gpt4_1Nano => "gpt-4.1-nano",
            OpenAIModel::Gpt5_1 => "gpt-5.1",
            OpenAIModel::Gpt5 => "gpt-5",
            OpenAIModel::Gpt5Mini => "gpt-5-mini",
            OpenAIModel::Gpt5Nano => "gpt-5-nano",
        };
        write!(f, "{name}")
    }
}

impl FromStr for OpenAIModel {
    type Err = AiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("models/").unwrap_or(s) {
            "gpt-4o-mini" => Ok(OpenAIModel::Gpt4oMini),
            "gpt-4o" => Ok(OpenAIModel::Gpt4o),
            "gpt-4-1-mini" => Ok(OpenAIModel::Gpt4_1Mini),
            "gpt-4-1-nano" => Ok(OpenAIModel::Gpt4_1Nano),
            "gpt-4.1" => Ok(OpenAIModel::Gpt4_1),
            "gpt-5.1" => Ok(OpenAIModel::Gpt5_1),
            "gpt-5" => Ok(OpenAIModel::Gpt5),
            "gpt-5-mini" => Ok(OpenAIModel::Gpt5Mini),
            "gpt-5-nano" => Ok(OpenAIModel::Gpt5Nano),
            _ => Err(AiError::InvalidModel),
        }
    }
}
