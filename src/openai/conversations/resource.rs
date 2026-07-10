use crate::{
    error::{AiResponse, AiResult},
    openai::{client::decode_openai_error, OpenAIClient},
};

use super::{
    ConversationId, ConversationItem, ConversationItemIncludeOptions, ConversationItemList,
    ConversationResource, CreateConversationItemsRequest, CreateConversationRequest,
    DeletedConversationResource, ListConversationItemsOptions, UpdateConversationRequest,
};
use crate::openai::responses::ResponseItemId;

#[derive(Clone, Copy)]
pub struct ConversationsResource<'a> {
    client: &'a OpenAIClient,
}

impl<'a> ConversationsResource<'a> {
    pub(crate) fn new(client: &'a OpenAIClient) -> Self {
        Self { client }
    }

    pub async fn create(
        &self,
        request: &CreateConversationRequest,
    ) -> AiResult<AiResponse<ConversationResource>> {
        self.client
            .transport()
            .post_json(
                "conversations.create",
                "conversations",
                request,
                decode_openai_error,
            )
            .await
    }

    pub async fn retrieve(
        &self,
        conversation_id: &ConversationId,
    ) -> AiResult<AiResponse<ConversationResource>> {
        self.client
            .transport()
            .get_json_segments(
                "conversations.retrieve",
                &["conversations", conversation_id.as_str()],
                &[],
                decode_openai_error,
            )
            .await
    }

    pub async fn update(
        &self,
        conversation_id: &ConversationId,
        request: &UpdateConversationRequest,
    ) -> AiResult<AiResponse<ConversationResource>> {
        self.client
            .transport()
            .post_json_segments(
                "conversations.update",
                &["conversations", conversation_id.as_str()],
                request,
                decode_openai_error,
            )
            .await
    }

    pub async fn delete(
        &self,
        conversation_id: &ConversationId,
    ) -> AiResult<AiResponse<DeletedConversationResource>> {
        self.client
            .transport()
            .delete_json_segments(
                "conversations.delete",
                &["conversations", conversation_id.as_str()],
                decode_openai_error,
            )
            .await
    }

    pub fn items(&self) -> ConversationItemsResource<'a> {
        ConversationItemsResource::new(self.client)
    }
}

#[derive(Clone, Copy)]
pub struct ConversationItemsResource<'a> {
    client: &'a OpenAIClient,
}

impl<'a> ConversationItemsResource<'a> {
    pub(crate) fn new(client: &'a OpenAIClient) -> Self {
        Self { client }
    }

    pub async fn create(
        &self,
        conversation_id: &ConversationId,
        request: &CreateConversationItemsRequest,
        options: &ConversationItemIncludeOptions,
    ) -> AiResult<AiResponse<ConversationItemList>> {
        self.client
            .transport()
            .post_json_segments_with_query(
                "conversations.items.create",
                &["conversations", conversation_id.as_str(), "items"],
                &options.query(),
                request,
                decode_openai_error,
            )
            .await
    }

    pub async fn list(
        &self,
        conversation_id: &ConversationId,
        options: &ListConversationItemsOptions,
    ) -> AiResult<AiResponse<ConversationItemList>> {
        self.client
            .transport()
            .get_json_segments(
                "conversations.items.list",
                &["conversations", conversation_id.as_str(), "items"],
                &options.query(),
                decode_openai_error,
            )
            .await
    }

    pub async fn retrieve(
        &self,
        conversation_id: &ConversationId,
        item_id: &ResponseItemId,
        options: &ConversationItemIncludeOptions,
    ) -> AiResult<AiResponse<ConversationItem>> {
        self.client
            .transport()
            .get_json_segments(
                "conversations.items.retrieve",
                &[
                    "conversations",
                    conversation_id.as_str(),
                    "items",
                    item_id.as_str(),
                ],
                &options.query(),
                decode_openai_error,
            )
            .await
    }

    pub async fn delete(
        &self,
        conversation_id: &ConversationId,
        item_id: &ResponseItemId,
    ) -> AiResult<AiResponse<ConversationResource>> {
        self.client
            .transport()
            .delete_json_segments(
                "conversations.items.delete",
                &[
                    "conversations",
                    conversation_id.as_str(),
                    "items",
                    item_id.as_str(),
                ],
                decode_openai_error,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        core::test_support::{json_response, one_shot_server},
        openai::conversations::ConversationMetadata,
        openai::responses::{ListOrder, OpenAIResponseInputItem, ResponseInclude, ResponseItemId},
    };

    const CONVERSATION_BODY: &str = r#"{
        "id":"conv_123",
        "object":"conversation",
        "metadata":{"topic":"testing"},
        "created_at":1741900000
    }"#;
    const ITEM_LIST_BODY: &str = r#"{
        "object":"list",
        "data":[],
        "has_more":false,
        "first_id":"item_first",
        "last_id":"item_last"
    }"#;

    fn make_client(base_url: String) -> OpenAIClient {
        OpenAIClient::builder()
            .api_key("test-key".into())
            .base_url(base_url)
            .build()
            .unwrap()
    }

    fn json_ok(body: &str) -> String {
        json_response("200 OK", &[], body)
    }

    fn request_target(request: &str) -> &str {
        request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap()
    }

    fn request_method(request: &str) -> &str {
        request
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
    }

    fn request_json(request: &str) -> serde_json::Value {
        let (_, body) = request.split_once("\r\n\r\n").unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn metadata() -> ConversationMetadata {
        ConversationMetadata::new(BTreeMap::from([("topic".into(), "testing".into())])).unwrap()
    }

    fn input_item() -> OpenAIResponseInputItem {
        serde_json::from_value(serde_json::json!({
            "type":"message",
            "role":"user",
            "content":"hello"
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn conversation_operations_use_exact_verbs_paths_and_bodies() {
        let id = ConversationId::new("conv_123").unwrap();

        let (base_url, captured) = one_shot_server("v1", json_ok(CONVERSATION_BODY)).await;
        let request = CreateConversationRequest::new().metadata(metadata());
        let client = make_client(base_url);
        let response = ConversationsResource::new(&client)
            .create(&request)
            .await
            .unwrap();
        assert_eq!(response.data().id.as_str(), "conv_123");
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "POST");
        assert_eq!(request_target(&captured), "/v1/conversations");
        assert_eq!(
            request_json(&captured),
            serde_json::json!({"metadata":{"topic":"testing"}})
        );

        let encoded_id = ConversationId::new("conv/a?b#c%").unwrap();
        let (base_url, captured) = one_shot_server("v1", json_ok(CONVERSATION_BODY)).await;
        let client = make_client(base_url);
        ConversationsResource::new(&client)
            .retrieve(&encoded_id)
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "GET");
        assert_eq!(
            request_target(&captured),
            "/v1/conversations/conv%2Fa%3Fb%23c%25"
        );

        let (base_url, captured) = one_shot_server("v1", json_ok(CONVERSATION_BODY)).await;
        let client = make_client(base_url);
        ConversationsResource::new(&client)
            .update(&id, &UpdateConversationRequest::new(metadata()))
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "POST");
        assert_eq!(request_target(&captured), "/v1/conversations/conv_123");
        assert_eq!(
            request_json(&captured),
            serde_json::json!({"metadata":{"topic":"testing"}})
        );

        let deleted = r#"{"object":"conversation.deleted","deleted":true,"id":"conv_123"}"#;
        let (base_url, captured) = one_shot_server("v1", json_ok(deleted)).await;
        let client = make_client(base_url);
        let response = ConversationsResource::new(&client)
            .delete(&id)
            .await
            .unwrap();
        assert!(response.data().deleted);
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "DELETE");
        assert_eq!(request_target(&captured), "/v1/conversations/conv_123");
    }

    #[tokio::test]
    async fn item_operations_preserve_repeated_queries_and_nested_paths() {
        let conversation_id = ConversationId::new("conv_123").unwrap();
        let item_id = ResponseItemId::new("item_123").unwrap();
        let include = ConversationItemIncludeOptions::new()
            .include(ResponseInclude::FileSearchCallResults)
            .include(ResponseInclude::ReasoningEncryptedContent);

        let (base_url, captured) = one_shot_server("v1", json_ok(ITEM_LIST_BODY)).await;
        let client = make_client(base_url);
        ConversationItemsResource::new(&client)
            .create(
                &conversation_id,
                &CreateConversationItemsRequest::new(vec![input_item()]).unwrap(),
                &include,
            )
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "POST");
        assert_eq!(
            request_target(&captured),
            "/v1/conversations/conv_123/items?include=file_search_call.results&include=reasoning.encrypted_content"
        );
        assert_eq!(
            request_json(&captured),
            serde_json::json!({
                "items":[{"type":"message","role":"user","content":"hello"}]
            })
        );

        let options = ListConversationItemsOptions::new()
            .limit(10)
            .unwrap()
            .order(ListOrder::Asc)
            .after(item_id.clone())
            .include(ResponseInclude::MessageOutputTextLogprobs);
        let (base_url, captured) = one_shot_server("v1", json_ok(ITEM_LIST_BODY)).await;
        let client = make_client(base_url);
        ConversationItemsResource::new(&client)
            .list(&conversation_id, &options)
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "GET");
        assert_eq!(
            request_target(&captured),
            "/v1/conversations/conv_123/items?limit=10&order=asc&after=item_123&include=message.output_text.logprobs"
        );

        let encoded_item_id = ResponseItemId::new("item/a?b#c%").unwrap();
        let item = r#"{"type":"message","id":"item_123","role":"user","content":[{"type":"input_text","text":"hello"}],"status":"completed"}"#;
        let (base_url, captured) = one_shot_server("v1", json_ok(item)).await;
        let client = make_client(base_url);
        ConversationItemsResource::new(&client)
            .retrieve(&conversation_id, &encoded_item_id, &include)
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "GET");
        assert_eq!(
            request_target(&captured),
            "/v1/conversations/conv_123/items/item%2Fa%3Fb%23c%25?include=file_search_call.results&include=reasoning.encrypted_content"
        );

        let (base_url, captured) = one_shot_server("v1", json_ok(CONVERSATION_BODY)).await;
        let client = make_client(base_url);
        ConversationItemsResource::new(&client)
            .delete(&conversation_id, &item_id)
            .await
            .unwrap();
        let captured = captured.await.unwrap();
        assert_eq!(request_method(&captured), "DELETE");
        assert_eq!(
            request_target(&captured),
            "/v1/conversations/conv_123/items/item_123"
        );
    }
}
