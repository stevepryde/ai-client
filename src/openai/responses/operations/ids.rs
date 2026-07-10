use std::fmt;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResponseOperationError {
    #[error("response ID must be non-empty and contain no whitespace or control characters")]
    InvalidResponseId,
    #[error("response item ID must be non-empty and contain no whitespace or control characters")]
    InvalidResponseItemId,
    #[error("page limit must be between 1 and 100")]
    InvalidPageLimit,
}

macro_rules! opaque_id {
    ($name:ident, $error:expr) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ResponseOperationError> {
                let value = value.into();
                if value.is_empty()
                    || value
                        .chars()
                        .any(|character| character.is_whitespace() || character.is_control())
                {
                    return Err($error);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl TryFrom<String> for $name {
            type Error = ResponseOperationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ResponseOperationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

opaque_id!(ResponseId, ResponseOperationError::InvalidResponseId);
opaque_id!(
    ResponseItemId,
    ResponseOperationError::InvalidResponseItemId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_reject_ambiguous_values_but_remain_opaque() {
        for invalid in ["", " ", "resp 123", "resp\n123", "resp\u{0000}123"] {
            assert!(ResponseId::new(invalid).is_err(), "accepted {invalid:?}");
        }
        let id = ResponseId::new("resp/a?b#c%").unwrap();
        assert_eq!(id.as_str(), "resp/a?b#c%");
        assert!(serde_json::from_str::<ResponseId>(r#""resp 123""#).is_err());
        assert_eq!(serde_json::to_value(id).unwrap(), "resp/a?b#c%");
    }
}
