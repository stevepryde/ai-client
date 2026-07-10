use std::{collections::BTreeMap, fmt};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::openai::responses::{ResponseItemId, ResponseOperationError};

const MAX_METADATA_ENTRIES: usize = 16;
const MAX_METADATA_KEY_LENGTH: usize = 64;
const MAX_METADATA_VALUE_LENGTH: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConversationError {
    #[error("conversation ID must be non-empty and contain no whitespace or control characters")]
    InvalidConversationId,
    #[error("conversation metadata may contain at most 16 entries")]
    TooManyMetadataEntries,
    #[error("conversation metadata keys may contain at most 64 characters")]
    MetadataKeyTooLong,
    #[error("conversation metadata values may contain at most 512 characters")]
    MetadataValueTooLong,
    #[error("at most 20 conversation items may be created at once")]
    TooManyItems,
    #[error(transparent)]
    ResponseOperation(#[from] ResponseOperationError),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConversationId(String);

impl ConversationId {
    pub fn new(value: impl Into<String>) -> Result<Self, ConversationError> {
        let value = value.into();
        if value.is_empty()
            || value
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(ConversationError::InvalidConversationId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ConversationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ConversationId")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for ConversationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for ConversationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ConversationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl TryFrom<String> for ConversationId {
    type Error = ConversationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for ConversationId {
    type Error = ConversationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ConversationMetadata(BTreeMap<String, String>);

impl ConversationMetadata {
    pub fn new(values: BTreeMap<String, String>) -> Result<Self, ConversationError> {
        validate_metadata(&values)?;
        Ok(Self(values))
    }

    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn as_map(&self) -> &BTreeMap<String, String> {
        &self.0
    }

    pub fn into_map(self) -> BTreeMap<String, String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for ConversationMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = BTreeMap::<String, String>::deserialize(deserializer)?;
        Self::new(values).map_err(D::Error::custom)
    }
}

fn validate_metadata(values: &BTreeMap<String, String>) -> Result<(), ConversationError> {
    if values.len() > MAX_METADATA_ENTRIES {
        return Err(ConversationError::TooManyMetadataEntries);
    }
    if values
        .keys()
        .any(|key| key.chars().count() > MAX_METADATA_KEY_LENGTH)
    {
        return Err(ConversationError::MetadataKeyTooLong);
    }
    if values
        .values()
        .any(|value| value.chars().count() > MAX_METADATA_VALUE_LENGTH)
    {
        return Err(ConversationError::MetadataValueTooLong);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Nullable<T> {
    Value(T),
    Null,
}

pub(crate) fn deserialize_response_item_id<'de, D>(
    deserializer: D,
) -> Result<ResponseItemId, D::Error>
where
    D: Deserializer<'de>,
{
    ResponseItemId::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
}

pub(crate) fn serialize_response_item_id<S>(
    item_id: &ResponseItemId,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(item_id.as_str())
}
