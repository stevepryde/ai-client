use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

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
    Gpt5Pro,
    Gpt5_2,
    Gpt5_2Pro,
    Gpt5_3Codex,
    Gpt5_4,
    Gpt5_4Pro,
    Gpt5_4Mini,
    Gpt5_4Nano,
    Gpt5_5,
    Gpt5_5Pro,
    Gpt5_6,
    Gpt5_6Sol,
    Gpt5_6Terra,
    Gpt5_6Luna,
}

#[cfg(feature = "chat-completions")]
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
            OpenAIModel::Gpt5Pro => false,
            OpenAIModel::Gpt5_2 => false,
            OpenAIModel::Gpt5_2Pro => false,
            OpenAIModel::Gpt5_3Codex => false,
            OpenAIModel::Gpt5_4 => false,
            OpenAIModel::Gpt5_4Pro => false,
            OpenAIModel::Gpt5_4Mini => false,
            OpenAIModel::Gpt5_4Nano => false,
            OpenAIModel::Gpt5_5 => false,
            OpenAIModel::Gpt5_5Pro => false,
            OpenAIModel::Gpt5_6
            | OpenAIModel::Gpt5_6Sol
            | OpenAIModel::Gpt5_6Terra
            | OpenAIModel::Gpt5_6Luna => false,
        }
    }

    pub(crate) fn supports_reasoning(&self) -> bool {
        match self {
            OpenAIModel::Gpt4oMini => false,
            OpenAIModel::Gpt4o => false,
            OpenAIModel::Gpt4_1 => false,
            OpenAIModel::Gpt4_1Mini => false,
            OpenAIModel::Gpt4_1Nano => false,
            OpenAIModel::Gpt5_1 => true,
            OpenAIModel::Gpt5 => true,
            OpenAIModel::Gpt5Mini => true,
            OpenAIModel::Gpt5Nano => true,
            OpenAIModel::Gpt5Pro => true,
            OpenAIModel::Gpt5_2 => true,
            OpenAIModel::Gpt5_2Pro => true,
            OpenAIModel::Gpt5_3Codex => true,
            OpenAIModel::Gpt5_4 => true,
            OpenAIModel::Gpt5_4Pro => true,
            OpenAIModel::Gpt5_4Mini => true,
            OpenAIModel::Gpt5_4Nano => true,
            OpenAIModel::Gpt5_5 => true,
            OpenAIModel::Gpt5_5Pro => true,
            OpenAIModel::Gpt5_6
            | OpenAIModel::Gpt5_6Sol
            | OpenAIModel::Gpt5_6Terra
            | OpenAIModel::Gpt5_6Luna => true,
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
            OpenAIModel::Gpt5Pro => false,
            OpenAIModel::Gpt5_2 => true,
            OpenAIModel::Gpt5_2Pro => false,
            OpenAIModel::Gpt5_3Codex => true,
            OpenAIModel::Gpt5_4 => true,
            OpenAIModel::Gpt5_4Pro => false,
            OpenAIModel::Gpt5_4Mini => true,
            OpenAIModel::Gpt5_4Nano => true,
            OpenAIModel::Gpt5_5 => true,
            OpenAIModel::Gpt5_5Pro => false,
            OpenAIModel::Gpt5_6
            | OpenAIModel::Gpt5_6Sol
            | OpenAIModel::Gpt5_6Terra
            | OpenAIModel::Gpt5_6Luna => false,
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
            OpenAIModel::Gpt5Pro => "gpt-5-pro",
            OpenAIModel::Gpt5_2 => "gpt-5.2",
            OpenAIModel::Gpt5_2Pro => "gpt-5.2-pro",
            OpenAIModel::Gpt5_3Codex => "gpt-5.3-codex",
            OpenAIModel::Gpt5_4 => "gpt-5.4",
            OpenAIModel::Gpt5_4Pro => "gpt-5.4-pro",
            OpenAIModel::Gpt5_4Mini => "gpt-5.4-mini",
            OpenAIModel::Gpt5_4Nano => "gpt-5.4-nano",
            OpenAIModel::Gpt5_5 => "gpt-5.5",
            OpenAIModel::Gpt5_5Pro => "gpt-5.5-pro",
            OpenAIModel::Gpt5_6 => "gpt-5.6",
            OpenAIModel::Gpt5_6Sol => "gpt-5.6-sol",
            OpenAIModel::Gpt5_6Terra => "gpt-5.6-terra",
            OpenAIModel::Gpt5_6Luna => "gpt-5.6-luna",
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
            "gpt-4.1-mini" => Ok(OpenAIModel::Gpt4_1Mini),
            "gpt-4.1-nano" => Ok(OpenAIModel::Gpt4_1Nano),
            "gpt-4.1" => Ok(OpenAIModel::Gpt4_1),
            "gpt-5.1" => Ok(OpenAIModel::Gpt5_1),
            "gpt-5" => Ok(OpenAIModel::Gpt5),
            "gpt-5-mini" => Ok(OpenAIModel::Gpt5Mini),
            "gpt-5-nano" => Ok(OpenAIModel::Gpt5Nano),
            "gpt-5-pro" => Ok(OpenAIModel::Gpt5Pro),
            "gpt-5.2" => Ok(OpenAIModel::Gpt5_2),
            "gpt-5.2-pro" => Ok(OpenAIModel::Gpt5_2Pro),
            "gpt-5.3-codex" => Ok(OpenAIModel::Gpt5_3Codex),
            "gpt-5.4" => Ok(OpenAIModel::Gpt5_4),
            "gpt-5.4-pro" => Ok(OpenAIModel::Gpt5_4Pro),
            "gpt-5.4-mini" => Ok(OpenAIModel::Gpt5_4Mini),
            "gpt-5.4-nano" => Ok(OpenAIModel::Gpt5_4Nano),
            "gpt-5.5" => Ok(OpenAIModel::Gpt5_5),
            "gpt-5.5-pro" => Ok(OpenAIModel::Gpt5_5Pro),
            "gpt-5.6" => Ok(OpenAIModel::Gpt5_6),
            "gpt-5.6-sol" => Ok(OpenAIModel::Gpt5_6Sol),
            "gpt-5.6-terra" => Ok(OpenAIModel::Gpt5_6Terra),
            "gpt-5.6-luna" => Ok(OpenAIModel::Gpt5_6Luna),
            _ => Err(AiError::Config {
                kind: ConfigErrorKind::InvalidModel,
                message: "unknown OpenAI model".to_string(),
            }),
        }
    }
}

impl OpenAIModel {
    pub const ALL: &'static [Self] = &[
        Self::Gpt4oMini,
        Self::Gpt4o,
        Self::Gpt4_1,
        Self::Gpt4_1Mini,
        Self::Gpt4_1Nano,
        Self::Gpt5_1,
        Self::Gpt5,
        Self::Gpt5Mini,
        Self::Gpt5Nano,
        Self::Gpt5Pro,
        Self::Gpt5_2,
        Self::Gpt5_2Pro,
        Self::Gpt5_3Codex,
        Self::Gpt5_4,
        Self::Gpt5_4Pro,
        Self::Gpt5_4Mini,
        Self::Gpt5_4Nano,
        Self::Gpt5_5,
        Self::Gpt5_5Pro,
        Self::Gpt5_6,
        Self::Gpt5_6Sol,
        Self::Gpt5_6Terra,
        Self::Gpt5_6Luna,
    ];
}

#[cfg(test)]
mod tests {
    use super::OpenAIModel;
    use std::str::FromStr;

    #[test]
    fn round_trips_recent_gpt_5_models() {
        let models = [
            (OpenAIModel::Gpt5Pro, "gpt-5-pro"),
            (OpenAIModel::Gpt5_2, "gpt-5.2"),
            (OpenAIModel::Gpt5_2Pro, "gpt-5.2-pro"),
            (OpenAIModel::Gpt5_3Codex, "gpt-5.3-codex"),
            (OpenAIModel::Gpt5_4, "gpt-5.4"),
            (OpenAIModel::Gpt5_4Pro, "gpt-5.4-pro"),
            (OpenAIModel::Gpt5_4Mini, "gpt-5.4-mini"),
            (OpenAIModel::Gpt5_4Nano, "gpt-5.4-nano"),
            (OpenAIModel::Gpt5_5, "gpt-5.5"),
            (OpenAIModel::Gpt5_5Pro, "gpt-5.5-pro"),
            (OpenAIModel::Gpt5_6, "gpt-5.6"),
            (OpenAIModel::Gpt5_6Sol, "gpt-5.6-sol"),
            (OpenAIModel::Gpt5_6Terra, "gpt-5.6-terra"),
            (OpenAIModel::Gpt5_6Luna, "gpt-5.6-luna"),
        ];

        for (model, name) in models {
            assert_eq!(model.to_string(), name);
            assert!(matches!(OpenAIModel::from_str(name), Ok(parsed) if parsed == model));
            assert!(
                matches!(OpenAIModel::from_str(&format!("models/{name}")), Ok(parsed) if parsed == model)
            );
        }
    }

    #[test]
    fn every_openai_model_round_trips_its_wire_id() {
        for model in OpenAIModel::ALL {
            let name = model.to_string();
            assert_eq!(OpenAIModel::from_str(&name).unwrap(), *model);
            assert_eq!(
                OpenAIModel::from_str(&format!("models/{name}")).unwrap(),
                *model
            );
        }
    }
}
