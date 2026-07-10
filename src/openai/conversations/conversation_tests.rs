use std::collections::BTreeMap;

use super::*;
use crate::openai::responses::{
    ListOrder, OpenAIResponseInputItem, OpenAIResponseItem, ResponseInclude, ResponseItemId,
    ResponseOperationError,
};
fn input_item(text: &str) -> OpenAIResponseInputItem {
    serde_json::from_value(serde_json::json!({
        "type": "message",
        "role": "user",
        "content": text
    }))
    .unwrap()
}

#[test]
fn ids_and_request_bounds_are_validated() {
    for invalid in ["", " ", "conv 123", "conv\n123", "conv\u{0}123"] {
        assert!(ConversationId::new(invalid).is_err());
    }
    assert_eq!(
        CreateConversationItemsRequest::new(vec![input_item("x"); 21]).unwrap_err(),
        ConversationError::TooManyItems
    );
    assert!(CreateConversationItemsRequest::new(Vec::new()).is_ok());
    assert!(CreateConversationItemsRequest::new(vec![input_item("x"); 20]).is_ok());
    assert!(
        serde_json::from_value::<ConversationResource>(serde_json::json!({
            "id":"conv invalid",
            "object":"conversation",
            "metadata":{},
            "created_at":1
        }))
        .is_err()
    );
    let id = ConversationId::new("conv_123").unwrap();
    assert_eq!(
        serde_json::to_value(&id).unwrap(),
        serde_json::json!("conv_123")
    );
    assert_eq!(
        serde_json::from_value::<ConversationId>(serde_json::json!("conv_123")).unwrap(),
        id
    );
    assert!(
        serde_json::from_value::<ConversationItemList>(serde_json::json!({
            "object":"list",
            "data":[],
            "has_more":false,
            "first_id":"item invalid",
            "last_id":"item_ok"
        }))
        .is_err()
    );
}

#[test]
fn metadata_constraints_are_checked_on_construction_and_decode() {
    let maximum = (0..16)
        .map(|index| (format!("key-{index}"), "v".repeat(512)))
        .collect();
    assert!(ConversationMetadata::new(maximum).is_ok());
    let too_many = (0..17)
        .map(|index| (format!("key-{index}"), "value".into()))
        .collect();
    assert_eq!(
        ConversationMetadata::new(too_many).unwrap_err(),
        ConversationError::TooManyMetadataEntries
    );
    assert_eq!(
        ConversationMetadata::new(BTreeMap::from([("k".repeat(65), "value".into())])).unwrap_err(),
        ConversationError::MetadataKeyTooLong
    );
    assert!(
        serde_json::from_value::<ConversationMetadata>(serde_json::json!({
            "x": "v".repeat(513)
        }))
        .is_err()
    );
}

#[test]
fn create_request_preserves_omitted_and_explicit_null() {
    assert_eq!(
        serde_json::to_value(CreateConversationRequest::new()).unwrap(),
        serde_json::json!({})
    );
    assert_eq!(
        serde_json::to_value(
            CreateConversationRequest::new()
                .null_metadata()
                .null_items()
        )
        .unwrap(),
        serde_json::json!({"metadata": null, "items": null})
    );
    assert_eq!(
        serde_json::to_value(UpdateConversationRequest::null_metadata()).unwrap(),
        serde_json::json!({"metadata": null})
    );

    let official_example = CreateConversationRequest::new()
        .items(vec![input_item("Hello!")])
        .unwrap();
    assert_eq!(
        serde_json::to_value(official_example).unwrap(),
        serde_json::json!({
            "items": [{"type":"message","role":"user","content":"Hello!"}]
        })
    );
}

#[test]
fn conversation_message_supports_all_pinned_roles_and_content_tags() {
    let content = serde_json::json!([
        {"type":"input_text","text":"input"},
        {"type":"output_text","text":"output","annotations":[],"logprobs":[]},
        {"type":"text","text":"plain"},
        {"type":"summary_text","text":"summary"},
        {"type":"reasoning_text","text":"reasoning"},
        {"type":"refusal","refusal":"no"},
        {"type":"input_image","image_url":null,"file_id":null,"detail":"original"},
        {"type":"computer_screenshot","image_url":null,"file_id":null,"detail":"original"},
        {"type":"input_file","file_id":"file_123"}
    ]);
    for role in [
        "unknown",
        "user",
        "assistant",
        "system",
        "critic",
        "discriminator",
        "developer",
        "tool",
    ] {
        let value = serde_json::json!({
            "type":"message",
            "id":"msg_123",
            "status":"completed",
            "role":role,
            "content":content.clone(),
            "phase":"commentary"
        });
        let item: ConversationItem = serde_json::from_value(value.clone()).unwrap();
        assert!(matches!(item, ConversationItem::Message(_)));
        assert_eq!(serde_json::to_value(item).unwrap(), value);
    }
}

#[test]
fn conversation_content_unknowns_round_trip_but_malformed_known_tags_fail() {
    let unknown = serde_json::json!({"type":"future_content","secret":"retained"});
    let content: ConversationMessageContent = serde_json::from_value(unknown.clone()).unwrap();
    assert!(matches!(content, ConversationMessageContent::Unknown(_)));
    assert_eq!(serde_json::to_value(content).unwrap(), unknown);

    assert!(
        serde_json::from_value::<ConversationMessageContent>(serde_json::json!({
            "type":"computer_screenshot",
            "image_url":null,
            "detail":"auto"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<ConversationMessageContent>(serde_json::json!({
            "type":"input_image",
            "image_url":"https://example.test/image.png",
            "file_id":null,
            "detail":"auto",
            "prompt_cache_breakpoint":{"mode":"automatic"}
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<ConversationItem>(serde_json::json!({
            "type":"message",
            "id":"msg invalid",
            "status":"completed",
            "role":"user",
            "content":[]
        }))
        .is_err()
    );

    let unknown_item = serde_json::json!({"type":"future_item","secret":"retained"});
    let item: ConversationItem = serde_json::from_value(unknown_item.clone()).unwrap();
    assert!(matches!(
        item,
        ConversationItem::Response(ref response)
            if matches!(response.as_ref(), OpenAIResponseItem::Unknown(_))
    ));
    assert_eq!(serde_json::to_value(item).unwrap(), unknown_item);
}

#[test]
fn list_query_defaults_and_repeated_includes_are_exact() {
    assert_eq!(ListConversationItemsOptions::default().page_limit(), 20);
    assert!(ListConversationItemsOptions::default().query().is_empty());
    assert_eq!(
        ListConversationItemsOptions::new().limit(0).unwrap_err(),
        ResponseOperationError::InvalidPageLimit
    );
    let query = ListConversationItemsOptions::new()
        .limit(100)
        .unwrap()
        .order(ListOrder::Asc)
        .after(ResponseItemId::new("item_123").unwrap())
        .include(ResponseInclude::FileSearchCallResults)
        .include(ResponseInclude::ReasoningEncryptedContent)
        .query();
    assert_eq!(
        query,
        vec![
            ("limit".into(), "100".into()),
            ("order".into(), "asc".into()),
            ("after".into(), "item_123".into()),
            ("include".into(), "file_search_call.results".into()),
            ("include".into(), "reasoning.encrypted_content".into()),
        ]
    );
}
