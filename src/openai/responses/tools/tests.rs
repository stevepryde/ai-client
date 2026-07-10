use super::*;

#[test]
fn unknown_tool_round_trips_and_known_malformed_tool_errors() {
    let value = serde_json::json!({"type":"future_tool","secret":"value","nested":{"x":1}});
    let parsed: OpenAIResponsesTool = serde_json::from_value(value.clone()).unwrap();
    let OpenAIResponsesTool::Unknown(raw) = &parsed else {
        panic!("expected unknown tool")
    };
    assert_eq!(raw.tag(), "future_tool");
    assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    assert!(serde_json::from_value::<OpenAIResponsesTool>(
        serde_json::json!({"type":"function","name":"missing required fields"})
    )
    .is_err());
}

#[test]
fn all_pinned_tool_tags_and_choice_forms_decode_typed() {
    // Minimal fixtures derived from specs/openai/openapi.documented.yml 2.3.0.
    let tools = [
        serde_json::json!({"type":"function","name":"f","strict":true,"parameters":{}}),
        serde_json::json!({"type":"file_search","vector_store_ids":[]}),
        serde_json::json!({"type":"computer"}),
        serde_json::json!({"type":"computer_use_preview","environment":"browser","display_width":1,"display_height":1}),
        serde_json::json!({"type":"web_search"}),
        serde_json::json!({"type":"web_search_2025_08_26"}),
        serde_json::json!({"type":"mcp","server_label":"server"}),
        serde_json::json!({"type":"code_interpreter","container":"auto"}),
        serde_json::json!({"type":"programmatic_tool_calling"}),
        serde_json::json!({"type":"image_generation"}),
        serde_json::json!({"type":"local_shell"}),
        serde_json::json!({"type":"shell"}),
        serde_json::json!({"type":"custom","name":"c"}),
        serde_json::json!({"type":"namespace","name":"n","description":"d","tools":[]}),
        serde_json::json!({"type":"tool_search"}),
        serde_json::json!({"type":"web_search_preview"}),
        serde_json::json!({"type":"web_search_preview_2025_03_11"}),
        serde_json::json!({"type":"apply_patch"}),
    ];
    assert_eq!(tools.len(), 18);
    for value in tools {
        let tool: OpenAIResponsesTool = serde_json::from_value(value).unwrap();
        assert!(!matches!(tool, OpenAIResponsesTool::Unknown(_)));
    }

    let choices = [
        serde_json::json!("auto"),
        serde_json::json!({"type":"allowed_tools","mode":"auto","tools":[]}),
        serde_json::json!({"type":"file_search"}),
        serde_json::json!({"type":"function","name":"f"}),
        serde_json::json!({"type":"mcp","server_label":"s","name":null}),
        serde_json::json!({"type":"custom","name":"c"}),
        serde_json::json!({"type":"programmatic_tool_calling"}),
        serde_json::json!({"type":"apply_patch"}),
        serde_json::json!({"type":"shell"}),
    ];
    assert_eq!(choices.len(), 9);
    for value in choices {
        serde_json::from_value::<OpenAIToolChoice>(value).unwrap();
    }
}

#[test]
fn closed_network_policy_and_skill_schemas_are_typed_and_strict() {
    let policy = serde_json::json!({
        "type":"allowlist",
        "allowed_domains":["api.example.test"],
        "domain_secrets":[{
            "domain":"api.example.test",
            "name":"API_KEY",
            "value":"private-value"
        }]
    });
    let parsed: OpenAIContainerNetworkPolicy = serde_json::from_value(policy.clone()).unwrap();
    assert!(matches!(parsed, OpenAIContainerNetworkPolicy::Allowlist(_)));
    assert_eq!(serde_json::to_value(&parsed).unwrap(), policy);
    assert!(!format!("{parsed:?}").contains("private-value"));
    assert!(serde_json::from_value::<OpenAIContainerNetworkPolicy>(
        serde_json::json!({"type":"allowlist"})
    )
    .is_err());

    let reference = serde_json::json!({
        "type":"skill_reference",
        "skill_id":"skill_123",
        "version":"latest"
    });
    let skill: OpenAIContainerSkill = serde_json::from_value(reference.clone()).unwrap();
    assert_eq!(serde_json::to_value(skill).unwrap(), reference);

    let inline = serde_json::json!({
        "type":"inline",
        "name":"review",
        "description":"Review code",
        "source":{
            "type":"base64",
            "media_type":"application/zip",
            "data":"private-base64"
        }
    });
    let skill: OpenAIContainerSkill = serde_json::from_value(inline.clone()).unwrap();
    assert_eq!(serde_json::to_value(&skill).unwrap(), inline);
    assert!(!format!("{skill:?}").contains("private-base64"));
    assert!(
        serde_json::from_value::<OpenAIContainerSkill>(serde_json::json!({
            "type":"inline",
            "name":"review",
            "description":"Review code",
            "source":{"type":"base64","media_type":"application/zip"}
        }))
        .is_err()
    );

    let future = serde_json::json!({"type":"future_policy","policy":{"x":1}});
    let policy: OpenAIContainerNetworkPolicy = serde_json::from_value(future.clone()).unwrap();
    assert!(matches!(policy, OpenAIContainerNetworkPolicy::Unknown(_)));
    assert_eq!(serde_json::to_value(policy).unwrap(), future);
}

#[test]
fn open_object_boundaries_reject_scalars_and_preserve_required_nulls() {
    let nullable = serde_json::json!({
        "type":"function",
        "name":"nullable_function",
        "strict":null,
        "parameters":null,
        "output_schema":null
    });
    let tool: OpenAIResponsesTool = serde_json::from_value(nullable.clone()).unwrap();
    assert_eq!(serde_json::to_value(tool).unwrap(), nullable);

    for malformed in [
        serde_json::json!({"type":"function","name":"f","parameters":{}}),
        serde_json::json!({"type":"function","name":"f","strict":true}),
        serde_json::json!({"type":"function","name":"f","strict":"yes","parameters":{}}),
        serde_json::json!({"type":"function","name":"f","strict":true,"parameters":[]}),
        serde_json::json!({"type":"function","name":"f","strict":true,"parameters":{},"output_schema":"text"}),
    ] {
        assert!(
            serde_json::from_value::<OpenAIResponsesTool>(malformed).is_err(),
            "accepted malformed function tool"
        );
    }

    let allowed = serde_json::json!({
        "type":"allowed_tools",
        "mode":"auto",
        "tools":[
            {"type":"function","name":"f"},
            {"future_definition":{"nested":true}}
        ]
    });
    let choice: OpenAIToolChoice = serde_json::from_value(allowed.clone()).unwrap();
    assert_eq!(serde_json::to_value(choice).unwrap(), allowed);
    for scalar in [
        serde_json::json!("function"),
        serde_json::json!(1),
        serde_json::json!(null),
    ] {
        assert!(
            serde_json::from_value::<OpenAIToolChoice>(serde_json::json!({
                "type":"allowed_tools",
                "mode":"required",
                "tools":[scalar]
            }))
            .is_err()
        );
    }
}

#[test]
fn mcp_debug_redacts_connection_and_auth_material() {
    let tool = OpenAIMcpTool {
        server_label: "private-server".into(),
        server_url: Some("https://example.test/mcp?token=private-token".into()),
        connector_id: None,
        tunnel_id: None,
        authorization: Some("Bearer private-auth".into()),
        server_description: None,
        headers: Some(std::collections::BTreeMap::from([(
            "x-secret".into(),
            "private-header".into(),
        )])),
        allowed_tools: None,
        allowed_callers: None,
        require_approval: None,
        defer_loading: None,
        extra: Default::default(),
    };
    let debug = format!("{tool:?}");
    assert!(!debug.contains("private-token"));
    assert!(!debug.contains("private-auth"));
    assert!(!debug.contains("private-header"));
}
