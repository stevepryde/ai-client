use serde::Serialize;

use crate::openai::responses::OpenAIResponseInputItem;

use super::{ConversationError, ConversationMetadata, Nullable};

const MAX_CREATE_ITEMS: usize = 20;

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Nullable<ConversationMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Nullable<Vec<OpenAIResponseInputItem>>>,
}

impl CreateConversationRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn metadata(mut self, metadata: ConversationMetadata) -> Self {
        self.metadata = Some(Nullable::Value(metadata));
        self
    }

    pub fn null_metadata(mut self) -> Self {
        self.metadata = Some(Nullable::Null);
        self
    }

    pub fn items(mut self, items: Vec<OpenAIResponseInputItem>) -> Result<Self, ConversationError> {
        validate_item_count(items.len())?;
        self.items = Some(Nullable::Value(items));
        Ok(self)
    }

    pub fn null_items(mut self) -> Self {
        self.items = Some(Nullable::Null);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateConversationRequest {
    metadata: Nullable<ConversationMetadata>,
}

impl UpdateConversationRequest {
    pub fn new(metadata: ConversationMetadata) -> Self {
        Self {
            metadata: Nullable::Value(metadata),
        }
    }

    pub fn null_metadata() -> Self {
        Self {
            metadata: Nullable::Null,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateConversationItemsRequest {
    items: Vec<OpenAIResponseInputItem>,
}

impl CreateConversationItemsRequest {
    pub fn new(items: Vec<OpenAIResponseInputItem>) -> Result<Self, ConversationError> {
        validate_item_count(items.len())?;
        Ok(Self { items })
    }

    pub fn items(&self) -> &[OpenAIResponseInputItem] {
        &self.items
    }
}

fn validate_item_count(count: usize) -> Result<(), ConversationError> {
    if count > MAX_CREATE_ITEMS {
        return Err(ConversationError::TooManyItems);
    }
    Ok(())
}
