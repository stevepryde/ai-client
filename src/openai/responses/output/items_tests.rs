use super::*;

#[test]
fn all_28_pinned_output_item_tags_decode_typed() {
    // Minimal fixtures derived from specs/openai/openapi.documented.yml 2.3.0.
    let values = [
        serde_json::json!({"type":"message","id":"msg_1","role":"assistant","content":[],"status":"completed"}),
        serde_json::json!({"type":"file_search_call","id":"fs_1","status":"completed","queries":[],"results":null}),
        serde_json::json!({"type":"function_call","call_id":"c","name":"f","arguments":"{}"}),
        serde_json::json!({"type":"function_call_output","id":"o","status":"completed","call_id":"c","output":"ok"}),
        serde_json::json!({"type":"web_search_call","id":"w","status":"completed","action":{"type":"search","queries":["rust"]}}),
        serde_json::json!({"type":"computer_call","id":"c","call_id":"x","pending_safety_checks":[],"status":"completed"}),
        serde_json::json!({"type":"computer_call_output","id":"c","status":"completed","call_id":"x","output":{"type":"computer_screenshot","file_id":"file_1"}}),
        serde_json::json!({"type":"reasoning","id":"r","summary":[]}),
        serde_json::json!({"type":"program","id":"p","call_id":"c","code":"x","fingerprint":"f"}),
        serde_json::json!({"type":"program_output","id":"p","call_id":"c","result":"ok","status":"completed"}),
        serde_json::json!({"type":"tool_search_call","id":"t","call_id":"c","execution":"server","arguments":{},"status":"completed"}),
        serde_json::json!({"type":"tool_search_output","id":"t","call_id":"c","execution":"server","tools":[],"status":"completed"}),
        serde_json::json!({"type":"additional_tools","id":"a","role":"developer","tools":[]}),
        serde_json::json!({"type":"compaction","id":"c","encrypted_content":"x"}),
        serde_json::json!({"type":"image_generation_call","id":"i","status":"completed","result":null}),
        serde_json::json!({"type":"code_interpreter_call","id":"c","status":"completed","container_id":"ctr","code":null,"outputs":null}),
        serde_json::json!({"type":"local_shell_call","id":"l","call_id":"c","action":{"type":"exec","command":["pwd"],"env":{}},"status":"completed"}),
        serde_json::json!({"type":"local_shell_call_output","id":"l","call_id":"c","output":"ok"}),
        serde_json::json!({"type":"shell_call","id":"s","call_id":"c","action":{"commands":["pwd"],"timeout_ms":null,"max_output_length":null},"status":"completed","environment":null}),
        serde_json::json!({"type":"shell_call_output","id":"s","call_id":"c","status":"completed","output":[],"max_output_length":1}),
        serde_json::json!({"type":"apply_patch_call","id":"a","call_id":"c","status":"completed","operation":{"type":"delete_file","path":"old.txt"}}),
        serde_json::json!({"type":"apply_patch_call_output","id":"a","call_id":"c","status":"completed"}),
        serde_json::json!({"type":"mcp_call","id":"m","server_label":"s","name":"n","arguments":"{}"}),
        serde_json::json!({"type":"mcp_list_tools","id":"m","server_label":"s","tools":[],"error":null}),
        serde_json::json!({"type":"mcp_approval_request","id":"m","server_label":"s","name":"n","arguments":"{}"}),
        serde_json::json!({"type":"mcp_approval_response","id":"m","request_id":"r","approve":true,"approval_request_id":"a","reason":null}),
        serde_json::json!({"type":"custom_tool_call","call_id":"c","name":"n","input":"{}"}),
        serde_json::json!({"type":"custom_tool_call_output","id":"c","status":"completed","call_id":"x","output":"ok"}),
    ];
    assert_eq!(values.len(), 28);
    for value in values {
        let item: OpenAIResponseOutputItem = serde_json::from_value(value).unwrap();
        assert!(!matches!(item, OpenAIResponseOutputItem::Unknown(_)));
    }
}

#[test]
fn nested_closed_output_variants_reject_malformed_known_shapes() {
    let malformed = serde_json::json!({
        "type":"web_search_call", "id":"w", "status":"completed",
        "action":{"type":"find_in_page","url":"https://example.test"}
    });
    assert!(serde_json::from_value::<OpenAIResponseOutputItem>(malformed).is_err());

    let malformed = serde_json::json!({
        "type":"computer_call_output", "id":"c", "status":"completed", "call_id":"x",
        "output":{"type":"future_screenshot"}
    });
    assert!(serde_json::from_value::<OpenAIResponseOutputItem>(malformed).is_err());
}

#[test]
fn nested_future_action_round_trips_losslessly() {
    let future = serde_json::json!({
        "type":"web_search_call", "id":"w", "status":"completed",
        "action":{"type":"browse_graph","node":"future"}
    });
    let item: OpenAIResponseOutputItem = serde_json::from_value(future.clone()).unwrap();
    assert_eq!(serde_json::to_value(item).unwrap(), future);
}
