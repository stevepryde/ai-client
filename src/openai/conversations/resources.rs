use serde::Deserialize;

use crate::openai::responses::ResponseItemId;

use super::{deserialize_response_item_id, ConversationId, ConversationItem, ConversationMetadata};

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationResource {
    pub id: ConversationId,
    pub object: String,
    pub metadata: ConversationMetadata,
    pub created_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationItemList {
    pub object: String,
    pub data: Vec<ConversationItem>,
    pub has_more: bool,
    #[serde(deserialize_with = "deserialize_response_item_id")]
    pub first_id: ResponseItemId,
    #[serde(deserialize_with = "deserialize_response_item_id")]
    pub last_id: ResponseItemId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeletedConversationResource {
    pub object: String,
    pub deleted: bool,
    pub id: ConversationId,
}
